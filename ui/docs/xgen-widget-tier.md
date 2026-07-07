# XGen UI — The `widget` Tier
> **Status**: ACTIVE  
> Version: 1.2  
> Date: Jul 2026  
> **Last updated**: 2026-07-07  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Formal definition of the `widget` tier — the Level-2, behaviour-carrying, pluggable assembly unit. Promotes the N-059 concept-lock (name + placement + home + one-tier-+-Phase, Joe-locked J-445) to a checkable specification. Crystallises into **D-102**.

**Provisional status (D-065).** This is **v1.0, first-instance-provisional**. The constraint set is drawn against the six closed di composites (N-054, N-060–N-066), not against a built widget. The first buildable widget — `substitutions-editor` (M-RP4.3) — dogfoods the spec and may surface a constraint needing amendment, exactly as `tag-select` amended N-064. The spec firms once an instance proves it.

**FIRMED at v1.1 (M-RP4.3, 2026-07-04).** The first widget shipped and verified two-layer (pure → sampler; persistence → real shell, seam-only). No W-clause was wrong; two were *firmed* with real-world detail: **W-3** (a `common` widget cannot import a shell dep — shell I/O is host-injected via a callback, not a bare `invoke`) and **W-8** (the first-run-no-config caveat). The spec stands; these are firmings, not rewrites.

---

## 1. What a widget is

A **widget is a UI plugin** — the pluggable Level-2 tier. It is a droppable unit that owns its own state and lifecycle, may perform host I/O through a defined seam, and plugs into a host behind a contract rather than being baked in. Its pluggability is the point.

The component taxonomy (Level 1: di/dd × atomic/composite) is entirely **passive** — props in → DOM out + an inspection getter, no side effects (N-054). A widget is **active** — it owns state, decides (parse/validate/dirty-track), and integrates with the host. The widget tier exists because the passive taxonomy has no rung for a behaviour-carrying assembly, and wedging one into the grid would break the concept-purity of di/dd/atomic/composite.

### 1.1 Placement — a new *level*, not a new grid cell

```
Level 0   substrate            common/base  (logic / envelope / debug)
Level 1   components           core         (di/dd × atomic/composite; passive; ceiling = a di composite)
Level 2   widget               common/…/widgets  (assembled FROM Level 1; active; pluggable)
```

The widget sits a storey **above** the arity axis — atomic → composite → **widget** — not a third rung between atomic and composite. This keeps the Level-1 grid pure.

### 1.2 The discriminator — passive (composite) vs active (widget)

The classifying test, primary form (**W-2**):

> A widget owns state with a **transition-lifecycle that persists across renders** — state that represents *progress through a task* (draft→dirty→saving→saved, load→loaded→error, committed-vs-uncommitted). A passive composite's state is a pure function of props plus **at most a single momentary view toggle**.

Plain-language gloss: *remove it — do you lose a **behaviour** (widget) or just a **layout/arrangement** (composite)?*

The passive-state family that does **not** cross the line (illustration, from the closed composites): `open` (combobox N-063), `revealed` (password-field N-060), `hovered` (star-rating N-061), `dragging` (file-field N-062). These are single reflected view toggles. State beyond that family — a buffer that diverges from its source, a status that advances through a lifecycle — is a widget. (This settles the N-063 correction: "owns one UI flag" ≠ widget; a behaviour contract = widget.)

---

## 2. Inheritance from the plugin model, and the one divergence

A widget is conceptually **the same mechanism as a protocol/auth plugin**: a unit behind a declared contract, host does not hardcode its innards, swappable behind the interface. Taken straight from the plugin shape: contract-not-hardcoded, capability + Phase declaration, one aggregate getter, clean mount/unmount.

**The one divergence — channel.** A protocol/auth plugin is **invocation-shaped**: the host *invokes* it, it returns/acts, the lifecycle is request-scoped (call → result, one-shot). A widget's data connection is **binding-shaped**: it sits **continuously bound to live state**, re-rendering as that state changes.

Three points where they part:

1. **Channel** — plugin = invoke/return (imperative, one-shot); widget = a reactive `$common` store handle (standing subscription). *This is the adaptation.*
2. **Lifetime** — plugin runs per-call; widget's binding lives for its mount duration.
3. **Direction** — plugin is call→result; widget is read + optional write-back through the same store.

> **Widget = the plugin contract with the invocation channel replaced by a reactive store binding.** That single swap is the whole divergence.

---

## 3. The constraint set (checkable)

Each clause is phrased so a candidate widget is yes/no conformant.

- **W-1 — Composes down only.** Logic-bearing / value-holding elements come from `core`; substrate from `common/base`. Raw native tags are allowed for layout only, never to carry the widget's own logic. *(Test: a native `<input>`/`<select>` with behaviour attached outside a core component = fail.)*
- **W-2 — Owns state + lifecycle.** Holds task-state with transitions that persist across renders (§1.2), beyond a single momentary view toggle. This clause **is** the discriminator.
- **W-3 — I/O only via declared seams.** All host interaction goes through Tauri `invoke` commands + `$common` stores. No `fs`, no bespoke IPC, no direct `fetch` in the widget body. **A widget in `common` never imports a shell dep (`@tauri-apps/api`) — it build-fails outside a Tauri host (M-RP4.3 finding). Shell I/O is *host-injected* (a callback prop the shell backs with `invoke`); the widget calls the callback, never `invoke` directly.** This is the imperative-one-shot form of the seam; the live in-app effect stays store-mediated.
- **W-4 — One aggregate getter.** Self-registers exactly one debug getter (N-054 model); child `core` components self-register under `<id>__<slot>`. The getter **publishes observable task-state** (`{dirty, valid, phase, …}` — the state that lets CDP drive the machine) and **never publishes payload or secrets** (the N-060 `hasValue` precedent).
- **W-5 — Clean mount/unmount.** A droppable unit: wires listeners / observers / store subscriptions on mount, tears them **all** down on unmount (the 0-orphans discipline extends to widget lifecycle). No cross-widget shared mutable state except through explicit `$common` stores.
- **W-6 — Skin L2 only; pure/effect separable.** Zero component `<style>` — all appearance in `skin.css` (N-066). Authored so the **presentational layer** (composed components + layout + validation display + the state machine) runs with **I/O stubbed** — this is what makes the two-home verify (§5) possible.
- **W-7 — Scoped home + a Phase.** Lives in `ui/common`. Declares a Phase (A pure Svelte / B +Tauri / C all three, N-028); the declaration matches the code (a Phase-A widget has no `invoke`).
- **W-8 — Surfaces honest phase-limits.** A partial capability (e.g. session-only write-back under D-101) is visibly flagged, not silently absent (D-065). **First-run caveat (M-RP4.3): on a genuinely fresh machine (no config yet) the strict write-back errors and the widget's `try/catch` swallows it — the in-app effect still applies, but the durable write silently no-ops until a config exists. Only bites once a widget is mounted in a real shell; surface it, don't rediscover it.**
- **W-9 — Representation.** A widget is an ordinary Svelte component (no `<component>` element exists) — a `.svelte` file in `ui/common/lib/components/widgets/`, marked Level-2 via `envelope` (optional tier arg / reflected `data-tier="widget"`) so `ids()` and the sampler's WIDGET tab partition it from composites. **Connection v1 = static import + placement** (`import`, `<WidgetName …/>`). A **widget registry + dynamic mount** (declarative `widget-id → component` placement — the true plugin-discovery layer) is **reserved**, triggered when dd-components give it a first consumer (D-065/D-069).
- **W-10 — Plugin contract.** The explicit seam a host relies on: mount lifecycle · one aggregate getter · store-mediated I/O · declared Phase + capability. A widget is swappable behind this contract exactly as a protocol plugin is.
- **W-11 — dd-socket.** A widget MAY expose typed **dd-slots**. Each slot = a `$common` **store handle** (read + optional write-back) + a **named mount point**. The slot is **source-agnostic** (N-057): the widget owns the store, a dd-component binds to it, and *who backs the store* (protocol vs literal) is the shell's job. **The dd-component binds to the store, never to widget internals.** This is the socket defined ahead of any dd-component so that when one is built it plugs in with zero widget rework.
- **W-12 — a widget owns exactly one region.** A widget MAY own a dockable **region** (a named surface in the layout descriptor). W-11 is the *data* seam; W-12 is its *layout* sibling. Every region-owning widget maps to exactly one surface. See `xgen-region-dock-model.md` (D-103).
- **W-13 — `system` widgets are non-removable.** Widgets carry a `kind`: `system` (built-in regions R1–R8: pre-installed, always in the default layout, configurable + redockable but never fully closed) or `custom` (install/remove; MAY also provide a region). Prevents a user closing the Composer with no way back.

> **Reframe (v1.2, D-103).** Every UI **region is a widget** — the client panel is a layout of dockable widgets (`system` R1–R8 + `custom`). The di/dd grid stays the **content** tier; **widgets are the dockable surfaces that host content**, so a custom widget can contribute a new dockable region. Renderers: config-grid (A) now → owned dock engine (B) at M-RP7, both reading one serializable layout descriptor. Full model in `xgen-region-dock-model.md`.

---

## 4. The I/O seam (W-3 / W-10 detail)

Default is **store-mediated**: the widget only touches `$common` stores; the store is backed by `invoke` in the real shell and by a literal in the sampler — the exact substitutions precedent (N-057/N-058). The widget contains no I/O in the common case, so "pure layer" = "the whole widget minus the store backing."

Two reserved fallbacks:

- **Callback / prop injection** — for a genuinely **imperative** one-shot action (a `save()` that does not fit a reactive store): the widget takes the command as a prop; the shell passes an `invoke`-backed one, the sampler passes a stub.
- **DEV hook** — for a **pure-compute core** (validate / parse): exposed on a DEV global (`__XGEN_…__`, the N-056 processor precedent) for sampler-side assertion.

Store-mediated first; callback for imperative actions; DEV-hook for pure compute. No new mechanism is invented — all three reuse existing precedents.

---

## 5. Verify home — the two-layer model

A widget's defining trait (host I/O + integration) is the sampler's **declared blind spot** (D-097 cedes integration + host-real behaviour to the real shells). So verification splits by layer:

**Pure / presentational layer** — composed components + layout + skin + validation + the state machine, **I/O stubbed**. Verified **in the sampler** (CDP 9422), a fifth **WIDGET** tab (mounted-not-`{#if}`, N-053 invariant — the registry stays complete). *Done when:* registry entry present; state-machine transitions CDP-asserted with I/O stubbed (e.g. set draft → `dirty:true`; fix invalid → `valid:true`); skin in cascade; both accents; 0 orphans. *(The component-DoD, extended.)*

**Effect layer** — the real host I/O: config read/write, command round-trip, session-vs-persistent behaviour. Verified **in the real shell** (client/node, CDP 9222/9322 — D-097's home for host-real + two-shells-together). *Done when:* the real `invoke` chain runs end-to-end incl. write-back; the honest phase-limit is demonstrated (e.g. session-only write-back survives within-session, resets on relaunch under D-101); real output quoted (Rule 2). *(This is the M-RP4.4 chain shape, but in the real shell — not the sampler host.)*

**One milestone, two verify homes.** The effect-layer DoD folds into the widget's build milestone as a distinct section. A widget is not "done" until **both** layers are green: the sampler-DoD stays standing for the pure layer; the dual-shell DoD returns specifically for the effect layer.

---

## 6. First widgets

| widget | status | why |
|---|---|---|
| `substitutions-editor` (M-RP4.3) | first **buildable** | in-app `[substitutions]` TOML editor + write-back. Composes core-di only (textarea / textfield / button) + host I/O; **no dd dependency**. Phase-B, session-only write-back under D-101. Dogfoods + firms this spec. |
| `temperature-indicator` | first **conceived**, **dd-blocked** | conceptually defined, but has nothing to plug into until a dd-component exists. Waits behind dd-components; will bind its `temperature` state through a **W-11 dd-socket**. |

The spec deliberately defines the **frame + connection generically** (proven by the dd-free `substitutions-editor`) so it is instance-independent; `temperature-indicator` validates the dd-socket later without frame rework.

---

## 7. Roadmap consequence

Widget-tier spec (this doc) → **M-RP4.3** (`substitutions-editor`, first widget) → **M-RP4.1** (kind-3 number-clamp) → kind-2 converter field → kind-4 `use:render` (deferred) → **dd-components** (unblocks `temperature-indicator` + the W-9 registry/dynamic-mount layer).

---

*UI-architecture spec. No protocol/data implication. Promotes N-059 (concept-lock, J-445) → D-102. v1.0 first-instance-provisional — firms once M-RP4.3 proves it.*
