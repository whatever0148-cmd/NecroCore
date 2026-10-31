# Requirements: Observatory cockpit and macOS notch UI (Phase 2)

## Introduction
The Observatory is the window where a maintainer watches an autopsy, a lift and
a parity run happen, and the notch UI is the always-visible strip on a MacBook
that shows the live patch table of a running ROM. Nothing in this spec is built
in Phase 1; it is written so Phase 2 starts from requirements, not from a blank
file.

## Requirement 1: Observatory
1. WHEN a `.cob` or `.ch8` is dropped on the window THE SYSTEM SHALL run the
   autopsy and show the CFG with Ghost, Vampire and Zombie blocks highlighted,
   each with its line and byte offsets on hover.
2. WHEN the user presses Prove THE SYSTEM SHALL run `necro-prover` with the
   configured vector count and stream parity, divergences and timings as they
   are measured.
3. WHEN the user presses Exorcise THE SYSTEM SHALL apply `heal`, rescan, and
   show the before and after counts side by side.
4. WHEN the user presses Spec THE SYSTEM SHALL call the Kiro bridge and write
   the result into `.kiro/specs/<program>/requirements.md`.

## Requirement 2: notch UI
1. WHEN a ROM is running THE SYSTEM SHALL show each patch point's name and
   current value in the notch strip.
2. WHEN the user drags a value THE SYSTEM SHALL write the byte through
   `PatchTable::apply` into the running module's memory without pausing it.
3. WHEN the display is not a notch MacBook THE SYSTEM SHALL fall back to a
   floating strip at the top of the screen.

## Requirement 3: honesty
1. WHEN any number is shown THE SYSTEM SHALL show the run it came from one hover
   away.
2. WHEN the bridge used the offline fallback THE SYSTEM SHALL say so in the UI.
