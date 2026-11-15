# Requirements: Observatory machine-room redesign

## Introduction
On 2026-11-15 we compared the Observatory with the 2025 Kiroween winners
(`research/kiroween-2025-winners.md`) and with Pinterest boards of 1970s computer
rooms (`research/moodboard/SOURCES.md`). It looked like a generic dark admin
dashboard. This spec gives the redesign checks that can be verified. It was written
alongside the build (done by Claude Code), so Kiro and the hooks can hold
later edits to it. The design system the build ended with is in DESIGN.md.

## Requirement 1: a judge understands it in ten seconds
1. WHEN the hosted page opens with no hash THE SYSTEM SHALL show PhantomArcade
   with a game already running as WebAssembly in the browser.
2. WHEN CryptLedger opens THE SYSTEM SHALL scroll the listing to the first
   finding and show the Ghost, Vampire and Zombie counts beside it, each with
   its technical term.
3. WHEN the page is the static copy THE SYSTEM SHALL label itself "static copy"
   and use the saved autopsy, the saved proof and the recorded Kiro turn, each
   marked as recorded.

## Requirement 2: the rot is visible on its own lines
1. WHEN a line is a Ghost THE SYSTEM SHALL flicker it in steps. WHEN it is a
   Vampire THE SYSTEM SHALL grow a band behind it. WHEN it is a Zombie THE
   SYSTEM SHALL shimmer it.
2. WHEN the user clicks or presses Enter on a haunted line THE SYSTEM SHALL move
   to that block in the control-flow graph.
3. WHILE prefers-reduced-motion is set THE SYSTEM SHALL show every finding
   without motion.

## Requirement 3: proof before trust
1. THE SYSTEM SHALL show the parity for the selected program from the saved
   benchmark file named in the build command, never a typed number.
2. WHEN the recorded Kiro plan is shown THE SYSTEM SHALL say it is a recording
   and offer a Replay button.

## Requirement 4: craft floor
1. THE SYSTEM SHALL meet WCAG AA for text (4.5:1) and 3:1 for graph lines and icons.
2. THE SYSTEM SHALL lay out at 390 px wide with no horizontal scroll. Tables
   stack into labelled rows.
3. THE SYSTEM SHALL load its fonts from its own `fonts/` folder (OFL), with no
   request to a font CDN.
