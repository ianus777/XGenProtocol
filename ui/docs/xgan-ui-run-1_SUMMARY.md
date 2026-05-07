# XGen UI Skeletons — Session Summary

> **Status**: COMPLETED
> Date: May 2026
> **Last updated**: 2026-05-07
> Author: JozefN
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.
> License: BSL 1.1 (converts to GPL upon project handover)

---

## Purpose

Record of the §7 work plan execution from `ui/docs/xgen-ui-chat-briefing.md`. All nine items shipped. Three verifier rounds resolved layout regressions caused by skin grid leakage, list-marker fallthrough, and implicit grid-row placement on the Console.

---

## Files in this folder

| File                  | Purpose                                                                |
| --------------------- | ---------------------------------------------------------------------- |
| `index.html`          | Skeleton viewer toolbar — switch app, switch skin, side-by-side split  |
| `client.html`         | Client room view — `data-xgen-screen="room"`                           |
| `node.html`           | Node admin dashboard — `data-xgen-screen="dashboard"`                  |
| `console.html`        | Client/Node Console overlay — VT220 default, Appendix E states         |
| `setup.html`          | First-run flow — three steps, purely local, zero network               |
| `modules.html`        | Module List + install/consent dialog                                   |
| `layer3-demo.html`    | Space-theme override cascade demo (4 themes side by side)              |
| `tokens.css`          | Spec-locked token slots (Ch6 §6.2). Single source of truth             |
| `skin-classic.css`    | Default operator skin — dark, ochre primary, steel-blue infra accent   |
| `skin-workshop.css`   | Alternate operator skin — warm paper, late-90s pro-tool flavour        |
| `skin-contrast.css`   | High-contrast skin — proves the same skeleton survives a different look |

---

## What landed (briefing §7 work plan)

### 1. CSS variables renamed to spec names

All token references now use the `--xgen-color-*`, `--xgen-space-*`, `--xgen-radius-*`,
`--xgen-font-*`, `--xgen-shadow-*`, `--xgen-layout-*`, `--xgen-transition-*` namespaces
declared by Ch6 §6.2. Old `--accent` / `--ink-*` / bare `--surface-*` names removed
across all skeletons and skins. No visual change — rename only.

### 2. Seven named widget slots + Module List

Both `client.html` and `node.html` now contain empty bounded containers:

- `space.header`
- `room.toolbar`
- `room.sidebar.top`
- `room.sidebar.bottom`
- `room.message.decorator`
- `node.dashboard.widget`
- `global.statusbar` (rendered as the bottom `<footer>`)

Bounded-container/`overflow: hidden` rule is baked into `tokens.css` so widget
modules cannot escape their slot. Empty slots render a dashed placeholder so the
contract is visible during skeleton work.

`modules.html` lists installed modules grouped by scope (System / Space / User),
documents the seven slot names, and exposes the install/consent dialog.

### 3. Base message header stripped to spec

Every message article now contains exactly Avatar / Display name / Timestamp /
Reply link in its header (Ch6 §6.7). Tier badges have been moved into the
`room.message.decorator` slot. The `TierBadge` is now a token-driven element
(`.xgen-tier-badge[data-tier="1..4"]`) using the four colours from briefing Q5
(T1 green / T2 blue / T3 amber / T4 red). The `NodeStatusIndicator`
(`.xgen-state-indicator[data-state="…"]`) is the single component used for the
left-side lifecycle pill in every status bar.

### 4. Console state indicator bound to Appendix E

`console.html`'s status-bar pill carries `data-state` values from the Appendix E
state machine: `SETUP`, `INITIALISING`, `CONNECTING`, `AUTHENTICATING`, `READY`,
`DEGRADED_AUTH`, `DEGRADED_FEDERATION`, `DEGRADED_NODE`, `DEGRADED_STORAGE`,
`RECONNECTING`, `DISCONNECTED`, `MAINTENANCE`, `CLOSING`. Click the pill to open
the built-in state-set reference dropdown — the briefing Q5 "double as
reference" affordance.

### 5. Token values committed (per briefing §D)

- **Surfaces** — broken blacks with a hint of warmth (`#16181c` … `#2c3038`),
  never pure `#000000`.
- **Text** — broken whites with warm tone (`#ece9e1` … `#7a7d85`), never pure
  `#ffffff`.
- **Primary (Client/identity layer)** — muted ochre as default working orange;
  saturated logo orange reserved for attention/CTA only.
- **Infra (Node/federation layer)** — steel-blue family derived from Node logo.
- **Tier glyph colours** — green/blue/amber/red per briefing Q5.

Hex values are committed but explicitly subject to UI testing per Ch6.

### 6. Layer 3 Space-theme demo

`layer3-demo.html` shows four mock Space surfaces side by side using the same
shell. Each Space sets `data-xgen-space-theme="…"`, which overrides only the
five permitted tokens from briefing §B:

- `--xgen-color-primary`
- `--xgen-color-primary-hover`
- `--xgen-color-surface`
- `--xgen-color-surface-raised`
- `--xgen-color-border`

A "Disable Space themes" toggle wires the briefing §C Identity Profile
preference: when set, the body attribute `data-xgen-disable-space-themes="true"`
neutralises Layer 3 overrides and the application skin applies everywhere.

### 7. First-run Identity Setup

`setup.html` is the three-step purely-local flow per briefing §J:

1. Display name
2. Passphrase (Argon2id KDF + ChaCha20-Poly1305 encryption disclosed in copy)
3. Generate keypair (the moment of identity creation)

Step 3 documents the silent `auto_connect_local` scan that follows — the client
reaches `READY` (unconnected) silently if no local Node responds.

### 8. Empty states

Left as labelled placeholders. Final content deferred to Ch6 second pass per
briefing §I.

### 9. Module List + consent dialog

`modules.html` includes the full Ch6 §6.8.4 user-mode install consent flow as
a `<dialog>`. The "send `room.message` events on your behalf" permission is
flagged as high-risk. Operator-warning copy explicitly states user-mode modules
are not vetted by the Node operator.

---

## Verifier round-trip log

Three rounds, all real layout bugs from the work plan execution.

### Round 1 — three regressions

| Issue                                                | Cause                                                                                  | Fix                                                                                                                  |
| ---------------------------------------------------- | -------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------- |
| `layer3-demo.html` body grid broken                  | `skin-classic.css` `body { display: grid; … }` rule leaked across to demo page         | Scoped the body grid to `body[data-xgen-screen="room"], [="dashboard"], [="modules"]`                               |
| `console.html` header/main/form overlapping at top:0 | `grid-template-rows: 1fr auto auto` had four children in three rows                    | Changed to four rows (`auto 1fr auto auto`)                                                                          |
| `node.html` overview cards prefixed `1. 2. 3. 4.`    | `tokens.css` list-reset enumerated specific `aria-label`s only, missed Node sections | Broadened reset to include `main > section > ol`                                                                     |

### Round 2 — Console still broken

The four-row grid without explicit `grid-template-areas` did not produce
source-order placement (browser was generating implicit rows around the inline
`<script>` child).

**Fix:** added `grid-template-areas: "head" "main" "form" "status"` to the body
rule and `grid-area: …` to each direct child. Form sits above the status bar,
status bar at the bottom — matching the Console design.

### Round 3 — clean

No further regressions reported.

---

## Open items for the next session

1. **Empty-state final copy** — deferred to Ch6 second pass.
2. **Per-message detail panel** — the briefing asked us to propose what the
   "context-on-demand" surface (Event ID / signature / DAG parent / room
   metadata) looks like. To be designed when we reach the message timeline in
   the next iteration; proposal will be handed back for formalisation into Ch6.
3. **Token hex values** — committed as starting points, explicitly revisable
   under UI testing per Ch6.
4. **Console undocking** — planned future capability per briefing Q4. The
   overlay is designed first; undocking is an extension.

---

## File-system layout reminder

Skeletons live under `ui/` in the source repo (`ianus777/XGenProtocol`). The
versions in this project mirror that folder for design iteration; they are
freestanding HTML and run with no build step.
