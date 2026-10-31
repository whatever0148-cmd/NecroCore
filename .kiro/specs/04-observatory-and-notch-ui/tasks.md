# Tasks: Observatory and notch UI (Phase 2, browser Observatory done, native app not started)

- [ ] 1. `necro-cli mcp` tool contract frozen: `necro_autopsy`, `necro_parity`, `necro_hotpatch`
  - _Requirements: 1.1, 1.2, 2.2_
- [ ] 2. Swift package skeleton with an MCP stdio client
  - _Requirements: 1.1_
- [ ] 3. CFG view with pathology highlighting and span hover
  - _Requirements: 1.1, 3.1_
- [ ] 4. Prove and Exorcise panels streaming results
  - _Requirements: 1.2, 1.3_
- [ ] 5. Notch strip with draggable patch values
  - _Requirements: 2.1, 2.2, 2.3_
- [ ] 6. Spec button writing through the Kiro bridge
  - _Requirements: 1.4, 3.2_
- [x] 7. Browser Observatory: `necro-cli observatory` writes one HTML page with autopsies, benchmark, and the lifted ROMs running live with hot-patch sliders (`necro-cli/src/observatory.rs`)
  - _Requirements: 1.1, 2.2, 3.1, 3.2 (partly: the page shows stored results, it does not run Prove, Exorcise or Spec itself)_
