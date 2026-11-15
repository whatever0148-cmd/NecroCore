# Kiroween 2025 winners: what actually won (read from their Devpost pages, 2026-11-15)

2025: 4,957 participants, 747 submissions. 2026: 13,950 registered as of Nov 15.

| Prize | Project | One line | Live link judges could try | Stack |
|---|---|---|---|---|
| 1st ($30k) | InteractionKit | template for gesture-controlled games and apps with MediaPipe; two demo apps (pumpkin math slicer, air drawing board) | GitHub + setup script, dev.to post | Python, MediaPipe, Pygame |
| 2nd ($20k) | UnitSet | Frankenstein UI builder: canvas + AI + code + live sandboxes | (not read yet) | |
| 3rd ($10k) | SpookBnB | vacation rental site; dark mode becomes a flashlight horror escape room with 4 puzzles | spook-bn-b.vercel.app | Next.js, Tailwind, Framer Motion |
| Best Resurrection ($5k) + Influencer Judges | Faxi | fax-to-internet bridge for offline seniors in Japan; you can fax a real number and get a reply | real fax numbers, qa.faxi.jp | Node, Gemini vision, MCP servers, Telnyx |
| Best Frankenstein | rumorNet | misinformation detection in 100+ languages | (not read yet) | |
| Best Skeleton Crew + Most Creative | Impulse Kit | Go/No-Go reflex trainer from a Swiss neuroscience study; 3 themes on one skeleton (sugar habit, French articles, trading) | GitHub Pages | plain HTML, CSS, JS |
| Best Costume Contest | Seance | world map of dark folklore; page-flipping book quiz, wrong answer flips you back, 3D smoke, owl and wolf sounds | seance-three.vercel.app | Next.js, Three.js, react-pageflip |
| Best Startup | Splat-n-Treat | spatial CMS for Gaussian-splat 3D worlds | | |
| Influencer Judges | BlueKai | BlueSky client for KaiOS flip phones | | |
| Blog bonus only | Dark Ledger | COBOL-verified payroll engine on Base | | (COBOL resurrection already done) |

## Patterns in what won
1. A person with a problem comes first (seniors left behind, people breaking a sugar habit, devs who cannot build CV apps fast). Technology is second.
2. Judges can try it in seconds: a public URL, a fax number, a one-line setup. Every winner had a hosted link or a trivial setup.
3. The experience is the hook: flashlight cursor, page-turning book, sounds, puzzles, a game loop. Costume and design are felt, not described.
4. Kiro use is told feature by feature with the reason each mattered (InteractionKit's table: vibe coding, 3 hooks, 2 specs, 2 steering docs, Context7 MCP; Faxi: 16 specs, 4 hooks, 2 custom MCP servers). Hooks that automate a real, repeated chore impress.
5. One sharp idea beats many features. Impulse Kit is simple HTML and won two prizes.
6. None of them led with benchmarks or parity tables.

## What this means for NecroCore
- Lead with a person: the bank's last COBOL programmer, a retro game fan whose childhood games are dying, not "parity 70,000/70,000".
- The hosted page must be playable in 10 seconds: open link, game runs, drag a slider, ghost appears.
- The UI must be an experience (crypt, CRT, séance) that still reads clearly, not a dark admin dashboard.
- Keep the proof as the evidence layer, one click deep.

## 2nd place UnitSet and Best Frankenstein rumorNet (read 2026-11-15)
- UnitSet (2nd, $20k): AI design-to-code canvas. Its Kiro section is the deepest of all winners: 21 specs, 15 steering docs, 9 hooks (code review that files Linear tickets, import cleanup on delete, component scaffolder on create, schema sync on save, docs lookup via Context7, pre-commit audit, DevTools UI sync, auto-docs, screenshot capture), 4 MCP servers (Chrome DevTools, Context7, Linear, 21st.dev Magic). Every item says what it automated and why it mattered. Live at unitset.in.
- rumorNet (Best Frankenstein): multilingual misinformation agents on AWS Lambda. Honest about Kiro's failures ("reward hacking": tests that only import, try/except blankets) and how they audited it. Spec-driven UI, vibe-coded backend, steering docs, Filesystem and Git MCP. Live dashboard on Render.
- Lesson: Technological Implementation is judged on how deeply and thoughtfully Kiro was used. Count and name specs, steering docs, hooks and MCP servers, give each one a real job, and be honest about where Kiro failed.
