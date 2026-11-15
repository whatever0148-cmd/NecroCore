# Tasks: Kiro guardrails

- [x] 1. `scripts/hooks/hook.py` with six jobs, each tested by hand on real files. _Requirements: 1-4, 6_
- [x] 2. Six hook configs in `.kiro/hooks/`. _Requirements: 1-4_
- [x] 3. Seven steering docs. _Requirements: 2, 5_
- [x] 4. Four agents, each passing `kiro-cli agent validate`. _Requirements: 5_
- [x] 5. Kiro (`scribe`) fixes the voice problems the hook found in README.md, with the hook firing on its writes. _Requirements: 2_
- [ ] 6. Kiro (`prover`) runs a live-bridge benchmark so the bridge column reads kiro-cli, saved under `benchmarks/`. _Requirements: 1_
- [ ] 7. Devpost and README Kiro sections list the real counts and point to `hook-firings.log`. _Requirements: 6_
