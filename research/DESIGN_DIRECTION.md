# NecroCore design direction, from research (2026-11-15)

Sources: `research/kiroween-2025-winners.md` (the 2025 winners' own pages) and the Pinterest
boards saved in `research/moodboard/` (searches: retro CRT terminal UI, 1970s mainframe room
amber screen, haunted computer horror interface, amber monochrome dashboard).

## The idea in one line
A 1978 bank computer room after midnight. The payroll job on the amber terminal is haunted,
and Kiro is the medium who exorcises it, then proves the ghosts are gone.

## Why this direction
- The winners sold an experience a judge feels in ten seconds (Seance's dark map, SpookBnB's
  flashlight). NecroCore today is a dark admin dashboard (`moodboard/10` to `12`).
- The 1970s mainframe board (`moodboard/02`) is the closest real reference to NecroCore's own
  story: COBOL banking programs from 1978 to 1985 ran on exactly these rooms and terminals.
- Amber phosphor on warm black reads as both "old bank computer" and "Halloween", without
  the generic purple-and-green spooky look.

## Visual system (taken from the boards, not invented)
| Element | Reference | Rule |
|---|---|---|
| Ground | mainframe room photos, warm near-black | background a warm black, never pure #000 or cold grey |
| Primary light | amber phosphor terminals (`01`, `02`) | amber for live text and focus, with a soft glow |
| Accent | rust and orange-red walls (`02`) | rust for structure, orange-red only for danger (a live ghost) |
| Paper | 1970s fanfold printouts | the autopsy report prints on green-bar continuous paper, cream ground |
| Arcade | green phosphor CRTs (`01`) | green only inside PhantomArcade's cabinet, so the two apps feel like two machines in one room |
| Horror | eye in the CRT, glitch, cursed dialog (`03`) | horror is reserved for findings: a Ghost line flickers, a Zombie loop keeps scrolling |
| Type | terminal screens | monospace for code and numbers; one heavy display face for titles |
| Effects | CRT glass | scanlines and bloom on the terminal only, never on reading text; all motion off under prefers-reduced-motion |

## Experience flow a judge sees from the hosted link
1. The room loads dark, the terminal powers on, `BANK-INTEREST-1978` boots line by line.
2. Ghost lines flicker in the listing. One click: "Summon Kiro".
3. A real recorded Kiro turn plays back (labelled as a recording, with its date), the patch is applied.
4. "Prove it": the counter runs to 10,000 of 10,000 against the reference interpreter.
5. The printer prints the autopsy on green-bar paper. Evidence sits one click deeper.
6. The arcade cabinet in the corner runs the lifted games live, sliders change physics.

## Story to lead with (from Faxi and InteractionKit: a person first)
Every year another COBOL programmer retires, and the programs that move money stay behind.
Use only sourced facts (the Reuters 2017 figure already cited in `deliverables/`).
Do not invent a named person.

## Not yet researched
- 2nd place UnitSet and Best Frankenstein rumorNet pages, and the winners' videos.
- Fonts: pick from real terminal references before choosing.
