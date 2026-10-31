# I brought a 1982 payroll job and a 1991 game back to life, and proved it with 70,000 test runs

*I wrote this post for the Kiroween 2026 hackathon on Devpost. The project is NecroCore, and it is my Resurrection entry.*

Old software has a quiet kind of death. It still runs, but nobody dares to touch it. I wanted to find out if I could take one of these programs, rebuild it for a modern runtime, and know for sure that the copy behaves like the original. Knowing for sure was the whole point. A rewrite that looks right is not a rewrite I would trust with payroll.

So I built NecroCore. It reads old programs, finds what is wrong with them, rebuilds them as WebAssembly, and then fights the rebuilt copy against the original until one of them blinks. This is how that went.

## Two kinds of dead

I picked two targets that look nothing alike.

**COBOL batch jobs.** I wrote three small programs in the style of 1978, 1982 and 1985 jobs, because I could not find real code I was allowed to publish. One posts savings interest, one runs a weekly payroll, one clears wire transfers. Each has defects on purpose.

**Retro game bytecode.** CHIP-8 is a tiny virtual machine from the 1970s. SUPER-CHIP is the 1991 upgrade that added a 128 by 64 screen, scrolling and bigger sprites. I wrote four original games for it, including a runner called NECRORUNNER-1991.

What do these two have in common? Both are old, both are small enough to understand completely, and both can be turned into the same intermediate form, so one scanner and one prover can handle them.

## What the autopsy looks for

The scanner builds a control-flow graph and reports three things.
- **Ghost.** Code that nothing can reach.
- **Vampire.** A counter that only grows inside a loop, and no exit condition ever reads it.
- **Zombie.** A loop whose exit can never change, so it cannot end.

The UI uses those names and the report uses the plain terms. My first graph was wrong in a way that took an afternoon to see. A COBOL paragraph that is only PERFORMed would fall into the next paragraph in my graph, so every loop looked like it had an exit. Once I made the graph remember where each PERFORM came from, the payroll program's endless loop showed up where it should.

The scanner also has a limit that I want to be honest about. The interest program does not finish on 2,845 of 10,000 inputs, and the scanner cannot see it, because two loops share one paragraph and merge in the graph. The proof step catches it anyway. That is why I wanted a proof step.

## Lifting to WebAssembly

For COBOL the hard part is arithmetic. Numbers are fixed-point decimals with a picture clause such as `PIC 9(7)V99`. If you store a result that is too wide, COBOL drops the high digits. If you mark a store ROUNDED, it rounds half away from zero. My interpreter and my WebAssembly emitter share one set of rules. Values live in a 64-bit integer scaled by 10,000, and every store truncates or rounds and then cuts to the picture width.

For ROMs the plan is simpler. The emitter writes one `br_table` case per ROM address and compiles each instruction inline. One trick makes live modding possible. A patchable byte, such as the speed of the ball, is not baked into the code. The compiled instruction loads it from the machine's memory every time it runs. So changing the game is a single byte write, and the registers, the stack and the screen stay exactly as they were.

SUPER-CHIP added the 128 by 64 mode, scrolling, a 16 by 16 sprite instruction and eight user flags. The machine image grew from 4.5 kilobytes to about 5.3 kilobytes, and one mode byte tells the drawing code which screen size to use. The assembler gained the new mnemonics, and NECRORUNNER uses all of them. The ground scrolls with a four-pixel left scroll each frame, the runner and the block are 16 by 16 sprites, and the best score is parked in the user flags.

## The proof

The prover runs 10,000 seeded vectors per program. For a COBOL program a vector is a set of input values with a bias toward edge cases such as zero, one and the largest value that fits. For a ROM a vector is a random number seed, a key schedule that changes every 25 steps, and a hot patch of every patch point halfway through. Both machines run, and I compare the full state at the end. For a ROM that means every byte of the machine image.

Seven programs, 10,000 vectors each, gives 70,000 vectors. All 70,000 matched. The COBOL programs ran 9 to 16 times faster as WebAssembly than in my interpreter. The ROMs did not get faster at all, between 0.92 and 1.06 times, because copying the machine image in and out costs about as much as the little program does. I left that in the results.

## Healing

The healer detaches the Ghosts and puts a fuel counter on every loop that cannot end, plus a cap on every runaway counter. After healing, the scanner runs again. It found 19 problems across the seven programs and 0 were left. The prover then checks that the healed program still agrees with its own interpreter.

## How Kiro fits in

Kiro shows up in three ways.

**It shaped the work.** I wrote four specs in EARS form before the Rust, plus three steering files and three hooks in Kiro's v1 hook format. The task lists in `.kiro/specs/` are the real to-do list. The Spec button writes new specs of the same shape from a live scan.

**It runs inside the product.** A small crate drives `kiro-cli chat --no-interactive` with tools off and a timeout. The Exorcise button asks Kiro for a plan. The benchmark has a `--live-kiro` mode, and in the last run all 7 of 7 turns were real, 142 seconds in total. The transcripts are saved in the repo.

**It is also a tool for the tool.** I wrote a small MCP server so Kiro can call the autopsy itself. In a real run, kiro-cli read the config, started the server, called `necro_autopsy` on the payroll program and listed the Zombie, the Vampire and the Ghost with their line numbers.

I should also say that I wrote most of the Rust with Claude Code as my coding partner. Kiro did the spec, steering, hooks, MCP and CLI parts above. I did not use Kiro Powers or Crew.

One small story from the mod box. You type a sentence like "double ball speed, widen paddles, and enable zero-gravity bounce". Kiro turns it into patch values, and the page writes them into the running game. The first time I tried it, Kiro skipped the zero-gravity part, so I added a hint to the prompt about which patch point controls vertical movement. On another run, Kiro's answer came back as valid JSON that was missing its final closing brace. The parser now repairs that case, labels the answer as repaired, and has a test for it. The last run answered with clean JSON, set three bytes, and the Pong game kept its score.

## A notch you can drop files on

I wanted one more thing that felt like a haunted house. There is a small dark pill that sits under the MacBook notch. Drop a `.cob` or `.ch8` file on it and it posts the file to the local daemon. The daemon runs the command from my `on-legacy-drop` hook and the pill shows the three counts, for example Ghost 1, Vampire 1, Zombie 0, then 0 left after healing. Then it opens the healed result in the browser. It is about 300 lines of Swift and AppKit.

## What I would do next

The COBOL subset needs REDEFINES and file handling before it can touch real programs. The two apps, CryptLedger and PhantomArcade, should live in their own folders. And I would like a Kiro Power that bundles the autopsy.

## Try it

The code is built with Rust, wasmtime and a bit of Swift. Build it with `cargo build --release`, run `necro-cli observatory --serve`, and open `127.0.0.1:4300`. Every number in this post comes from a run that is saved in the repo under `benchmarks/`.
