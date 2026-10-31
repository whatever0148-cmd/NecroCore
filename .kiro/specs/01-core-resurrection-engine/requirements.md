# Requirements: core resurrection engine (necro-core)

## Introduction
One intermediate representation for every legacy front end, a control-flow graph
over it, and a static scanner that finds three pathologies and writes them up as
EARS requirements.

## Requirement 1: unified IR
User story: as a lifter author, I want one IR with blocks, terminators and typed
variable declarations so that COBOL and CHIP-8 share the scanner.

1. WHEN a front end lowers a program THE SYSTEM SHALL produce a `NecroIR` with an
   entry block, a block list, variable declarations (integer digits, scale, sign,
   input flag) and an origin tag.
2. WHEN a block is created THE SYSTEM SHALL record a `Span` with byte start, byte
   end and line of the source it came from.

## Requirement 2: control-flow graph
1. WHEN `Cfg::build` runs THE SYSTEM SHALL add one edge per jump, two per branch,
   an edge from a `Perform` to its entry, and an edge from each paragraph end to
   the return point of every `Perform` whose range ends at that paragraph.
2. WHEN the graph is built THE SYSTEM SHALL compute reachability from the entry
   and the strongly connected components that contain a cycle.

## Requirement 3: pathologies
1. WHEN a block with code is not reachable from the entry THE SYSTEM SHALL report
   a Ghost (unreachable code) with the block label, span and incoming edges.
2. WHEN a loop has no exit edge, or every conditional exit reads only variables
   the loop never writes, THE SYSTEM SHALL report a Zombie (non-terminating loop)
   listing the loop blocks and edges.
3. WHEN a variable inside a loop is only ever assigned `var + positive constant`
   (or appended) and no exit condition reads it THE SYSTEM SHALL report a Vampire
   (unbounded accumulator) naming the variable and its storage width.
4. WHEN the scan finishes THE SYSTEM SHALL render every finding as EARS text in
   `requirements_markdown`, ready for a `.kiro/specs/` directory.

## Requirement 4: healing
1. WHEN `heal` is given a report THE SYSTEM SHALL detach each Ghost, route each
   Zombie loop's back edges through a fuel guard, and route each Vampire loop's
   back edges through a cap guard.
2. WHEN the healed IR is rescanned THE SYSTEM SHALL report zero Ghosts and zero
   Zombies for the three sample COBOL programs.
