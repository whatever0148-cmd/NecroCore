---
inclusion: always
---

# Proof before trust

NecroCore's promise is that nothing is called fixed or modernised until the prover
shows every tested input still matches the original. These rules keep that promise
in the code and in the words.

- A number that appears in the UI, the README or a deliverable comes from a saved
  run in `benchmarks/`. Cite the file. Never round it up, never type it by hand.
  The `guard-benchmarks` hook blocks writes to `benchmarks/*.json` and `*.txt`.
- To change a number, change the code and rerun
  `cargo run --release --bin necro-cli -- benchmark --vectors 10000 --json <file>`.
- A heal is accepted only when the rescan finds zero pathologies and parity on the
  healed program stays at 100.000%.
- When Kiro is not available the bridge says "offline" in its output. Keep that
  label. Never present an offline plan as a live Kiro turn.
- No invented users, banks, testimonials or partners. The only outside fact about
  COBOL we cite is the Reuters 2017 figure already in `deliverables/`.
