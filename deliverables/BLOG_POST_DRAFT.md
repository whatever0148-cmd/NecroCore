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

Seven programs, 10,000 vectors each, gives 70,000 vectors. All 70,000 matched. In the latest run the COBOL programs ran 9.0 to 17.1 times faster as WebAssembly than in my reference interpreter, and the lifted CHIP-8 and SUPER-CHIP games ran 1.6 to 2.3 times faster.

## Healing

The healer detaches the Ghosts and puts a fuel counter on every loop that cannot end, plus a cap on every runaway counter. After healing, the scanner runs again. It found 18 problems across the seven programs and 0 were left. The prover then checks that the healed program still agrees with its own interpreter.

## How Kiro fits in

Kiro shows up at three levels.

**It shaped the work.** I wrote four specs in EARS form before the Rust, and two more later, six in all. There are ten steering files. Four load every time, and the rest load only for the files they cover, like the COBOL rules for the COBOL lifter. The task lists in `.kiro/specs/` are the real to-do list.

**It wrote a feature.** Late in the build I added a requirement to spec 03: a `step_batch` entry point for the lifted ROMs that keeps the program counter and the timer divider in WebAssembly locals and reads key presses from a table, so one host call covers a whole test vector. Then I ran `kiro-cli` on it in real coding turns. It wrote the new export, the layout constants, the prover change and a parity test. It could only read and write files, so I ran the compiler between turns and pasted the errors back. It fixed two of them. One turn ended in an error and I added a single missing parenthesis myself, and the log says so. While it worked, Kiro fired my `on-exorcise-defect` hook 18 times and my `on-parity-verify` hook twice, and I saved the payloads it received. Every turn is in `benchmarks/kiro-coding-sessions.json`.

Parity stayed at 70,000 of 70,000 after every change. I then kept going on speed myself: fused straight-line runs of up to six instructions into one dispatch, held the sixteen registers and the I register in WebAssembly locals for the whole batch, and rewrote sprite drawing to XOR whole rows instead of single pixels. Together they took the lifted ROMs from about the same speed as my native Rust interpreter to 1.6 to 2.4 times faster in my Nov 1 runs, and the draw and scroll routines are the next place to look.

**It runs inside the product.** A small crate drives `kiro-cli chat --no-interactive` with tools off and a timeout. The Exorcise button asks Kiro for a plan. The benchmark has a `--live-kiro` mode. In the latest run all 7 of 7 turns were real, 102.7 seconds in total, with no offline fallback. I also wrote a small MCP server so Kiro can call the autopsy itself. In a real run, kiro-cli read the config, started the server, called `necro_autopsy` on the payroll program and listed the Zombie, the Vampire and the Ghost with their line numbers.

**It is checked by the repo itself.** Near the end I moved my house rules into nine hooks. One blocks any write to a saved benchmark file, so nobody, Kiro included, can type a nicer number. One checks the README and these posts for em dashes, semicolons and buzzwords and hands back the line numbers. One rebuilds the hosted page when the UI changes, and one runs the tests of whatever crate was just edited. I also gave Kiro four custom agents with one job each. The `scribe` agent fixed four semicolons that the voice hook had found in my README, and the hook came back clean on its last write. The `prover` agent ran the full live benchmark. Two things went wrong first. Only `kiro-cli --v3` could find my workspace agents, and a catch-all deny rule I wrote blocked the writes I had allowed. Two of the nine hooks, `task-proof` and `on-legacy-drop`, have not been fired from inside a Kiro session yet. I did not use Kiro Powers or Crew.

One small story from the mod box. You type a sentence like "double ball speed, widen paddles, and enable zero-gravity bounce". Kiro turns it into patch values, and the page writes them into the running game. The first time I tried it, Kiro skipped the zero-gravity part, so I added a hint to the prompt about which patch point controls vertical movement. On another run, Kiro's answer came back as valid JSON that was missing its final closing brace. The parser now repairs that case, labels the answer as repaired, and has a test for it. The last runs answered with clean JSON, set three bytes, and the Pong game kept its score.

## Why it looks like a 1978 machine room
My first interface was a dark dashboard with green buttons, and it looked like every other developer tool. Before redoing it I did two things. I read the pages of last year's Kiroween winners, and the ones that won on design made you feel something in the first ten seconds: a flashlight cursor in a haunted cabin, a folklore book whose pages turn. Then I made mood boards on Pinterest from retro CRT terminals, 1970s mainframe rooms and haunted computer interfaces.

The mainframe rooms won. COBOL banking programs from 1978 ran in exactly those rooms, on amber screens, so the look tells the story without a word. Now the listing glows amber on warm black, Kiro's fix plans print on bone paper, and the arcade games sit in green phosphor. The haunting does a job: a Ghost line flickers because nothing reaches it, a Zombie loop keeps shimmering because it never ends, and a Vampire line pulses because its counter only grows. Click one and the graph jumps to that block.

## A notch you can drop files on

I wanted one more thing that felt like a haunted house. There is a small dark pill that sits under the MacBook notch. Drop a `.cob` or `.ch8` file on it and it posts the file to the local daemon. The daemon runs the command from my `on-legacy-drop` hook and the pill shows the three counts, for example Ghost 1, Vampire 1, Zombie 0, then 0 left after healing. Then it opens the healed result in the browser. It is about 300 lines of Swift and AppKit.

## What I would do next

The COBOL subset needs REDEFINES and file handling before it can touch real programs. And I would like a Kiro Power that bundles the autopsy.

## Try it

The code is built with Rust, wasmtime and a bit of Swift. Build it with `cargo build --release`. Run `cryptledger` and open `127.0.0.1:4301` for the COBOL studio, or run `phantomarcade` and open `127.0.0.1:4302` for the games. Every number in this post comes from a run that is saved in the repo under `benchmarks/`.

- Demo video (2:36): https://youtu.be/blxbArocAas
- Try it in your browser: https://whatever0148-cmd.github.io/NecroCore/
- Code (MIT): https://github.com/whatever0148-cmd/NecroCore
- Devpost: https://devpost.com/software/necrocore
