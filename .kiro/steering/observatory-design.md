---
inclusion: fileMatch
fileMatchPattern: ["crates/necro-daemon/src/ui.html", "necro-cli/src/observatory.rs", "docs/**"]
---

# Observatory design

The Observatory is a working tool lit like a 1978 bank machine room at midnight.
Its design system is recorded from the shipped page in DESIGN.md. Follow it, do
not restyle it.

#[[file:DESIGN.md]]

- Amber is for live text, focus and the proof. Rust holds the frame. Oxide red
  means live rot. Green phosphor lives only inside the PhantomArcade cabinet.
- Three type voices: VT323 for titles and readouts, IBM Plex Mono for code and
  counts, the system sans for prose and controls. Fonts are self-hosted.
- Every motion turns off under prefers-reduced-motion.
- Text meets WCAG AA (4.5:1), graph lines and icons meet 3:1.
- `docs/index.html` is generated. Never edit it by hand. The `sync-hosted-page`
  hook rebuilds it after edits to the UI or the embedding code.
