# Social posts for NecroCore

Rule from the Devpost Resources page: the post has to START with #KiroweenHackathon26, be posted during the submission period, and its link goes on the submission form. Each post below starts with the tag. Post the link to the repo or the demo video where it says LINK.

## X (Twitter) thread

**1/**
#KiroweenHackathon26 I rebuilt a 1982 payroll job and a 1991 game as WebAssembly, then fought the copies against the originals with 70,000 test runs. Zero mismatches. Built with @kirodotdev. NecroCore, my Resurrection entry. 🧵 #Kiroween

**2/**
The tool reads old programs and finds three kinds of rot. Ghost is code nothing can reach. Vampire is a counter that only grows. Zombie is a loop that can never end. Every finding comes with a line number.

**3/**
Then it lifts the program to WebAssembly and runs 10,000 seeded vectors through the original and the copy. For game ROMs it compares every byte of the machine's memory. 7 programs, 70,000 vectors, 70,000 matches.

**4/**
Honest results: the COBOL copies run 9 to 16 times faster than my interpreter. The game ROMs got no faster, about 1x, because copying the machine state costs as much as the game. I kept that in the table.

**5/**
Kiro does real work in it. I wrote 4 specs in EARS form first. A small MCP server lets kiro-cli call the autopsy itself. And a live kiro-cli turn writes the fix plan for each program. 7 of 7 benchmark turns were real, and the transcripts are in the repo.

**6/**
My favourite part. You type "double ball speed, widen paddles, and enable zero-gravity bounce" and Kiro turns it into patch values. The page writes them into the running Pong game. The score and the screen stay. No reset.

**7/**
There is also a dark pill that sits under the MacBook notch. Drop a .cob or .ch8 file on it and it shows the Ghost, Vampire and Zombie counts, then opens the healed result. Swift and AppKit, about 300 lines.

**8/**
Everything is written in Rust with wasmtime, and every number I posted comes from a run saved in the repo. LINK #KiroweenHackathon26

## LinkedIn

#KiroweenHackathon26

I am a first-year student, and this month I built NecroCore for the Kiroween 2026 hackathon on Devpost.

The idea is simple. Old programs are hard to trust because nobody can say what they really do. NecroCore reads a COBOL batch job or a retro game ROM, reports dead code, runaway counters and loops that cannot end, rebuilds the program as WebAssembly, and then proves the copy behaves like the original.

What the last run showed:
- 7 sample programs, 10,000 test vectors each, 70,000 of 70,000 matched
- COBOL copies ran 9 to 16 times faster than my reference interpreter
- Game ROMs ran at about the same speed, and I report that too
- 19 problems found, 0 left after healing

What I liked about building it with @kirodotdev. I wrote the specs in EARS form before the code. A small MCP server lets kiro-cli call the scanner directly. A live kiro-cli turn writes the fix plan, and I saved all 7 benchmark transcripts in the repo. I also used Claude Code for most of the Rust, and I say so in the write-up.

The part that made me smile is a mod box for the retro games. You type what you want in plain words, and Kiro answers with patch values that get written into the running game without a reset.

If you work on legacy migration, I would like to hear which proof you would want to see before you trusted a rewrite. LINK

#Kiroween #WebAssembly #Rust #COBOL

## Instagram caption

#KiroweenHackathon26 I made old software come back to life. 👻🧛🧟

NecroCore reads a 1982 COBOL payroll job and a 1991 game ROM, finds the code that is dead, the counters that never stop and the loops that never end, rebuilds both as WebAssembly, and proves the copy matches the original with 70,000 test runs.

You can type "double ball speed, widen paddles" and watch the running Pong game change without a reset. A little dark pill under the MacBook notch takes dropped files and shows the monsters it found.

Built with Rust, WebAssembly and @kirodotdev for the Kiroween 2026 hackathon. Link in bio.

#Kiroween #coding #webassembly #rust #retrogaming #studentdeveloper
