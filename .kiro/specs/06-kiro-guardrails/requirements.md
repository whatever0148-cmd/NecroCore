# Requirements: Kiro guardrails for NecroCore

## Introduction
NecroCore's promise is proof before trust. Writing it in the README does not stop
an agent from typing a nicer number or pasting an em dash into the write-up. This
spec moves the house rules into hooks, steering and agents so Kiro is checked by
the repo itself.

## Requirement 1: numbers come only from runs
1. WHEN an agent tries to write a file under `benchmarks/` ending in `.json` or
   `.txt` THE SYSTEM SHALL block the write and tell the agent which command
   regenerates it.

## Requirement 2: judges' text keeps one voice
1. WHEN an agent writes README.md or a file under `deliverables/` THE SYSTEM
   SHALL list every em dash, semicolon and buzzword with its line number and
   return the list to the agent.

## Requirement 3: the hosted page never drifts
1. WHEN an agent edits `ui.html`, the daemon's `lib.rs` or `observatory.rs` THE
   SYSTEM SHALL rebuild `docs/index.html` and `observatory/index.html`.

## Requirement 4: every edit is tested where it lands
1. WHEN an agent edits a Rust file in `crates/<crate>/` THE SYSTEM SHALL run
   that crate's tests and return failures to the agent.
2. WHEN a spec task finishes THE SYSTEM SHALL run the workspace tests and a
   1,000-vector proof and report the task as not done if either fails.

## Requirement 5: agents with one job each
1. THE SYSTEM SHALL provide `exorcist`, `prover`, `scribe` and `medium` agents
   whose tools and write permissions match their job.

## Requirement 6: an honest log
1. WHEN any hook fires THE SYSTEM SHALL append one line to
   `benchmarks/hook-firings.log` with the hook, the event, the path and whether
   it was fired by Kiro or run by hand.
