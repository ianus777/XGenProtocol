# XGen Protocol — M-RP5.6 B Runbook: `message-stream` scroll machine (Clair)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-09  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. Read-order (Rule 0) + spec of record

Before touching code: CLAUDE.md PLAY → JOURNAL J-484 → this runbook. **Spec of record = `docs/xgen-dd-message-family-phase0.md` v1.2 §9** — §9.3 (scroll machine, concept) + **§9.10 (B implementation refinement, locked J-484)**. This runbook is the build sheet for **step B only**; if it and §9 disagree, §9 wins — stop and flag (Rule 6).

## 1. Scope — B (scroll machine), not A (shell, done)

A shipped the shell (J-482/J-483): the root **is** the scroll viewport (`overflow-y:auto`, `max-height:340px`), and getter G already exposes `atBottom` (hardcoded `true` in A). **B builds only the scroll behaviour** on that existing viewport:
1. **`atBottom` live** (drops the A stub).
2. **Stick-to-bottom** on append when at/near bottom; **no yank** + **jump-to-latest pill** when scrolled up.
3. **Preserve-position-on-prepend** (older-load anchor).
4. **Initial scroll to bottom** on mount.

**Do NOT touch:** `MessageDescriptor`, `stream/grouping.ts`, `message.svelte` (§9.7 — no descriptor change; grouping/dividers recompute for free via the existing `$derived computeRows`). Only `message-stream.svelte` + the sampler + `skin.css`. Fixtures only — no node↔client channel (J-476).

## 2. `atBottom` computation (Phase-0 §9.10, Q1 — single 80px, rAF-throttled)

One build-time const governs everything:
```ts
const BOTTOM_THRESHOLD_PX = 80; // Joe-tunable
```
- `atBottom` (a `$state`) = `scrollHeight − scrollTop − clientHeight ≤ BOTTOM_THRESHOLD_PX`.
- Recompute in a `scroll` listener on the root viewport, **rAF-throttled** (coalesce bursts to one recompute per frame — a clean, non-flickering CDP read). No hysteresis / second threshold.
- Pill visible ⇔ `!atBottom`. Scrolling back within 80px flips `atBottom` true → pill auto-hides.
- Getter G's `atBottom` now reads the live `$state` (was `true` stub in A).

## 3. Stick-to-bottom + jump pill (Phase-0 §9.10, Q2 — inline chrome)

**On append** (message count grew, first-id unchanged → newest added at bottom):
- if `atBottom` was true **before** the update → after flush, scroll to bottom (`scrollTop = scrollHeight`) so the newest stays in view (stick).
- if `atBottom` was false → **do NOT move** the viewport (no yank); the pill shows (`!atBottom`).

**Pill = inline chrome** (the `day-divider` precedent — stream-owned, NOT a component, NOT through the widget registry, NOT registered in `__XGEN_DEBUG__`):
- `<button class="jump-to-latest" onclick={jumpToBottom}>` inside the stream markup, rendered `{#if !atBottom}`.
- `position:absolute` bottom-right over the viewport (appearance in `skin.css`; structure — position/z-index above rows — in the component `<style>`).
- `jumpToBottom()` → `scrollTop = scrollHeight` → `atBottom` recomputes true → pill hides.
- a11y: `type="button"`, `aria-label` (e.g. "Jump to latest"); it's a scroll affordance, not a message.

## 4. Preserve-position-on-prepend (Phase-0 §9.10, Q3 — scrollHeight-delta + first-id heuristic)

**Prepend-detection heuristic** (keeps the stream self-contained, no new consumer prop): a prepend is when `messages.length` grew **AND** the previous first-descriptor id is **no longer at index 0** (older messages inserted on top). Distinguish from append (count grew, first-id stable) and replace/reset.

**Anchor mechanism** (deterministic, CDP-assertable — the "prepend `scrollTop` invariance" the DoD names):
- Capture `prevScrollHeight = el.scrollHeight` **before** the DOM flushes (Svelte 5: `$effect.pre`, tracking `messages`).
- After the flush (`await tick()`), if the update was a prepend, apply `el.scrollTop += el.scrollHeight − prevScrollHeight`.
- Net effect: the first previously-visible message stays at the same visual y-offset (no jump). `atBottom` is unaffected by a prepend (older content above the fold).
- Track the previous first-id + previous length in local `$state`/refs so the heuristic can compare across updates.

**Interaction with §3:** append and prepend are mutually exclusive per update; branch on the heuristic. Grouping/dividers recompute automatically (`computeRows` is `$derived`) — do not re-solve.

## 5. Initial scroll on mount (Phase-0 §9.10, sub-point 1)

On mount, scroll the viewport to the bottom (newest visible) — a chat opens at the latest. Do it after first paint (`$effect` + `tick()` / mount hook): `scrollTop = scrollHeight`. This matches the `atBottom:true` init and must not fight the prepend anchor (mount is neither append nor prepend).

## 6. Sampler — the `stream-scroll` live fixture (Phase-0 §9.10, Q4)

Add **one** new DD·composite fixture; leave the four static ones (basic / days / empty / bg) untouched (they proved A).
- **`stream-scroll`** — a **mutable `$state` array** seeded with ~10 `text` messages (enough to overflow `max-height:340px` → the viewport is actually scrollable), a couple of authors so grouping shows.
- **Controls** in sampler chrome, same pattern as the existing `streamBgLive` toggle button:
  - **append** → `push` a `text` message dated `now` (same author as the current last, so it also extends a group).
  - **prepend** → `unshift` a `text` message dated ~10 min **before** the current first (older → also re-exercises grouping/divider recompute).
  - **reset** → restore the seed array.
- Reassign the array (`arr = [...arr, m]` / `[m, ...arr]`) so Svelte reactivity + the stream's `$derived` fire. Bind `id="stream-scroll"`.
- CDP drives by **clicking the real sampler buttons** (honest user path) + setting `el.scrollTop` to force the scrolled-up state.

## 7. Skin

Pill appearance only in `ui/assets/skin.css` (`.jump-to-latest` — pill shape, accent-tinted is fine but must swap gold↔blue if accent is used, or keep accent-neutral). Structure (absolute position, bottom-right offset, z-index above rows) in the component `<style>`. Keep new tokens at zero if achievable.

## 8. CDP verification (D-097, sampler 9422 — real output, Rule 2)

Harness (now restored, D-105): kill stale `node`/`cargo`/`xgen-sampler` + free 5175/9422 → Joe launches `.\run-sampler.ps1 -Debug` (Chat runs short `.\cdp-debug.ps1 -App sampler -Mode eval -Expression "..."` probes; single-line evals only; iterate elements rather than quoting `[data-debug-id="…"]` selectors).

Must show (real output):
1. **Initial:** `stream-scroll` getter `atBottom:true` on load; viewport `scrollTop` at/near max (mounted scrolled to bottom).
2. **Append while at bottom (stick):** getter `atBottom:true` → click **append** → still `atBottom:true`, `scrollTop` advanced to new max (newest in view), no pill (`!atBottom` false → `.jump-to-latest` absent).
3. **Scroll up → append (no yank + pill):** set `el.scrollTop = 0` → getter `atBottom:false` + `.jump-to-latest` present → click **append** → `scrollTop` **unchanged** (no yank), `atBottom:false`, pill still shown.
4. **Jump pill:** click `.jump-to-latest` → `scrollTop` = max, `atBottom:true`, pill hides.
5. **Prepend invariance:** note the top visible message + `scrollTop` → click **prepend** → the previously-top message stays at the same visual offset; `scrollTop` increased by exactly the inserted block height (capture `scrollHeight` delta), `atBottom` unchanged.
6. **Grouping/divider recompute free:** after append/prepend, `groupedCount`/`dividerCount` reflect the mutated array (no regression).
7. **Both accents** (`--accent2` `#c28840` ↔ `#3a7ab0`); pill legible in both (or accent-neutral).
8. `vite build` clean (module count quoted); screenshot to `temp/`.

## 9. Definition of Done (Rule 7 — verify each with real output)

- [ ] `message-stream.svelte` scroll machine added (scroll listener + rAF-throttle + `atBottom` live + inline pill + `$effect.pre`/`tick` prepend-anchor + mount-to-bottom); `MessageDescriptor`/`grouping.ts`/`message.svelte` untouched.
- [ ] `atBottom` live and CDP-readable: `true` on mount, `false` when scrolled up, back to `true` within 80px (CDP-quoted transitions).
- [ ] Append sticks when at bottom (`scrollTop`→max) and does **not** yank when scrolled up (CDP-quoted `scrollTop` before/after).
- [ ] Jump-to-latest pill (inline chrome) shows ⇔ `!atBottom`; click → bottom + hide (CDP-quoted).
- [ ] Prepend preserves position: previously-top message unmoved, `scrollTop` += inserted height (CDP-quoted delta).
- [ ] Sampler `stream-scroll` live fixture + append/prepend/reset controls; four static fixtures untouched.
- [ ] Grouping/dividers recompute correctly after mutation (no regression; CDP-quoted counts).
- [ ] `vite build` clean (module count quoted) + both accents + screenshot (`temp/`).

*(DoD never includes "commit pushed" — the `Status: COMPLETED` header is the close signal.)*

## 10. Close (D-074 two-commit)

Clair feat commit first (component + sampler + skin), then Chat Claude doc-bridge (JOURNAL J-485 + registry `xgen-ui-components.md` if the count moves — the `stream-scroll` fixture adds a live stream subtree, so expect a registry delta + note it honestly + ROADMAP M-RP5.6 B ✅ / M-RP5.6 ✅ DONE + CLAUDE.md PLAY + this runbook → COMPLETED). Joe pushes both. **Closes M-RP5.6** (message-stream dd-composite, A+B) → message dd sub-family complete; next-active = M-RP6.1 client UI panel arc (R5 wrap + live-wiring).
