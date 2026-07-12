# XGen — Plugin Taxonomy Phase-0 (and the module-UI sandbox boundary)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-12  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

The D-071 Phase-0 for the plugin taxonomy — filed at `ui/docs/xgen-region-dock-model.md` §11 and `docs/xgen-widget-surfaces-phase0.md` §9. It has **two required outputs**: the **taxonomy** (§4–§9) and the **module-UI sandbox boundary** (§10), and they must lock together — *you cannot classify a thing while leaving open what the thing you are classifying is allowed to do.*

**Gates:** M-RP6.1l (the plugin list must list every species) · surfaces §6 item ① (Settings' own surface) · M-RP7.4 (custom-widget-contributed regions).

**Status of the decisions below.** Joe delegated this walk (2026-07-12: *"you have autonomy in this part; do as you propose"*). The decisions in §4–§10 are therefore **taken**, not merely proposed — but each carries its reasoning in the open, and **any one of them reverses on one word.** Canonical records (DECISIONS.md, Ch6, widget-tier, region-dock, surfaces) do **not** move until Joe reads this doc and says so (D-069).

---

## 0. Vocabulary

**Not restated here.** The canonical table is `ui/docs/xgen-region-dock-model.md` §0 (N-100, LOCKED): **tile** · **region** · **face** · **window** · **slot**. This document uses those words exactly as defined there and adds **no** new placement word.

---

## 1. Why this document exists

Two specs disagree about what a "widget" is and where it is placed.

- **Ch6 §6.8.3** (D-036, written April 2026) — a **widget** is an HTML file in an **isolated webview**, talking to its module backend over a **local WebSocket**, placed by a **named slot** from a fixed inventory (`room.sidebar.bottom`, `global.statusbar`, …), shipped as a **package + manifest** by a third party.
- **D-102 / D-103** (written July 2026) — a **widget** is a **Svelte component**, in-process, fed by a **`$common` store**, placed by a **dockable region** in the layout descriptor, written by us in the client tree.

`self-panel` and `inspector-panel` are the latter: no webview, no socket, no slot, no manifest.

**This is a D-067 drift surface sitting in the SPECS, not in the code.** Joe's reconciliation frame is the right one and is not re-litigated here: ***"module and widget is the plugin in the two areas: system and ui"*** — **one plugin, one list, several UI forms.** The work is **alignment**, not choosing a winner.

---

## 2. Grounding — what the code actually says (2026-07-12)

**Grepped before proposing.** Three shipped Rust artefacts that neither §11 nor surfaces §9 knew about.

### 2.1 `xgen-common/src/module.rs` — the plugin spine ALREADY EXISTS

Shipped under the Storage-Engine / Plugin-Framework milestone (**SE-D2**). Its own module doc comment carries **Joe's frame verbatim, in code, months before this session**:

> *"There is one unified handshake mechanism; the code term **`kind`** carries the system/ui distinction: a **module** is a *system* plugin (`host = node`), a **plugin** is a *ui* plugin (`host = client`)."*

It also ships:

- **Identity.** `ModuleKindId` (the **slot** GUID — generated once per slot, copied by every implementer) + `ModuleImplId` (the **implementation** GUID). **UUIDv4, never `Xgid`** — *"an `Xgid` is protocol-assigned and federates; a module GUID is local, developer-assigned, and never crosses the wire."*
- **Trust posture.** *"The descriptor is a **const in the plugin's own code** — there is **no manifest file**, because a compile-time framework has no host↔plugin gap to bridge. Metadata is **authoritative**, location is **never trusted**, and an unknown descriptor is **rejected loudly** at the registration site. The GUID/descriptor handshake verifies **declaration honesty**, never **behaviour**."*

> **⚠️ Honest limit, stated so nobody later over-reads this section.** The shipped `Descriptor` struct is `{ kind_id, impl_id, name, assurance }`. There is **no `host` field, no `kind` field, no `ui_form` field**, and `assurance: AssuranceClass` is **storage-engine-specific** (the tier→engine gate, SE-D4). **The vocabulary is shipped; the fields are not.** This taxonomy therefore *extends* a real spine — it does not describe fields that already exist.

### 2.2 `xgen-core/src/auth/module_registry.rs` — a THIRD species, in neither list

An **Auth Module** is a **protocol principal**: `AuthModuleXgid` (the seventh XGID flavour, AMR-D2) + an **`endpoint_url`** + a trusted/revoked registry on the Node (`revoked` is block-only, A2-D1: retained, marked untrusted, never removed).

**It is a remote service.** It is not a compiled plugin and it is not a UI widget.

> **⚠️ Ch6 §6.8.7 says the Auth Module is *"the reference implementation of a Window-form module"* — a package in `modules/` with a manifest. The shipped code says otherwise.** The correction is §8.3.

### 2.3 `xgen-node/src/plugins/temperature.rs` — in-process trait, loader deliberately open

A Rust `trait TemperaturePlugin` + `NoOpTemperaturePlugin`. Its header states the loading mechanism is *not* decided: *"config-driven loading, dynamic libraries, WASM, external process… a future Phase 2 implementation decision."*

### 2.4 The negative grounding

- **No `xgen-module.json` exists anywhere in the tree** (searched, whole repo).
- **No manifest loader. No `modules/` directory scan. No local WebSocket server in the client.**
- **No HTTP client in any crate** (established J-506).

---

## 3. 🔑 The finding — the drift is not "Ch6 vs D-102"

> ### It is **Ch6 §6.8 versus everything that was actually built.**

Ch6 §6.8 was written in **Session 2 (April 2026)** — before the plugin spine (`module.rs`), before the Auth Module registry, before the widget tier (D-102), before the region model (D-103), before `WidgetMount`. Every later artefact converged on a *different* shape, and none of them consulted it.

**This is the J-502 "first bird" shape a second time:** a chapter section named a thing before every convention it would live among existed, and the specs have been quietly disagreeing with it ever since. **The deferral is why nothing hardened wrong.** Nothing shipped is broken; **§6.8 is the outlier and it is §6.8 that moves.**

---

## 4. ✅ The model — one plugin, three orthogonal axes

A **plugin** is one thing: a unit with a **descriptor** (`kind_id` = the slot it fills, `impl_id` = which implementation it is) that a host **registers**. It is described by three axes.

| axis | values | notes |
|---|---|---|
| **host** | `node` (the **system** area) · `client` (the **ui** area) | Joe's frame; already the words in `module.rs`. A package MAY have both halves. |
| **delivery** | `compiled` · `service` · `packaged` | **← the axis nobody had. This is where trust lives.** See §10. |
| **surface** *(client only)* | `none` (headless) · `region` · `shelf` · `window` | surfaces §3.3, **unchanged**. At most one (W-12). |

**"Module" and "widget" are not two things.** They are **`host = node`** and **`host = client`** on one plugin — exactly as `module.rs` already says. `self-panel` is a plugin with `host=client, delivery=compiled, surface=region, kind=system`.

### 4.1 The delivery axis, grounded

| delivery | what it is | shipped instance | trust floor |
|---|---|---|---|
| **`compiled`** | const `Descriptor` in the implementation's own code, linked into our binary, in-process | node storage-engine slot · `TemperaturePlugin` · `self-panel` · `inspector-panel` · `substitutions-editor` · `entity-context-menu` | **source review + build.** No sandbox is possible *or meaningful* — it **is** our binary. |
| **`service`** | its own process, its own **XGID**, reached at an **endpoint**, speaks protocol Events | **Auth Module** (`AuthModuleRegistry`) | **structural**: never in our address space, never touches our DOM. Trust = the registry + revocation + the tier gate. **Already shipped.** |
| **`packaged`** | third-party code + assets + a **manifest**, installed into `modules/` | **NONE. Zero lines exist.** | **⚠️ NONE. This is the entire open trust surface of the project.** |

> ### 🔑 The reframe that makes §10 tractable.
> Ch6 §6.8.8 has asked since April: *"Widget sandboxing — what CSP and iframe sandboxing apply?"*
>
> But **`self-panel` needs no CSP, and a compiled Rust storage engine needs no CSP.** **Nothing about *being a widget* is dangerous. Being `delivery: packaged` is.**
>
> **The question was attached to the wrong noun for three months.** The boundary is the **delivery** axis, and it applies identically to a `packaged` module's **window** as to its **widget** — which is why §10 is not a widget rule.

---

## 5. ✅ Placement vs containment — the slot inventory does NOT retire, and it is NOT a rival

The apparent "two placement models" collapse once surfaces §3.2 is applied to Ch6's own slot list.

> **surfaces §3.2 (shipped clause): content rendered *inside* another widget is NOT a surface — it is content.**

Split Ch6 §6.8.3's slot inventory against that line:

| Ch6 slot | what it actually is |
|---|---|
| `node.dashboard.widget` | **a region** — a tile in a layout descriptor. *(Placement.)* |
| `room.sidebar.top` / `room.sidebar.bottom` | **regions** — tiles, in the language the descriptor already speaks. |
| `room.toolbar` | **a content anchor** inside R6 (the composer). |
| `room.message.decorator` | **a content anchor** inside a `message`. |
| `space.header` | **a content anchor** inside R4 (the room/space header). |
| `global.statusbar` | **a content anchor** inside the `status-bar` (frame chrome, outside the descriptor by D-107). |

> ### 🔑 And the containment mechanism is ALREADY SHIPPED.
> `message.svelte` takes **`details: WidgetMount[]`** and **`bodyExtras`**, resolves them against a prop-injected `widgetId → Component` registry, and **drops unresolvable ids** (W-13, M-RP5.5 / J-478). Renderer A reuses that exact shape (J-499).
>
> ***`room.message.decorator` is `message.details` under another name.*** Nobody noticed, because one was written in April and the other in July.

**→ The resolution, and it costs no new mechanism:**

- **ONE placement model** — the D-103 `Layout` descriptor. A plugin that *is drawn as a place* takes a **surface** (`region` / `shelf` / `window`).
- **ONE containment model** — **host-declared mount points**. A host widget declares named `WidgetMount[]` sockets; a plugin fills one. **This spends no surface** (§3.2) and it is not a placement.
- **Ch6's slot table is therefore not a rival placement model. It is a stale, guessed inventory of mount points** — written against a Room view that does not exist in the shipped UI. **It is regenerated from the widgets that actually exist**, at the milestone that needs it (M-RP7.4), not copied forward.

**A slot is declared by the HOST, not by the guest.** That is the anti-drift property: a mount point exists because a shipped widget offers it, never because a manifest asked for one.

---

## 6. ✅ The four questions from surfaces §9, answered

**1. Is a D-102 widget a module? Does a module contribute one?**
A D-102 widget **is a plugin** with `host = client`. "Module" was only ever the `host = node` word (`module.rs`). **One list.** A package MAY ship both halves (a node module *and* a client plugin) — that pair is what Ch6 §6.8.3 was reaching for with *"a single module package may declare more than one UI form."*

**2. Do the slot inventory and the descriptor unify, coexist, or does one retire?**
**Neither unifies nor retires: they are different mechanisms** — *placement* (surface, descriptor) vs *containment* (host-declared mount). §5. The apparent collision was two words for the tile, plus one guessed list.

**3. Does the surface vocabulary need a `screen` kind?**
**No.** §9.

**4. What does the plugin list render for a built-in with no package, no manifest, no socket?**
**Ch6 §6.8.5 already drew it: the `[system]` / `[user]` mode badge.** That is **W-13 pre-figured**. `self-panel` / `inspector-panel` list as `[system]` — configurable + redockable, **Remove disabled**. No new mechanism.

---

## 7. ✅ Consequences for the plugin list (M-RP6.1l)

The list is **one pane, two entry points** (Joe, J-502) and it lists **every plugin, of every species** (Ch6 §6.8.5's *universal registry* principle survives intact).

Each row renders from the three axes:

- **`host`** → a section or a badge (**system** / **ui**). *Joe's two areas, made visible.*
- **`delivery`** → the **trust badge**, and it is the load-bearing one: **`built-in`** (compiled) · **`service`** (an XGID + an endpoint — this row shows *revoke*, not *remove*) · **`installed`** (packaged — the only row that can ever carry an untrusted author).
- **`surface`** → where it lives (and whether a *Launch* button exists — Ch6 §6.8.5's `window` rule, unchanged).

**Remove/Disable semantics fall out of the axes rather than being special-cased:** `kind: system` → no Remove (W-13) · `service` → **revoke** (block-only, the shipped `revoked` flag) · `packaged` → Remove, with confirmation for `identity_mode: user` (Ch6 §6.8.5, unchanged).

---

## 8. ✅ The manifest, reconciled — and the two corrections it forces

### 8.1 A manifest is not a descriptor

The shipped spine says: *"there is **no manifest file** … metadata is **authoritative**, location is **never trusted**."* Ch6 §6.8.2 specifies `xgen-module.json`. **Both survive, because they are different objects:**

- **A compiled `Descriptor` is AUTHORITATIVE.** It is our code; an unknown one is rejected loudly at the registration site.
- **A `packaged` manifest is UNTRUSTED INPUT.** It **declares intent**; the **host enforces**. It is a request, not a fact.

**They must never be merged into one type.** A manifest that is read as authoritative is a third party writing our registry.

### 8.2 Module signing (Ch6 §6.8.8, open since April) resolves on the delivery axis

**Mandatory for `packaged`. Meaningless for `compiled` (it is our build). Already solved for `service`** (an Auth Module *is* its key — `module_id.pubkey()`, AMR-D3 derive-don't-store).

*Two of Ch6 §6.8.8's five open questions dissolve the moment the delivery axis exists. That is the sign the axis is real.*

### 8.3 ⚠️ Ch6 §6.8.7 is factually wrong against the shipped code

Ch6 §6.8.7: *"The XGen Auth Module … is the reference implementation of a **Window-form module** … uses the same manifest format and the same module list entry as any third-party module."*

**It does not.** The shipped Auth Module is `delivery: service` — an `AuthModuleXgid` principal at an `endpoint_url`, in a Node-side trust registry with revocation. **It has no manifest, no package, no `modules/` folder, and no webview.** A *different* thing (an operator-facing verification-flow UI) may later be a `window` plugin **on top of** it, but the module itself is a network service.

**Corrected in the Ch6 rewrite (§12), not left standing.**

---

## 9. ✅ Settings' own surface — `window`, and NO `screen` kind (surfaces §6 item ①)

**Decision: Settings is a plugin with `surface = window`. The `screen` kind is NOT added.**

- The `window` form **already exists in the model** (surfaces §3.3, Ch6 §6.8.3) and has **another customer** waiting (a `packaged` plugin's Launch button). **One mechanism, two consumers** — the D-102/N-086 rule: do not add an abstraction with one user.
- Ch6 §6.8.5's phrase *"a screen of its own"* is **prose, not a surface kind**. Adding a fifth kind to satisfy a phrase is exactly the re-derivation this Phase-0 exists to stop.
- A grid **tile** is rejected: Settings is a *task you enter and leave*, not a panel you arrange around. Putting it in the descriptor would let a user dock the plugin manager into a 200px column — and then remove the widget that removes widgets.

> **⚠️ The one thing this forecloses, recorded so it is not a surprise:** the **Discord full-window overlay** shape (settings covering the whole client, chrome included, with an ✕). That is **not** `window` — it is a fifth surface kind, and it would be `screen`.
>
> **If Joe wants the Discord shape, this decision reverses and `screen` costs exactly one word in the surface enum.** It is a **product choice**, not a structural one, and it should be a lock — never a drift. *(The plugin list itself is unaffected either way: it is **one pane with two entry points** whichever surface hosts it — J-502, unchanged.)*

**→ surfaces §6 item ① is now CLOSED. Only ④ (top-shelf pinning) remains open.**

---

## 10. 🔒 THE SANDBOX BOUNDARY — the largest open trust surface in the project

**Ch6 §6.8.8 has carried *"Widget sandboxing: what CSP and iframe sandboxing apply?"* since Session 2 (April 2026), untouched.** It is answered here.

### 10.1 What makes it the largest surface

Every other content channel in XGen has a **structural** foreclosure — not a filter, a **foreclosure**:

- **Blobs** are content-addressed. **A hash cannot name a host** (D-111). The beacon is not blocked; it is **unsayable**.
- **Space themes** are a colour-only allowlist applied through CSSOM. **A colour cannot be a `url()`** (D-110).
- **Glyphs** are banned from Space override (D-110). **Fonts are bundled** (Ch6 §6.2).

**A `packaged` module UI has NONE OF THESE.** It is arbitrary third-party markup and script **with a network stack**. It is not "one more item on the list" — **it is the only one with no floor under it.**

### 10.2 The floor — and it is built the way this project always builds one: **foreclose, don't filter**

> ### **S-1 — A packaged module UI is a webview with NO NETWORK.**
> Its **only** egress is the local IPC channel to its **own backend**, which runs on the Node/Client **we** ship.
> CSP: `default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self' data:; font-src 'self'; connect-src <its own local channel>; frame-ancestors 'none'; form-action 'none'`.
> **No `http:` or `https:` scheme is reachable at all.**
>
> **This makes D-111's beacon UNSAYABLE inside a module widget — exactly as it is unsayable in a message.** *The same property, taken twice.*

- **S-2 — Assets are PACKAGED, never fetched.** Everything the UI renders ships inside the package. *The bundled-fonts rule, generalised. A packaged asset cannot phone home.*
- **S-3 — Isolation.** Each module UI is its **own webview, its own origin**. No `allow-same-origin` with the host. **No Tauri IPC exposure** (no `withGlobalTauri`, no `invoke` in its context). No access to the host DOM, the host stylesheet, or another module's webview.
- **S-4 — The module never holds the key.** Ch6 §6.8.4's `identity_mode: user` is **consent, not key handover**: the module **requests**, and **our Rust signs, per event**, against the capabilities it declared. *A module that can sign as you at will is not a module — it is you.*
- **S-5 — A module UI may not draw trust chrome.** The **D-110 lesson, generalised beyond CSS.** It renders inside a **bounded, attributed frame**, and it **may not occupy the identity / verified / AI-badge zone** (Ch6 §6.13). *Icon spoofing does not become acceptable because the spoofer arrived as a package instead of a theme.* **Corollary:** because S-3 gives it a separate origin, it never sees our glyph tokens — the foreclosure is structural here too.
- **S-6 — Capabilities are declared and enforced host-side, DENY-BY-DEFAULT.** **Allowlist, never denylist** (D-110's rule). A manifest **declares**; the host **enforces** (§8.1). This is also the answer to Ch6 §6.8.8's *"module permissions"* question.
- **S-7 — The sequencing lock.** **No `delivery: packaged` plugin may load until S-1…S-6 ship.** Until then the plugin list contains **`compiled` + `service` species only** — which is *exactly what exists today*, so the lock costs nothing and forecloses everything.

### 10.3 Why lock it now, with no code to lock it against

**Because that is the cheapest moment a trust boundary can ever be set** — D-110's timing win, taken a second time, deliberately.

`state.space_theme` was locked before a line of it existed. `delivery: packaged` is in the same position **today**: zero lines, no manifest, no loader, no webview. **The first packaged module that ever loads will load into a floor that already exists.** *(D-071 paying out in advance rather than in arrears — again.)*

### 10.4 What is NOT decided here, and must not be smuggled in

- **The `compiled` plugin LOADING mechanism** — `temperature.rs` itself says it is open (*"dynamic libraries, WASM, external process"*). **A dynamic library is not a sandbox** — if the loader ever becomes `dlopen`, that plugin is `compiled`-trust with **none** of `compiled`'s review, and this taxonomy would be quietly lying. **Filed, not solved: whichever loader is chosen, it must land on the delivery axis, and if it admits third-party code it inherits §10.2.**
- The **backend** half of a packaged module (Ch6 §6.8.1's *any language + WebSocket + `meta_atts`*) is **not** re-opened here. It is a **protocol participant** — the same posture as any other Node client — and its risk is the Node's authorisation model, not the client's DOM. **The dangerous half is the UI half.**

---

## 11. Open items this Phase-0 does NOT close

- **surfaces §6 item ④ — top-shelf pinning.** Untouched. The top shelf still mounts **empty** (no dead controls, D-065).
- **surfaces §6 item ⑤ — glyph licence provenance.** **⚠️ VERIFIED, NOT ASSUMED (2026-07-12): D-108 has DISSOLVED it as a design question.** `docs/xgen-icon-adoption.md` §3f: *"Licence + source live in `icons.manifest.json`, per glyph. **A glyph with no licence entry fails the build.**"* **No audit can forget what the compiler enforces.** What remains is **mechanically sourcing gear / diskette / load** — a task, not a decision. *(Item ⑤ should be struck from §6's open list on lock.)*
- **M-RP6.6 client resident**, the **read-marker protocol gap**, `temperature-indicator` ⏸️ — untouched, and **none of them may be smuggled into a UI milestone.**

---

## 12. Records to change ON LOCK (not before — D-069)

| record | change |
|---|---|
| `DECISIONS.md` | **+D-112** the plugin taxonomy (host · delivery · surface; one plugin, one list) · **+D-113** the packaged-module-UI sandbox floor (S-1…S-7). *Appended at the bottom — D-099…D-111 already are; do not re-sort a 4,000-line record.* |
| `docs/xgen_ch6_client_design.md` | **v0.5 → v0.6.** §6.8.3 re-expressed as **surface + anchor** · the slot table marked **STALE, regenerated at M-RP7.4** · **§6.8.7 corrected** (the Auth Module is `service`, not a Window-form package) · **§6.8.8 CLOSED** (widget sandboxing → D-113; module permissions → S-6; module signing → §8.2) |
| `ui/docs/xgen-widget-tier.md` | **v1.2 → v1.3.** The **delivery axis** added; W-12 amended per surfaces §3.3; a widget is restated as *a plugin with `host = client`*. |
| `ui/docs/xgen-region-dock-model.md` | **v1.6 → v1.7.** **§11 CLOSED** (it is the doc that filed the gap). |
| `docs/xgen-widget-surfaces-phase0.md` | **v1.3 → v1.4.** **§9 CLOSED** · **§6 item ① CLOSED** (`window`) · **item ⑤ STRUCK** (dissolved by D-108). Only **④** remains → **M-RP6.1i–l UNGATED.** |
| `docs/ROADMAP.md` | version bump; the taxonomy Phase-0 → ✅; M-RP6.1i–l ungated. |
| `JOURNAL.md` | J-507. |

---

## 13. Open for Joe — five reversals, one word each

1. **The three axes** (host · delivery · surface). *If this is wrong, everything below it is wrong.*
2. **Slots survive as CONTENT ANCHORS, not a second placement model** — and `message.details` is the shipped proof that we already built one.
3. **Settings = `window`; no `screen` kind** — *unless* you want the Discord full-overlay shape, in which case `screen` is a fifth surface kind (§9).
4. **S-1 — no network in a packaged module UI.** **The load-bearing one.** Everything else in §10 follows from it.
5. **S-7 — packaged plugins are structurally unloadable until the floor ships.** *Costs nothing today; forecloses everything later.*

---

*Plugin taxonomy Phase-0. Design only — no code, no canonical record moved. Crystallises into D-112 (taxonomy) + D-113 (packaged-UI sandbox) on Joe's lock.*
