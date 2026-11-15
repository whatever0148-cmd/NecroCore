---
name: NecroCore
description: A working COBOL and CHIP-8 analysis tool lit like a 1978 bank machine room at midnight.
colors:
  machine-room-black: "#140d09"
  well-black: "#1a110b"
  console-brown: "#1d130c"
  raised-console: "#26180f"
  hairline-umber: "#3a2516"
  rivet-umber: "#5c3820"
  rust: "#b3602f"
  rust-deep: "#7d3d1c"
  amber-phosphor: "#ffb22e"
  amber-dim: "#e0952a"
  amber-hover: "#ffc354"
  amber-pale: "#ffd27a"
  amber-ink: "#1e1204"
  listing-amber: "#f0c27a"
  bone-text: "#f2e2c4"
  faded-ledger: "#bba184"
  disabled-ink: "#8c735a"
  listing-number-umber: "#a87444"
  graph-edge-umber: "#8c6849"
  oxide-red: "#ff6a45"
  oxide-pale: "#ffb39e"
  ghost-slate: "#a9bdd6"
  zombie-lichen: "#c3d24f"
  crt-glass: "#0e0805"
  bone-paper: "#efe2c6"
  paper-ink: "#2a1b10"
  green-phosphor: "#3dff8e"
  patch-add: "#2a2410"
  patch-remove: "#3a160f"
typography:
  wordmark:
    fontFamily: "VT323, ui-monospace, Menlo, monospace"
    fontSize: "38px"
    fontWeight: 400
    lineHeight: 1
    letterSpacing: "0.02em"
  display:
    fontFamily: "VT323, ui-monospace, Menlo, monospace"
    fontSize: "44px"
    fontWeight: 400
    lineHeight: 1
    letterSpacing: "0.01em"
  title:
    fontFamily: "VT323, ui-monospace, Menlo, monospace"
    fontSize: "24px"
    fontWeight: 400
    lineHeight: 1
    letterSpacing: "0.03em"
  readout:
    fontFamily: "VT323, ui-monospace, Menlo, monospace"
    fontSize: "52px"
    fontWeight: 400
    lineHeight: 0.95
    letterSpacing: "0.01em"
    fontFeature: "tnum"
  stat:
    fontFamily: "VT323, ui-monospace, Menlo, monospace"
    fontSize: "34px"
    fontWeight: 400
    lineHeight: 1
    fontFeature: "tnum"
  lead:
    fontFamily: "ui-sans-serif, system-ui, -apple-system, Segoe UI, sans-serif"
    fontSize: "16px"
    fontWeight: 400
    lineHeight: 1.55
  body:
    fontFamily: "ui-sans-serif, system-ui, -apple-system, Segoe UI, sans-serif"
    fontSize: "15px"
    fontWeight: 400
    lineHeight: 1.55
  control:
    fontFamily: "ui-sans-serif, system-ui, -apple-system, Segoe UI, sans-serif"
    fontSize: "15px"
    fontWeight: 600
    lineHeight: 1.55
  label:
    fontFamily: "ui-sans-serif, system-ui, -apple-system, Segoe UI, sans-serif"
    fontSize: "12px"
    fontWeight: 600
    lineHeight: 1.3
    letterSpacing: "0.05em"
  code:
    fontFamily: "IBM Plex Mono, ui-monospace, SF Mono, Menlo, Consolas, monospace"
    fontSize: "13px"
    fontWeight: 400
    lineHeight: 1.6
  count:
    fontFamily: "IBM Plex Mono, ui-monospace, SF Mono, Menlo, Consolas, monospace"
    fontSize: "22px"
    fontWeight: 600
    lineHeight: 1
    fontFeature: "tnum"
rounded:
  none: "0px"
  paper: "2px"
  tag: "3px"
  control: "4px"
  panel: "6px"
  screen: "8px"
  cabinet: "10px"
spacing:
  hair: "2px"
  xs: "6px"
  sm: "8px"
  md: "16px"
  lg: "20px"
  xl: "32px"
  section: "48px"
components:
  button:
    backgroundColor: "{colors.raised-console}"
    textColor: "{colors.bone-text}"
    typography: "{typography.control}"
    rounded: "{rounded.control}"
    padding: "9px 12px"
    height: "40px"
  button-go:
    backgroundColor: "{colors.amber-phosphor}"
    textColor: "{colors.amber-ink}"
    typography: "{typography.control}"
    rounded: "{rounded.control}"
    padding: "9px 12px"
    height: "40px"
  button-go-hover:
    backgroundColor: "{colors.amber-hover}"
    textColor: "{colors.amber-ink}"
  button-disabled:
    backgroundColor: "transparent"
    textColor: "{colors.disabled-ink}"
    rounded: "{rounded.control}"
  select:
    backgroundColor: "{colors.raised-console}"
    textColor: "{colors.bone-text}"
    typography: "{typography.code}"
    rounded: "{rounded.control}"
    padding: "9px 12px"
    height: "40px"
  textarea:
    backgroundColor: "{colors.raised-console}"
    textColor: "{colors.bone-text}"
    typography: "{typography.code}"
    rounded: "{rounded.control}"
    padding: "9px 12px"
  tab:
    backgroundColor: "transparent"
    textColor: "{colors.faded-ledger}"
    typography: "{typography.control}"
    rounded: "{rounded.none}"
    padding: "10px 14px 8px"
  tab-selected:
    backgroundColor: "transparent"
    textColor: "{colors.bone-text}"
  status-tag:
    backgroundColor: "transparent"
    textColor: "{colors.faded-ledger}"
    rounded: "{rounded.tag}"
    padding: "5px 10px"
  status-tag-on:
    backgroundColor: "transparent"
    textColor: "{colors.amber-phosphor}"
  panel:
    backgroundColor: "{colors.console-brown}"
    textColor: "{colors.bone-text}"
    rounded: "{rounded.panel}"
    padding: "16px"
  rail-section:
    backgroundColor: "transparent"
    textColor: "{colors.bone-text}"
    rounded: "{rounded.none}"
    padding: "14px 0 16px"
  listing-screen:
    backgroundColor: "{colors.crt-glass}"
    textColor: "{colors.listing-amber}"
    typography: "{typography.code}"
    rounded: "{rounded.screen}"
  printed-report:
    backgroundColor: "{colors.bone-paper}"
    textColor: "{colors.paper-ink}"
    rounded: "{rounded.paper}"
    padding: "12px 14px"
  proof-readout:
    textColor: "{colors.amber-phosphor}"
    typography: "{typography.readout}"
  finding-chip-ghost:
    textColor: "{colors.ghost-slate}"
  finding-chip-vampire:
    textColor: "{colors.oxide-red}"
  finding-chip-zombie:
    textColor: "{colors.zombie-lichen}"
  keypad-key-on:
    backgroundColor: "{colors.green-phosphor}"
    textColor: "#02140a"
    rounded: "{rounded.control}"
    padding: "12px 0"
---

# Design System: NecroCore

## Overview

**Creative North Star: "The Machine Room at Midnight"**

NecroCore is a working analysis tool lit the way a 1978 bank machine room is lit after hours: a warm brown-black room, amber phosphor on the screens, rust on the racks and rails, and printed reports on bone paper. The light comes from the instruments, never from the chrome around them. Everything that glows is either live data or the one thing you can do next.

The shell is an Operate surface, dense and calm. A sticky top bar holds the wordmark, a status tag and three tabs. Below it, wide content columns carry the program listing on its amber screen, a control-flow graph, and tables, while a narrow right rail stacks the findings counts, the proof readout, Kiro's plan on paper and the run actions, each separated by a rust rule rather than boxed. Controls are standard web controls in the room's materials: no invented widgets.

One signature move carries the identity: haunted lines. Rot is shown on the program's own lines with a quiet, specific animation per kind of defect, and selecting any haunted line or graph block jumps to its counterpart. Motion is short and exponential (ease-out), and all of it stops under reduced motion.

**Key Characteristics:**
- Warm brown-black ground with a faint amber glow from the top left. No neutral greys in the ground or chrome.
- Amber phosphor for live text, focus and the proof. Rust for structure and selection. Oxide red only for a live finding.
- Three type voices: VT323 for headings and readouts, IBM Plex Mono for code and numbers, system sans for prose and controls.
- Flat panels with hairline umber borders. The only depth belongs to screens (listing and cabinet) and to the printed paper.
- Haunted lines as the single signature interaction.
- Green phosphor exists only inside the PhantomArcade cabinet.

## Colors

A low-light, warm, single-temperature palette: umber and brown-black surfaces, amber and rust as the two working accents, and three finding hues that each mean one kind of rot.

### Primary
- **Amber Phosphor** (amber-phosphor): live text, the focus ring, caret, link colour, checkbox and slider accents, the parity figures and the proof readout. As a solid fill it appears only on the one action button of a group (button-go). Its darker sibling **Amber Dim** (amber-dim) sets the denominator of the proof readout; **Amber Hover** (amber-hover) is the go-button hover fill; **Amber Pale** (amber-pale) is link hover and the ink of added or planned lines in diffs and patch notes. **Listing Amber** (listing-amber) is the body text of the program listing on its screen. **Amber Ink** (amber-ink) is text on any amber fill.

### Secondary
- **Rust** (rust): structure and selection. The selected tab's 2px underline, hover borders on controls, the selected listing line's outline, hovered or selected graph blocks, and dashed loop-back edges.
- **Rust Deep** (rust-deep): the 1px rule above each rail section and the border of a status tag that is on.

### Tertiary (finding hues)
- **Oxide Red** (oxide-red): a live finding only. The Vampire (unbounded accumulator) chip, line and graph block, and failure text. **Oxide Pale** (oxide-pale) is the line text inside a Vampire row and removed lines in a diff.
- **Ghost Slate** (ghost-slate): the Ghost (unreachable code) chip, line text and graph block stroke. It is the only cool hue in the system and it is cool on purpose: the dead line is the one thing in the room without warmth.
- **Zombie Lichen** (zombie-lichen): the Zombie (non-terminating loop) chip, shimmer and graph block stroke. A sickly yellow-green, deliberately distinct from the arcade's green phosphor.
- **Green Phosphor** (green-phosphor): the PhantomArcade cabinet only (the pressed keypad key; the canvas pixels are drawn in the same green on a near-black green glass).

### Neutral
- **Machine-Room Black** (machine-room-black): the page ground, under a fixed radial glow of warm brown (#2a170b) from the upper left.
- **Well Black** (well-black): recessed wells such as the control-flow graph frame.
- **Console Brown** (console-brown): panels.
- **Raised Console** (raised-console): controls (buttons, selects, text areas) and graph block fills.
- **CRT Glass** (crt-glass): the listing screen and the diff screen.
- **Hairline Umber** (hairline-umber): panel borders, table rules, footer rule.
- **Rivet Umber** (rivet-umber): control borders, the screen bezel line, scrollbar thumb.
- **Bone Text** (bone-text): primary text. **Faded Ledger** (faded-ledger): secondary text, table headers, technical terms, tab labels at rest. **Disabled Ink** (disabled-ink): text of disabled controls only.
- **Bone Paper** (bone-paper) with **Paper Ink** (paper-ink): printed reports (Kiro's plan, specs).
- **Patch Add** (patch-add) and **Patch Remove** (patch-remove): line backgrounds for added and removed lines.

### Named Rules
**The One Amber Field Rule.** Solid amber fill is reserved for the single action you can take (the go button). The proof readout is the only other large amber element, and it is amber text, not a field. Everything else uses amber as text, ring or accent at most.

**The Rust Holds the Frame Rule.** Rust marks structure and selection, never content. If something is selected, hovered or ruled off, it is rust. If something is read, it is bone or amber.

**The Oxide Means Live Rot Rule.** Oxide red appears only where a live finding or a failure is shown. It is never decoration, never a brand accent.

**The Green Lives in the Cabinet Rule.** Green phosphor never leaves the PhantomArcade cabinet.

**The Warm Ground Rule.** Every ground, panel, border and neutral text is warm (brown, umber, bone). The only cool hue is Ghost Slate, and only on Ghost findings.

## Typography

**Display Font:** VT323 (with ui-monospace, Menlo, monospace), self-hosted woff2, weight 400 only.
**Body Font:** the system sans (ui-sans-serif, system-ui, -apple-system, Segoe UI).
**Label/Mono Font:** IBM Plex Mono 400 / 500 / 600 (with ui-monospace, SF Mono, Menlo, Consolas), self-hosted woff2.

**Character:** VT323 is the terminal's own voice for names and readouts; Plex Mono is the line printer for code and numbers; the plain system sans keeps prose and controls quiet and readable so the instruments stay the loudest thing on screen.

### Hierarchy
- **Wordmark** (VT323 400, 38px, 1): the NecroCore wordmark in the top bar, with "Core" in amber. 32px under 600px.
- **Display** (VT323 400, 44px, 1): one view title per tab. 36px under 600px.
- **Title** (VT323 400, 24px, 1): section heads (Listing, What is rotten, Proof, Findings).
- **Readout** (VT323 400, 52px, 0.95, tabular): the proof readout in amber, with the denominator at 26px in Amber Dim. 44px under 600px.
- **Stat** (VT323 400, 34px, tabular): amber figures in stat rows.
- **Lead** (sans 400, 16px, 1.55, max 68ch): the one-paragraph intro under each view title, in Faded Ledger.
- **Body** (sans 400, 15px, 1.55): prose, notes, table cells (14px). Detail cells cap at 64ch.
- **Control** (sans 600, 15px): button and tab text (tabs at 14.5px).
- **Label** (sans 600, 12px, 0.05em, uppercase): table column headers only.
- **Code** (Plex Mono 400, 13px, 1.6): the listing, logs, diffs, register grids; 12.5px on paper and diffs; 13.5px in numeric cells.
- **Count** (Plex Mono 600, 22px, tabular): the three findings counts in the rail.

### Named Rules
**The Three Voices Rule.** VT323 only for headings, the wordmark and big numerals. Plex Mono for anything a machine wrote: code, program and block names, numbers, registers. System sans for anything a person reads as prose or clicks.

**The Label Beside the Term Rule.** A finding label (Ghost, Vampire, Zombie) is always set next to its technical term in Faded Ledger at the same size or one step smaller.

## Layout

A centred column, max 1320px, with 20px gutters (14px under 600px) and 26px top padding. The top bar is sticky, 94% opaque machine-room black with a hairline umber bottom rule; tabs sit at its right edge.

The CryptLedger view is a two-column grid: content at 2fr and a rail at 1fr (min 300px), 20px gap. The rail is sticky at 76px from the top on wide screens. Rail sections are not boxed: each is separated by a 1px Rust Deep rule above it, with 6px between sections. PhantomArcade uses a cabinet column plus a 340px side panel; registers and RAM sit in a two-up split below.

Spacing rhythm is small and dense: 2px between count rows, 6 to 8px between tools and controls, 16px panel padding and split gaps, 20px column gaps, 32px above section titles, 48px above the footer.

Breakpoints: at 1020px the CryptLedger grid collapses to one column and reorders (listing, then the rail, then the graph). At 960px splits and the arcade layout collapse. At 600px wide tables become stacked grid rows with named areas, the tab list takes the full width, and the type sizes step down as noted in Typography. Coarse pointers get 44px minimum targets on buttons, selects and tabs; checkbox labels are 44px tall everywhere.

## Elevation & Depth

The room is flat. Panels and controls carry no shadow; they separate by tone (console brown on machine-room black) and hairline umber borders. Depth belongs to the instruments: the listing screen is a recessed glass with a radial amber-brown glow, an inner vignette and a soft drop, the arcade canvas sits in a bezel ring, and printed paper lifts slightly off the console.

### Shadow Vocabulary
- **Screen recess** (`box-shadow: inset 0 0 0 1px #0a0603, inset 0 0 60px rgba(0,0,0,.65), 0 18px 40px -24px rgba(0,0,0,.9)`): the listing screen only.
- **Cabinet bezel** (`box-shadow: inset 0 0 40px rgba(0,0,0,.8), 0 0 0 6px #0b0805, 0 0 0 7px #5c3820, 0 14px 30px -18px rgba(0,0,0,.9)`): the arcade canvas only.
- **Paper lift** (`box-shadow: 0 10px 24px -14px rgba(0,0,0,.8)`): printed reports in the rail.

### Named Rules
**The Only Glass Is the Screen Rule.** Shadows and glows belong to screens and paper. Panels, buttons and tags stay flat.

## Shapes

Corners are small and get rounder as the object becomes more like a physical instrument: paper 2px, status tags 3px, controls 4px, panels and graph frames 6px, the listing screen 8px, the arcade cabinet glass 10px. Tabs and rail sections have no corners at all: tabs are an underline, rail sections are a rule. Borders are 1px throughout; dashed borders mean "not available here" (disabled buttons) or "drop zone" (file drop), and dashed graph edges mean a loop back. Haunted lines carry a 1px inset left edge in their finding hue.

## Components

### Buttons
Standard, quiet and solid; only the one action you should take next is lit.
- **Shape:** gently squared (4px), 40px minimum height, 9px 12px padding, 1px Rivet Umber border.
- **Default:** Raised Console fill, Bone Text, sans 600.
- **Go (primary):** Amber Phosphor fill and border with Amber Ink text; hover lifts to Amber Hover. One per action group.
- **Hover / Focus:** default buttons shift their border to Rust over 0.2s ease-out (cubic-bezier(.16,1,.3,1)). Focus everywhere is a 2px Amber Phosphor outline at 2px offset.
- **Disabled:** transparent, dashed Hairline Umber border, Disabled Ink text, not-allowed cursor. A disabled go button loses its amber entirely.
- **Rail actions:** full width, left aligned, stacked with 8px gaps.

### Tabs
- **Style:** sans 600 14.5px, no fill, no corners. At rest Faded Ledger; hover Bone Text; selected Bone Text with a 2px Rust underline.

### Status tag
- **Style:** Plex Mono 500 12px in a 3px-cornered 1px Rivet Umber outline, Faded Ledger text. When live, amber text and a Rust Deep border.

### Inputs / Fields
- **Style:** selects and text areas share the button's fill, border and 4px corners; their text is Plex Mono (14px / 13.5px). Checkboxes, sliders and progress bars are native, tinted with Amber Phosphor.
- **Focus:** the global amber outline. Hover on a select shifts its border to Rust.
- **Drop zone:** a dashed Rivet Umber outline with Faded Ledger text that turns amber-bordered with Bone Text while a file is over it.

### Cards / Containers
- **Corner Style:** 6px.
- **Background:** Console Brown; recessed wells use Well Black.
- **Shadow Strategy:** flat (see Elevation).
- **Border:** 1px Hairline Umber.
- **Internal Padding:** 16px. Rail sections drop the box and keep only a Rust Deep top rule with 14px above and 16px below.

### Tables
Full width, collapsed, 1px Hairline Umber row rules, 9px 10px cells. Column headers use the Label style in Faded Ledger. Numeric columns are right aligned, tabular and in Plex Mono. Clickable rows tint with translucent rust on hover and focus. Under 600px rows become stacked grids.

### Finding chips
- **Style:** a 16px inline line-drawn icon (1.4 stroke, currentColor) followed by the label, sans 600 13px, in the finding's hue: Ghost Slate, Oxide Red or Zombie Lichen. Always followed by the technical term.

### Printed report
Bone Paper with Paper Ink, Plex Mono 12.5 to 13px, 2px corners, 260 to 360px max height with internal scroll, paper lift shadow. Used for Kiro's fix plan and generated specs.

### Proof readout
The rail's VT323 readout: matched count in Amber Phosphor at 52px, the slash and denominator at 26px in Amber Dim, with a sans meta line beneath naming the saved run.

### Listing screen (signature: haunted lines)
Plex Mono 13px on CRT Glass with Listing Amber text, a 52px right-aligned line-number gutter, 8px corners and the screen recess. On load it powers on (brightness and saturation rise over 0.9s, ease-out). Haunted lines:
- **Ghost:** a faint slate wash and slate left edge; the text sits at 85% opacity and flickers in hard steps every 5.5s.
- **Vampire:** an oxide wash and left edge; a deeper oxide band swells from left to right every 3.2s.
- **Zombie:** a lichen wash and left edge; a soft lichen shimmer scrolls across the line every 3.4s, linear.
- **Selected:** a 1px Rust outline inset and a translucent rust tint. Detached (healed) lines strike through in a muted brown.
Selecting a haunted line jumps the graph and the finding; selecting a graph block highlights its lines. All animation is removed under prefers-reduced-motion.

### Control-flow graph
Plex Mono 11px labels on Raised Console blocks with a 1px Rivet Umber stroke, inside a Well Black frame. Hover or selection draws a 2px Rust stroke; keyboard focus a 2px amber stroke. Finding blocks take their hue as stroke over a tinted dark fill. Loop-back edges are dashed Rust.

### Arcade cabinet
A panel with a vertical brown gradient (#22170f to #170f0a) holding the pixelated canvas on near-black green glass inside the cabinet bezel. The 4x4 hex keypad uses standard buttons in Plex Mono; a pressed key fills with Green Phosphor.

## Do's and Don'ts

### Do:
- **Do** keep every ground, panel and border warm: Machine-Room Black, Console Brown, Hairline and Rivet Umber.
- **Do** reserve the solid Amber Phosphor fill for the single go action of a group, and make the proof readout amber text in VT323.
- **Do** use Rust for selection, hover borders, rail rules and the selected-tab underline.
- **Do** pair every finding label with its technical term and its hue, using the finding chip.
- **Do** set code, block names and numbers in IBM Plex Mono with tabular figures in numeric columns.
- **Do** keep controls as standard web controls in the room's materials, at 40px (44px on coarse pointers).
- **Do** use ease-out cubic-bezier(.16,1,.3,1) for state changes and turn every animation off under prefers-reduced-motion.

### Don't:
- **Don't** use cold neutral greys for grounds, panels, borders or text. The one cool hue is Ghost Slate, on Ghost findings only.
- **Don't** use Oxide Red for anything except a live finding or a failure.
- **Don't** use Green Phosphor outside the PhantomArcade cabinet.
- **Don't** fill large areas with amber, or light more than one amber button per action group.
- **Don't** put shadows or glows on panels, buttons or tags; depth belongs to screens and paper.
- **Don't** set prose or controls in VT323, or headings in the system sans.
- **Don't** build a dark dashboard of metric cards with one neon accent; the instruments (listing, graph, readout, tables) carry the screen.
