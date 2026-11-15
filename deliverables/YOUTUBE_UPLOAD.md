# YouTube upload text for the Kiroween demo video

## Title
NecroCore: dead COBOL and CHIP-8 programs, healed with Kiro and proven (Kiroween 2026 demo)

## Description
NecroCore takes dead programs, a 1978-style COBOL payroll job and four CHIP-8 games, lifts them to WebAssembly and finds three kinds of rot: Ghost (unreachable code), Vampire (unbounded accumulator) and Zombie (non-terminating loop). Kiro writes the fix plan, NecroCore heals the program, and a prover checks the healed version against the original on 10,000 seeded inputs per program.

From the saved run (benchmarks/2026-11-15-r2-live-kiro.json):
- 70,000 of 70,000 inputs matched across 7 programs
- 18 problems found, 0 left after healing
- 7 of 7 real kiro-cli turns

Built with Kiro: 6 specs, 10 steering docs, 9 hooks, 4 custom agents and an MCP server.

The COBOL programs are samples I wrote for this project in a 1978 style. They are not real bank code. The four games are original.

Try it in your browser: https://whatever0148-cmd.github.io/NecroCore/
Code (MIT): https://github.com/whatever0148-cmd/NecroCore

Music and sound were made in code or come from CC0 packs.

#KiroweenHackathon26

## Settings
- Visibility: Public (the Kiroween rules require a public video). Published 2026-11-15 at https://youtu.be/blxbArocAas
- Audience: No, it's not made for kids
- Captions: upload the .srt if the final file comes with one
