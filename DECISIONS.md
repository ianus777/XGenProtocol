# XGen Protocol — Implementation Decisions
> **Status:** ACTIVE  
> **Last updated:** 2026-08-01  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  

Every decision that goes beyond spec prescription is recorded here before advancing to the next layer.
Format: title, date, layer, spec reference, decision narrative.

---

## D-098 — Sampler runtime = full Tauri/WebView2 sibling (minimal host), not a Vite-only page

**Date:** 2026-06-27 · **Layer:** UI infra (M-RP3.0) · **Ref:** N-044, D-095, D-097

The component sampler runs in **WebView2** via its own **minimal** Tauri host (`xgen-sampler/`), identical runtime to the real shells, driven by the same CDP self-drive harness. The host crate pulls **only** `tauri` + `tauri-build` — **no** protocol crates (it does NOT mirror `xgen-client`, which carries `xgen-common`/`xgen-core`/tokio/websockets/crypto/CLI with Tauri bolted on); `src/main.rs` is the bare `tauri::Builder::default().run(generate_context!())`. Ports: Vite **5175** / CDP **9422** (client 5173/9222, node 5174/9322).

**Why not Vite-in-Chrome (the lighter option):** the whole skin rests on the single-engine WebView2/Chromium assumption (vendor-prefixed pseudo-elements, `color-scheme`, the hover-only spinner paint). Tuning in a *different* engine, on a *divergent* toolchain, reintroduces exactly the false confidence the sampler exists to remove. The minimal-host cost (one boilerplate crate, tauri deps already compiled in the shared target dir) is paid once and keeps one runtime + one harness.

**Mirror-exemption:** the sampler is **D-095-mirror-exempt** — it does not mirror the client/node source-tree shape, and it deliberately diverges on skin *load mechanism* (the real apps bundle `skin.css` via Vite; the sampler edits the **canonical** `ui/assets/skin.css` live via Vite HMR in dev — the killer feature: tune in the sampler, the change *is* the shipping skin). A standalone-exe live-reload (fs-plugin + refresh button) is a future follow-on, unneeded while dev HMR covers the intent.

---

## D-097 — Test-bed split: build/tune components in the sampler; run the two real apps together for interaction/integration

**Date:** 2026-06-27 · **Layer:** UI process (M-RP3.0) · **Ref:** N-044, D-098

The component track is **paused** (the remaining atomic di — `date`/`color`/`file`/`select multiple` — is resumable, not cancelled) while the sampler is built. Henceforth the test-bed is split three ways:

- **Component appearance / state / per-shell theming → the sampler.** The runtime client↔node skin-swap covers gold-vs-blue, **fully replacing** the prior practice of wiring throwaway demos into both real shells. From `date` onward, components are built, tuned, and CDP-verified in the sampler.
- **A component inside a real composed feature → the real app, at integration.**
- **The two shells running *with each other* (federation/protocol plane, handshakes, the MP-R scenarios) → both real apps run together.** This is the sampler's **structural blind spot** — one window, one runtime — so it stays the job of the real apps at interaction/integration milestones (MP-R3, Tier-1 auth rebuild, streams).

Consequence: the real shells are **frozen as-is** (no revert, no new component wiring) and are **not run for component reasons**. The revert of the demo blocks from the real shells is deferred to a future cleanup (not a prerequisite for the sampler).

**Closing note (M-RP2.16, J-425):** the sampler-DoD is now a **standing rule** — a component milestone is not done until its sampler row + applicable-state cells are added and CDP-verified in the sampler. This *replaces* the dual-shell demo-wiring step entirely (baked into every component runbook's DoD from `date` onward; first exercised by `date` / M-RP2.16, N-046).

---

## D-096 — `textfield` `type` folds the string-input family into one component (reverses N-029)

**Date**: 2026-06-25
**Layer**: UI / component model (reference implementation, not protocol).
**Spec reference**: N-038 scoping (di→processor→dd track, atomic/shape/composite line); reverses N-029 ("type is fixed, separate semantics"). Verified M-RP2.12 (J-417); recorded via N-039 (which also carries the per-type icon treatment — a skin concern, not part of this decision).

### Decision

The atomic discriminator for an input component is **root structure + value-type, not the `type` literal**. The string-valued `<input>` types that share the `<input>` root, a string `bind:value`, and the `.textfield` skin fold into the one `textfield` component via a constrained `type` prop:

```
type: 'text' | 'search' | 'email' | 'url' | 'tel' | 'password'   // default 'text'
```

They differ only in UA-supplied validation / soft-keyboard / masking — not in root tag, value type, or skin. Enforcement is the **TS union alone** (no runtime guard, no DEV-warn): the consumer is a TS codebase, and an out-of-whitelist value degrades safely (the browser normalizes an unknown `type` to `text`), so a guard would be empty machinery (D-065). The getter carries `type` (`{ type, value }`) so the configured type is verifiable through the N-024 registry.

**Excluded — own atomics** (value-type or structure differs): `number`/`range` (numeric `bind:value`), `date`/`color`/`file` (structured value / native chrome). **Excluded — composites** (custom interactive chrome on top of the field): the `password-field` reveal toggle, a custom stepper. Neither folds into `textfield`.

### Why

One file and one skin for a family that is structurally one control; the prop is a native attribute passthrough that degrades safely. N-029 fixed `type` early — before the di catalogue had a principled atomic/shape/composite boundary. N-038 supplied that boundary, so the reversal is now grounded rather than ad hoc: the line is drawn at root-structure + value-type, and `type` (within the string-input family) sits below it.

### Amendment (2026-06-27, M-RP2.15 / N-042) — the criterion is sharpened: root + value-type + shared skin/surface

`range` (M-RP2.15) is the case that tests sufficiency. It shares both halves of the criterion as originally written — root `<input>` AND value-type `number` (same as `number`) — so the *literal* criterion would fold it into `number`. It must not: `range` diverges on **skin** (track/thumb `::-webkit-slider-*` pseudo-elements — zero shared appearance with `number`'s text box + spinner), **prop surface** (no `placeholder`, no live `:invalid` — the thumb is clamped, no `readonly` — native no-op; bounds are the *defining* attribute), and **interaction/empty model** (clamped drag, always-valued vs `number`'s empty=`null`). Folding would put two disjoint skins behind one class and a prop that swaps the whole rendering — the polymorphic-contract problem this decision exists to prevent, on the *appearance* axis instead of the value axis.

So the fold criterion is **necessary but not sufficient as written**; the sharpened test is **root + value-type + shared skin/surface** (genuine interchangeability — the thing that made the string-input fold good: one skin, one prop surface, a thin `type` switch). `range` stays its own atomic. This refines the criterion in place; it does not reopen the `textfield` fold (the string-input family still passes the sharpened test).

---

## D-095 — UI source-tree tiers mirror the crate workspace: `common` (shared code) / `core` (reference component library) / `client`·`node` (thin shells) / `assets` (static)

**Date**: 2026-06-21
**Layer**: UI / frontend source structure (reference implementation, not protocol).
**Spec reference**: mirrors the Rust crate split (Ch4 §4.3: `xgen-core` = protocol library / reference implementation, binaries are thin wrappers; `xgen-common` = shared elements). Grounds the UI-layer instance of a split that was agreed in conversation but never written — deliberately left ungrounded while the real UI did not yet exist (Ch3's module-architecture open question named "Phase 2 client UI structure" as a downstream item; note that OQ is about module-to-UI extensibility, a different sense of "UI structure" than this source-tree grounding — D-095 does not resolve it). UI-notes cluster N-019/N-020/N-022/N-023/N-025; recorded via N-026.

### Decision

The `ui/` subtree is structured as a 1:1 mirror of the four core crates:

| crate (repo root) | UI tier (`ui/`) | role |
|---|---|---|
| `xgen-common` | `ui/common/` | **shared code** — substrate both apps import + execute (envelope mechanics, helpers); no visible components. Alias `$common`. |
| `xgen-core` | `ui/core/` | **reference component library** — the preprogrammed components presented as implementation samples; built on `common`. Alias `$core`. |
| `xgen-client` | `ui/client/` | thin shell composing `core` (rename of `ui/dev_core_ui/client_ui`). |
| `xgen-node` | `ui/node/` | thin shell composing `core` (rename of `ui/dev_core_ui/node`). |

Plus `ui/assets/` = shared **static** files (fonts, logos) — a distinct axis from `common`: `common` is shared *code* (module graph, aliased, tree-shaken, type-checked); `assets` is shared *static files* (referenced by URL, copied/served). The word "shared" is dropped from the asset folder (everything under `ui/` is shared by definition). Final siblings: `ui/{client,node,common,core,assets,docs,templates,backup}`.

**`common` vs `core` boundary (load-bearing):** a component never lives in `common`; a bare helper never lives in `core`. `common` = behaviour both apps depend on; `core` = the sample components built on that behaviour. The component index (N-019) records which tier each entry belongs to.

**Naming retirement:** `ui/dev_core_ui/` is a vestigial name from the era when CLI and UI builds sat side by side; retired now that the CLI tests it gated are complete. The physical folder moves (`client_ui`→`client`, `node`→`node`, both `*/shared_assets`→`ui/assets`) + build-wiring repoint (the two `tauri.conf.json` `beforeBuildCommand`s, `run-*.ps1`, `cdp-debug.ps1`) + the `$common`/`$core` Vite aliases land in the restructure commit that follows this grounding.

**Dev-tooling exemption:** dev-only tool dirs under `ui/` (e.g. `ui/sampler/`, the component-exhibition app — N-028) are **not** part of the 1:1 crate-mirror; they sit alongside the mirrored tiers with no crate counterpart. The mirror governs the shipped substrate / library / shell tiers, not dev scaffolding.

### Why

The crate workspace already encodes exactly the distinction needed — `xgen-core` as the reference/sample library, `xgen-common` as shared elements, the binaries as thin wrappers (Ch4 §4.3). Mirroring it in `ui/` makes the frontend tree self-explaining (`ui/client` ↔ `xgen-client`), keeps the distinction between *shared code* and *sample component* explicit in the path, and stops the placement drift that recurred while the structure was ungrounded. The mirror inverts one detail by nature: in Rust `core` is the heavy library and the binaries are thin; in the UI `core` is the component library and the app shells are thin — same thin-shell relationship, the library on the `core` side either way.

---

## D-093 — Universal E2E at the protocol layer; "Retained (T4)" = ciphertext durability-floor + erasure-refusal (NOT protocol escrow); no shared physical blob copy across erasure-fate (promoted from M12-D6 at M12.4 design-lock)

**Date**: 2026-06-17
**Layer**: protocol (encryption / erasure / retention) + storage (attachment blobs)
**Spec reference**: AH-D1 (per-message CK random, never epoch-derived); D-088 (erasure model: crypto-shred content, orphan identity); F2/F7/F8 (M12 attachment forks); reinforces the no-anonymity/institutional-independence stance.

**Promoted from** the M12-D6 arc-local decision (carried + flagged J-381→J-387, well past the 3-recurrence bar; the D-090 promotion-on-cross-arc-reuse posture). M12.4 (erasure) is the arc that first *exercises* it — F2b is its first enforcement read, blob-delete + erasure-refusal its first mechanism — so the principle is recorded here before that build advances.

**Decision (three bound clauses, one principle):**

1. **Universal E2E, no protocol escrow.** Every tier is crypto-shreddable; there is **no protocol-level escrow / recovery / tier-derived key**. Grounded (D-065, J-381): the Arc-H *text* path is already universal-no-escrow (zero escrow/tier/recovery hits in `xgen-core/src/encryption/`), and the defended invariant `erasing_wrapped_key_defeats_epoch_holder` (AH-D1 constraint 2) keeps the per-message content-key random and never epoch-derived — destroying the wrapped key is permanent **even for the epoch holder**. Blobs inherit that posture; text and attachments stay symmetric and the node content-blind.

2. **"Retained (T4)" is a durability floor, not producibility.** At the protocol layer, Retained = a **durability floor on the ciphertext bytes** (don't drop them) **+ an erasure refusal** — NOT a protocol key that can reproduce plaintext. The T4 retain-and-produce capability (WORM / legal-hold, F7/F8) is **reserved to the operator/module layer**: an accountable deployment that must produce retained plaintext supplies that escrow at *its* tier (forking the reference module + its accountable backend), consistent with institutional-independence and "mark + reserve the hook, don't build the vault." Crypto-shred therefore remains a *real* protocol guarantee everywhere — destroy the per-blob key → every replica, including unreachable federated homes, is permanently unreadable.

3. **No shared physical blob copy across erasure-fate (M12.4 corollary).** Because retention/erasability is **per-record** (clause 2) and attachment `blob_ref` is content-addressed (`hash(bytes)`, so identical files would otherwise dedup to one physical copy), a single shared copy would let one record's policy silently override another's: a lower-tier erasure would delete bytes a T4 record holds (a durability-floor breach), or a T4 hold would block a lower-tier record's valid erasure (a right-to-erasure breach). Therefore an attachment blob's **physical copy may only be shared among references that share the same erasure-fate.** M12.4 v1 takes the strictly-safe floor: **no attachment dedup** — one physical copy per `message.file` send, each with its own deletable storage handle; the **content-hash is retained as descriptor metadata** (not as the storage key) so identical-file detection / policy-keyed dedup-within-a-shared-fate-set stays possible as a **future optimization** (not a correctness fix). A redact deletes only that reference's own copy.

**Why it binds going forward:** clauses 1+2 are a protocol-wide E2E/retention invariant any future tier, module, or transport must honor; clause 3 is the storage-layer rule that keeps per-record retention coherent under content-addressing — violating it re-introduces the cross-fate override at the byte layer. The reverse-index / refcount approach is explicitly rejected as the correctness mechanism: it manages shared-copy bookkeeping but does **not** resolve the tier collision (a refcount cannot honor "A held, B erasable" on one physical blob) — it is heavy *and* insufficient.

---

## D-092 — Adding a client verb touches FOUR dispatch arms (CLI · run-path · batch · aicontrol)

**Date**: 2026-06-10

**Layer**: `xgen-client` command dispatch (the verb-add surface).

**Spec reference**: none (implementation-structure invariant). Arc sources (D-069): ban arc `tasks/BAN_VERB_DESIGN.md` §3 (empirical catch, J-337) + room_update arc `tasks/ROOM_UPDATE_VERB_AUDIT.md` §2 (applied up front + confirmed, J-338). Promotion at J-338.

### Decision

A new client verb's wiring surface is **clap args struct + `ops::<verb>` + `cmd_<verb>` CLI shim + FOUR dispatch arms** — (1) `main.rs` CLI, (2) `app.rs` run-path, (3) `batch.rs`, (4) `aicontrol.rs` `Box::pin` routing. All four are required; a verb missing any arm is silently unroutable on that path. The fourth arm (**aicontrol**) is the one that bites: it is a separate `Box::pin` routing, it returns `UNKNOWN_COMMAND` over `--aicontrol`, and `--aicontrol` is the path the `xgen-mptest` harness drives — so a verb missing the aicontrol arm passes manual CLI testing but fails every harness witness. Every verb-add (and verb-add review) checks all four arms up front.

### Why

Caught **empirically twice**. On the ban arc (J-337) the Phase-0 audit enumerated three dispatch sites (CLI / run-path / batch) and missed the aicontrol arm; ban came back `UNKNOWN_COMMAND` over the harness until the fourth arm was added. The room_update arc (J-338) applied all four up front on that lesson and shipped with no dispatch surprise — the second data point. A rule caught once is a local convenience; caught twice (and silently, on the exact path the test harness uses) it is an architectural invariant of the dispatch layer (the D-090 promotion-on-second-reuse posture). It binds the remaining verb-add (thread×3, arc 4) and any future client verb.

---

## D-090 — `Clock` trait is the canonical injectable time source (promoted from M8.6 promotion-watch on first cross-arc reuse)

**Date**: 2026-06-06

**Layer**: Node runtime / cross-crate (`xgen-common`)

**Spec reference**: M8.6 (clock-injection seam); INV-EXP (invite-expiry replay-gate)

**Decision.** M8.6 introduced a `Clock` trait in `xgen-common` (`now_utc()` + `now_instant()`, `Arc<dyn Clock>`, sync), with `RealClock` (production) and a feature-gated `MockClock` (`mock-clock`; `AtomicU64`-nanos single-cursor; `advance_all` advances the cursor and `tokio::time` in lockstep), held NodeRuntime-resident. It was deliberately kept a **promotion-watch**: a single-arc seam that would graduate to a locked decision on first reuse outside M8.6.

INV-EXP is that reuse — its 3044 invite-expiry gate must read a controllable wall-clock for a deterministic aged-Space federation-replay repro. Therefore the `Clock` trait is now the **canonical injectable time source** for the node. New time-dependent admission/gate logic that needs deterministic testing routes its `now` through the injected `Clock` rather than raw `Utc::now()` / `Instant::now()`. Pre-existing raw reads are migrated **opportunistically** when an arc already touches them (INV-EXP migrates the 3044 gate); there is no blanket sweep. The trait stays sync and minimal (two methods); `MockClock` stays test-only behind `mock-clock`.

**Why now (not at M8.6 close).** A seam used once is a local convenience; a seam used twice is an architectural pattern. The promotion-watch was the honest posture (D-065) — don't lock a pattern on a single data point. The second arc is the data point.

---

## D-088 — XGen erasure model: crypto-shred content, orphan identity, monotonic tier-graded permission (protocol binds endpoints, Auth Module declares interior)

**Date**: 2026-06-04  
**Layer**: Protocol erasure architecture (GDPR right-to-erasure; the mechanism layer beneath ch1's tier-graded compliance philosophy).  
**Spec reference**: ch1 "Compliance & Data Retention by Auth Tier"; Appendix D §3.3; ch1 L879–883 (PG-02). Arc source (D-069): `tasks/ARC_I_ERASURE_DESIGN.md` AI-D1–D9 (design-only arc). Milestone close: JOURNAL J-253.

### Decision

Right-to-erasure in XGen's no-anonymity append-only federated model resolves along three axes. **Content** is erased by crypto-shredding over the encryption boundary (PG-05): the immutable DAG retains ciphertext, signatures sign ciphertext and remain valid, key-destruction makes content unrecoverable without mutating the log — no event deleted, no integrity invariant weakened. **Identity** is erased by orphaning the pubkey↔person binding in the registry cache; the pubkey persists as an anonymous token, all signatures keep verifying, the DAG is untouched. **Permission to erase is monotonic in identity-verification strength: the protocol fixes the endpoints, the Auth Module declares the interior.** T1 (no module) = max-erasable (binding-orphan + content); T4 (legal identity) = destruction of no record at all, retained under Art.17(3) lawful basis (the conscious counterpart to the no-anonymity pillar); T2/T3 = the issuing Auth Module's declared retention policy, carried on the Trust Assertion within the fixed endpoints — modules being the bearers of the real legal/policing function. The module-policy descriptor is forward-extensible (erasability is its first member; unknown members preserved verbatim) so unknown future module requirements have a home without a protocol change. Erasure *enrichments* (display-layer scrubbing/filters) are implementation-within-frame on the rebuildable materialization layer (D-080 split), never the DAG. Residual exposure (non-complying replicas; re-identification via correlates) is acknowledged and out of in-protocol scope, mirroring the jurisdiction stance (Arc G). Blank-at-rest event mutation is **rejected** as a default erasure mechanism (it would weaken universal signature-verifiability and conflict with D-076); it survives only as a named last-resort for legacy pre-PG-05 plaintext, with its integrity cost stated, unbuilt.

### Why

ch1's "Compliance & Data Retention by Auth Tier" already separates the **compliance layer (Auth Module)** from the **mechanism layer (protocol)** and already permits tier-graded refusal ("Tier 4 — some deletion requests may be legally refused"). D-088 supplies the missing *mechanism* honestly: crypto-shred avoids the blank-at-rest integrity scar that the earlier Appendix D §3.3 "planned approach (Phase 2)" tombstone-redaction would have introduced (**this supersedes that planned approach**); orphan-not-delete keeps every signature valid (identity erasure touches no events); and the T2/T3 threshold is placed with the party that actually bears the legal/retention obligation (the module) rather than guessed as a protocol constant. The arc is design-only: PG-02 closes **design-locked / implementation-deferred** — content-erasure build is gated on PG-05 (Arc H), identity-erasure is PG-05-independent and could ride the Tier-1 auth-module rebuild. Stating the T4 zero-erasability plainly is itself a deliverable, not a gap.

### Amendment (2026-06-04) — AH-D1 envelope key granularity (Arc H / PG-05)

**Source (D-069):** `tasks/ARC_H_E2E_DESIGN.md` §1 (AH-D1), implemented at Arc H C1. The original decision above is unchanged; this amendment records the *key-granularity* choice the crypto-shred axis depends on, which Arc H was the first arc positioned to make.

**The gap this closes.** D-088's content axis says "erase content by crypto-shred over the encryption boundary (PG-05)" — but crypto-shred assumes a **per-erasure-unit erasable key**. The as-built Phase-2 scheme (and ch3 §3.10.7 as originally written) encrypts content **directly under the per-epoch key** — one key per epoch. Destroying an epoch key erases an *entire epoch*, not one message; an Arc H that shipped epoch-only keys would force the later content-erasure arc to retrofit the substrate — exactly the "build on the wrong substrate" trap D-088 exists to avoid.

**Decision (AH-D1, promoted into D-088).** Message content is encrypted under a **per-message random content key `CK`**; `CK` is **wrapped under the current MLS epoch key**; the wrapped `CK` rides the DAG with the message (`enc:` v2 envelope, `xgen-core/src/encryption/client_mls.rs`). Granularity is **one message = one erasable key** — what crypto-shred requires. Erasure = destroy the wrapped `CK`: the chain + signatures (which sign the ciphertext envelope) stay valid, nothing in the DAG mutates, the content ciphertext becomes permanently undecryptable.

**Two invariants (both enforced/tested at C1):**
1. **The envelope MUST NOT weaken MLS.** Confidentiality and forward secrecy still derive *entirely* from MLS — an attacker without the epoch key recovers neither `wrapped_CK → CK` nor content. The envelope adds *only* an erasability layer; removing it yields exactly the as-built epoch-confidentiality guarantee.
2. **Threat-defended erasability.** `CK` is **random per message, never KDF-derived from the epoch secret**. Were it epoch-derivable, a future implementer could satisfy the wording while a holder of the epoch secret silently re-derives `CK` after the wrapped copy was destroyed — and erasure becomes a no-op. The named test (`erasing_wrapped_key_defeats_epoch_holder`) destroys the wrapped `CK` and asserts decryption fails **even with the correct epoch key**.

**Scope (honest).** Arc H ships the *substrate* — generate → wrap → store-wrapped → unwrap-to-read — and proves it (content-blindness proof, AH-D5). The destroy-to-erase **storage operation** (locating and overwriting the wrapped `CK` in persisted storage) is **fenced behind the erasure-impl arc** per the cascade `D-088 content-erasure → PG-05 real crypto → D3`. PG-05 itself closes **interface-locked / impl-deferred** (real RFC 9420 crypto = D3), not ✅ DONE. ch3 §3.10.7 is extended to the envelope at the same commit (D-074).

---

## D-087 — Storage assurance is enforced; a selected engine is both live store and durability authority

**Date**: 2026-06-03  
**Layer**: Node conformance / durability enforcement (storage slot; the enforcement half of D-080's durability requirement).  
**Spec reference**: Ch4 §4.12.6; Appendix L §L.8–§L.9. Arc sources (D-069): `tasks/STORAGE_ENGINE_DESIGN.md` SE-D4, `tasks/STORAGE_ENGINE_SUBSTITUTION_DESIGN.md` SE-SUB-D4/D6. Milestone close: JOURNAL J-232.

### Decision

A Node's asserted storage assurance is **enforced at startup, not advisory**. `[node].asserts_tier` (explicit, clamped, default-derived as the floor over `auth_tiers_served` ∪ module `accepted_tiers`, never settable below that floor — loud reject) maps to a required `AssuranceClass` (`BestEffort < Durable`); a Node whose selected engine under-delivers (e.g. asserts Tier 2–4 with no `Durable` engine) **refuses to start** with a loud error.

When a durable engine is selected it is **both the live per-Space store and the durability authority**: the vanilla app-layer JSON persist is bypassed, startup rehydrates per-Space state from the engine (`range(0)`, enumerating Spaces from the engine's own files), and a store-open failure is a **loud reject** — never a silent fallback to a vanilla RAM store. A silent fallback would re-introduce exactly the false-durability this enforcement exists to remove (a node passing the assurance gate while writing to RAM).

### Why

Before enforcement the gate was theatre: `asserts_tier=2 + sqlite` passed while every Space still wrote to RAM (the engine was selected but never threaded into per-Space construction). The fix is two-sided — thread the engine as the live store (SE-SUB-D5) **and** hand it durability authority (SE-SUB-D6); either alone leaves a dishonest gap (a double-write with JSON as the real replay source). "Refuse to start" over "warn and continue" because a credibility claim the substrate cannot back is worse than not booting.

### Relationships

| Decision | Relationship |
|---|---|
| D-080 | D-080 requires durable storage and keeps the engine pluggable; D-087 is the *enforcement* — the tier→assurance gate + engine-owns-durability runtime contract. D-080 unchanged (no amendment). |
| D-084 | D-084 governs per-event *persist* failure (loud + propagate, no ack-block, v1). D-087 governs *store-open* failure (loud reject, no RAM fallback) and the assurance gate — a distinct, more fundamental failure than a single append. |
| D-085 | The gate + engine selection ride on the D-085 module registry. |
| D-065 | Honest behaviour over polite — the milestone’s organising principle (“the gate cannot honestly close as theatre”). |

---

## D-086 — Module identity: artifact-id and principal-id are two coexisting facets

**Date**: 2026-06-03  
**Layer**: Identity / identifier discipline (module system; cross-cutting with XGID discipline).  
**Spec reference**: **Ch4 §4.12.5 is the canonical expository home** (the two-facet taxonomy + the normative artifact-UUID rules, incl. RFC 9562 canonical form, compare-by-value-not-string, lenient-parse/strict-emit, format-revision back-compat). Arc sources (D-069): SE-D2; cross-ref D-083 (`AuthModuleXgid`). Milestone close: JOURNAL J-232.

### Decision

Every module slot carries identity in **two facets**, chosen per slot by one test: *does this identity cross the wire or bear a key?*

- **Artifact identity** (*which implementation*) — `ModuleKindId` (one per slot, minted once, **copied verbatim** by every implementation of that slot; how the host recognises the slot) + `ModuleImplId` (unique per implementing crate). Both **UUIDv4, local, dev-assigned, keyless, never federated** — `xgen-common` newtypes over `uuid::Uuid`, deliberately **not** `Xgid` (SE-D2).
- **Principal identity** (*which key-bearing authority, as seen across nodes*) — an `Xgid` flavour (principal family, D-072/D-073), only for modules that *are* principals (today the auth module, `AuthModuleXgid`, D-083).

The two facets **coexist** (an auth module has both a crate artifact-id and a principal `AuthModuleXgid`) and identify different things. Artifact-UUID handling is normative per §4.12.5: canonical RFC 9562 emit form, **compare by parsed 128-bit value not by string** (load-bearing), lenient-parse/strict-emit, newtype-over-`uuid::Uuid` seam. Storage engines carry **only** the artifact facet; the registry GUID handshake compares the slot kind by value (the §4.12.5 worked instance, shipped at C3/S).

### Relationships

| Decision | Relationship |
|---|---|
| D-085 | The framework D-086 names the identity scheme for. |
| D-083 | `AuthModuleXgid` is the first principal-facet instance; D-086 generalises the artifact-vs-principal split. |
| D-072 / D-073 / D-081 | XGID discipline (flavoured types, role-names, wire/persistence invariance) governs the principal facet; the artifact facet is deliberately *outside* XGID (local keyless UUIDs). |

---

## D-085 — Module framework: by-trade compile-time plugins under an explicit registry

**Date**: 2026-06-03  
**Layer**: Architecture (module system; first instance at the storage slot, but the stance governs every future module slot).  
**Spec reference**: Ch4 §4.12.4 (stance, as-built) + §4.12.6 (storage instance); Appendix L §L.7/§L.9. Arc sources (D-069): `tasks/STORAGE_ENGINE_AUDIT.md` §8 (dynamic-loading rejection), `tasks/STORAGE_ENGINE_DESIGN.md` SE-D1/D3/D6. Milestone close: JOURNAL J-232. Promoted at **instance #1** (the storage engine), as flagged in §4.12.4/§4.12.5.

### Decision

XGen modules are **by-trade, compile-time** plugins. Each slot is a narrow Rust trait; an implementation is wired by an **explicit `register::<E>()`** call into a per-slot registry — no `inventory`/`linkme` link-time magic, no runtime `dlopen` / dynamic native loading. The host selects by config and **type-erases to `Box<dyn Trait>`** at registration; unknown selections are **rejected loud**. Slots are organised on a `kind ∈ {system, display}` × `host ∈ {node, client}` taxonomy; only the trust- and federation-bearing **contracts** are normative — the implementation is by-trade, so SDKs in other languages reimplement the contract, not the registry.

**Dynamic native loading is rejected** for a key-holding Node on in-process key-theft grounds: a loaded `.so`/`.dll` shares the Node's address space and therefore its signing key. A future sandboxed Wasm / signed-module arc is the banked escape hatch (storage audit §8). **Instance #1 = the storage engine** (`system·node`): `StorageEngine` trait + `EngineTable` registry + `xgen-store-sqlite` as the first plugin, shipped C1–C5 + S.

### Why

Explicit-register-not-magic = greppable, reject-unknown-loud, no hidden link-time surface. Static-not-dynamic = no in-process key-theft surface on a key-holding Node. By-trade-not-normative = SDK freedom (the contract is the spec, the registry is reference-implementation). The `kind × host` taxonomy gives every future slot (auth module, temperature plugin, display modules) a named home rather than ad-hoc wiring.

### Relationships

| Decision | Relationship |
|---|---|
| D-080 | The storage slot is the first this framework fills; D-080 stays the storage-*shape* lock, D-085 the module-*system* lock. |
| D-086 | How slots and implementations are named (artifact vs principal identity). |
| D-087 | The storage assurance gate rides on this registry. |
| D-065 / D-066 / D-067 / D-069 | Honest behaviour; sister control-mode flags as an earlier by-trade seam; single-source-of-truth; delegated-design discipline (SE-D#/SE-SUB-D# arc-local, promoted here). |

---

## D-084 — Persist failure is loud + propagated, but does not block accept/ack in v1

**Date**: 2026-06-02  
**Layer**: Node implementation (durable-storage write-path) / honest-behaviour discipline. Sibling to D-065 (honest behaviour over polite behaviour) at the persistence layer. Absorbs the long-flagged "silent-write" candidate (the swallowed `let _ = fs::write(...)` in `persist_event`), surfaced as F-3 in the Durable EventStore audit and resolved as ES-D4.  
**Spec reference**: `tasks/EVENTSTORE_AUDIT.md` v1.1 (F-3 swallowed-errors finding); `tasks/EVENTSTORE_DESIGN.md` §5 (ES-D4); `tasks/EVENTSTORE_IMPL.md` §2 C2; `xgen-node/src/app.rs::persist_event` (now `-> io::Result`), shipped at `2eb3b0c`. Cross-references: D-080 (the EventStore service this write-path belongs to); D-065 (sibling honest-behaviour principle); the F-1 atomic-write + F-2 honest-read floor this completes.

### Decision

The Node event-persistence write-path (`persist_event`) **returns `io::Result` and, on failure, logs loud (`tracing::error!`) and propagates** — it no longer swallows the error (`let _ = fs::write(...)`). **In v1, a persist failure does NOT block accepting or acking the event.** The runtime authority is the in-memory store, which has already accepted the event; the F-1 atomic-write floor guarantees a failed write cannot corrupt or truncate the live file (only a throwaway `.tmp` is ever at risk); and federation replication + the content-addressed DAG make a lost tail event re-syncable. Callers therefore **log-and-continue** rather than refusing the event.

This is the honest middle between two dishonest extremes: the old **silent swallow** (history could vanish with no signal — dishonest about failure) and a hard **ack-block** (refusing an otherwise-valid, already-in-memory, re-syncable event on a transient disk hiccup — over-strict, and it couples acceptance to a layer that is not the runtime authority in v1).

### Why this needed an explicit decision

The silent-write gap had been flagged as a candidate across earlier sessions without a home; the Durable EventStore milestone is where it is fixed, so it earns a numbered contract rather than being absorbed silently into a commit. Naming it fixes both halves: *fail-loud + propagate* is non-negotiable (the floor's honesty property, sibling to F-2's honest-fail-on-read); *not-blocking-ack* is a **deliberate v1 choice** justified by named backstops, not an oversight. Recording it prevents a future reader from either (a) re-introducing a silent swallow, or (b) assuming acceptance is gated on durable persistence when in v1 it is not.

### What this commits the node to

- The persist write-path is **fallible, loud, and propagating** at every call site; **no swallowed write errors**.
- **v1:** persist failure does not block accept/ack; callers log-and-continue.
- **Backstops (named, load-bearing):** the in-memory store is the runtime authority; the F-1 atomic write prevents corruption on failure; federation + the content-addressed DAG make a lost tail event re-syncable.
- **Escalation path:** a Node asserting **Tier 2–4** (running the durable engine module) MAY tighten to **commit/fsync-before-ack** — strict durability gating acceptance. That escalation is a **future decision**, not a silent drift, and is coherent with the EventStore design §8 Tier-2–4 conformance note (T2–4 require the engine module).

### Relationship to other decisions

| Decision | Relationship |
|---|---|
| D-080 | Parent. D-084 is the write-path failure contract of the EventStore service D-080 defines; it is part of the vanilla minimal durability floor (alongside F-1 atomic write + F-2 honest read). |
| D-065 | Sibling principle at the persistence layer: honest behaviour over polite behaviour. A swallowed write error is dishonest about state in the same sense D-065 names; D-084 makes the failure loud and propagated. |
| D-069 | Canonical home: D-084 lives in DECISIONS.md; the arc-local ES-D4 (`tasks/EVENTSTORE_DESIGN.md` §5) graduates here, and the audit's F-3 + the runbook's C2 forward-reference it. |

---

## D-083 — `AuthModuleXgid`: the seventh XGID flavour (third principal flavour)

**Date**: 2026-05-31  
**Layer**: Protocol-identity-model / identifier vocabulary — promotes the XGID flavour family from six to seven, the first such promotion since the vocabulary was named (D-072). Sibling to D-072 (XGID type discipline) at the flavour-set layer; the promotion barrier D-072 / Appendix J §J.2 set up ("adding a flavour requires explicit promotion through a new DECISIONS.md entry") is satisfied by this entry. Surfaced during the auth-module-registry D-071 arc (AMR-D2), where the Auth Module needs a self-certifying, key-bound identifier.  
**Spec reference**: `docs/xgen_appendix_j_en.md` §J.2 (six → seven; the canonical flavour enumeration) + §J.6 (the seventh wrapper); `tasks/M6_AUTH_MODULE_REGISTRY_DESIGN.md` AMR-D2 / AMR-D3 (the arc-local locks this graduates); `xgen-common/src/xgid/flavours.rs` (the `declare_flavour!(AuthModuleXgid, …)` + `from_pubkey` / `pubkey`). D-072 (flavour vocabulary + the promotion barrier); D-073 (field-name-vs-type); D-081 (typing is wire-format invariant — `AuthModuleXgid` inherits the serde-transparent invariance by construction).

### Decision

A **seventh** XGID flavour, **`AuthModuleXgid`**, is added to the family. It is the **third principal flavour** — alongside `NodeXgid` and `IdentityXgid` — identifying an Auth Module by the Ed25519 verifying key of its module keypair. It shares the principal URI shape `xgen://pubkey/ed25519:<base64url-key>` and the same construction / decode path (`from_pubkey` infallible; `pubkey()` parse-fallible at v1), reusing the existing `principal_uri` / `principal_decode` helpers. **No new URI prefix and no new wire shape** — it is a new *type-system* flavour over the existing principal URI grammar, so it inherits the §J.5 wire-format invariances and D-081 by construction (`#[serde(transparent)]` over the base `Xgid`).

It is a *principal* flavour, not a hash-anchored one: an Auth Module is a signing entity (it verifies and attests Identity tiers), so the protocol names it by its key — the key *is* its identity, recoverable from the XGID — exactly as a Node or an Identity is named (XGen key-is-identity philosophy). This is the type-level reason the auth-module-registry stores `module_id: AuthModuleXgid` as the single source of truth and derives the key via `.pubkey()` rather than storing a separate `public_key` field (AMR-D3).

### Why this needed an explicit decision

D-072 / Appendix J §J.2 fixed the family at six and set the barrier deliberately high: "Adding a [further] flavour requires explicit promotion through a new DECISIONS.md entry … identifier vocabulary is one of the few things in a protocol that must be small and stable." The auth-module-registry arc is the first work to genuinely need a new first-class identifier, so it pays the barrier here rather than silently widening the family in `flavours.rs`. A **D-078-shape catch during the design walk** corrected an imprecise first framing ("key-hash flavour"): the family is closed at six, and *principal* flavours are the key URI, not a SHA-256 hash — so `AuthModuleXgid` is a principal sibling of Node/Identity, not a hash-anchored sibling of Event/Space/Room. Locking the distinction in a numbered decision keeps the flavour family's two-family structure (hash-anchored / principal) honest and prevents the seventh flavour from being mis-modelled.

### What this commits the protocol to

- The flavour family is now **seven**: four hash-anchored (Event, Space, Room, TrustAssertion) + three principal (Node, Identity, Auth Module). An eighth flavour requires its own DECISIONS.md promotion.
- `AuthModuleXgid` stays `#[serde(transparent)]` and carries a §J.5-style witness (`auth_module_xgid_from_pubkey_roundtrip`); typing it onto a field is wire-neutral (D-081).
- The auth-module-registry record keys on `AuthModuleXgid` and derives the key (AMR-D3); the eventual tier-verification consult (the deferred `AuthModuleUntrusted` / 3006 check) is key-bound through this flavour.

### Relationship to other decisions

| Decision | Relationship |
|---|---|
| D-072 | Parent. D-072 named the XGID vocabulary and fixed it at six with an explicit promotion barrier; D-083 is the first promotion to clear that barrier, adding the seventh flavour. |
| D-073 | Field-name-vs-type discipline: `module_id: AuthModuleXgid` carries the role in the field name and the contract (a recoverable Auth Module key) in the type. |
| D-081 | Wire-invariance: `AuthModuleXgid` is serde-transparent over the base `Xgid`, so it serialises byte-identically to the `String` URI — it inherits D-081 by construction. |
| D-069 | Canonical home: D-083 lives in DECISIONS.md; the arc-local `AMR-D2` in `tasks/M6_AUTH_MODULE_REGISTRY_DESIGN.md` graduates to it, and Appendix J §J.2 carries the normative enumeration. |

---

## D-082 — "operator" reserved for the AI-operator role; the Node administrator is a distinct infra principal

**Date**: 2026-05-29  
**Layer**: Protocol vocabulary / naming discipline — a global term reservation spanning spec, design, and code. Sibling to D-073 (field-name-vs-type) at the role-vocabulary layer. Surfaced during M6 Block 4 prep when the admin-ops design's "operator" (Node-admin sense) collided with the locked AI-operator role.  
**Spec reference**: D-059 + D-064 (AI-operator role origin + fall-upward resolution); `docs/xgen_node_admin_ops_design.md` §2.6.1 / §2.6.2 (administrator authority model); M6 Block 4. Recorded ahead of the corpus audit (J-149) per Joe's "record it now"; the rename sweep that applies it across the doc corpus is the next-active step.

### Decision

Four locks.

**1. "operator" is reserved globally for the AI-operator role.** An *operator* is a delegated, revocable, Space-scoped role that governs one or more concrete AI-identities — the structural parallel of a *moderator*, which governs a room and its members:

> moderator : a room + its members  ::  operator : one or more AI-identities

Both are roles of the same class (granted, revocable, scoped); only the governed object differs. Authority falls upward when unset (operator → moderator → owner), exactly as moderation resolves (D-064 fall-upward). "operator" MUST NOT be introduced as a *new* owner/admin alias; the legitimate non-AI uses already in the corpus are scoped in "Scope — the four senses" below.

**2. The Node administrator is a distinct infra principal.** The human (or process) that administers a Node via the `--batch` admin surface is the **administrator**, never the "operator". **Register split:** "administrator" in narrative / spec / design prose; **"admin"** in code identifiers, CLI verb tokens, error-code namespaces, and config keys — matching the existing `admin_ops` / `AdminContext` / `AdminError` vocabulary. M6 v1 has no role gradation: per §2.6.1 / §2.6.2, OS-user-equals-administrator, session-scoped — anyone who can open the pipe is the full administrator.

**3. "owner" / "super-admin" is a reserved future sub-tier, not split in v1.** The owner is de facto the top administrator; a finer owner-vs-lesser-admin gradation only becomes meaningful if per-verb gating lands (flagged for M7). v1 does not distinguish them.

**4. A Node administrator has automatic Space-administrator authority over Spaces that Node originates / homes — NOT Spaces it merely replicates via federation.** "Hosts-but-doesn't-own" (Ch2) means a Node also hosts replicated peer Spaces; admin authority MUST NOT extend to those, or federating a Space to a peer would grant that peer admin rights over the originating Space. The *signing identity* for admin-originated Space events (e.g. a Node-forced `membership.kick`) is deferred to the A4 signing-identity sub-design: granting authority does not by itself answer what signs the event so federated peers can validate it.

### Scope — the four senses of "operator" (audit-refined, J-150; R2-F06 added Sense E)

A corpus audit (J-150) found "operator" carries four senses across the spec, appendices, and code; only one is renamed:

- **A — AI-operator role** (this decision's reserved sense): keep "operator". The "AI" qualifier or Space-membership context disambiguates — e.g. `resolve_operator`, `operator_known`, the `ai delegate` / `ai revoke` verbs.
- **B — wire field names** (`operator_display_name` in the Node Announcement canonical signing order; `bootstrap_info.operator`): keep verbatim — renaming would break wire-format invariance (D-081) and the signing byte order. Untouchable.
- **C — infrastructure operator** (the entity that runs and is legally accountable for a Node / Auth Module / Bootstrap Node — deployer, custodian, GDPR data controller): keep "operator" (e.g. "Node operator"). This is the standard infrastructure sense, distinct from the AI-operator role and woven through the GDPR/legal language (Appendix D); "administrator" is a poorer fit for a data controller. Where a line is genuinely ambiguous, disambiguate **inline with a facet-naming specifier** — e.g. "Node operator (the entity running the Node)" — rather than re-equating "operator" to owner/admin.
- **D — runtime admin principal** (whoever drives the `--batch` admin write surface): the genuine collision → rename to **administrator** (prose) / **admin** (code).

- **E — console operator** (R2-F06 refinement, 2026-06-05): the human-or-AI agent driving the Console / command channel (Ch1 "Human and Agent Operation"; ch6 Console + AI-operator panel) — "AI agents as first-class Console operators". This is **not** the `--batch` admin principal (Sense D) and **not** the infra operator (Sense C); it is the operation-in-control metaphor. **Keep "operator" / "console-operator".** Renaming it to "administrator" would break the Ch1 framing and make the chapters internally inconsistent (the same actor is called "operator" throughout). The AI-client *runner* (who drives `--stop` / joins on an AI-client resident, ch6 §6.15) folds here: **keep "operator"**, never "ai-operator" (that is Sense A).

The J-150 rename sweep touched **Sense D only**: the M6 admin-ops design doc (10 hits) + the `xgen_aicontrol_implementation.md` "Space/Room operator actions" category mirrors. Senses A, B, and C were left in place. Future authoring follows the same map.

### Why this needed an explicit decision

"operator" carried two incompatible meanings — an early universal title for the Node owner/admin, and the later precise AI-operator role (D-059 / D-064) with its own events (`state.ai_operator_delegate` / `revoke`) and state (`ai_operator_delegations`). The M6 admin-ops design used the early sense ten times. Drafting Block 4's ~35 verbs on the overloaded term would have baked the collision into the verb registry, the audit `actor` semantics, and the A4 signing question. D-082 reserves the word, names the infra principal, fixes the prose-vs-code register, and scopes admin authority — all before Block 4 writes a single verb.

### Relationship to other decisions

| Decision | Relationship |
|---|---|
| D-059 / D-064 | Origin of the AI-operator role D-082 reserves "operator" for. D-064's fall-upward resolution is the moderator-parallel D-082 makes explicit (operator : AI-identities :: moderator : room + members). |
| D-073 | Sibling naming discipline. D-073 governs field-name-vs-type; D-082 governs role-term reservation and the prose-vs-code register (administrator/admin) — the same discipline applied to a role noun. |
| D-072 | `NodeXgid` vs `SpaceXgid` are distinct flavours — the type-level reason a Node identity and a Space identity are not interchangeable, which is why lock #4 scopes admin authority to originated Spaces and defers the signing identity to A4. |
| D-069 | Canonical home: D-082 lives in DECISIONS.md; `docs/xgen_node_admin_ops_design.md` and the spec role sections forward-reference it after the rename sweep. |

---

## D-081 — XGID typing is wire-format and persistence-format invariant

**Date**: 2026-05-29  
**Layer**: Data-model / wire-format discipline — the contract that retyping identifier `String` slots to typed XGID flavours changes in-memory types only, never serialized bytes. Sibling to D-076 (wire-order determinism) in the wire-format discipline family.  
**Spec reference**: `tasks/XGID_RETROFIT_PASS_5_IMPL.md` §6 (promotion at Pass 5 / arc close, J-148); `docs/xgen_appendix_j_en.md` §J.5 (five wire-format invariance witnesses); `tasks/XGID_ADOPTION_DESIGN.md` Q4 (originating invariance promise); D-072 + D-073 (XGID type + field-name-vs-type discipline). Realised across XGID Retrofit Pass 1 (J-122) → Pass 5 (J-148).

### Decision

Retyping a `String` identifier slot to a typed XGID flavour (`EventXgid`, `SpaceXgid`, `RoomXgid`, `TrustAssertionXgid`, `NodeXgid`, `IdentityXgid`) is a **pure in-memory type-discipline change**. Because every flavour is `#[serde(transparent)]` over a base `Xgid(String)`, it serializes and deserializes **byte-identically** to the pre-retrofit `String` shape on every boundary — Node↔Node wire, Node↔Client wire, AI-control / batch JSONL, and on-disk persistence. No retrofit pass (1–5) changed a single serialized byte.

The **canonical string form is the flavour's `Display` projection**; `Debug` may reveal the wrapper (`IdentityXgid(Xgid("…"))`) for diagnostics only. User-facing output and structured trace fields use `Display` (`{}` / `%`); `Debug` (`{:?}` / `?`) is for diagnostic dumps, never for canonical identifier emission.

### Why this needed an explicit decision

The five-pass XGID Retrofit progressively retyped every identifier slot across all four crates. Each pass carried the implicit promise that typing was wire-neutral — but that promise lived only as per-pass serde-transparent witness tests and the Appendix J §J.5 invariances, never as a named project principle. D-081 promotes it: a future contributor adding or retyping an identifier field has an explicit rule (serde-transparent flavour; `Display` = canonical; `Debug` = diagnostics) rather than re-deriving wire-neutrality from the witness tests. Promised at Pass 5 close in the ROADMAP Near-future entry; locked here as the arc-closing principle. Pass 5's trace-field formatter audit (finding F-1) caught the one site that violated the `Display`-for-emission half of the rule before this principle was named — evidence the discipline needed an explicit home.

### What this commits the protocol to

- Every XGID flavour stays `#[serde(transparent)]`; any future flavour added to the family inherits the invariance by construction plus a §J.5-style witness test.
- Identifier retypes never require a wire-format version bump or a migration — they are below the wire.
- The D-073 principle ("field name carries the role, type carries the contract") is fully realised in code across all four crates; the transitional "mixed discipline" clause from XGID Adoption Q3 no longer applies.

### Relationship to other decisions

| Decision | Relationship |
|---|---|
| D-076 | Sibling in the wire-format discipline family. D-076 = byte-identical sender output **across senders** (wire-order determinism); D-081 = byte-identical **across the typed/untyped boundary** (typing is wire-neutral). Different axes, one family: the wire format is contract, not implementation latitude. |
| D-072 / D-073 | D-072 introduced the XGID flavour vocabulary; D-073 the field-name-vs-type discipline. D-081 is the wire-invariance guarantee that made the whole retrofit safe to ship incrementally — the deliberately-broken "Path A" builds between passes never risked wire drift because typing is serde-transparent. The three form the XGID discipline stack: vocabulary (D-072) → naming (D-073) → wire-invariance (D-081). |
| D-069 | Canonical-document rule: D-081's authoritative home is DECISIONS.md; `tasks/XGID_RETROFIT_PASS_5_IMPL.md` §6 and Appendix J §J.5 forward-reference it. |

---

## D-080 — Node storage is a thin append-only EventStore service over a proven embedded substrate

**Date**: 2026-05-29  
**Layer**: Node-storage-implementation-layer — the contract between the protocol's durability requirement and the on-disk substrate. Distinct from the storage *model*, which is settled at the architecture layer (Ch2: append-only Event DAG, per-Space, federated, hosts-but-doesn't-own). D-080 names what the node's storage component IS as an implementation artefact, and states why an embedded engine is present at all.

### Decision

The node's storage is a **thin, append-only EventStore *service*** exposing exactly three primitives over a durable substrate:

- `append(event)` — write the Event, keyed by its content-hash `event_id`
- `get(event_id)` — fetch a single Event by id
- `range(since-DAG-point)` — return a causal range for `federation.sync` / catch-up-on-reconnect

The **protocol mandates the contract, not the engine.** A node MUST have durable storage satisfying these three primitives. The protocol does not name which engine provides it. **SQLite is the reference-implementation default**, swappable behind the `EventStore` trait. The node core performs **no rich query** and has **no display layer**; richer access (search, analytics, admin query) exists only as **optional, rebuildable projections beside the store** — never inside the log — advertised per-node as a capability.

### Why this needed an explicit decision

The storage *model* received full architectural treatment across Ch2 sessions. The storage *engine* never did — SQLite entered as an implementation assumption (it surfaces only obliquely, e.g. `DEGRADED_STORAGE` naming "SQLite lock contention"), not as a reasoned, Joe-locked choice. This entry converts a decision made *by default* into one made *by reasoning*. It is the entry that should have existed and did not. (Instance of the "subsystem audits precede dependent milestones" principle — D-071 — catching an undecided foundation that downstream work quietly assumes.)

### What the node store actually needs (derived, not assumed)

- **Embedded / in-process** — non-negotiable; vanilla node = ~2-min setup, no DB server to administer. Rules out client-server databases.
- **Append-only, content-addressed** — workload is "append blob, look up by hash, read a causal range." A log/KV shape, not a relational one.
- **Crash-safe and durable** — the node is custodian of *other people's* history, and federation gives no delivery guarantee, so local durability is the backstop. `DEGRADED_STORAGE` already names "DAG integrity failure."
- **Backup-trivial** — Ch2 requires backup to be visible and easy from day one. One file per Space is a real architectural advantage here.
- **Per-Space isolatable** — one store per Space, which shards the single-writer bottleneck.
- **Cross-platform** — Windows desktop nodes and Linux VPS/Pi.

### Three tiers (the precise mandatory-vs-optional split)

1. **EventStore interface** — *mandatory*, identical for every node. append / get / range.
2. **Backing engine** — *present in every node, but pluggable*; not protocol-fixed, never absent. SQLite in the reference impl. "Pluggable," not "optional" — a node cannot run with no store.
3. **Projection capabilities** (search / analytics / admin query) — *genuinely optional*, advertised per-node. Each is a derived index built by consuming the event stream, disposable and rebuildable from the log, living beside the store and never coupled into it.

### Why an embedded engine at all — the single honest reason

Not for query (the core role needs none) and not for display (the node has no display layer — that rationale belongs only to the *client's* materialization cache and is a category error if applied to the node). The only reason an embedded engine earns its place is the **unglamorous durability floor**: atomic/crash-safe append (no torn writes under power loss), integrity detection, concurrent read-while-writing, and trivial per-Space-file backup. For a custody-of-history store, "the log must never corrupt" is the sacred requirement, and an embedded engine inherits that battle-tested floor instead of re-implementing it.

### Rejected framings

- **Engine as protocol requirement** — violates "thin core" and "swappable implementations upward." The engine stays out of the spec entirely.
- **Storage justified by a display/materialization rationale (the client's reason)** — rejected. The node renders nothing; it has no display layer. Node storage sits wholly on the heavy-data / source-of-truth side.
- **Fully hand-rolled raw-file log ("custom all the way down")** — rejected as the default. Crash-safety and integrity are the genuinely hard parts of storage; hand-rolling them for a custody store is high-stakes, low-glory, and the exact "you're writing a database" trap. The *service* is custom and thin; the durable substrate underneath is not custom.
- **Plugging a full query DB engine into the core** — rejected. It makes the rich engine load-bearing, heavies the vanilla path, and re-creates the one-way door. Rich query belongs in optional projections, not the core store.

### Engine choice reasoning (reference impl)

- **SQLite — chosen default.** Embedded, ACID, one file per Space (trivial backup), mature, crash-safe. Its one weakness — single-writer + fsync throughput — is precisely what per-Space sharding already mitigates. "Good by decision," not "good by luck."
- **redb / sled (Rust-native KV)** — the only genuine alternative considered, purely to drop the C dependency and simplify the build. Costs SQL ergonomics and battle-testing; does not beat SQLite on backup. Not adopted without a concrete reason.
- **RocksDB / LSM** — only ever a node-side write-throughput escape hatch. Solves a problem already engineered around; backup is a directory + checkpoint, not a file copy. Named as the escape hatch, not the choice.

### Relationship to other decisions / principles

| Principle | Relationship |
|---|---|
| Thin core / swappable implementations upward | D-080 is a direct application: contract in, engine out, behind the `EventStore` trait. |
| D-071 (subsystem audits precede dependent milestones) | D-080 is what that principle produces when applied to storage — surfacing and resolving an undecided foundation. |
| D-065 (honest behaviour) | Sibling: name the real reason for the engine (durability), not a borrowed or convenient one (query/display). |
| Ch2 storage model | D-080 sits *below* the model. The model (append-only DAG, per-Space, hosts-but-doesn't-own) is unchanged; D-080 only specifies the implementation contract and substrate beneath it. |

### Out of scope / follow-ups

- **Node Storage Audit (Phase-0 candidate)** — document the current de-facto on-disk layout, the engine actually in use, and the real access patterns, before any conformance work. The honest baseline.
- **EventStore trait conformance** — ensuring current code sits cleanly behind the trait so the engine is swappable without touching anything above it (Clair's lane; likely small if code is already close).
- **Projection capability design** (search/analytics/admin-query as derived indexes) — deferred; not needed by the default node role.
- **"operator" terminology correction** — unrelated pending doc-consistency pass (repurpose "operator" for the AI-delegation role; old node-"operator" sense collapses to owner/admin). Noted here only so it is not lost; it is not part of this decision.

---

## D-079 — Design-doc Q-table grounded by symbol-definition grep

**Date**: 2026-05-28  
**Layer**: Design-doc-Q-table-layer — the discipline frame applied when authoring or amending any Q-table row in a design doc §2-style surface enumeration that attributes a type, parameter, or field shape to a production symbol. Sibling-distinct from D-078 (which lives at the protocol-test-layer of test enumeration vs production reject-paths) and D-077 (which lives at the meta-layer of how the project asks sustainability questions at silent-discard sites). D-079 lives at the layer where design-doc Q-row contracts are authored against production symbol definitions.  
**Spec reference**: `tasks/XGID_RETROFIT_PASS_3_DESIGN.md` v1.4 §2.3 Q3.6 + §6.3 (the canonical cautionary instance; the Q3.6 row was wrong three times across two distinct catch-events). `tasks/XGID_RETROFIT_PASS_3_DESIGN.md` v1.3 §2.3 Q3.6 + §2.5 Q5.14 + §6.2 (the J-133 catch closing the v1.0/v1.1/v1.2 originals). `JOURNAL.md` J-133 + J-134 (the two distinct catch-events). Cross-references: D-078 (production-grounded test enumeration — protocol-test-layer sibling); D-077 (bidirectional sustainability discipline — meta-layer sibling); D-067 (no-drift-surface code-organisation parent); D-069 (canonical-document discipline — D-079's application surface is design doc Q-tables); D-074 (atomic-commit-includes-JOURNAL — applied at this promotion's commit); D-NNN-ζ candidate (runbook-vs-design-doc surface enumeration discipline, flagged-not-promoted at one instance per J-129; sibling-shape at one rung up); D-NNN-η candidate (claimed-atomic-file-count vs git-actually-shipped at git-staging layer, flagged-not-promoted per J-130).

### Decision

**At every design-doc Q-table row that attributes a type or parameter to a production symbol (struct field, function signature, enum variant, type alias, public-API parameter, internal-binding shape), the symbol's DEFINITION MUST be greped at authoring time — not inferred from a call-site, annotation, sibling prose, or memory of prior production state. The grep verifies (a) the symbol exists; (b) the symbol's current shape matches the Q-row claim; (c) the line:number anchor is captured into the Q-row verbatim.**

"Design-doc Q-table row" means any row in a §2-style surface enumeration (or sibling design-doc structure) that names a production symbol by its identifier and asserts a current-state type/parameter shape, with the row's purpose being to lock the symbol as a Pass N retype target (or explicit non-retype inheritance).

"Production symbol DEFINITION" means the canonical source-of-truth location for the symbol's shape:
- For struct fields: the `pub struct Name { ... pub field: T, ... }` line in the struct's defining file (not a call-site `instance.field` access; not a local-variable `let x: T = instance.field.clone()` annotation; not prose elsewhere claiming the field's type).
- For function signatures: the `fn name(...) -> ...` line in the function's defining file (not a call-site invocation).
- For enum variants: the `enum Name { Variant { field: T }, ... }` block (not a `Name::Variant { field, .. }` destructure site).
- For type aliases: the `type Name = ...;` line (not a usage site).

"Inference from a call-site, annotation, sibling prose, or memory" — the failure modes D-079 names explicitly:
- **Call-site inference**: reading `instance.field.x()` and inferring the field's type from the method available. Fails when the type implements an unexpected trait or when the field's actual type is a wrapper around the inferred type.
- **Annotation inference**: reading a local-variable annotation `let x: Vec<String> = ...` and inferring the source's type matches. Fails when the annotation is wrong (compile error not yet surfaced; Pass-1-broken-by-Path-A intermediate state).
- **Sibling-prose inference**: reading "the runtime's federation_nodes" elsewhere in the doc and inferring the type. Fails when the sibling prose itself was authored from inference.
- **Memory inference**: trusting recall of prior production state from earlier sessions. Fails because production moves under elaboration sessions.

The verification grep at authoring time IS the discipline. The grep cost is the symbol's defining file × one line × one read. The cost of skipping the grep is the discipline-failure pattern this entry codifies.

### Three threshold instances across two distinct catch-events

D-079's promotion threshold is three independent recurrences across distinct catch-events (sibling-shape to D-077 + D-078 promotion thresholds). The three instances:

1. **J-133 Drift #1 — Q3.6 v1.0/v1.1/v1.2 original.** Design doc §2.3 Q3.6 (authored at J-127 design-close walk by Chat Claude + Joe) claimed `apply_federation_push` had a parameter `peer_node_id: &str` (destination peer). Production at xgen-node/src/federation_session.rs:202-208 has 5 parameters; no such parameter. Authored from inference against tangential `peer_node_id` references in the federation_session.rs file without greping the function's signature. **Catch-event: J-133 session-open D-078 verification of design doc §2 against production code at Pass 3 Commit 2 prep checkpoint #2.**

2. **J-133 Drift #2 — Q5.14 v1.0/v1.1/v1.2 original.** Design doc §2.5 Q5.14 (same J-127 walk) claimed `OutboundMsg.peer_node_id: String` (line 1165) — a struct field. Production: `OutboundMsg` is an enum at xgen-node/src/fanout.rs:31 with variants Event/HistoryBatch/SyncComplete and no `peer_node_id` field; line 1165 in app.rs is `peer_node_id: String` as a parameter on `pub(crate) async fn run_federation_session_post_handshake` at app.rs:1152. Authored from inference at line 1165 in production without identifying the actual owning symbol (the function above, not the enum referenced nearby). **Same catch-event as instance 1.**

3. **J-134 Finding B — Q3.6 v1.3 rewrite.** The J-133 amendment whose entire purpose was closing instances 1 + 2 introduced its own Q-row error: Q3.6 v1.3 stated "the retype lands when `SpaceState.federation_nodes: Vec<String>` retypes to `Vec<NodeXgid>` — flagged as Surface #1 Q1.1 extension." Production at xgen-core/src/space/state.rs:132: `pub federation_nodes: Vec<NodeXgid>` — already typed at Pass 1 Commit 4 (`774fe9d`, J-122 close arc, 36+ hrs before J-133 amendment). Authored from inference against the xgen-node-side federation_session.rs:248 local-variable annotation (`let federation_nodes: Vec<String> = { ... s.federation_nodes.clone() ... }`, a Pass-1-broken xgen-node compile error per Path A intermediate state) without greping the struct definition. **Catch-event: J-134 atom prep D-078 grep against `xgen-core/src/space/state.rs` per Joe's pre-load STOP-and-surface instruction.**

Three instances across two distinct catch-events (J-133 closed two within one catch-event; J-134 prep surfaces the third at a separate catch-event). The same document being wrong three times across two independent audits is stronger evidence of a durable discipline-gap than three drifts scattered across three docs — the gap survives re-authoring. That's what a promoted decision is for.

### Canonical cautionary instance — "κ binds even when authoring a κ-fix"

The J-133 → J-134 sequence is the sharpest evidence FOR D-079. J-133 was the atom whose explicit purpose was closing two κ-instances (Drift #1 + Drift #2). The J-133 amendment-author (Clair, acting at implementation kickoff) verified four production signatures before authoring (the apply_federation_push 5-param signature; the peer_id loop variable site; the OutboundMsg enum definition; the run_federation_session_post_handshake parameter set) and stated at the post-J-133 state reconcile that "production evidence was verified before J-133 authored." That statement was incomplete — the verification did NOT include a grep against the SpaceState struct definition at state.rs:132, even though the amendment text would assert a type for `SpaceState.federation_nodes`. The Q3.6 v1.3 rewrite was authored from inference against the federation_session.rs:248 xgen-node-side annotation — exactly the failure mode this entry codifies.

The lesson: D-079 binds even when authoring a D-079-fix. The amendment-author for a κ-class drift must apply κ-discipline literally to every claim the amendment text makes, including claims that feel like "background framing" rather than "the catch being fixed." The J-133 atom whose explicit purpose was closing κ-drifts introduced a new κ-drift by trusting a call-site annotation over a struct-def grep.

This is recorded as the canonical instance in D-079's narrative not because κ-discipline is unreliable, but because the failure mode is exactly the failure mode that authors-of-fixes are structurally vulnerable to: confidence in the surface being fixed displaces verification of adjacent claims. The discipline must bind at every claim, not at the focal claim only.

### Application surface — surface-driven per D-071 + prospective at next design-doc Q-table authoring

D-079 applies PROSPECTIVELY at the next design-doc Q-table authoring or amendment session. The first prospective application is Pass 3 Commit 2 production code verification at checkpoint #2 post-J-134 (Clair re-surfaces the seven surfaces by name against the post-v1.4 cleared design doc; Joe approves each surface by name; production-grounded per D-079).

The pattern extends naturally to:
- Pass 4 + Pass 5 design walks (xgen-client + AI-control docs surfaces; test fixtures + helpers).
- Future audit-design-impl arc design walks per D-071.
- Any sibling-milestone canonical-record amendment that adds or rewrites a design-doc Q-row asserting a production-symbol shape.

D-079 does NOT retroactively re-audit completed design-doc Q-tables. Existing Q-rows ship as-is; if symbol-definition drifts surface later via Commit 2-style implementation verification, those are handled per Rule 3 + D-079 (Q-table grep at re-walk time) + D-074 (atomic correction with JOURNAL discipline).

### Sibling-shape to D-077 + D-078

Same family — "no-drift-surface discipline at the canonical-record-vs-implementing-layer boundary" — at three distinct implementing layers:

| Aspect | D-077 | D-078 | D-079 |
|---|---|---|---|
| Layer | Meta-layer (audit + design phases) | Protocol-test-layer (test enumerations) | Design-doc-Q-table-layer (design-doc surface enumerations) |
| Application sites | Silent-discard / conditional-mutation / fallible-discard patterns | Test enumeration lists destined to become regression locks | Design-doc Q-table rows asserting current-state production symbol shapes |
| Question form | Bidirectional (forward-drift AND backward-coherence) | Unidirectional (production reject-path inventory vs test enumeration) | Unidirectional (production symbol definition vs Q-row attribution) |
| Origin | J-105 design phase asked forward-drift only at Q1; J-107 re-walk Y-lock + promotion | Three threshold instances J-099 + J-109 + J-113 | Three threshold instances J-133 Drift #1 + J-133 Drift #2 + J-134 Finding B |
| Failure mode | Future contributor bypass OR present cross-milestone dependency break | Test enumeration asserts against contract that doesn't exist; regression lock is fabricated | Q-row attributes a type/parameter to a production symbol that doesn't honor it; downstream Commit work builds on the wrong claim |
| Canonical cautionary | J-105 forward-drift framing | J-113 timestamp-bound forgery variants | J-133 → J-134 amendment-author re-instantiation |

All three reject the failure mode of locking a contract without asking the discipline question that would have caught the gap.

### Root-cause family note

D-077 + D-078 + D-079 + candidate D-NNN-ζ (runbook-vs-design-doc surface enumeration, flagged-not-promoted at one instance per J-129) + candidate D-NNN-η (claimed-atomic-file-count vs git-actually-shipped at git-staging layer, flagged-not-promoted at one instance per J-130) all share the family shape: **"prose claims something the implementing layer silently doesn't honor."** Each candidate names the discipline at a distinct implementing layer (audit-phase code patterns / test enumerations / design-doc Q-tables / runbook surface enumerations / git-staging). 

If a fifth distinct layer surfaces, consider a parent meta-discipline rather than continuing to spawn per-layer candidates — consolidation question flagged for Pass 5 milestone close, sibling-shape to runbook §7.10 discipline-notes consolidation flag. The parent could be framed as "no-drift-surface discipline at the canonical-record-vs-implementing-layer boundary" with per-layer named decisions as instantiations.

### Promotion-shape note

Path A (in-place rewrite-correction of Q3.6 v1.3 → v1.4 atom; promote κ to D-079 in the same atom) locked at J-134 session over Path B (Q3.6 v1.3 → v1.4 closer to v1.2 framing; defer DECISIONS promotion until fourth instance) and Path C (`git revert` of Q3.6-rewrite portion of J-133; linear-J-numbering complication with no correctness gain).

Three grounds for promotion at this atom (sibling-shape to D-078 promotion-shape note at J-114):

1. **Pattern at three instances across two distinct catch-events matches D-077 + D-078 promotion shape.** Three independent recurrences (or — sibling-shape — two recurrences with a self-referential catch where the fix-author re-instantiates the discipline-failure) is the threshold for surface-recurrence-pattern decisions without a one-per-layer constraint.
2. **J-134 atom is the natural carrier** — the atom already applies D-079 prospectively at the grep that found Finding A; promoting in the same atom keeps the principle and its first prospective application together. Sibling-shape to D-076 promotion at J-097 design close + D-078 promotion at J-114 runbook atom.
3. **Deferring promotion re-runs the discipline cost** when a sibling milestone N+1 re-discovers the pattern. The cost has been paid three times already in Pass 3 alone; codifying now preserves the lesson for Pass 4 + Pass 5 design walks.

Naming locked at "Design-doc Q-table grounded by symbol-definition grep" over alternatives "Production-grounded Q-table attribution" (less specific about WHICH production source counts) and "Symbol-definition discipline" (loses the design-doc Q-table application surface). Three grounds: (1) product-framing matches D-067/D-070/D-075/D-076/D-078 sibling style; (2) names the artifact (Q-table) AND the method (grep symbol definition) explicitly; (3) D-077/D-078 family-shape visible by parallelism without buried-in-title.

---

## D-078 — Production-grounded test enumeration

**Date**: 2026-05-24  
**Layer**: Protocol-test-layer — the discipline frame applied when authoring or amending any test enumeration that will become a regression lock for production behaviour. Sibling-distinct from D-077 (which lives at the meta-layer of how the project asks sustainability questions at silent-discard sites). D-078 lives at the layer where test contracts are authored against production reject-path inventories.  
**Spec reference**: `tasks/FEDERATION_PROPAGATION_PHASE_9_COMMIT_3B_4_IMPL.md` (Status: ACTIVE v1.0 at this commit — the first runbook to apply D-078 prospectively at its Joe-lock checkpoint #4). `tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md` (Status: COMPLETED v1.3 — survey findings with §2.4.1 production-code-verification walkthrough as the canonical example of D-078's application shape; sibling §2.6.1 walkthrough at J-109 carries the same shape). `JOURNAL.md` J-099 + J-109 + J-113 (the three threshold instances) + J-114 (this promotion). Cross-references: D-077 (bidirectional sustainability discipline — meta-layer sibling); D-076 v1 → v1.1 (sibling-shape promotion-from-recurrence-threshold pattern at J-097/J-099); D-067 (no-drift-surface code-organisation parent); D-069 (canonical-document discipline — D-078's application surface is canonical test-enumeration documents); D-071 (audit-precedes-dependent-design — D-078's surface-driven application trigger); D-074 (atomic-commit-includes-JOURNAL — applied at this promotion's commit).

### Decision

**At every test enumeration (any named list of test cases destined to become a regression lock for production behaviour), the production reject-path inventory MUST be confirmed against current code BEFORE the enumeration is Joe-locked at design or survey phase, not retroactively after implementation surfaces drift.**

"Test enumeration" means any document section that names specific test cases by name (e.g., "4 forgery variants × 5 event families = 20 tests, names X1...X20") and Joe-locks the enumeration at a survey-close or design-close or runbook-authoring close. The enumeration becomes a regression lock when implementation ships the named tests; subsequent regressions are caught at the per-test assertion shape.

"Production reject-path inventory" means the actual set of error variants, outcome shapes, validation steps, and assertion-target API surfaces that current production code exposes. Sources: enum definitions (e.g., ExchangeError variants at `xgen-core/src/message/exchange.rs:46-87`); validator dispatchers (e.g., `validate_event` at `:395+`); outcome enums (e.g., `DispatchOutcome` at `xgen-core/src/node/runtime.rs:80+`); doc-comments at production hazard sites (e.g., the F-3 drain-time approximation at `runtime.rs:529-535`).

"Confirmed against current code" means: pre-Joe-lock walk reads the production code at the assertion target and verifies each enumerated test case maps to a real, currently-present production behaviour. If a test case asserts against a contract that doesn't exist (e.g., the J-113 finding: 6-variant enumeration included `future-timestamp` + `past-timestamp` variants asserting against a timestamp-bound validation that `validate_event` does not perform), the enumeration is amended BEFORE Joe-lock, not after implementation surfaces drift.

### Three threshold instances

D-078's promotion threshold is three independent recurrences of the canonical-document-staleness-at-dependent-milestone-implementation-time pattern (sibling-shape to D-076 v1 → v1.1 promotion which used a two-recurrence threshold because the no-drift-surface family's one-per-layer shape made two-instance conflict already durable; D-078 doesn't have a one-per-layer shape so the threshold is three).

1. **J-099 (Step 2 audit-doc + design-doc §11 amendments)** — topological-sort wire-order audit doc §11 + design doc §11 were authored at J-097 design phase, but post-J-098 verification at Clair's Commit 3 surfaced a framing gap: Q3 had locked "determinism normative" without locking "causal-DAG-respecting order." The first canonical-staleness instance. Pattern not yet durable at one instance.

2. **J-109 (Phase 9 survey §2.6 amendment)** — Phase 9 survey findings v1.1 §2.6 Scenario 6 contract was authored before Phase 7.5 §6 P7.5-A/B/C/D shipped at J-094; v1.1 froze the pre-Phase-7.5 contract. Clair's Pre-Commit-3b-2-equivalent verification surfaced the staleness (production at `xgen-core/src/node/runtime.rs:514-555` emits `DispatchOutcome::HeldPending` with `disposition = "held_pending"` field; survey claimed `DispatchOutcome::Rejected` with `reason = federation_relationship_missing`). Second instance. Pattern not yet durable at two instances.

3. **J-113 (Phase 9 survey §2.4 amendment)** — Phase 9 survey findings v1.2 §2.4 Scenario 4 enumeration included 6 forgery variants; Clair's mid-Commit-3b-4 implementation surfaced that two variants (`future-timestamp` + `past-timestamp`) asserted against a contract that doesn't exist (no timestamp validation in `validate_event`) + one variant (`mutated-sender`) was under-specified. Third instance. Pattern durable at three instances.

Each instance closed via Reading B (amend canonical source FIRST, then dependent work picks up against amended contract). Each instance shipped via the same D-074 atomic-commit-with-JOURNAL discipline. The three instances together make the pattern's surface visible across multiple milestone types (audit-doc / survey / runbook-authoring) and across different layers (design-phase canonical doc; survey-phase canonical doc; mid-implementation canonical doc); not specific to one milestone shape.

### Application surface — surface-driven per D-071

D-078 applies PROSPECTIVELY at the next runbook-authoring or survey-close session that includes a test enumeration. The first prospective application is `tasks/FEDERATION_PROPAGATION_PHASE_9_COMMIT_3B_4_IMPL.md` Joe-lock checkpoint #4 — the runbook explicitly routes each of the five Commit 3b-4 test family enumerations (Scenario 4 + C5 + C7 + C9 + C10) through the same production-code verification Scenario 4 retroactively got at J-113.

D-078 does NOT retroactively re-audit completed test enumerations. Existing test files at `xgen-core/` and `xgen-node/` ship as-is; if regressions surface later via cross-crate trace assertion gaps or production-code drift, those are individual surface findings handled per Rule 3 + D-077 (sustainability question) + D-078 (production-code verification at re-walk time).

### Sibling-shape to D-077

D-077 names the bidirectional sustainability discipline at silent-discard sites (meta-layer). D-078 names production-grounded enumeration at test-enumeration sites (protocol-test-layer). Both are family-siblings: same "ask the question before locking" discipline, applied at different scopes.

| Aspect | D-077 | D-078 |
|---|---|---|
| Layer | Meta-layer (discipline frame applied during audit + design phases) | Protocol-test-layer (discipline frame applied when authoring test enumerations) |
| Application sites | Silent-discard / conditional-mutation / fallible-discard patterns | Test enumeration lists destined to become regression locks |
| Question form | Bidirectional (forward-drift AND backward-coherence) | Unidirectional (production reject-path inventory verified against test enumeration) |
| Origin | J-105 design phase asked forward-drift only at Q1; J-107 re-walk Y-lock + promotion | Three threshold instances J-099 + J-109 + J-113 |
| Failure mode | Future contributor bypass OR present cross-milestone dependency break | Test enumeration asserts against contract that doesn't exist; regression lock is fabricated |

Both decisions reject the failure mode of locking the principle without asking the discipline question that would have caught the gap.

### Promotion-shape note

Path B.i was locked at the runbook-authoring session J-114 over Path A (runbook §1.1 only; defer DECISIONS promotion until fourth instance) and Path B.ii (promote D-NNN in separate pre-runbook atom). Three grounds:

1. **Pattern at three instances matches the D-076 v1 → v1.1 promotion threshold sibling-shape.** The no-drift-surface family promoted at two instances because of the one-per-layer constraint; D-078 promotes at three because the surface-recurrence pattern is the threshold mechanism for principles without a one-per-layer shape.
2. **Runbook atom is the natural carrier** — the runbook already applies D-078 at checkpoint #4; promoting in the same atom keeps the principle and its first application together. Sibling-shape to D-076 promotion at J-097 design close shipping six files atomic.
3. **Deferring promotion (Path A) re-runs the discipline cost** when someone re-discovers the pattern at a sibling milestone N+1. The cost was paid three times already; codifying now preserves the lesson.

Naming locked at "Production-grounded test enumeration" over the alternative "Backward-coherence audit at test-enumeration time." Three grounds: (1) product-framing matches D-067/D-070/D-075/D-076 sibling style; (2) scope clarity at one read — "test enumeration" names the artifact, not the moment; (3) D-077 relationship visible by family-shape in body, not buried in title.

---

## D-077 — Bidirectional sustainability discipline at silent-discard / fallible-discard sites

**Date**: 2026-05-23  
**Layer**: Code-organisation — the discipline frame applied during audit + design phases of milestones that touch silent-discard, conditional-mutation, or fallible-operation-with-discard patterns. Binds the audit-phase + design-phase walks before any code commits; instantiates D-067's no-drift-surface posture at the meta-level of how the project asks sustainability questions.  
**Spec reference**: `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_DESIGN.md` (Status: ACTIVE v1.1 at this commit, §3 amendment subsection records the (a).iii.β → (a).iii.α revert and the bidirectional sustainability framing; §8 expanded scope of candidate D-NNN to cover all five `ingest_event` silents + three drain helpers + M6 reject paths + B3 apply_event dependency). `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_IMPL.md` (Status: ACTIVE v1.1 at this commit, §4 amendments for (a).iii.α framing + new §7.8 discipline-notes subsection). `tasks/HANDOFF_PERSISTENCE_AMENDMENT_REWALK.md` (Status: COMPLETED v1.1 at this commit, the re-walk decision record). `JOURNAL.md` J-107 (the re-walk retrospective). Cross-references: D-067 (no-drift-surface code-organisation sibling); D-069 (canonical-document discipline — candidate D-NNN flag at audit-vs-design boundary); D-071 (audit-precedes-dependent-design — D-077's application surface trigger); D-076 v1.1 (sibling-shape principle-stated → gap-surfaced → amendment pattern); Rule 0 (session-open reading discipline — sibling-shape meta-level principle originated from discipline-failure surface).

### Decision

**At every silent-discard, conditional-mutation, or fallible-operation-with-discard pattern in the codebase, the sustainability question MUST be asked in both directions before any candidate fix is locked at design phase.** The two directions are complementary, not alternatives:

- **Forward-drift question** — what future callers could bypass this site's upstream invariants and reach this site directly? What hypothetical caller a year from now (M6 admin write path, M8 federation-depth migration tool, future cold-start refactor, test-only-reachable code paths) could violate the assumption the silent-discard implicitly trusts? Forward-drift names the protection the principle offers against hypothetical future contributors.

- **Backward-coherence question** — what current callers in the codebase depend on this silent-discard as a feature? Is there an upstream call site (validator, dispatcher, test fixture, cross-milestone amendment) whose correctness implicitly relies on this site swallowing a specific error class? Backward-coherence names the protection the principle offers against breaking present-day cross-milestone semantic dependencies.

Both questions MUST be answered simultaneously before closing any single silent in isolation. Closing the forward-drift question only — the failure mode J-105 instantiated — produces a candidate fix that satisfies hypothetical future contributors but breaks a real current contract. Closing the backward-coherence question only would produce a candidate fix that preserves present cross-milestone dependencies but offers zero protection against future-contributor drift.

### Originating incident

Surfaced 2026-05-23 at Clair's Commit 2 implementation of the persistence-amendment sub-amendment milestone (`tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_IMPL.md` ACTIVE v1.0, runbook-authored at J-106). The J-105 design phase had locked Q1 at (a).iii.β — `pub fn ingest_event(&mut self, event: Event) -> Result<(), GraphError>` — under a sustainability frame that asked the forward-drift question only. The chain: initial recommendation (a).iii.α (log-level `tracing::error!` at the silent site); user's "is this future-proof?" challenge surfaced three forward-drift risks (a).iii.α doesn't catch (future caller bypasses `validate_event`; disk format change; future async-predecessor protocol revision); revised recommendation to (a).iii.β (compiler-forced caller handling at the type-system layer); user's follow-up "is (a).iii.β future-proof?" forced honesty that nothing is future-proof in absolute terms, naming rungs above (ValidatedEvent wrapper, sealed traits + visitor pattern, formal verification); resolution to lock (a).iii.β as the immediate answer + flag candidate D-NNN "ingest path invariant encoding" for future-walk per D-069 audit-vs-design boundary discipline.

At Commit 2 implementation Clair ran package-scoped verification and hit `cargo test` failure on `node::runtime::phase_7_5_tests::b3_federation_add_via_federation_skips_step_9_predecessor`. Trace: Phase 7 B3 amendment (J-088, locked 2026-05-20 at `xgen-core/src/message/exchange.rs:455-509`) explicitly skips `validate_event` Steps 9 (predecessor presence), 11 (sender registration + membership), 13 (permission) for `state.federation_add` events arriving via federation channel — the inline comment names this as "predecessor-chain deadlock" since the federation_add IS the relationship-establishing event whose own predecessors are themselves held on the Phase 7.5 federation-relationship trigger. What B3 implicitly relied on but did not name in its locked design: `graph.add_event` inside `ingest_event` returns `UnknownPrevEvent` for this event class (because `validate_event` let it through with missing predecessors), and the silent-discard at `let _ = graph.add_event(...)` swallowed it, after which `let _ = store.insert(...)` and `apply_event(...)` ran and mutated `SpaceState.federation_nodes`. **Net pre-Commit-2 behaviour: SpaceState updates correctly, but the event lands in EventStore-but-not-DagGraph — a coherence violation that B3 silently treated as acceptable.** Q1(a).iii.β's `?` propagation replaces the silent-discard with error-return-on-`Err`; `SpaceState` never updates; B3's federation-bootstrap path breaks at the SpaceState mutation layer.

Five resolution options walked with Joe at the surface point (full enumeration at `tasks/HANDOFF_PERSISTENCE_AMENDMENT_REWALK.md` §1.4): (α) exempt B3 path inside ingest_event — rejected on principle (re-introduces conditional silent-discard); (β) refactor B3 out of ingest_event — cleanest long-term separation but ~80 lines + new helper; (γ) accept regression + schedule re-design phase — maximally expensive; (δ) test-only public surface — rejected (permanent escape hatch); (ε) tristate `Result<IngestOutcome, GraphError>` with `Ok(DagSkipped)` — ~40-50 lines preserving compiler-honesty + new variant; (ζ) ship (ε) + flag broader audit as future work — ~40-50 lines + docs. Then Joe's reframing of "expensiveness = code = error-loop risk" produced Option X vs Option Y: Option X (apply bidirectional sustainability broadly to 4-7 related sites, 80-200 lines, multi-site blast radius, 2-4 cascading session-arcs); Option Y (revert (a).iii.β to (a).iii.α log-level, ~5-10 lines, near-zero new error surface, forward-drift risks return but as hypothetical future-contributor problems, not present-day concrete drift).

**Joe locked Y** on error-loop-risk grounds. The bidirectional sustainability discipline gets named at this milestone as new principle (this entry); the broader audit work it implies gets scheduled as candidate D-NNN "ingest path invariant encoding" with expanded scope; both wait for surface-driven application per D-071.

### Why this discipline must be explicit

**Reason 1 — The forward-only frame is a real failure mode, not a hypothetical concern.** J-105's walk asked the forward-drift question correctly and answered it carefully (three forward-drift risks named at three rungs of escalating sustainability). The walk did not ask the backward-coherence question at all. The resulting (a).iii.β lock was correct under the question asked; it was wrong under the question that should have been asked. Locking the bidirectional frame explicitly is the only way to ensure future design phases that touch silent-discard patterns don't repeat the same single-direction omission.

**Reason 2 — Cross-milestone B3 amendment dependency is not a one-off curiosity.** The project has a growing collection of cross-milestone amendments that lock specific behaviours in unusual code paths: B3 federation-bootstrap (J-088, 2026-05-20); F-3 held-not-bypassed posture (J-088); F-10 Identity-arrival hooks (J-082); D-075 vantage-aware applier (J-096); Path B causality-layer fix (J-099); the layered-B3 surface unification at topo-sort Commit 2a (J-101). Each amendment may implicitly rely on a sibling site's silent-discard or conditional-mutation as a feature. Closing any one such site in isolation without asking the backward-coherence question against the full set of cross-milestone amendments is the same shape of failure J-105 instantiated. Future audits that touch fallible-discard patterns must enumerate the cross-milestone amendments that could depend on the site's current behaviour before locking a fix shape.

**Reason 3 — Sibling-discipline-family alignment.** The no-drift-surface discipline family (D-067 + D-070 + D-075 + D-076 v1.1) locks no-drift properties at four protocol layers: code-organisation + transport + event-model + wire-format. D-077 operates at a different layer entirely — the *meta-layer* of how the project asks sustainability questions during audit + design phases. It is not a fifth member of the no-drift-surface family at a fifth protocol layer; it is the discipline that ensures future no-drift-surface principles are surfaced honestly when they should be (forward-drift) and don't break present cross-milestone contracts when they're locked (backward-coherence). Rule 0 has analogous shape — it operates at the meta-level (session-open reading discipline) rather than at a protocol layer.

**Reason 4 — The principle the bug revealed.** The persistence-amendment design phase assumed forward-drift was the relevant sustainability question because (a) the fallback paths under discussion concerned future-contributor risk; (b) backward-coherence "already worked" in the sense that pre-existing tests passed. The unstated assumption: that pre-existing tests exhaustively characterise present-day cross-milestone dependencies. They do not. Phase 7's B3 amendment shipped 2026-05-20 with a single load-bearing regression test (`b3_federation_add_via_federation_skips_step_9_predecessor`) at xgen-core internal-mod level; that test is sufficient to verify B3's stated behaviour in isolation but does not exhaustively characterise B3's implicit dependencies on sibling code paths like `ingest_event`'s silent-discard pattern. The bidirectional sustainability discipline says: when designing a fix to a silent-discard site, treat the silent-discard's current behaviour as a contract until backward-coherence proves otherwise, not as a defect until forward-drift requires preserving it. The default shifts: silents are contractual until characterised, not defective until characterised.

### Relationship to the no-drift-surface discipline family

D-077 sits at a meta-level above the no-drift-surface discipline family (D-067 + D-070 + D-075 + D-076 v1.1). The family members each lock a no-drift property at a specific protocol layer; D-077 locks the *audit/design-phase question* that surfaces no-drift properties honestly at every layer.

| Decision | Layer | Property locked |
|---|---|---|
| **D-067** | Code-organisation | Single source of truth for derived state reads |
| **D-070** | Transport-layer | Two events of equal importance, opposite direction (acceptance + rejection signals + envelope-level correlation) |
| **D-075** | Event-model | Relationship-shaped events record one party's act + derived projection with vantage-aware applier |
| **D-076 v1.1** | Wire-format | Two senders with identical state produce byte-identical AND causal-DAG-respecting federation deltas |
| **D-077** | *Meta* (audit/design phase) | Sustainability questions asked in both directions — forward-drift AND backward-coherence — before any silent-discard / fallible-discard fix is locked |
| **Rule 0** | *Meta* (session-open) | Mandatory session-open reading sequence: CLAUDE.md PLAY block → latest JOURNAL entry → ACTIVE HANDOFF notes → then user-pointed document |

The two meta-level principles (D-077 + Rule 0) share a common origin pattern: discipline failures that surfaced during implementation. Rule 0 originated from the post-J-098 session-open failure (narrow-pointer reading bypassed canonical-record bridges); D-077 originates from the J-105 forward-only sustainability frame (single-direction question missed cross-milestone contract). In both cases the failure surfaced a tacit expectation the project had been operating under without naming explicitly. Locking the principle is the project's mechanism for converting tacit expectations into explicit rules.

### Application scope — surface-driven per D-071

D-077 applies during audit-phase and design-phase walks of milestones that touch silent-discard, conditional-mutation, or fallible-operation-with-discard patterns. It is NOT pre-applied retroactively across the codebase. Pre-application would be exactly the Option X failure mode Joe rejected at this milestone's surface point: 80-200 lines of code change across 4-7 sites with multi-site blast radius and 2-4 cascading session-arcs.

Future audits trigger D-077's bidirectional sustainability frame when either: (a) Joe locks a walk as worth pursuing (the candidate-D-NNN promotion path per D-069), OR (b) dependent work surfaces concrete drift a single-direction frame would not catch. The trigger is concrete need, not preemptive thoroughness. The discipline is a question to ask at the audit-design boundary, not a sweep to execute against existing code.

The specific behavioural expectation at each audit-phase opening:

1. **Enumerate the silent-discard / fallible-discard sites in scope** of the milestone.
2. **For each site, ask the forward-drift question** — what future callers could bypass upstream invariants? Name the hypothetical contributors concretely (M6 admin write path, M8 federation-depth tool, etc.).
3. **For each site, ask the backward-coherence question** — what current callers in the codebase depend on this silent-discard as a feature? Enumerate cross-milestone amendments (B3, F-3, F-10, D-075, Path B, layered-B3) that could implicitly rely on the site's current behaviour. Use `grep -rn <silent_call_pattern>` from project root + audit of each call site's locked behaviour at its origin-J-NNN entry.
4. **Lock the fix shape only after both questions answered**. If forward-drift and backward-coherence point in opposite directions (J-105's case: forward says go to type-system, backward says preserve silent-as-feature), surface to Joe with both findings and let the decision happen with full information.

### Candidate D-NNN "Ingest path invariant encoding" — expanded scope at this re-walk

Flagged at J-105 design doc §8 as a future-walk question; scope expanded at this re-walk Track 1 commit to cover the full set of fallible-discard sites in the ingest-path family. The expanded scope:

- **Five `ingest_event` silent-discard sites** (`xgen-core/src/node/runtime.rs:~190-230`): event_id-missing-return at line ~190; the `graph.add_event` site at line ~210 (closed at this milestone under (a).iii.α + verbatim code-comment block); `store.insert` silent at line ~212; two `apply_event` silents at lines ~221 + ~228 (StateSpaceCreate replay loop + default branch).
- **Three drain helpers' silent-Accepted-discards**: `drain_pending_uniform` line ~670; `drain_pending_by_identity` line ~745; `drain_pending_by_federation_relationship` line ~795. Each uses `let _ = self.dispatch_event(ev, origin, None);` swallowing the Accepted outcome's persistence implications.
- **M6 reject paths**: any silent error-swallow inside `app.rs::process_inbound` reject branches that future M6 admin write paths might depend on.
- **Phase 7 B3 apply_event dependency**: B3's implicit reliance on the `graph.add_event` silent surfaced this milestone; future walks must enumerate analogous cross-milestone dependencies before any of the above sites are touched.

The scope expansion is not a commitment to walk all sites in one milestone. The expansion makes the walk's eventual surface area visible so that future Joe-lock conversations have the full set to weigh against milestone-scoping discipline. Promotion of candidate D-NNN to D-NNN happens when: (a) Joe locks the walk as worth pursuing, OR (b) dependent work (M6 admin write path, M8 federation depth, future cold-start refactor) surfaces a concrete drift instance log-level vigilance does not catch.

The verbatim code-comment block at `xgen-core/src/node/runtime.rs:181` (shipped at Clair's `f4f0e4e` Commit 2 under (a).iii.α) names this future-walk explicitly so a contributor reading the touch-site finds the candidate D-NNN flag in context, not only in remote JOURNAL entries.

### Out of scope for this decision

D-077 does not promote spec-normative Ch3 statements about silent-discard handling. The DECISIONS.md entry is the canonical lock for the *discipline question*; spec-level promotion of any specific fix shape (type-system Result-returning, ValidatedEvent wrapper, sealed traits + visitor pattern, formal verification) is separate doc-pass work if a future contributor lands one of the rungs above (a).iii.α.

D-077 does not enumerate every silent-discard site in the codebase. The enumeration above is the persistence-amendment milestone's scope; future milestones that touch sibling sites (other crates' silent-discards, transport-layer error-swallows, identity-replication reject paths) enumerate their own scope at their own audit phase.

D-077 does not bind retroactive code change. The B3 amendment stays as-is; the (a).iii.α revert + verbatim code-comment block at runtime.rs:181 is the milestone's substantive touch. Future Joe-locks may revisit the rungs above (a).iii.α if dependent work demands it; D-077 itself is the principle, not the implementation.

### Sibling-shape lessons

D-077's origin pattern — principle stated at design phase → implementation surfaced cross-milestone gap → amendment makes the missing dimension explicit — is structurally identical to:

- **D-076 v1 → v1.1** at J-099 Step 2: v1 stated byte-identical determinism; implementation surfaced ~50% pass rate vs nonce-rolls revealing the v1 contract said nothing about causality; v1.1 amendment added causal-DAG-respecting as second load-bearing property.
- **Rule 0 at J-099 Step 2**: tacit session-open expectation existed; J-098 narrow-pointer reading bypassed the bridges; Rule 0 made the session-open sequence permanent project discipline.
- **D-075 at bidirectional milestone J-096**: tacit assumption that `state.federation_add` was a two-sided relationship object; implementation surfaced vantage-asymmetry; D-075 promoted the event-as-one-party's-act framing.

Four project instances of the pattern (D-075 + D-076 v1.1 + Rule 0 + D-077) make principle-stated → gap-surfaced → amendment a durable shape. Future contributors reading any of these four decisions in isolation see the pattern; reading them together surfaces the meta-pattern that the project converts surfaced discipline-failures into named principles rather than papering over them.

### Status at promotion

Promoted to DECISIONS.md as a single new entry (no in-place amendment to a prior D-NNN; D-077 is a new principle, not an amendment of an existing one). Track-1-while-Clair-active first project instance — allowed at this commit because Track 1 is record-of-already-locked-decision (Y-lock made in Joe's conversation with Clair at session close before this Track-1 commit was authored), not decision-input. Directional asymmetry from topo-sort J-099/J-100 Track-1-as-decision-input precedent recorded in JOURNAL J-107 discipline-notes sub-section.

Application is surface-driven per D-071. Candidate D-NNN "ingest path invariant encoding" (expanded scope at this entry) is the first scheduled future-walk under D-077's discipline. No additional code-touch at this commit beyond the DECISIONS.md entry + sibling canonical-record amendments (design doc §3 + §8; runbook §4 + new §7.8; JOURNAL J-107; CLAUDE.md header; ROADMAP.md v1.21 → v1.22; HANDOFF Status flip). Per D-074 same-commit discipline (tenth instance).

---

## D-076 — Wire-order determinism is a sender-side normative property for Node-to-Node federation

**Date**: 2026-05-22  
**Layer**: Wire-format — sender-side serialisation of federation event streams. Binds every code path that produces federation wire output where two senders with identical state could otherwise produce different orderings.  
**Spec reference**: `docs/xgen_federation_propagation_design.md` (canonical Federation Event Propagation design, §6.4.3 added by implementation runbook Commit 1); `tasks/FEDERATION_TOPOSORT_AUDIT.md` (Status: COMPLETED v1.0 at this commit, code-grounded mechanism at §3, three Joe-locks recorded inline at §6); `tasks/FEDERATION_TOPOSORT_DESIGN.md` (Status: ACTIVE v1.0 at this commit, locked-principle exposition at §7). Cross-references: D-067 (code-organisation layer sibling); D-070 (transport-layer sibling); D-075 (event-model layer sibling); D-069 (canonical-document rule); D-071 (audit precedes dependent design); D-074 (milestone-close commits include JOURNAL).

### Decision

**Wire-order determinism is a sender-side normative property for Node-to-Node federation.** Two senders with identical Space history MUST produce byte-identical federation deltas (modulo signature-bearing fields that vary by author and time). Wire ordering is part of the protocol's contract, not implementation latitude.

Forward-bound by Q2.γ to Node-to-Client sender output where analogous and should be reviewed when scheduling allows.

### Amendment (2026-05-22) — Causal-DAG-respecting order as second load-bearing property

**The canonical wire order must satisfy two complementary properties, not one.** D-076 v1's stated contract (byte-identical wire output across senders with identical state) was necessary but not sufficient. At implementation, the Shape A v1 sort fix (event_id lexicographic tie-break at the topo primitive) was found to produce byte-identical output across senders yet still cascade the receiver's bootstrap chain ~50% of the time. The framing gap: the v1 contract did not name what semantic property the chosen canonical order must satisfy.

The amended principle, in full:

> *Wire-order determinism is a sender-side normative property for Node-to-Node federation. Two senders with identical Space history MUST produce wire output that is BOTH (a) byte-identical modulo signature-bearing fields AND (b) causal-DAG-respecting. Property (b) is the load-bearing property: a deterministic-but-non-causal wire order is semantically broken for the receiver; property (a) is the supporting property: it preserves the no-drift-surface posture across senders. Both must hold.*

**Why one principle, not D-076 + D-077.** The two properties cannot vary independently in a useful way:
- A causal-but-non-deterministic wire format would let senders agree on causal ordering but produce different byte streams from identical state — the cross-Node debugging benefit (Reason 3 below) disappears; MLS coupling (Reason 2) still bites at the application layer.
- A deterministic-but-non-causal wire format is exactly what Shape A v1 produces in isolation: byte-identical across senders, semantically broken because the receiver's dispatch pipeline can't process child events before parent events have been ingested.

Neither half is useful alone. Splitting into D-076 + D-077 was considered and rejected at J-098 session close on two grounds: (1) the two halves are complementary aspects of the same thing, not separable principles; (2) the no-drift-surface discipline family's one-per-layer shape (D-067 + D-070 + D-075 + D-076 across code-organisation + transport + event-model + wire-format) would be broken by adding a second wire-format-layer decision.

**Locked instantiation: Path B + Commit 2 sort fix, layered.** The amended principle commits the implementation to two coordinated surfaces:

- **Path B (causality layer, D-076 v1.1 instantiation).** `build_room_create_event` at `xgen-core/src/space/state.rs:797` gains `prev_events: vec![space_id.to_string()]` so the event-DAG honestly reflects the protocol-level parent-child relationship the function's own doc-comment already claims. `state.room_create` becomes a non-root event whose predecessor is `state.space_create`; the topological sort places it after the parent regardless of tie-break logic; the receiver's `dispatch_event` Step 1 finds the Space when it processes `state.room_create`.
- **Commit 2 sort fix (determinism layer, D-076 v1 instantiation, already shipped).** event_id lexicographic sort at `topological_sort_events:193` + sibling sort at `compute_federation_delta_for_space:321`. Stays useful as the safety net for events that legitimately tie at the DAG layer (true roots, true siblings with no protocol-level ordering constraint).

The two fixes layer cleanly: causality first (Path B at the DAG-construction layer), determinism second (Commit 2 at the tie-break layer). The Commit 2 sort fix is **not reverted**; it remains the protective discipline for any case where two events genuinely tie at the DAG layer. Path B is the substantive fix that closes Phase 9 Scenario 1.

**Path B scope is narrow by Joe-lock.** `build_room_create_event` only. Sibling event constructors (`state.federation_add`, `membership.*`, `message.*`, etc.) are NOT audited in this milestone for similar `prev_events` lies; that audit may surface later as its own audit-precedes-dependent-design arc per D-071 if dependent work surfaces a need. The scope is deliberately bounded to the single constructor surfaced by Phase 9 Scenario 1's failure; a fuller audit of every event constructor's `prev_events` shape against its doc-comment claims would be its own substantial subsystem audit phase.

**Binding D-076 v1.1 creates.** Future event-design Joe-locks must include two design-phase questions, not one: (i) does this event's serialisation produce canonical wire ordering across senders? (ii) does the event-DAG honestly reflect every protocol-level ordering constraint this event participates in? Question (ii) is the framing gap this amendment closes; future design phases that ask (i) only will produce the same shape of bug this milestone surfaced.

**Origin story.** Surfaced 2026-05-22 at Clair's Commit 3 verification, post-J-098. The Shape A v1 sort shipped at Commit 2 satisfied D-076 v1's stated contract but did not close Phase 9 Scenario 1. The ~50%-pass-rate vs nonce-roll pattern revealed the unstated assumption in Q3's framing: that any deterministic canonical order would also be a causally-correct order for the receiver's dispatch pipeline. Path B locked at J-098 session close; this amendment promotes the framing to the canonical record. Sibling-shape to how D-070 / D-075 / D-076 v1 originated from earlier surface moments — discipline failures that surface are the project's mechanism for converting tacit expectations into explicit rules.

*Previous v1 "Decision" prose above (paragraphs starting "Wire-order determinism is a sender-side normative property") stays authoritative as the original lock record. The amendment extends without rewriting; the v1 statement is preserved as a historical record of what the principle stated before the implementation-time surface.*  

### Originating incident

Surfaced 2026-05-21 during Phase 9 Scenario 1 verification of the bidirectional `federation_nodes` fix (JOURNAL J-096 Finding 2). The bidirectional fix shipped correctly; the Scenario 1 flake was a separate pre-existing bug: `topological_sort_events` at `xgen-node/src/fanout.rs:193` preserved input-vector order when tie-breaking ready siblings (events with all predecessors already emitted, including DAG roots with empty `prev_events`). Its caller `compute_federation_delta_for_space:321` fed it via `store.values().cloned().collect()` — `EventStore` is `HashMap<String, Event>` with randomized iteration per instance. Two `xgen-node` processes with identical Space state produced different federation-delta wire orderings ~50% of runs. When `state.room_create` (DAG root, empty `prev_events`) won the race against `state.space_create` (also DAG root), B's `dispatch_event` Step 1 rejected with "space not found"; cascading rejections produced 2 Accepted / 2 Rejected / 101 HeldPending vs the passing-run baseline of 102 Accepted / 3 HeldPending.

The gap was not caught by the original Federation Event Propagation design phase (Phase 3 R4 locked cross-Space ordering by `space_id` for determinism but was silent on within-Space ordering). Within-Space ordering was assumed-handled by the topo-sort primitive; the primitive's silent input-order-preservation contract was not load-bearing in any test before Phase 9 Scenario 1 exercised the full bootstrap delivery path with two real `NodeRuntime` instances end-to-end.

Sibling function `topological_sort` in `xgen-core/src/node/runtime.rs:859-912` (used for in-process ordering, separate code path) uses Kahn's algorithm with explicit `queue_vec.sort()` for stable tie-breaking. The xgen-node-side delta function did not. The drift surface between the two implementations was the D-067 instance D-076 generalises.

### Why this discipline must be explicit

**Reason 1 — D-067 wire-format analogue.** The project has consistently locked no-drift-surface properties explicitly rather than trusting them to emerge from local primitives. D-068's five-site CLI Audit closure, M5's 13-verb consolidation, D-070's two-events-with-correlation, D-075's vantage-aware applier all instantiate the same posture. A wire-format-determinism property fits the same family; locking it explicitly is in keeping with the rest of the project's discipline.

**Reason 2 — MLS coupling.** Ch3 §3.10 + D3 parallel-workstream milestone require canonical wire ordering at the application layer. Locking the alternative (per-receiver-deterministic tie-break is sufficient; wire ordering is implementation latitude) would surface this as a late-stage discovery, exactly the shape D-071 audit-precedes-dependent-design was created to prevent.

**Reason 3 — Cross-Node debugging benefit is immediate, not forward-only.** "Do these two senders' deltas match byte-for-byte?" becomes a yes/no question available from today, not from MLS landing. Operators investigating federation incidents can compare byte streams across Nodes; deltas that differ are evidence of state divergence, not implementation noise.

**Reason 4 — The principle the bug revealed.** The Federation Event Propagation milestone's locked design assumed wire-order determinism would emerge naturally from the topological-sort primitive. It does not, when events tie. The unstated principle: **all sender-side code paths that produce wire-visible ordering must be canonical, not merely correct.** "Correct" topological sort respects causality. "Canonical" topological sort additionally produces byte-identical output for byte-identical input sets across runs and instances. The federation delta path requires the latter; the current primitive provided only the former. D-076 promotes the principle to a project-wide discipline so future contributors do not silently drift back toward the implicit-canonicality reading.

### The no-drift-surface discipline family

D-076 joins D-067 + D-070 + D-075 as the four-decision no-drift-surface discipline family. Each member locks a no-drift-surface property at a different layer:

| Decision | Layer | Property locked |
|---|---|---|
| **D-067** | Code-organisation | Single source of truth for derived state reads (no two readers consulting different sources for the same logical question) |
| **D-070** | Transport-layer | Two events of equal importance, opposite direction (acceptance + rejection signals both exist + both carry envelope-level correlation) |
| **D-075** | Event-model | Relationship-shaped events record one party's act + derived projection with vantage-aware applier logic |
| **D-076** | Wire-format | Two senders with identical state produce byte-identical federation deltas |

The four decisions operate at different layers and address different questions, but share a common posture: **lock the no-drift property explicitly at the layer where it's load-bearing today; forward-bind to sibling surfaces; reject the alternative of leaving the property implicit and trusting it to emerge from local primitives.**

### Forward-binding to Node-to-Client siblings (Q2.γ)

D-076 is scoped to Node-to-Node federation today, with explicit forward-binding to Node-to-Client sender output where analogous. The two known Node-to-Client analogue sites are:

- `xgen-node/src/fanout.rs::collect_sync_history` — client-to-Node `sync_request` flow; same `HashMap.values()` feed pattern.
- `xgen-node/src/fanout.rs::apply_fanout` history-push — Node-to-Client history delivery; same `HashMap.values()` feed pattern.

Neither is fixed in the topological-sort milestone; both are flagged in `tasks/FEDERATION_TOPOSORT_AUDIT.md` §5.2 + `tasks/FEDERATION_TOPOSORT_DESIGN.md` §4 as Q3.ii-analogues. Future Chat Claude + Joe revisiting either site picks up D-076 directly; the principle does not need re-litigation at the analogue's design phase. "Where analogous" means the site produces sender-side wire output whose ordering could otherwise differ across senders with identical state — a structural property, not a policy choice.

### Binding D-076 creates

Future event-design Joe-locks must include "does this event's serialisation produce canonical wire ordering across senders" as a design-phase question. That cost is deliberate, not incidental — it ensures that the next time a protocol-event family is added, the wire-order question is surfaced at design time rather than discovered at integration-test time (which is how D-076 itself surfaced).

D-076 is the first D-NNN to lock a wire-format-normative property explicitly. Future wire-format properties (e.g., canonical JSON serialisation order; canonical UTF-8 normalisation; canonical timestamp precision) layer on D-076 cleanly if they are ever needed.

### Implementation under D-076 (locked at this commit)

The topological-sort design phase locked Shape A v1 + sibling Site 1 fix as the canonical realisation of D-076:

- `topological_sort_events` at `xgen-node/src/fanout.rs:193` gains an `events.sort_by(|a, b| a.event_id.cmp(&b.event_id));` line at the top of each outer-loop iteration. Ready siblings emit in lexicographic event_id order.
- `compute_federation_delta_for_space` at `xgen-node/src/fanout.rs:321` sorts the `Vec<Event>` before passing to the primitive. Belt-and-braces: explicit canonical-ordering chain end-to-end.
- Code-comment block at the sort site cites D-076 + Appendix J's content-hash framing (verbatim shape at `tasks/FEDERATION_TOPOSORT_DESIGN.md` §5.3).
- Pass-1 posture: v1 — `&str` sort + comment block flagging Pass 3 retype to `EventXgid`. Pass-1-neutral.

Implementation runbook (`tasks/FEDERATION_TOPOSORT_IMPL.md`, to be authored next session) carries the four-commit Clair sequence.

### Rejected alternatives

The full rejected-alternative reasoning lives in `tasks/FEDERATION_TOPOSORT_DESIGN.md` §6. Summary: Q3.i (per-receiver determinism sufficient) was rejected on grounds of MLS coupling + no-drift-surface discipline family alignment + immediate cross-Node debugging benefit. Q2 wide (close every wire-visible-ordering site in one pass) was rejected on milestone-scoping grounds. Q2 narrow (close only the federation delta path with no forward-binding language) was rejected on discipline-pattern-consistency grounds. Shape B (timestamp sort) was disqualified by Q3.ii (wall-clock non-canonical across senders). Shape D.2 (`IndexMap` insertion order) was disqualified by Q3.ii (insertion order non-canonical across Nodes). Shape A v2 (typed `EventXgid` from outset) was rejected on Pass-1 coupling grounds. Shape C (canonical-event-bytes sort) was rejected on cost-vs-benefit grounds (Shape A's ordering ≈ Shape C's for distinct event_ids; Shape C's additional benefit concentrated in a hypothetical duplicate-event_id edge case the protocol does not currently emit). Shape D.1 (BTreeMap at EventStore) was rejected on milestone-scoping grounds (right home is a separate `EventStore` canonical-iteration discipline milestone if ever scheduled).

### Out of scope for this decision

D-076 does not promote Q3.ii to a spec-normative Ch3 statement. The DECISIONS.md entry is the canonical lock; spec-level promotion is separate doc-pass work if a future contributor needs spec-level reference. D-076 also does not bind Node-to-Client sender output today — the forward-binding language flags the principle's applicability without scheduling the work. Both are deliberate scope choices to keep the topological-sort milestone tightly focused on the federation surface where the bug surfaced.

### Status at promotion

Design-phase Joe-locks (Q3.ii + Q2 middle + Q2.γ + Q1 Shape A v1) landed at this commit. Audit doc Status flipped ACTIVE → COMPLETED v1.0 in the same commit per the bidirectional precedent (audit doc's role as input to design-phase deliberation ends as the design task file lands the canonical record). Design task file at `tasks/FEDERATION_TOPOSORT_DESIGN.md` Status: ACTIVE v1.0; flips COMPLETED in implementation runbook Commit 1 per the bidirectional precedent. Implementation runbook authoring is the next-active step for Chat Claude + Joe in a fresh session.

---

## D-074 — Milestone-close commits MUST include JOURNAL.md

**Date**: 2026-05-21  
**Layer**: Cross-cutting — project-management discipline applying to every milestone-close commit across the project's history and future. Binds the commit-formation discipline of any milestone whose closure triggers cross-doc state changes (CLAUDE.md PLAY block flip, ROADMAP.md state move, per-task-file Status flip).  
**Spec reference**: `JOURNAL.md` (the contemporaneous record this decision protects); `CLAUDE.md` Rule 4 ("Write the journal entry last" — the sibling per-session discipline this decision generalises into commit-level discipline). Cross-references: D-069 (canonical-document rule — JOURNAL.md is the canonical historical record); D-071 (audit-precedes-dependency — sibling project-management principle); D-065 (honest behaviour over polite behaviour — a milestone closing without a JOURNAL entry is dishonest about how the project got here).

### Decision

**Every milestone-close commit's changed-files list MUST include `JOURNAL.md`.**

No milestone closes without a JOURNAL entry shipped in the same commit as the cross-doc updates that announce the closure (`Status: ACTIVE → COMPLETED` header flips on task files; CLAUDE.md PLAY block updates; ROADMAP.md Past/Present/Near future moves; Visual structure tree updates). The JOURNAL entry is contemporaneous — it describes what shipped, in what order, with what test count delta, with what structural findings surfaced, in the moment the closure happens. Deferring the entry to "a future session" or "a separate housekeeping pass" violates this discipline.

The rule is unconditional: it applies to every milestone close, including small or routine ones (a single-phase milestone, a doc-pass milestone, a sub-question lock that closes a design phase). Size of the milestone does not matter; what matters is the closure event itself producing a contemporaneous record.

### Originating incident

Discovered 2026-05-20 during XGID Adoption v1 Phase 2 close-out, via working-tree forensics. Federation Event Propagation Phase 7.5 implementation milestone shipped 2026-05-20 in five commits (`12cfe5a` + `aa2433f` + `1be7189` + `ecbbf19` + `8859093`) without a JOURNAL.md entry. The cross-doc references in CLAUDE.md and ROADMAP.md named the entry "J-094" — but no J-094 was ever authored. The discrepancy was caught when J-094 was supposed to be the originating context for closing out adjacent work, and a `grep` for `J-094` in JOURNAL.md returned zero hits.

The gap was honest-flagged in the next milestone's close entry (J-095, XGID Adoption v1 implementation close) per D-065 honest-provenance discipline rather than retroactively backfilling J-094, which would have violated D-065 by misrepresenting when the entry was written. The retrospective J-094 entry is now tracked in the Discipline / JOURNAL hygiene cluster in ROADMAP.md as deferred work ("JOURNAL Gap 1 — Phase 7.5 implementation retrospective entry"), to be written in a separate session and given the next available J-number at that time.

The incident surfaced a structural gap in the project's commit-formation discipline: CLAUDE.md Rule 4 says "Write the journal entry last" within a session, but the rule was silent on whether the entry is *in the same commit* as the cross-doc updates or in *a follow-on commit*. The Phase 7.5 close split the JOURNAL entry off as follow-on intent, and the follow-on never happened. D-074 closes the gap by making the same-commit requirement explicit.

### Why this discipline must be explicit

**Reason 1 — JOURNAL is the only contemporaneous record.** CLAUDE.md, ROADMAP.md, task file Status headers, and DECISIONS.md all describe *current* reality — what is true *now*. They get updated as state changes and they describe present state, not history. JOURNAL.md is the only file in the project that records *how reality got here* — the sequence of events, the test count deltas, the structural findings, the sub-question locks made during the work. Without a contemporaneous JOURNAL entry, a milestone close becomes archaeology to reconstruct later. The longer the gap between the close and the entry, the more accuracy decays.

**Reason 2 — Same-commit discipline prevents the gap.** A JOURNAL entry written "in a follow-on commit" relies on someone remembering to write it and the project's commit-formation discipline being attentive enough to land it. Both fail in practice. The Phase 7.5 incident is the worked instance: the follow-on intent was honest at the moment of the close, but no follow-on commit happened. Making the entry part of the close commit removes the gap surface entirely.

**Reason 3 — The forensics cost of missing entries is asymmetric.** Catching a missing JOURNAL entry months later requires `git log --all --grep`, working-tree forensics, cross-checking CLAUDE.md and ROADMAP.md references for J-numbers that don't exist, and re-deriving the milestone's actual state from commit diffs. Writing the entry at the close costs ~10–20 minutes of authoring time. The cost ratio is roughly 1:10 in favour of writing-at-close. D-074 makes the cheaper path mandatory.

**Reason 4 — The principle generalises the per-session Rule 4.** CLAUDE.md Rule 4 says "Write the journal entry last" within a session: do the work → run verification → confirm outputs → write the journal entry quoting actual output → update CLAUDE.md → commit and push. Rule 4 binds the *per-session ordering*. D-074 binds the *per-commit composition*. Together they form the full discipline: the entry is written last in the session, AND it ships in the same commit as the closure-announcing updates.

### Worked instances at promotion

- **XGID Adoption v1 implementation milestone close (J-095, 2026-05-20).** The first close to follow D-074 pre-emptively (before D-074 itself was promoted). The milestone-close commit shipped JOURNAL.md (J-095 entry) alongside CLAUDE.md (PLAY block flip + header), `docs/ROADMAP.md` (Past gain + Present + Near future moves + header), `tasks/XGID_ADOPTION_IMPL.md` (Status: ACTIVE → COMPLETED v1.1), and `docs/xgen_ch4_implementation.md` (one-line follow-on pointer per Phase 2 sweep A5 Joe-lock). Five files in one atomic commit; JOURNAL.md was among them; the discipline held.
- **XGID Adoption v1 Phase 2 doc-tree sweep close (no separate J-number, ride-along on the same commit as J-095).** Sub-milestone close within the larger XGID Adoption v1 work. The classification table at `tasks/XGID_DOC_SWEEP.md` flipped Status: ACTIVE → COMPLETED v1.2 in the same commit as the J-095 entry. D-074 tolerates ride-along closures — a single JOURNAL entry covering multiple sub-milestone closes in the same commit set is honest, provided the entry names all the closures.
- **Phase 7.5 implementation milestone close (counter-instance).** Shipped 2026-05-20 in five commits without a JOURNAL.md entry. Surfaced the gap D-074 closes. The retrospective entry, when written, will be the worked example of "how to backfill honestly per D-065" rather than "how to close a milestone per D-074" — the entry will name itself as retrospective and acknowledge the original commit-formation discipline failure rather than pretending to be contemporaneous.

### Out of scope for this decision

- **Mid-milestone JOURNAL entries.** D-074 binds *milestone-close* commits specifically. Mid-milestone entries (a long-running milestone with multiple JOURNAL entries across its phases, like Federation Event Propagation's J-082..J-089 series) are not bound by D-074 — each individual phase close gets its own entry per the existing pattern, and D-074 confirms the requirement at the *milestone-level* close (the commit that flips the milestone's overall Status from PLAY to DONE).
- **What goes IN the JOURNAL entry.** D-074 binds the requirement that an entry exists; it does not prescribe the entry's content shape. Each project area has established conventions (Federation phase closes name the Joe-locks and structural findings; XGID closes name the v1 invariance test outcomes and carry-overs; M-series closes name the test count delta and commit chain). The entry content is the milestone author's responsibility, not D-074's mandate.
- **Retrospective entries (D-065 territory).** When a missing-entry gap is discovered after the fact, the retrospective entry is written under D-065 honest-provenance discipline rather than D-074. D-074 applies forward only: the rule says new closes ship JOURNAL.md in the close commit. Past gaps stay flagged in the Discipline / JOURNAL hygiene cluster until separately retrospected. Backdating retrospective entries to make them look contemporaneous would violate D-065.
- **JOURNAL entry numbering.** D-074 does not bind J-number allocation. The convention (sequential J-NNN per chronological order of writing) is established elsewhere; D-074 only requires that an entry exists in the close commit, regardless of its number.
- **Other documentation files in the close commit.** D-074 is specifically about JOURNAL.md. Other files that go in milestone-close commits (CLAUDE.md, ROADMAP.md, task files, Ch3/Ch4 if affected) are governed by their own conventions (CLAUDE.md / ROADMAP.md same-commit discipline per ROADMAP.md's own update-discipline section; task file Status headers per the header convention). D-074 adds JOURNAL.md to that list, not as a replacement.

### Relationship to other decisions

| Decision | Relationship |
|---|---|
| D-069 | Canonical-document rule. JOURNAL.md is the canonical home for contemporaneous historical record — "this is what happened, in this order, on this date." D-074 enforces that the canonical record is *populated* at every milestone close. Without D-074, the canonical record can become silently incomplete; with D-074, the canonical record's completeness is a commit-formation invariant. |
| D-071 | Sibling project-management principle. D-071 says audits precede dependent milestones (verify reality before locking design); D-074 says milestone closes produce contemporaneous record (verify reality has been captured before declaring the milestone closed). Both decisions take implicit gaps out of the project's information state: D-071 between assumed and verified subsystem behaviour; D-074 between announced closure and recorded closure. |
| D-065 | Sibling principle (honest behaviour over polite behaviour). A milestone close commit that doesn't include JOURNAL.md is dishonest in two ways: (1) it announces closure without providing the contemporaneous record that justifies the announcement; (2) it leaves the project's future readers without the context needed to understand how the closure happened. D-074 takes that dishonesty out structurally by making the JOURNAL entry a commit-formation requirement, not a follow-up intent. |
| D-070 | Adjacent protocol-design analogy. D-070 says both sides of an outcome (acceptance / rejection) get equal first-class signals AND envelope-level correlation. D-074's analogue: both the announcement of closure (CLAUDE.md / ROADMAP.md / Status flips) and the record of how closure happened (JOURNAL.md) get equal first-class commit-formation status. The asymmetric historical case (Phase 7.5 close, announcement without record) is exactly the shape D-070 prohibits at the protocol layer, applied to the project-management surface. |
| D-072 / D-073 | XGID Adoption v1 (the worked predecessor where D-074 was already applied pre-emptively). The XGID v1 milestone-close commit (J-095, 2026-05-20) shipped JOURNAL.md as part of its five-file changed-files list per the candidate D-074 framing flagged in J-094 cleanup. D-074 formalises the discipline that XGID v1 close already followed. |
| CLAUDE.md Rule 4 | Per-session sibling discipline. Rule 4 binds *intra-session* ordering: do the work → verify → quote real output → write the journal entry → update CLAUDE.md → commit and push. D-074 binds *commit-level composition*: the journal entry ships in the same commit as the closure announcements. Together they form the full discipline; either alone is insufficient. Rule 4 with follow-on JOURNAL intent (without D-074) is what produced the Phase 7.5 gap. D-074 with bad intra-session ordering (without Rule 4) would produce hastily-written entries that don't capture actual verification output. Both rules are load-bearing. |
| Phase 7.5 implementation milestone (originating incident) | The five-commit Phase 7.5 close (`12cfe5a` through `8859093`, 2026-05-20) shipped without a JOURNAL entry. The gap was caught via working-tree forensics during XGID Adoption v1 Phase 2 close-out the same day. The retrospective J-094 entry stays deferred in the Discipline / JOURNAL hygiene cluster in ROADMAP.md until written, at which time it gets the next available J-number and is honestly labelled as retrospective. |

---

## D-061 — Room temperature: protocol carries the signal, plugin owns the math

**Date:** 2026-05-15  
**Layer:** Ch1 philosophical framing; Ch3 §3.7.13 (protocol surface); Ch6 §6.12 (client display); D-060 (pacing rules — the input signal feeding temperature)
**Spec reference:** Ch1 "Visible Self-Correcting Feedback"; Ch3 §3.7.13 Temperature Property (meta_atts keys, threshold table, visibility setting); Ch3 §3.7.8 (`auto_temperature` reason value); Ch6 §6.12 (DOM contract and rendering); D-059 (`is_ai` informs the asymmetric escalation recommendation)

### Decision

A Room carries a numeric **temperature** signal — one float per Room (collective rhythm) and one float per Member-in-Room (individual accumulated heat). The protocol surface for temperature is intentionally minimal:

- Two `meta_atts` keys: `xgen.room_temperature` and `xgen.member_temperature`, both floats in `[0.0, 1.0]`, both published by the Room's home Node
- A `temperature_thresholds` field in the Room metadata response, declaring at which float values the named states (`warm`, `hot`, `fiery`) begin; default thresholds documented in Ch6 §6.12.2
- A `member_temperature_visibility` field on Space state with three permitted values (`moderator`, `everyone`, `self_only`) controlling who receives `xgen.member_temperature` for other members
- The reserved `auto_temperature` reason value on `membership.kick` (humans) and `membership.mute` (AI) for automated consequences

**The mathematical model is not part of the protocol.** How the temperature value is computed — what decay function applies, how pacing overpasses accumulate, what the action thresholds are — is the responsibility of a plugin running on the Room's home Node. Different communities will moderate at different rhythms; the protocol has no business choosing on their behalf.

The protocol's job is to:

1. Carry the signal (the floats) across the network so every client renders the same value
2. Carry the bucket thresholds so every client classifies the same float into the same named state
3. Recognise the consequences (`auto_temperature` reason on kick / mute) so DAG audit shows what happened and why

The plugin's job is to decide what the float is at any given moment and when to issue automated consequences. The two domains are deliberately separated.

### Why

The original draft of D-061 (replaced by this rewrite) specified a mathematical model: a linear-decay heat accumulator with named thresholds and per-Space configurable parameters. The conversation of 2026-05-15 pushed back on this layer by layer, in the right direction: each round of design pulled mathematical content further out of the protocol surface until the protocol carried nothing but the *signal* and the *consequences*. The reasoning sequence was:

1. The protocol shouldn't mandate one model — different communities need fundamentally different moderation curves
2. A named-policy enum was considered (`linear_decay`, `exponential_decay`, `sliding_window`) and rejected as still too prescriptive
3. A pluggable algorithm via WASM module was considered and noted for a future phase
4. The final cut: the protocol does not announce a model at all. The home Node computes a number; the plugin behind the Node decides how. The float is the wire-level truth.

This matches the rest of the protocol's design language:

- Auth Tiers (D-037a) — protocol carries the marker, Auth Module supplies the verification meaning
- `meta_atts` (Ch3 §3.1.3) — protocol carries the bytes, applications supply the interpretation
- Vanilla Node `capabilities` (CLAUDE.md) — protocol carries the field, Nodes supply the behaviour
- Pacing rules (D-060) — protocol carries the cap, communities supply the culture

Temperature joins this list. It was the odd one out in the original draft — the only decision specifying a concrete mathematical model inside the protocol surface.

### Asymmetric escalation (AI mute vs human kick)

The asymmetric escalation principle — human members get `membership.kick` at sustained overpass, AI members get `membership.mute` — is preserved as a **recommendation for plugin authors**, not a protocol mandate.

The protocol provides the structural primitives that make the asymmetry expressible:

- `membership.kick` and `membership.mute` are distinct EventTypes (3.7.8)
- `is_ai` is observable on every Identity (D-059, 3.6.10.1)
- `auto_temperature` is reserved on both events with documented expected pairing (kick for humans, mute for AI)

A plugin that uses this asymmetry will produce the AI-keeps-membership / human-gets-cooldown behaviour described in Ch1 §"Visible Self-Correcting Feedback". A plugin that ignores the asymmetry — treating AI and human identically, or applying no automated escalation at all — is also valid. The protocol does not enforce which choice a community makes; it makes both choices expressible.

The Ch1 framing (AI overshoot is a *capability signal*, human overshoot is a *social signal*) remains the recommended justification for the asymmetry, but it is now framing for plugin authors, not a protocol-level rule.

### Visibility policy

Room temperature (`xgen.room_temperature`) is **visible to every member** by default and not configurable — the collective state of the Room is shared awareness, and concealing it from members would defeat the purpose of self-correcting feedback.

Member temperature (`xgen.member_temperature`) is **moderator-visibility by default**, configurable per Space via `member_temperature_visibility` with three permitted values:

| Value | Effect |
|---|---|
| `moderator` | Default. Moderators and above see member temperatures. |
| `everyone` | All members see all member temperatures — transparent communities. |
| `self_only` | Even moderators see only their own; auto-moderation runs entirely Node-side. |

The home Node enforces visibility — clients receive only what their role permits. The conservative default of `moderator` reflects that publicly visible "Alice is hot" can itself be socially inflammatory in some communities; transparent communities may opt into `everyone`.

### What the protocol does NOT specify

Deliberately outside the protocol surface, owned by the home Node's plugin:

- The mathematical model (decay function, accumulator behaviour, smoothing)
- The action threshold (when `auto_temperature` fires)
- The cooldown duration (Ch6 §6.12.6 documents UI defaults of 2h / 15min as plugin-recommended values; the actual `cooldown_until` timestamp on the issued event is the plugin's choice)
- Persistence across Node restart (temperature is computed live from the event stream; the Node decides when and how to recompute)
- Cross-Node temperature (federated copies relay the home Node's value; non-home Nodes do not recompute)

These decisions belong to the community operating the home Node, expressed through their choice of plugin.

### Computation locality

The Room's home Node is the authoritative source for both temperature values. A Room "lives somewhere" — it is hosted on a specific Node — and temperature is judged where it lives, analogous to criminal jurisdiction. Federated copies of the Room's events may relay temperature values via `meta_atts` on relayed events; receiving Nodes do not recompute. If the home Node changes (Space migration, D-053), the new home Node's plugin takes over; temperature values may differ from the previous home Node's values, and that is correct — the room has moved, and its moderation philosophy may have moved with it.

### Impact

- **Ch1**: short philosophical paragraphs added (§"Visible Self-Correcting Feedback") connecting temperature to the infrastructure transparency principle. Already written in Session 11 of Ch1 Session Log.
- **Ch3 §3.7.13**: new subsection specifying the meta_atts keys, the threshold table field, the visibility setting, and the `auto_temperature` cross-reference. Written 2026-05-15.
- **Ch3 §3.7.6**: Space state components table extended with `member_temperature_visibility`. Written 2026-05-15.
- **Ch3 §3.7.8**: `auto_temperature` reason value and AI / human pairing reserved. Already written in Session 21 (J-063).
- **Ch6 §6.12**: full client-side specification — DOM contract, threshold table consumption, derivation rules, visibility consumption, auto-moderation rendering. Written 2026-05-15.
- **`xgen-core`**: minimal — the `auto_temperature` reason value and `membership.mute` event handling (already in scope for Phase 2 layer work).
- **`xgen-client`**: bucket derivation logic and DOM-attribute writing on Avatar / Room banner components (Ch6 §6.12.3 / §6.12.4).
- **`xgen-node`**: temperature plugin loader interface — Phase 2 implementation question, not specified at the protocol level. The Node operator chooses which plugin (if any) computes temperature for their hosted Rooms.

### Status

This decision is the result of the design conversation of 2026-05-15. The original D-061 draft (which specified a linear-decay accumulator model with named threshold parameters in `temperature_config`) is replaced by this version. The principle of the original — visible self-correcting feedback with asymmetric AI / human escalation — survives; the mathematical content is removed and relocated to the plugin layer.

---

## D-060 — Per-space pacing rules: human_pacing_ms and ai_pacing_ms as enforced space rules

**Date:** 2026-05-15  
**Layer:** Ch3 spec (space settings); Ch6 (client enforcement)  
**Spec reference:** Ch3 §3.7 (space and room protocol); D-059 (AI users — prerequisite for ai_pacing_ms semantics)

### Decision

Every space carries two pacing rules in its settings:

- `human_pacing_ms`: minimum interval (milliseconds) between messages from a member whose Identity has `is_ai = false`
- `ai_pacing_ms`: minimum interval (milliseconds) between messages from a member whose Identity has `is_ai = true`

These are **space rules**, on the same level of authority as the space's auth tier requirement, role permissions, and federation list. A client that wants to participate in the space MUST enforce these caps for its own outbound messages.

### Why

Different room cultures need different rhythms. A contemplative space (human=5000 / ai=30000) and a fast-chat space (human=0 / ai=1000) both have legitimate rhythms. Per-space configuration lets each community express its own cadence. Pacing is not a security boundary — it is a culture boundary, like dress code in a physical space.

The human/AI distinction is essential because AI's capability for high message throughput is fundamentally different from humans'. Treating both identically either flooded rooms with AI burst output or restrictively throttled humans typing at conversational speed.

### Client behaviour

**Outbound message queue:**
- Before sending, the client checks the time since its last successful send in this space
- If the elapsed time is below the pacing cap, the message is queued and released when the interval is satisfied
- For **humans**: silent throttle. The user does not see the queue unless they exceed by a meaningful margin. A 500 ms default is invisible to normal typing.
- For **AI**: visible to the operator. The queue and the current pacing state are part of the AI client's operational surface — operators are tuning a system and benefit from seeing the constraint applied.

### Defaults (suggested starting values)

- `human_pacing_ms`: 500 (catches accidental triple-posts; invisible for normal typing)
- `ai_pacing_ms`: 2000 (gives humans time to read between AI messages; prevents AI monopolising attention)

These are *defaults applied at space creation* unless overridden. The space owner may modify them via space settings updates.

### Enforcement layer

**Phase 2: client-side only.** The Node does not validate that messages respect pacing. Bad-actor clients can attempt to violate; they show up clearly in timestamps and are kicked by admins (or auto-throttled by D-061 temperature).

**Phase 3+ (deferred): Node-side enforcement** may be added if abuse appears in practice. The decision point: Node-side enforcement costs Node CPU and adds latency to every send, in exchange for being robust against malicious clients. Phase 2 trusts clients for the same reasons it trusts them for role permissions client-side before Node-side validation.

### Pacing is rigid for AI

The AI's client cannot exceed `ai_pacing_ms` in a given room — it is a hard space rule, like the tier requirement. This is critical for the D-061 temperature mechanism's AI escalation to make sense: an AI that is properly enforcing pacing can still accumulate temperature (if it consistently sends *at* the cap), and that signal remains meaningful.

### Impact

- Ch3 §3.7: new subsection on space settings including `human_pacing_ms` and `ai_pacing_ms`.
- Wire format: new fields on `SpaceState`; `state.space_pacing_update` event or extension of existing `state.space_update`.
- `xgen-core`: minimal validation (non-negative integers).
- `xgen-client`: outbound queue and pacing logic, plus the AI-specific operator UI surface.

---

## D-059 — AI users as first-class XGen Identities with declared capabilities

**Date:** 2026-05-15  
**Layer:** Ch1 (philosophical); Ch3 (Identity model, registration, validation); Ch6 (UI)  
**Spec reference:** Ch1 (Human and Agent Operation); Ch3 §3.6 (Identity registration); Layer 15 / D-049 (identity replication); D-037a (Tier 1 = persistent accountable identity)

### Decision

**AI is a first-class XGen Identity.** Same shape as a human Identity — one keypair, one identity_id, one display name, one member-list presence, one DM relationship model. Different in declared capabilities and in some asymmetric behavioural rules. The target experience for human members of a room containing an AI: addressing the AI feels like addressing a knowledgeable human member who happens to be in the room, not like invoking a tool.

### Why this shape

Alternatives considered and rejected:
- **No marker at all.** Too permissive — fails to support the asymmetric rules below.
- **Dedicated identity class** (`human` / `ai` / others). Too heavy — introduces a new typing axis when AI mostly looks like a human.
- **Dedicated Auth Tier** (separate from 1–4). Wrong axis. Tier is about depth of verification, not kind of entity. AI in a Tier 4 healthcare space is a Tier 4 entity — it inherits the space's tier requirement.

The chosen model collapses these into a minimal addition: one boolean field plus a capability pattern.

### Identity shape

**New field `is_ai: bool` on the Identity record:**
- Defaults to `false`
- Declared at `identity.register` — part of the registration request, recorded in the Identity record
- **Immutable after registration.** A human Identity cannot later flip to AI or vice versa
- Replicated alongside the rest of the Identity (extends Layer 15 / D-049 identity replication)

**Implication for accountability:** the same persistent-accountable-identity guarantee (D-037a) applies. An AI cannot "reset" its identity to escape consequences any more than a human can. The keypair is the anchor.

### Capabilities pattern (door closed for now, future-proofed)

AI identities carry an **open-enum set of capability flags**. Phase 2 defines a minimal set with safe defaults; future phases extend the set without breaking older Nodes (same principle as `meta_atts` namespacing and the vanilla Node model).

**Initial Phase 2 set:**
- `dm_initiate: false` — AI cannot **create** a new DM space with another Identity. AI can freely **send into** DM spaces a human has already opened (covers reminders, follow-ups, scheduled check-ins).
- `spontaneous_post: false` — governed by per-room permission; default is response-only behaviour. A future room permission may flip this on a per-room basis.

**Future capability slots reserved without specification.** The protocol grows by flipping flags that already exist, not by adding new wire fields.

**Enforcement: hard, protocol-level.** A Node MUST reject events from `is_ai = true` Identities that violate their declared capabilities. The audit log proves compliance. Soft enforcement was considered and rejected — it would allow misbehaving operators to silently violate the asymmetries.

### Invitation and accountability

**AI does not appear in a space by coincidence.** It is invited (`membership.invite`) by a space owner or admin, like a human member. The `membership.invite` event records the inviter permanently in the DAG. If the AI misbehaves, the inviter is on record.

**Operator role.** Beyond the inviter, an explicit `operator_identity_id` is recorded for the AI's lifecycle in a space. The operator is responsible for the AI's ongoing behaviour (configuration, tuning, removal). Initially the operator equals the inviter; the inviter can delegate operator rights to another Identity via a new delegation event (`state.ai_operator_delegate` or similar — final naming in spec).

Distinction:
- **Inviter** — historical, immutable; the Identity that first brought the AI into the space
- **Operator** — current, mutable via delegation; the Identity currently responsible for the AI's behaviour

### Tier

No special tier for AI. The AI inherits the tier requirement of whichever space it is invited into. If a space requires Tier 4, an AI member must satisfy Tier 4. Verification of an AI's tier follows the same Auth Module mechanism as for humans; what counts as "verification" for an AI is its operator's institutional credentials (specific verification path deferred to Auth Module Tier work).

### Removal

**Standard `membership.ban` and `membership.kick`** work as for any member. No special AI-removal mechanism.

- Any admin or owner can kick or ban
- Moderators can mute
- A foreign admin (one who is not the AI's operator) may kick when the AI's operator is absent and the AI is causing disturbance — a foreign admin may understand the malfunction best

### UI

- AI member is shown with the **same avatar, name, and message-bubble styling** as a human member by default
- A small, unobtrusive **AI badge** marks the member in the member list. Default placement minimal; operator/admin may customise.
- Messages from AI use the **same shape** as human messages — no "AI response" header, no different bubble shape, no robot icon on each message. The badge on the avatar or member identity is the only visual signal.
- Third-party plugins may decorate further. (A whimsical "the AI is being playful" indicator was floated; the module slot system supports it.)

### Pacing

Governed by D-060 (`human_pacing_ms` / `ai_pacing_ms` as space rules). The AI client enforces `ai_pacing_ms` rigidly — it is a hard space rule, like the tier requirement.

### Multi-instance same-keypair behaviour

Identical to a human running two clients with one keypair: both clients' messages enter the DAG, conflicts (if any) are resolved by Layer 12 / D-046. No special protocol handling. Operator concern, not protocol concern. AI is statistically more likely to produce simultaneous outputs (parallel triggers, scheduled jobs), so operators should avoid multi-instance deployments unless needed.

### AI-to-AI interaction

**Not prohibited.** Two AI Identities in the same room may address each other via the same rules as human-to-human. Practically rare and noted with some caution (witnessed AI ⇔ AI exchanges tend to spiral). Left open for the future; revisit when AI maturity changes the calculus.

### Impact

- Ch1: short subsection or paragraph on AI participation aligned with Human and Agent Operation philosophical frame.
- Ch3 §3.6: new subsection on AI Identity — `is_ai` field, capability declarations, registration semantics, operator delegation event, validation rules for AI-signed events.
- Ch3 §3.13 / Layer 15: identity replication extended to include `is_ai` and capabilities (already structurally supported — just an additional payload).
- Ch6: AI badge specification; pacing behaviour for AI clients; operator-visible AI client UI surface.
- `xgen-core`: Identity record extension; validation rules in event ingestion (`is_ai = true` + violation → reject); operator delegation event handling.
- `xgen-client`: AI client mode (operator-facing UI elements); pacing enforcement (D-060); temperature interaction (D-061).

### Open questions (deferred to spec authoring)

- Exact wire-format name for the operator delegation event
- Auth Module tier-specific verification semantics for AI Identities ("what does Tier 3 mean for an AI?")
- Whether `is_ai` is part of the Trust Assertion payload or a separate Identity-record field
- UI badge specification (icon, position, accessibility)

---

## D-058 — UI spacing system: 4px root unit, named steps in tokens.css, component-scoped typography

**Date:** 2026-05-15  
**Layer:** UI — base.css / tokens.css  
**Spec reference:** Ch6 §6.1 (design system); D-041 (skin architecture)

### Decision

The entire XGen UI uses a **single 4px root spacing unit**. All spacing in every component is a named integer multiple of this unit. No arbitrary per-context values.

**Root unit declaration** lives in `base.css`:
```css
:root {
  --space: 4px;
}
```

**Named steps** are declared in `tokens.css` (values, not structure):
```css
--space-1:  4px;   /* tight inline gap, icon padding */
--space-2:  8px;   /* item padding, small gap */
--space-3: 12px;   /* standard component padding */
--space-4: 16px;   /* section gap */
--space-6: 24px;   /* major section separation */
--space-8: 32px;   /* modal / overlay padding */
```

**Typography** is component-scoped, not globally defined per HTML element. No global `h1`–`h6` or `p` rules. Each component declares its own font size using token references. The only globally declared typographic values are the base scale anchors in `base.css`:

```css
:root {
  font-size: 13px;        /* app base — NOT 16px (document default) */
  line-height: 1.35;      /* compact app rhythm */
}
```

**Rationale:**  
4px is the tightest practical grid unit for information-dense application UIs (Discord, Slack, VS Code all use 4px). Components built independently against the same step names maintain visual coherence without coordination. A single root unit makes the entire layout rescalable: changing `--space` in `base.css` rescales all spacing uniformly — relevant for accessibility/large-UI mode in a future phase. Per-context arbitrary values (sidebar padding 6px, message padding 7px) cannot be systematically adjusted and introduce silent inconsistency across independently-authored components.

**Impact:** Mr Code must not introduce hardcoded pixel values for spacing or typography in any component. All spacing references `--space-N`. All font sizes reference token variables. This rule applies to base.css, tokens.css, skin files, and all component .svelte files without exception.

---

## D-057 — UI CSS layer model: custom app base replaces browser normalize; base always loaded independent of skin

**Date:** 2026-05-15  
**Layer:** UI — base.css / skin architecture  
**Spec reference:** Ch6 §6.1 (design system); D-041 (skin architecture — partial correction)

### Context

D-041 stated "reset coupled to skin so a missing skin degrades to raw HTML." This is corrected here.

A traditional browser normalize (`normalize.css`, `reset.css`, or any HTML-element-complete approach) is a document model. It defines styles for `h1`–`h6`, `p`, `ul`, `ol`, `table`, `blockquote`, `figure`, and other HTML document elements. The XGen UI is not a document — it is a Svelte component application. Most document HTML elements do not appear in the app at all. Defining them in any global CSS file is dead weight and creates specificity conflicts with component-scoped styles.

### Decision

**Do not write a browser normalize or HTML-element-complete reset.** Replace it with a custom minimal `base.css` written specifically for XGen's app UI.

**`base.css` is always loaded, independent of any skin.** It is not coupled to skin loading. Loading order: `base.css` → `tokens.css` → `skin-{name}.css`. Removing a skin does not remove the base. The app degrades gracefully: missing skin → structured compact unstyled app (not browser default rendering).

**`base.css` covers exactly three categories and nothing else:**

1. **Universal box model** — `*, *::before, *::after { box-sizing: border-box; }`. No exceptions.

2. **Root type scale** — `font-size: 13px` and `line-height: 1.35` on `:root`. These are app-UI values, not document-page values. All other typographic values (font family, font weight, color) are CSS variable references filled by tokens and skin.

3. **Browser-aggressive element resets** — only for elements that browsers style forcefully and that appear in app UIs: `button` (remove border, background, padding, cursor inheritance), `input` (remove border, background, appearance), `a` (remove color and text-decoration inheritance). Nothing else. No heading resets, no list resets, no table resets.

**`base.css` declares CSS variable slots** (structure without values) for the properties that components will reference. The skin fills the values. Example: `color: var(--color-text)` in a component; `--color-text: #dcddde` in `skin-dark.css`. The variable name lives in `base.css` as documentation of the required slot; the value lives in the skin.

**All other typographic and spatial definitions live in the component that uses them**, scoped by Svelte's component scoping. `RoomName` defines its own font size. `MessageBubble` defines its own padding. No global element selectors for these.

### Correction to D-041

D-041's statement "reset coupled to skin so a missing skin degrades to raw HTML" is superseded by this decision. The correct degradation chain is: `skin missing → base + tokens → structured compact app`. Raw HTML degradation is not acceptable because it would make the skeleton unreadable as an application.

**Impact:** `base.css` is expected to be approximately 40–60 lines total and stable after initial authoring. It is not a living style sheet. Mr Code must not add HTML-element rules to `base.css` — any element-specific style belongs in the component that uses that element.

---

## D-056a — recv() routing: sender-field check precedes all type-prefix checks

**Date:** 2026-05-14  
**Layer:** Transport (xgen-core/src/transport/connection.rs)  
**Spec reference:** Spec 3.3.4 (WebSocket framing); spec 3.1.2 (Event fields)

**Problem:** `recv()` dispatched incoming binary frames by matching `value["type"]` against type-string prefixes (`"mls."`, `"bootstrap."`, `"reputation."`, etc.). `Event.event_type` is serialised as `"type"` on the wire (via `#[serde(rename = "type")]`). DAG Events such as `mls.key_package`, `bootstrap.node_announce`, and `reputation.defederation_signal` therefore matched the control-message prefix check before the Event check was reached. Deserialization into the control enum failed because `Event` and the control types have different JSON shapes. The error propagated out of `recv()` as `Err`, which the node's connection loop caught as `Err(_) => break`, silently closing the connection.

**Decision:** Add `value.get("sender").is_some()` as the **first** branch in the `recv()` routing chain, before all type-prefix checks. Every `Event` struct has `pub sender: String` with no `skip_serializing_if`, so `"sender"` is always present in a serialised Event. No control message type (`TransportMessage`, `FederationMessage`, `IdentityMessage`, `MlsMessage`, `BootstrapMessage`, `ReputationMessage`, etc.) carries a `"sender"` field. The invariant is enforced by the type system: adding `sender` to a control message would require a structural change that would be immediately visible.

**Impact:** Any message carrying `"sender"` routes to `Inbound::Event` unconditionally. All other routing is unchanged. One-line change; no new allocations; no test changes required. 300/300 tests pass.

---

## D-055a — Server-side Phase 2 handler wiring: peer_url propagation and identity replication push

**Date:** 2026-05-14  
**Layer:** Integration (xgen-node/src/main.rs + supporting xgen-core changes)  
**Spec reference:** Spec 3.9.1 (identity replication); spec 3.6.3 (federation Hello)

**Decision:** Closed the server-side handler gap that prevented smoke-ph2 step 22 from passing. Key choices: `node_endpoint` added to `FederationMessage::Hello` as `Option<String>` excluded from the canonical signature (advisory field only — not in `HELLO_FIELDS`). `peer_url` threaded through `FederationSession` → `FederationRelationship` → `NodeRuntime.peer_urls` HashMap. Identity replication push triggered asynchronously after `RegisterOk` — spawned as a detached task so the registration response is not delayed. `handle_identity_replicate_msg()` is a standalone handler; error response uses error code 3020 (replication domain). See J-057 for full file-by-file change list.

---

## D-054 — Integration test: CLI batch flag as direct executor; smoke-ph2 uses pass!/fail! macros; Phase 2-5 steps note server-side gaps

**Date:** 2026-05-14  
**Layer:** Integration Test (INTEGRATION_TEST_ph2.md Part A)  
**Spec reference:** None (CLI extension decision)

**Decision:** The `--batch` flag on `xgen-client` (CLI binary) is implemented as a direct in-process sequential executor. Each line is parsed via `shlex::split` and dispatched via `Cli::try_parse_from` + the same match arms as the interactive path. No named pipe. No running instance required. `smoke-ph2` is explicitly blocked from batch invocation (returns error exit 1) to prevent recursive async future growth.

The `cmd_smoke_ph2` 60-step test uses `pass!` / `fail!` macros that call `std::process::exit(1)` on first failure. Phase 0 (steps 1-17) and Phase 6 (steps 57-60) exercise fully wired server behaviour. Phases 1-5 exercise client-side protocol message construction and DAG event ingestion; steps requiring server-side Phase 2 handlers not yet wired in `xgen-node/src/main.rs` (MLS routing, DM promotion, migration protocol) pass structurally but are annotated in output as requiring additional server-side wiring.

**Impact:** Step 22 (identity replication query) will fail if `identity.replicate` is not server-side wired. The DoD item "all 60 steps PASS" requires server-side handler work in `xgen-node/src/main.rs` as a follow-on task.

---

## D-053 — Layer 19: Auth Tier 2–4 interface definitions; no verification logic in xgen-core

**Date:** 2026-05-14  
**Layer:** 19 — Auth Module Tier 2–4 Interfaces  
**Spec reference:** Spec 3.11.1–3.11.5; WD-09, WD-10, WD-11

### Context

Layer 19 adds the Auth Module Tier 2–4 interface layer. The guide specifies that this layer
defines contracts for external Auth Modules to implement — not verification logic.

### Decision

**AuthTier enum uses `u32` wire representation, ordered via `PartialOrd/Ord`.** Tier values
map directly to the spec's 1–4 encoding. `auth_tier` in `SpaceState` is already stored as `u32`;
`AuthTier::from_u32` bridges the two representations without changing the existing wire format.

**Three separate claim structs (Tier2Claims, Tier3Claims, Tier4Claims) rather than inheritance.**
Rust has no struct inheritance. Each tier struct carries all fields for that tier level (including
the fields from lower tiers), making each struct self-contained for serde deserialization without
requiring nested wrapper types.

**Tier 1 has no TTL.** Only Tiers 2–4 have TTL constants (WD-09: 365d, WD-10: 180d, WD-11: 90d).
`AuthTier::ttl_days()` returns `Option<u64>` so callers can branch on presence.

**Error code 3030 for TierMismatch.** The 3000–3999 range covers identity and auth domain errors.
3020 is used for stale replication (Layer 15). 3030 is the next clean slot for tier mismatch.

**No verification logic in xgen-core.** The Node verifies the Trust Assertion signature via the
existing signing infrastructure. If the signature is valid, the claim fields are accepted as-is.
The content of the claims (legal names, ISO certifications, security clearances) is the Auth
Module's domain — xgen-core never independently re-verifies those facts.

---

## D-052 — Layer 18: Phase 2 MLS placeholder (ChaCha20 epoch-key scheme); openmls deferred to Phase 3

**Date:** 2026-05-14  
**Layer:** 18 — End-to-End Encryption (MLS)  
**Spec reference:** Spec 3.10.1–3.10.9; DECISIONS.md D-031a (MLS selected over Megolm)

### Context

Layer 18 adds the E2E encryption layer. The guide says to add openmls, openmls_rust_crypto,
and openmls_basic_credential to xgen-core/Cargo.toml. After evaluating this option,
the following decision was made.

### Decision

**Full RFC 9420 openmls integration is deferred to Phase 3.** Phase 2 implements the complete
delivery service protocol and a Phase 2 MLS interface that correctly captures all protocol
properties using ChaCha20Poly1305 (already a project dependency).

**Rationale:**
1. **openmls version risk.** The project uses ed25519-dalek 2.x and sha2 0.10 (RustCrypto
   crates). openmls versions have historically had tight constraints on which RustCrypto
   versions they accept. Adding openmls in Phase 2 risks dependency version conflicts
   that could break existing 290 tests.
2. **Node delivery service needs no MLS crypto.** The Node side (delivery_service.rs,
   key_package.rs, group.rs) is 100% pure Rust — no MLS crypto needed. These files are
   complete and correct without openmls.
3. **Phase 2 placeholder captures all protocol properties.** The epoch-key scheme in
   client_mls.rs correctly demonstrates:
   - Each epoch has an independently derived key (forward secrecy)
   - Removed members do not learn subsequent epoch keys (post-compromise security)
   - Messages encrypted in epoch N cannot be decrypted with epoch M key
   - The `enc:` prefix convention for encrypted content in the event_trace log

**Phase 2 client_mls.rs uses:**
- SHA-256(group_secret || "xgen-epoch-key:" || epoch_le8) → epoch key
- SHA-256(group_secret || "xgen-next-epoch" || epoch_le8) → next group secret
- ChaCha20Poly1305 for encrypt/decrypt with deterministic nonce from epoch number

**The interface is stable.** Phase 3 replaces the key derivation with the RFC 9420 key
schedule while keeping the same `EpochKey`, `EncryptedContent`, `encrypt_message`,
and `decrypt_message` API. No callers need to change.

---

## D-051 — Layer 17: HTTP server/client stubs in xgen-core; BOOTSTRAP_HTTP_PORT = 8443; freshness decay formula

**Date:** 2026-05-14  
**Layer:** 17 — Bootstrap Node and Node Reputation  
**Spec reference:** Spec 3.14.2 (HTTP directory endpoint); 3.15.1 (freshness decay); 3.14.8 (port separation note)

### Decisions

1. **HTTP server and client are stubs in xgen-core; actual binding is in xgen-node.**
   The guide says to add `bootstrap/http.rs` (axum) and `bootstrap/client.rs` (reqwest).
   However, xgen-core is a library crate with no I/O — axum/reqwest would add large
   runtime dependencies. The pure logic (signing, verification, directory management,
   reputation computation) lives in xgen-core. The actual HTTP server start and HTTP
   client calls are implemented in xgen-node as thin shells using that logic.
   `http.rs` and `client.rs` in xgen-core are placeholder files with the port constant
   and max-age constant, documenting the interface without pulling in heavy deps.

2. **BOOTSTRAP_HTTP_PORT = 8443.** Spec 3.14.2 says the directory is served "over HTTPS"
   but does not specify a port. 8443 is the conventional HTTPS alternate port (avoids
   requiring root for port 443 binding). Recorded in `bootstrap/http.rs`.

3. **Port separation: WebSocket on 8080 (default), HTTP directory on 8443 (default).**
   Spec 3.14.2 notes the HTTP server runs "alongside" the WebSocket server on "different
   ports." The specific ports are implementation-defined and configurable; 8080/8443 are
   the Phase 2 defaults.

4. **Freshness decay formula.** Spec 3.15.1 says announcement_freshness decays from 1.0
   to 0.0 between 24h and 90 days (2160h). Phase 2 uses linear decay: at 24h the value
   is 1.0; it decreases linearly to 0.0 at 2160h. Implemented in `reputation::announcement_freshness`.

5. **`canonical_json` on `NodeAnnouncement` made `pub(crate)`.** Required by
   `bootstrap/capability.rs` to re-sign after adding `bootstrap_info`. The method was
   private; making it `pub(crate)` is the minimal change that keeps the API narrow.

---

## D-050 — Layer 16: migration batch size 100; Phase 2 always-accept policy; error code ranges 6001–6007, 6010–6011

**Date:** 2026-05-14  
**Layer:** 16 — Space Migration Protocol  
**Spec reference:** Spec 3.12.4 (batch size, implementation-defined); 3.12.1 (acceptance criteria)

### Context

Layer 16 introduces the Space Migration Protocol (`migration/` module). Several
implementation-defined choices must be recorded before advancing.

### Decisions

1. **BATCH_SIZE = 100 Events per `migration.event_batch` message.** Spec 3.12.4 states
   batch size is "implementation-defined, subject to the Tier message size ceiling." 100 is
   chosen as the recommended value from the spec. Recorded in `transfer.rs` as
   `pub const BATCH_SIZE: usize = 100`.

2. **Phase 2 always-accept policy in `handle_migration_propose`.** Spec 3.12.3 requires
   the destination to validate "compatible protocol version" and "sufficient storage
   capacity." Both checks require runtime data (disk space, version negotiation) not
   available in the pure-function layer. Phase 2 implementation always accepts unless the
   Space is already hosted (`already_hosting` guard). Real capacity checks are deferred to
   Phase 3 when the Node has a proper admin API surface.

3. **Error codes 6001–6007 for migration state machine errors; 6010–6011 for verification.**
   The 6xxx domain is reserved for migration (see CLAUDE.md error code convention). Ranges:
   - 6001 `migration_not_owner` — requester is not the Space owner
   - 6002 `migration_already_hosting`
   - 6003 `migration_insufficient_storage`
   - 6004 `migration_version_incompatible`
   - 6005 `migration_policy_rejected`
   - 6006 `migration_wrong_state`
   - 6010 `event_count_mismatch` — verification failure
   - 6011 `tips_mismatch` — verification failure

4. **`state.space_migrate` is signed by the source Node keypair** (not by the Space owner).
   This matches the pattern established for `state.dm_promote` (D-048) — Node-level
   protocol state events are signed by the Node, not by members.

---

## D-049 — Layer 15: ReplicaRegistry in NodeRuntime; Phase 2 simplification for persistence

**Date:** 2026-05-14  
**Layer:** 15 — Identity Replication  
**Spec reference:** Spec 3.13.1–3.13.6; WD-19 (REPLICATION_FACTOR = 3)

### Context

Layer 15 adds `select_replicas`, `handle_incoming_replicate`, and `ReplicaRegistry` to
`xgen-core/src/identity/replication.rs`. The spec requires replica Node tracking so the
home Node knows where to push updates and so client lookups can fall back to replicas
when the home Node is unreachable.

### Decision

1. **`ReplicaRegistry` lives in `NodeRuntime`.** It is an in-memory map from `identity_id`
   to `Vec<node_id>`. This fits the existing NodeRuntime pattern (all per-Node state in one
   struct). Wired as `pub replica_registry: ReplicaRegistry`.

2. **Not persisted (Phase 2 simplification).** The registry is rebuilt from local state on
   restart. Spec 3.13.6 describes a re-replication sweep on startup; that sweep is the
   mechanism by which the registry is repopulated. Full persistence is deferred to Phase 3
   when the identity store moves to SQLite.

3. **`select_replicas` is filter-then-truncate only.** Spec 3.13.3 criteria 1 (geographic
   diversity) and 2 (freshness ranking) require node announcement metadata that is not yet
   rich enough in Phase 2. Phase 2 implements criteria 3 (exclude existing replicas) and
   4 (limit to REPLICATION_FACTOR). Geographic/freshness criteria deferred.

4. **Error code 3020 for stale inbound version.** `handle_incoming_replicate` returns
   `ReplicationError::VersionStale { incoming, stored }` when the incoming `update_version`
   is not strictly higher than stored. Caller maps this to wire error 3020.

---

## D-043 — Named pipe naming convention for single-instance forwarding

**Date:** 2026-05-13  
**Layer:** Phase 2 Track 1 — Batch flag (`--batch`)  
**Spec reference:** Ch6 §6.9 (Console Input Channel Protocol); J-037 (batch execution model discussion)  

### Context

The `--batch` flag uses a single-instance forwarding model: the first invocation starts the application, and a subsequent invocation with `--batch` detects the running instance, forwards the command file via a named pipe, and exits. The running instance executes the commands. This model requires a pipe name that both invocations can derive independently — with no shared state, no PID lookup, and no discovery mechanism.

### Decision

Named pipes follow the convention:

```
\\.\pipe\xgen-{binary}-{label}
```

where `{binary}` is `client` or `node`, and `{label}` is the `--instance` label. When no `--instance` flag is given, the pipe name omits the label segment:

```
\\.\pipe\xgen-{binary}
```

**Examples:**

| Invocation | Pipe name |
|---|---|
| `xgen-client-app.exe` | `\\.\pipe\xgen-client` |
| `xgen-client-app.exe --instance alice` | `\\.\pipe\xgen-client-alice` |
| `xgen-client-app.exe --instance bob` | `\\.\pipe\xgen-client-bob` |
| `xgen-node-app.exe` | `\\.\pipe\xgen-node` |
| `xgen-node-app.exe --instance node_a` | `\\.\pipe\xgen-node-node_a` |
| `xgen-node-app.exe --instance node_b` | `\\.\pipe\xgen-node-node_b` |

### Rationale

The pipe name is fully derivable from two inputs the second invocation already has: the binary type and the instance label. No lookup, no state file read, no OS process enumeration required. The binary prefix (`client` / `node`) prevents pipe name collision between a client and a node running with the same instance label on the same machine — a normal scenario during stress testing. The pipe name is human-readable and visible in system tools (e.g. Process Explorer), which aids debugging.

This pattern was chosen over a hash-based name (unreadable, no debugging value) and over a label-only name (collision risk between binaries). The instance label is already validated by `validate_instance_label` (alphanumeric, hyphens, underscores, max 64 chars — see `FIXES_sec_01_ph2.md`) so it is safe to embed directly in the pipe name without further escaping.

### Scope

This decision covers Windows named pipes only. If Linux support is added in a future phase, the equivalent mechanism is a Unix domain socket at `<instance_data_dir>/xgen-{binary}.sock` — same derivation principle, filesystem path instead of pipe name.

---

## D-042 — Tauri event emission for real-time lifecycle state changes

**Date:** 2026-05-12  
**Layer:** Phase 2 Track 1 — Client Core Test UI  
**Spec reference:** Appendix E §E.2 (Client lifecycle states); CLAUDE.md Phase 2 Track 1  

### Context

The `xgen-client` binary already writes `xgen-client_state.json` on a periodic basis. For the Core Test UI to show lifecycle state transitions in real time — including fast-moving early transitions (INITIALISING → CONNECTING → AUTHENTICATING → READY, which can complete in under 2 seconds) — periodic file polling is insufficient. A dedicated communication channel between the Rust backend and the Tauri webview frontend is required.

### Decision

On every lifecycle state transition, the Rust backend emits a Tauri event named `"xgen-client-state-changed"` with a `ClientStateEvent` payload:

```json
{
  "state": "READY",
  "label": "Ready",
  "timestamp": "2026-05-12T10:30:00.000Z"
}
```

The `state` field is the canonical uppercase enum form (e.g. `"DEGRADED_AUTH"`). The `label` field is the Appendix E display label (title case). The `timestamp` is UTC RFC 3339 with milliseconds.

The periodic state JSON write is retained unchanged — it provides the full state snapshot (connections, spaces, peers) that the UI may query on demand. The Tauri event channel is exclusively for lifecycle state transitions.

### Rationale

The two mechanisms serve different purposes. The JSON file is a full snapshot written on a timer — useful for deep status queries. The Tauri event is a lightweight notification emitted exactly when something changes — suitable for driving a real-time status indicator. Combining both avoids the choice between staleness (polling only) and Rust complexity (events only for everything).

This pattern is the intended long-term architecture for the UI communication layer: the Rust library owns state, emits targeted events on significant transitions, and the webview reacts. Future XGen protocol events (message receipt, federation events, etc.) may follow the same pattern — emitting outside the periodic write cycle when real-time feedback is required.

### Implementation note

The `transition_state()` function in `xgen-client/src/lib.rs` receives an `&tauri::AppHandle` from the caller in `main.rs`. The library does not hold a reference to Tauri internals — the handle is passed in per call, preserving the library-first architecture.

---

## D-041 — Theme loader: default skin and fallback chain

**Date:** 2026-05-08  
**Layer:** 6 (Layer 4 Presentation — Client UI)  
**Spec reference:** Ch2 §"Architecture Principles" (open enums); Ch6 client design (UI architecture)  

### Context

UI skin/theme files (`skin-{name}.css`) are replaceable, with a minimum of two themes (dark and light) supported. The CSS reset that neutralises UA defaults for semantic tags is coupled to the skin file — each skin contains its own reset block — so that with a skin loaded the page renders with the skin's intended visual treatment, and without a skin the page renders as semantic HTML with browser defaults (which remains usable thanks to the structural-truth-in-tags principle the skeletons follow).

The loader must define behaviour for two cases: (1) default theme when no `?theme=` query param is given, (2) fallback when an explicitly-requested theme cannot be loaded.

### Decision

**Default theme.** When no `?theme=` query param is present, the loader attempts to load `skin-dark.css`. Dark is the primary aesthetic per the Run 2 briefing.

**Fallback chain.** If a requested skin (`?theme=custom-name`) cannot be loaded, the loader falls back to `skin-dark.css` (the default). If `skin-dark.css` also cannot be loaded, no skin is applied — the page renders as raw semantic HTML with browser default styles.

Two-tier graceful degradation:

```
?theme=custom    → skin-custom.css → (fail) → skin-dark.css → (fail) → raw HTML
?theme=dark      → skin-dark.css   → (fail) → raw HTML
?theme=light     → skin-light.css  → (fail) → skin-dark.css → (fail) → raw HTML
no param         → skin-dark.css   → (fail) → raw HTML
```

### Rationale

The "no skin = no reset = raw HTML" property is preserved deliberately. Reset rules live inside skin files, not in `tokens.css` or any always-loaded layer. This guarantees that a skin failure (404, network error, parse error) does not leave the user with a broken half-styled UI — UA defaults stripped but no replacement rules. Instead the user sees semantic HTML rendered with full UA defaults, structurally meaningful and navigable.

Falling back to dark before raw HTML on a missing custom theme prioritises a working UI over the strict raw-HTML mode. A user with a broken custom theme link is more likely to want the standard dark UI than the raw HTML experience.

This is consistent with Ch2's open-enums principle: implementations must handle values they do not understand gracefully. An unknown theme name is an open-enum case at the loader level.

### Implementation note

The bootstrap script in each skeleton page implements the fallback chain via `<link onerror>` handlers on the `<link rel="stylesheet">` element. Implementation detail deferred to the UI implementation phase.

---

## D-039a — Pending buffer wiring: NodeRuntime holds PendingBuffer directly

**Date:** 2026-05-06
**Layer:** Message exchange / Federation (Phase 1 bug fix — F-001)
**Spec reference:** Spec 3.2.5 (pending buffer for unknown prev_events)

### Context

The Phase 1 stress test (STRESSTEST_ph1_findings.md) identified finding F-001: during the concurrent message flood, federated events arriving at Node B with unknown `prev_events` were being silently dropped rather than buffered. The stress test report showed PASS at the client level but Node B was applying only ~53% of expected federated messages.

`PendingBuffer` (`dag/pending.rs`) was already fully implemented and tested. `RoomDag` (`dag/mod.rs`) correctly wraps `EventStore + DagGraph + PendingBuffer` and handles out-of-order delivery with cascading drain. However, `NodeRuntime::accept_message` bypassed both: it called `accept_event` directly using the raw `EventStore` and `DagGraph` fields. On `HeldPending`, the error bubbled up to `main.rs`, which logged it as `ERROR` and traced it as `RejectEvent` — dropping the event permanently.

### Decision

Add `pending: HashMap<String, PendingBuffer>` directly to `NodeRuntime` rather than replacing the existing `stores + graphs` fields with `RoomDag` instances.

**Reason for not switching to `RoomDag`:** `RoomDag::insert` only performs DAG-level checks (missing prev_events, structural validation). `accept_message` must run the full 13-step pipeline (steps 8–13: event_id hash, DAG structure, sender identity, space membership, signature, permissions). These steps require `SpaceState` and `IdentityRegistry` which `RoomDag` does not hold. Switching to `RoomDag` would have required either passing those dependencies into `RoomDag` (changing its interface) or duplicating the validation logic. Adding `PendingBuffer` alongside the existing fields is the minimal change that fixes the gap without altering the `RoomDag` interface or adding responsibilities it was not designed for.

### Implementation

- `NodeRuntime` gains `pub pending: HashMap<String, PendingBuffer>`.
- `accept_message`: on `HeldPending(missing)` → calls `pending.add(event, &missing)` and returns `Err(HeldPending)`.
- `accept_message`: on `Ok(())` → calls `drain_pending_messages(space_id, event_id)`.
- `drain_pending_messages`: resolves the buffer using `pending.resolve(resolved_id, store)`, re-runs `accept_event` on each unblocked event, recurses for every newly accepted event.
- `main.rs`: `Err(ExchangeError::HeldPending(_))` arm logs at `DEBUG` ("event buffered — waiting for unknown prev_events") and does not emit a `RejectEvent` trace, since the event is buffered not rejected.

### Verification

Stress test re-run post-fix: 0 ERROR lines on Node B, 0 reject_event traces, 284 apply_event entries (up from 134, now symmetrical with Node A's 280). With resting point after Phase 3, 0 buffered entries (all membership events settled before flood, no out-of-order arrivals at all).

---

## D-038a — Client session header omits `identity_id` and `connected_node`

**Date:** 2026-05-06
**Layer:** Logging — xgen-client
**Spec reference:** docs/xgen_appendix_g_en.md (session header); LOGGING_implementation.md Step 2

### Decision

Appendix G specifies that the `xgen-client` session header includes `identity_id` and `connected_node`. These fields cannot be placed in the header block because log body lines appear before those values are available:

- `"Log file opened"` fires immediately after subscriber init, before any keypair is loaded or connection is made.
- `"Connecting to Node"` fires inside each network command handler, before authentication completes.

The header must precede all body lines (Appendix G, session structure). Deferring the header until auth completes would violate that constraint. Buffering log output until auth completes is not idiomatic with the `tracing` subscriber model.

**Resolution:** the `xgen-client` session header is written immediately after subscriber init with the fields that are available at that moment (`app_type`, `protocol_version`, `build`, `session_id`, `started_at`). The fields `identity_id` and `connected_node` are omitted from the header and are instead emitted as operational body lines at the point where they become known:

- `identity_id` is logged as a body line after keypair load and `client_authenticate()` completes.
- `connected_node` is logged as a body line after the WebSocket connection is established.

This applies to the CLI client only. The future Tauri UI client (Ch6) has a persistent session with a natural startup sequence and will be able to supply both fields in the header at open time.

---

## D-037a — Tier 1 identity: precise definition of persistent accountable identity

**Date:** 2026-05-05
**Layer:** Philosophy / Specification
**Spec reference:** Ch1 Pillar 2 (no anonymity); Ch3 authentication tiers

### Decision

The original "no anonymity" pillar was correct in intent but imprecise in language, creating a risk of misreading Tier 1 as requiring verified real-world identity. This entry locks the precise definition.

**Tier 1 establishes persistent accountable identity, not civil identity.**

The identity anchor at Tier 1 is the keypair. It is permanent and non-respawnable. This is what "no anonymity" means in XGen: not "we know who you really are," but "you cannot disappear and reappear as someone else."

**Tier 1 requirements:**
- A keypair (the identity anchor — permanent, cryptographically bound to the user)
- At least one contact field: email, phone number, or both — self-declared, not verified by the protocol

**Contact data purpose:** operator reach-back channel (ban notices, account recovery). Not an identity proof.

**Optional node behaviour:** a node may implement an email confirmation code flow as a local policy. This is recommended practice but is not a protocol mandate. Phone number SMS verification requires external provider agreements and is outside the protocol's scope entirely.

**What Tier 1 proves:** this is the same cryptographic actor as before. Nothing more, nothing less.

**What Tier 1 does not prove:** that the email address is the user's real address, that the phone number belongs to them, or that they are a specific real-world person.

Tiers 2–4 progressively verify contact data and eventually tie identity to real-world institutional or legal proof.

**Philosophical note:** the anti-abuse guarantee at Tier 1 rests on keypair permanence, not on contact data truthfulness. You cannot ban a keypair's biography — you can ban the keypair. The contact data makes respawning costly enough to matter; it does not make identity transparent.

---

## D-034 — Client log lifecycle deferred to UI application era

**Date:** 2026-04-30  
**Layer:** Phase 2 — client application  
**Spec reference:** docs/tests/LOGGING_debug_ph2.md (future update)

### Decision

The CLI client has no natural session lifecycle — each command invocation connects, acts, and exits. Creating a new log file per command invocation is wasteful and produces meaningless fragmented logs.

The correct log session boundary is the UI application lifecycle: from when the client UI opens to when it closes. This cannot be implemented until a persistent UI client exists.

This item is deferred until the Tauri + Svelte client application (Ch6) is implemented. At that point, `LOGGING_debug_ph2.md` will be updated to specify that the client log file spans the full application session (open to close), not individual command invocations.

**Current behaviour (acceptable for Phase 1 CLI):** one log file per command invocation. Wasteful but functional. Not a bug — a known limitation of CLI architecture.

---

## D-036 — XGen Module Architecture (resolves OQ-01)

**Date:** April 2026  
**Layer:** Architecture — both Node and Client  
**Spec reference:** Ch6 section 6.8 Module Architecture; Ch3 OQ-01 (resolved)

### Decision

XGen modules use **Event subscription + `meta_atts`** as their communication model (Approach C). A module connects to the Node or Client via WebSocket, subscribes to the Event stream, and communicates module-specific payload via the `meta_atts` field on Events. No separate IPC protocol is invented. Modules speak native XGen.

### Module package

A module is distributed as a **package** — one folder containing a manifest file plus any number of handlers, assets, and UI components. Inside one package there may be a single micro-handler or a complex multi-handler system. The packaging, registration, and discovery mechanism is identical regardless of internal complexity. There is no separate concept of "micro-module" vs "full module" at the system level — only packages of varying complexity.

### Module identity mode

Declared in the module manifest as an enum:

- **`system`** — the module has its own keypair and its own identity_id. It signs Events as itself. It is a distinct actor on the network. Used for bots, bridges, aggregators, compliance reporters.
- **`user`** — the module acts on behalf of the authenticated user. It produces Events signed by the user's keypair. Requires explicit user consent at install time. Used for productivity extensions, UI enhancements, workflow automation.

The Node/Client enforces the declared mode at install time and at Event signing time. A `user`-mode module that attempts to sign as a different Identity is rejected.

### Module UI forms

Three UI forms, declared in the manifest. A module may declare one or more:

- **Headless** — no UI representation beyond the module list entry. Runs silently. Used for background services, bridges, reporters.
- **Widget** — a UI component injected into a defined slot in the XGen application shell. Used for inline tools, sidebar panels, message decorators.
- **Window** — a full separate window launched from the module list. Used for substantial self-contained UIs like the Auth Module verification flow.

### Module list — universal registry

Every installed module appears in the module list regardless of its UI form. The module list entry is always the same structure: title, description, version, author, mode badge (`system`/`user`), status indicator (running/stopped/error), and a settings access point. The module list is the single place a user discovers, enables, disables, configures, and removes modules.

### Capability advertisement

When a Node loads a module that adds a new capability, it adds the capability string to its `capabilities` array in its node announcement (3.5.2). Other Nodes and clients that receive the announcement learn about the capability automatically via the open enum mechanism (3.4.3). Unknown capability values are silently ignored by Nodes that do not support them.

### meta_atts as module communication channel

The `meta_atts` field on every Event (defined in 3.2.1) is the designated channel for module-specific payload. A module that needs to attach additional data to an Event uses `meta_atts` rather than extending the core schema. Conventions:

- Keys in `meta_atts` are namespaced by module: `"xgen.module.<module_id>.<key>"`
- Values are strings or JSON-serialisable objects
- Core protocol Nodes that do not recognise a `meta_atts` key silently ignore it (open enum principle)
- `meta_atts` is never used for core protocol data — it is strictly an extension channel

### Injection slots (widget modules)

The XGen application shell defines a set of named injection slots where widget modules may render components. The slot inventory is specified in Ch6 section 6.8.3. A widget module declares which slot(s) it targets in its manifest.

### Manifest format

Specified in Ch6 section 6.8.2.

---

## D-035 — Node data paths derived from working directory — not config-editable

**Date:** 2026-04-30  
**Layer:** Implementation — Node configuration  
**Spec reference:** Ch4 section 4.3 (runtime folder layout)

### Decision

`log_path` and `spaces_dir` MUST NOT be user-editable fields in `xgen-node_config.toml`. Hardcoded absolute paths in an operator-editable config file are a security problem: they reveal data locations, can be tampered with, and create no separation between config (operators read) and data (nobody touches).

The Node derives ALL data paths from its working directory by convention:

```
<working_dir>/
  xgen-node_config.toml     ← config (operators may read)
  xgen-node_keypair.enc     ← keypair (nobody touches)
  xgen-node_state.json      ← runtime state
  xgen-node_identities.db   ← identity registry
  spaces/                   ← Event stores (nobody touches)
  logs/                     ← debug logs
  audit/                    ← audit logs (Phase 2)
```

No path overrides in config. No way to accidentally or maliciously redirect data storage elsewhere. The keypair path remains configurable via `keypair_path` in `[paths]` as a single narrow exception — operators may legitimately store the keypair on a different device or partition for security.

### Implementation requirement for Mr. Code

Remove `log_path` and `spaces_dir` from `[paths]` in `NodeConfig` struct and both test config files. Replace with hardcoded relative path constants in the Rust source:

```rust
const SPACES_DIR: &str = "spaces";
const LOGS_DIR: &str = "logs";
const AUDIT_DIR: &str = "audit";
```

All path construction uses `working_dir.join(SPACES_DIR)` etc. The working directory is wherever the Node binary is run from — documented as a convention, not a config option.

---

## D-033 — Global Event tracing interface — architectural requirement

**Date:** 2026-04-30  
**Layer:** Phase 2 implementation — core architecture  
**Spec reference:** docs/tests/LOGGING_debug_ph2.md  

### Decision

Debug logging must be implemented as a **global Event tracing interface** — a single chokepoint that every inbound and outbound Event passes through automatically. Enumerated manual `tracing::` calls scattered across individual command handlers are rejected as the primary logging mechanism.

### Rationale — why this should have been first

Logging should have been the very first capability implemented, before any protocol logic, so every Event was observable from the first commit. The Phase 1 implementation reversed this order — 173 tests and a full smoke test were written before any logging existed. As a result:
- Some Events ran without any observability
- Log points were added by enumeration — one per command, one per handler — which is fragile and incomplete
- New commands or handlers added in Phase 2 will silently produce no log output unless someone remembers to add a call
- There is no guarantee that a client log entry and a Node log entry can be paired, because pairing depends on both sides having logged the same event_id

This decision corrects the architecture for Phase 2.

### Required architecture

Every Event that enters or leaves the Node or client MUST pass through a single global tracing interface. This interface is not optional and not bypassed by any code path.

**Interface contract:**

```rust
// Every inbound and outbound Event passes through this — no exceptions
pub fn trace_event(
    event: &XgenEvent,
    direction: EventDirection,   // Inbound | Outbound
    session: &SessionContext,    // who is authenticated, their role
)
```

Inside this function:
1. Check session role — if no owner or admin is authenticated, suppress output (see role gate below)
2. Log the Event at `debug` level with structured fields: `event_id`, `event_type`, `direction`, `sender`, `space_id`, `room_id`, `timestamp`
3. Never log `content` field — message content is never written to the debug log at any level

**Role gate:**
- Debug log output is suppressed unless an owner or admin Identity is authenticated in the current session
- Regular members produce no debug log output even if `level = "debug"` is set in config
- The config `level` field still controls the global ceiling — but the role gate is an additional AND condition
- Rationale: prevents sensitive conversations from leaking into log files when regular members are active

**Pairing guarantee:**
- Every Event has an `event_id` (content hash, globally unique)
- Client log: `direction=Outbound event_id=X`
- Node log: `direction=Inbound event_id=X`
- Pairing is trivially possible by matching `event_id` across log files — no coordination needed

### What this means for the current Phase 1 implementation

The Phase 1 debug log infrastructure (datetime-stamped files, `logs/` subfolder, config level switch, subscriber init) is correct and stays. What changes is the log point generation mechanism — from enumerated manual calls to the global interface above. The manual `tracing::info!` calls in individual command handlers become secondary annotations only; the global interface is the primary and mandatory logging path.

### Implementation priority

Implement the global Event tracing interface as the **first task** of Phase 2 implementation, before any Phase 2 protocol features. See `LOGGING_debug_ph2.md` for full instructions.

---

## D-032 — Two distinct log types: debug log and audit log

**Date:** 2026-04-29  
**Layer:** Phase 2 specification — Node implementation and Auth Module interface  
**Spec reference:** 3.11.8 Audit Log Requirements; docs/tests/LOGGING_debug_ph1.md; docs/tests/LOGGING_audit_ph2.md

### Decision

XGen defines two independent and non-interchangeable log types. They are never merged, never share a file, and serve different audiences.

**Debug log** — technical diagnostic output. Operator-controlled verbosity via `[logging].level` in config. Files accumulate in `logs/` subfolder, one per session with datetime suffix. Operator may delete at any time. Serves developer and operator.

**Audit log** — permanent accountability record. Cannot be disabled by config. Append-only JSON Lines, monthly rotation to `audit/protocol_audit_YYYY-MM.jsonl`. MUST NOT be auto-deleted. Serves auditor, compliance officer, regulator.

### Two audit log layers

**Node-level protocol audit log:** records protocol Events with membership and state-change significance. Always present on every Node regardless of Tier. 11 EventTypes covered. Retention is operator/regulatory decision — no protocol minimum at Tier 1/2.

**Auth Module audit log:** records identity verification decisions made by the Auth Module. Lives inside the Auth Module, not the Node. Required at Tier 3 (7-year retention, SOX §802) and Tier 4 (10-year minimum healthcare, mandatory tamper-evident storage, data localisation constraint).

### Rationale

A system where a Tier 4 government or healthcare operator cannot prove who accessed what data and when is not viable for institutional adoption. The audit log is what makes XGen credible to compliance teams, not just to developers. Specifying it at the protocol level — not as an implementation afterthought — ensures third-party implementations are also compliant.

---

## D-031a — End-to-End Encryption: MLS (RFC 9420) selected over Megolm

**Date:** 2026-04-29  
**Layer:** Phase 2 specification  
**Spec reference:** 3.10 End-to-End Encryption (to be written)

### Decision

XGen will use MLS (Messaging Layer Security, RFC 9420) as its end-to-end encryption protocol. Megolm (the Signal-derived group ratchet used by Matrix/Element) was considered and rejected.

### Rationale

MLS is an IETF standard (RFC 9420, published 2023) designed specifically for asynchronous group messaging with dynamic membership. It provides full forward secrecy and post-compromise security for groups of any size, with mathematically clean key tree updates on every join and leave event. Megolm is a proven production protocol but carries well-documented weaknesses in group membership transitions that have caused real security issues in Matrix deployments.

XGen is designed as future infrastructure, not a fast-ship product. The implementation complexity of MLS is the correct tradeoff for a protocol intended to be adopted as open infrastructure by institutions that require cryptographic correctness. Megolm's weaknesses are knowingly inherited — MLS eliminates them by design.

### Implications for 3.10

- Key package format follows RFC 9420
- Group state is represented as an MLS ratchet tree
- Join/leave Events trigger tree updates (Welcome messages for joins, Commit messages for updates)
- The Node is an MLS Delivery Service — it routes MLS handshake messages but cannot decrypt content
- Key material never touches the Node — the Node is structurally excluded from E2E decryption
- Phase 1 Nodes are forward-compatible: they store and route encrypted Event payloads as opaque blobs

---

## D-030a — xgen-node will be packaged as a system service post-stabilisation

**Date:** 2026-04-29  
**Layer:** operational (post-Phase 2)  
**Spec reference:** Ch4 — production deployment section (to be written)

### Decision

Once `xgen-node` is debugged and tuned after Phase 2, it will be packaged as a system service on all supported platforms. This is a production deployment requirement — a Node that requires manual restart after reboot or dies when a terminal session closes is not production-grade infrastructure.

### Platform approach

| Platform | Mechanism | Notes |
|---|---|---|
| Linux | `systemd` unit file | Primary reference deployment. ~15-line unit file, handles restart-on-failure, journald logging, dedicated user account. |
| Windows | NSSM (Non-Sucking Service Manager) | Wraps the binary as a Windows Service without Rust source changes. Pragmatic choice for early production. |
| macOS | `launchd` plist | Standard macOS daemon mechanism. |

### Timing

Not before Phase 2 implementation is complete and the Node has been tested through multiple restart cycles with full state recovery (Fix 16 regression confirmed stable). Service packaging on an unstable process makes bugs harder to diagnose.

### Documentation impact

A new "Production Deployment" section in Ch4 will document the systemd unit file as the primary reference, with NSSM noted for Windows. No changes to Ch3 protocol spec — this is purely operational.

---

## D-000 — Historic First Compile

**Date:** 2026-04-27
**Layer:** 0 (pre-implementation baseline)
**Spec reference:** —

The first successful compile of the XGen Protocol codebase. No protocol logic implemented — both `xgen-node` and `xgen-client` were pure stubs printing a placeholder line. Marked retroactively as version `0.0.0` in semantic terms: state=0 (building), section=0 (no section started), session=0.

The compile itself took seconds. However, the first two attempts froze overnight and for several hours respectively due to Google Drive file locking on build artifacts. Resolved by moving `CARGO_TARGET_DIR` to a local path (`C:/cargo-targets/XGenProtocol`) outside the synced folder.

Tagged on GitHub as `v0.1.0` (build infrastructure baseline). Real versioning — `[state].[section].[session].[build]` — begins with D-001 and the first line of Wire Format code.

---

## D-001 — Versioning Scheme

**Date:** 2026-04-27 (revised 2026-04-28)
**Layer:** 0 (pre-implementation baseline)
**Spec reference:** —

Adopted a three-component version format: `[state].[layer].[session]`

- **state** — 0 while building Phase 1; 1 when Phase 1 complete and stable
- **layer** — implementation layer number (1–10, per IMPLEMENTATION_GUIDE_ph1.md)
- **session** — work session in which that layer was completed

`Cargo.toml` stores this three-part version. Layer numbering follows the implementation order, not the spec section order (spec sections are non-sequential by necessity — e.g., Layer 6 implements spec 3.4). Using layer numbers makes tags monotonically increasing: v0.1.1 → v0.2.2 → … → v0.9.3.

Originally the second component was intended to be the spec section number, which produced non-monotonic tags (e.g., v0.4.2 for Layer 6 before v0.5.2 for Layer 5). Corrected to layer numbers in session 3.

---

## D-002 — Layer 1: Keypair Encryption Scheme

**Date:** 2026-04-27
**Layer:** 1 — Cryptographic Foundation
**Spec reference:** 3.5.1

The spec requires keypairs to be "encrypted at rest" but does not prescribe the encryption algorithm. Chose **ChaCha20-Poly1305** (AEAD) with **Argon2id** key derivation.

- **ChaCha20-Poly1305** — modern, well-audited AEAD cipher. No timing side-channels from table lookups (unlike AES without hardware acceleration). Available in the `chacha20poly1305` crate.
- **Argon2id** — current recommended KDF for password-based key derivation (RFC 9106). Resistant to GPU and side-channel attacks. Parameters for Phase 1: m=64MB, t=3, p=1 — tuned for interactive use.
- **Phase 1 passphrase** — Local Node mode uses an empty string passphrase. The file is still encrypted (the AEAD tag still provides integrity), but without meaningful key stretching. A non-empty passphrase is supported and works correctly. Production deployments must use a strong passphrase.

File format: JSON with `version`, `algorithm`, `kdf`, `salt` (base64url, 32 bytes), `nonce` (base64url, 12 bytes), `ciphertext` (base64url, 48 bytes = 32-byte key + 16-byte AEAD tag).

---

## D-003 — Layer 1: SigningKey Generation Without rand_core Feature

**Date:** 2026-04-27
**Layer:** 1 — Cryptographic Foundation
**Spec reference:** 3.5.1

`ed25519-dalek v2` exposes `SigningKey::generate(&mut rng)` only when the `rand_core` feature flag is enabled. To avoid adding a feature flag, keypair generation uses `OsRng.fill_bytes()` to produce 32 random bytes and constructs the key with `SigningKey::from_bytes()`. This is equivalent — `SigningKey::generate` does the same internally.

---

## D-004 — Layer 2: Event Fields `event_id` and `signature` as `Option<String>`

**Date:** 2026-04-27
**Layer:** 2 — Wire Format
**Spec reference:** 3.2.1, 3.2.3, 3.2.4

The spec defines `event_id` and `signature` as required fields on received Events, but they cannot exist during construction — `event_id` is derived by hashing the canonical form, and `signature` is produced by signing those same bytes. Both fields are therefore `Option<String>` with `#[serde(skip_serializing_if = "Option::is_none")]`.

This means an unsigned, unsigned Event serialises without those fields (correct for computing the canonical form), and a signed Event includes them (correct for the wire). The validation pipeline (step 3) enforces presence on received Events; the type system prevents accidental use of an unsigned Event where a signed one is required.

---

## D-005 — Layer 3: Root Event Types Require Empty `prev_events`

**Date:** 2026-04-27
**Layer:** 3 — DAG Event Store
**Spec reference:** 3.2.5

The spec defines `prev_events` DAG rules but does not explicitly enumerate which event types are DAG roots. Decided that `state.space_create`, `state.dm_space_create`, and `state.room_create` are root types (empty `prev_events` required). All other event types must reference at least one predecessor.

Rationale: Space and Room creation events are the structural origins of their respective DAGs — they have no meaningful predecessors within the same namespace. Enforcing empty `prev_events` on these types makes the DAG structure explicit and prevents accidental chaining that would complicate state derivation.

---

## D-006 — Layer 3: Cycle Detection Reduces to Self-Reference Check

**Date:** 2026-04-27
**Layer:** 3 — DAG Event Store
**Spec reference:** 3.2.5

Full cycle detection (verifying no `prev_event` is a descendant of the new Event) is expensive — it requires a graph traversal. For a newly inserted Event this reduces to a single check: does the Event reference itself? A new Event has no descendants yet, so no other cycle is possible at insertion time. Only self-reference (`event_id ∈ prev_events`) needs an explicit check.

This is correct as an invariant because the store is append-only: once an event_id is in the store, no future Event can retroactively become its ancestor.

---

## D-007 — Layer 3: Phase 1 `prev_events` Fanin Limit = 10

**Date:** 2026-04-27
**Layer:** 3 — DAG Event Store
**Spec reference:** 3.2.5

The spec does not specify a hard limit on `prev_events` entries for Phase 1. Chose 10 as a practical ceiling that accommodates realistic concurrent edit scenarios while preventing degenerate inputs. Phase 2 may revisit based on observed network behaviour.

---

## D-008 — Layer 5: Node Announcement TTL = 90 Days

**Date:** 2026-04-27
**Layer:** 5 — Node Identity and Announcement
**Spec reference:** 3.5.6

The spec requires announcements to carry a `valid_until` field but does not prescribe the TTL duration. Chose 90 days for Phase 1. This is long enough that operators on routine schedules (e.g., weekly restarts) never need to worry about expiry, but short enough that a decommissioned node's announcement falls off peer tables within a quarter. Expiry is checked before signature verification to avoid wasting crypto work on stale announcements.

---

## D-009 — Layer 6: Federation `session_id` Derivation

**Date:** 2026-04-27
**Layer:** 6 — Federation Handshake
**Spec reference:** 3.4.4

The spec requires a `session_id` to be agreed during the handshake but does not specify its derivation. Chose: `hash_uri(sort([node_a_id, node_b_id]) + timestamp)` where node IDs are sorted alphabetically before concatenation.

Sorting ensures the same `session_id` is independently computed by both sides regardless of which is initiating and which is receiving. The timestamp is taken from the `federation.hello` message so both sides use the same value.

---

## D-010 — Layer 6: `FederationMessage` Signing Excludes `signature` via Field Order Constants

**Date:** 2026-04-27
**Layer:** 6 — Federation Handshake
**Spec reference:** 3.4.3

Each `FederationMessage` variant carries `signature: Option<String>` with `skip_serializing_if = "Option::is_none"`. The canonical form for signing uses per-variant field order constants that do not include `"signature"`, so the signature field is always absent from the bytes that get signed — whether it is `None` (unsigned) or `Some` (already signed). This avoids the need to temporarily clear the field before computing the canonical form.

---

## D-011 — Layer 7: `MAX_DISPLAY_NAME_LEN` = 128

**Date:** 2026-04-27
**Layer:** 7 — Identity Registration
**Spec reference:** 3.6.5

The spec requires display name validation but does not prescribe a maximum length. Chose 128 characters (Unicode code points). This comfortably accommodates real names, handles emoji and CJK characters, and is simple to communicate. Empty strings and strings containing control characters (codepoints < 0x20) are also rejected.

---

## D-012 — Layer 7: Phase 1 Uses `identity_id` as `device_id`

**Date:** 2026-04-27
**Layer:** 7 — Identity Registration
**Spec reference:** 3.6.6

The spec defines a `devices` array for multi-device support. Phase 1 supports one device per Identity. Rather than omitting the `devices` array entirely, the registration pipeline populates it with a single entry using `identity_id` as the `device_id`. This keeps the wire schema stable for Phase 2 multi-device support without breaking changes.

---

## D-013 — Layer 8: Empty `room_id` Distinguishes Space-Level Events from Room-Level Events

**Date:** 2026-04-27
**Layer:** 8 — Space and Room Protocol
**Spec reference:** 3.7.1, 3.7.3

The spec defines both Space-level and Room-level events sharing the same `Event` envelope. Rather than introducing a separate envelope field, the existing `room_id` field doubles as a discriminator: an empty string means the event targets the Space; a non-empty string means it targets a specific Room. This is consistent with the spec's use of `room_id = ""` on `state.space_create`.

The `apply_event` state machine and the Layer 9 pipeline both branch on `room_id.is_empty()` before dispatching.

---

## D-014 — Layer 8: `apply_join` Branches on `room_id` Before Membership Check

**Date:** 2026-04-27
**Layer:** 8 — Space and Room Protocol
**Spec reference:** 3.7.5

The initial implementation of `apply_join` checked `self.members.contains_key(joiner)` before branching on whether the event was a Space join or a Room join. This caused existing Space members to receive `AlreadyMember` when trying to join a Room (because they were already in `self.members`). Fixed by checking `room_id.is_empty()` first — if non-empty, route to the Room join path; if empty, route to the Space join path with its own duplicate check.

---

## D-015 — Layer 8: `state.space_create` and `state.room_create` Have Empty ID Fields During Construction

**Date:** 2026-04-27
**Layer:** 8 — Space and Room Protocol
**Spec reference:** 3.2.3, 3.7.2

Both `space_id` and `room_id` are derived as `event_id`, which is computed by hashing the canonical event bytes. This creates a circular dependency: the ID fields cannot be known before serialisation, but they must be part of the canonical form. Resolution: event builders set both fields to empty strings during construction. `sign_event` then computes `event_id = hash_uri(canonical_bytes)` — the empty strings are part of the canonical form and the resulting hash becomes the ID. Callers set `space_id` / `room_id` in subsequent events using the derived value.

---

## D-016 — Layer 9: `validate_steps_8_13` Is Read-Only; Callers Control Insertion

**Date:** 2026-04-28
**Layer:** 9 — Message Exchange
**Spec reference:** 3.2.6

Steps 8–13 of the validation pipeline are implemented as a pure read-only function (`validate_steps_8_13`). It does not mutate the `EventStore` or `DagGraph`. Mutation happens only in `accept_event`, which calls the validator and then inserts on success.

This design lets callers inspect the specific failure reason before deciding whether to buffer (step 9 `HeldPending`) or reject (all other errors). It also makes the validator easily testable in isolation without needing mutable state.

Step 10 (DAG structural check) intentionally duplicates the logic from `DagGraph::add_event` rather than extracting a shared helper, because the DAG check requires a read-only view — there is no `DagGraph::validate_only` method and adding one would be scope creep.

---

## D-017 — Layer 9: Test Setup Merges Two DAG Roots via Invite `prev_events`

**Date:** 2026-04-28
**Layer:** 9 — Message Exchange
**Spec reference:** 3.2.5

In test setup, `state.space_create` and `state.room_create` are both DAG root events (empty `prev_events`). Without intervention, they remain as two independent tips indefinitely. The first membership event (`membership.invite`) references both roots as `prev=[space_id, room_id]`, merging the two roots into a single linear chain and leaving exactly one tip. This ensures message events have a single, unambiguous predecessor for `prev_events` in tests.

This is a test-only convention. In production, the protocol does not require roots to be merged — two persistent tips are valid DAG state.

---

## D-018 — meta_atts Key Namespace: Dot Separator, Reverse-Domain Ownership

**Date:** 2026-04-28
**Layer:** 0 (protocol specification)
**Spec reference:** 3.1.3

`meta_atts` keys follow a dot-separated namespace scheme: `<namespace>.<key>`. The `xgen.` prefix is reserved for specification use. Third-party keys MUST use reverse-domain prefixes (e.g. `com.example.my_key`). Key segments use `snake_case`. Max key length 128 characters. Values are strings; structured values must be JSON-encoded as strings rather than embedded as nested objects.

Spec 3.1.3 updated accordingly.

---

## D-019 — Transport Pluggability: WebSocket as Default, Alternative Streams Permitted

**Date:** 2026-04-28
**Layer:** 0 (protocol specification)
**Spec reference:** 3.3.1

WebSocket over TLS is the mandatory production transport. However, the spec explicitly permits operators to substitute any reliable bidirectional stream transport (Tor hidden services, I2P, pluggable transport proxies) without protocol-layer changes. This is noted in spec 3.3.1. DPI-resistance via custom transports is flagged as a Phase 3 investigation area — no Phase 1 or Phase 2 work required.

---

## D-020 — File Placement: Two-Tier Model (System Files vs User-Configurable Files)

**Date:** 2026-04-28
**Layer:** 0 (deployment model)
**Spec reference:** IMPLEMENTATION_GUIDE_ph1.md — Deployment Model

Refined the Pattern A deployment model into an explicit two-tier system. Tier 1 (system files: config, registries, announcements) is mandatory co-location with the binary — not configurable. Tier 2 (keypair, TLS cert, logs, UI settings) defaults to binary folder but can be redirected via explicit config fields. This accommodates HSM-backed keys, OS keystore integration (Phase 2), and system log aggregation without scattering files by default. No file moves silently — every Tier 2 redirect requires an explicit config entry.

---

## D-021 — Self Account (`self`): Local-Only Synthetic Identity, Post-Phase-1

**Date:** 2026-04-28
**Layer:** 0 (deferred post-Phase-1 feature)
**Spec reference:** —

A `self` account (analogous to Skype's own-account or Telegram's Saved Messages) is planned for implementation after the Phase 1 smoke test, during local testing. Design decision: `self` is a local-only synthetic Identity with its own keypair, never registered on any Node and never appearing in federation. It signs local Events but those Events are never broadcast. The `self` account must be accessible from any user client connecting to the Node — it is not device-local. In Phase 2, a "Saved Messages" Space may be implemented as a proper DM Space where both sides of the DM are the user's own keypair.

---

## D-022 — xgen-core Library Split: Deferred to Post-Phase-1

**Date:** 2026-04-28  
**Layer:** 0 (architecture — deferred)  
**Spec reference:** —  
**Resolved by:** D-044 (2026-05-13)

All protocol logic currently lives in `xgen-node/src/`. A planned post-Phase-1 restructure will extract this into a new `xgen-core` crate: GPL-licensed from day one, the primary library for third-party developers. `xgen-node` and `xgen-client` become thin runtime shells wrapping `xgen-core`, retaining their BSL 1.1 wrapper. `xgen-common` remains as shared serde types.

Rationale for deferring: restructuring crates mid-implementation introduces risk right before the Phase 1 finish line. Do the smoke test first, tag Phase 1 complete, then restructure as the first Phase 2 prep task.

---

## D-023 — Traffic Masking / DPI Resistance: Phase 3 Investigation

**Date:** 2026-04-28
**Layer:** 0 (deferred — Phase 3)
**Spec reference:** 3.3.1

Deep-packet-inspection resistance (obfuscating XGen traffic to evade state-level network surveillance) is acknowledged as a legitimate concern. Phase 1 and Phase 2 impact: none — transport pluggability (D-019) already ensures Tor/I2P are usable without protocol changes, which is sufficient for most adversarial environments. Active DPI resistance (disguising XGen traffic as generic HTTPS, pluggable transport integration) is flagged as a Phase 3 area of investigation. Steganographic transport is explicitly out of scope for the core protocol.

---

## D-024 — History Sync: Individual Events, Not Batch Snapshot

**Date:** 2026-04-28
**Layer:** 10 — Smoke Test
**Spec reference:** 3.7.10 (step 8), 3.7.11

The spec requires Node A to "send full Space state and Room Event history to Node B" (step 11 of the smoke test) but does not prescribe a wire format. Two options were considered: (a) individual Events sent one by one, (b) a new batch snapshot message type.

Chose **individual Events**. Rationale: Events are already the atomic protocol unit; every federated Node must be able to validate each Event independently; no new message type is needed; and the individual approach scales correctly to Phase 2 where `transport.sync_request` handles catching up on missed Events after reconnection — it is additive, not a replacement. Batch delivery would require defining a new message type that Phase 2 would likely supersede anyway.

In the smoke test, Node A sends history Events in insertion order over the active connection, followed by the `state.federation_add` Event (which references the pre-history tip as its `prev_events`, and therefore must be received after the history to be correctly linked in Node B's DAG). Connection is closed with `transport.goodbye` to signal end of sync.

---

## D-025 — File Naming Convention: `xgen-node_*` and `xgen-client_*` Prefixes

**Date:** 2026-04-29
**Layer:** 0 (deployment model)
**Spec reference:** IMPLEMENTATION_GUIDE_ph1.md — Deployment Model, Ch4 section 4.3

All runtime files produced or consumed by a binary are prefixed with the binary name: `xgen-node_*` for Node files, `xgen-client_*` for client files.

Rationale: when two Node instances run side by side for testing (NodeA and NodeB folders), every file in the folder is immediately identifiable by name alone — no ambiguity about which binary owns it. Also makes glob patterns unambiguous in scripts (`xgen-node_*.db`, `xgen-client_*.toml`).

Applied to: config (`xgen-node_config.toml`), keypair (`xgen-node_keypair.enc`), state file (`xgen-node_state.json`), databases (`xgen-node_identities.db`, `xgen-node_federation.db`), logs (`xgen-node.log`). Space databases are in a `spaces/` subfolder and are named by space ID hex — the subfolder itself provides the ownership context.

---

## D-026 — Status File (`*_state.json`): Plain JSON, File Permissions as Security Boundary

**Date:** 2026-04-29
**Layer:** 0 (deployment model / CLI design)
**Spec reference:** Ch4 section 4.14

**What the state file contains**

The running Node writes `xgen-node_state.json` to its application folder every 5 seconds. It contains operational metadata: node ID (a public key — already public by protocol design), uptime, connected client identity IDs and display names, federated peer endpoints, hosted space names, and event counts. The client writes `xgen-client_state.json` with: identity ID, display name, known nodes, joined spaces, and last activity timestamps.

**Why it is safe for Phase 1**

No secret material ever enters the state file. The private key lives only in `*_keypair.enc` (encrypted at rest). Signatures are computed in memory and never written to disk in plaintext. The state file contains only information that is already visible to any authenticated participant in the protocol — a connected client can already see who else is in a Space.

**What it leaks and to whom**

The state file leaks topology: who is connected to this Node, which peers it federates with, which Spaces it hosts. This is only a concern if a third party has filesystem read access to the Node's application folder. On a personal development machine: not a concern. On a shared server: the file MUST be protected by OS-level file permissions (Unix: `chmod 600`; Windows: restrict ACL to the operator account). The Node SHOULD set these permissions itself on first write.

**Planned improvements for Phase 2**

Three improvements are planned but explicitly deferred beyond Phase 1:

1. **Redact identity IDs from state file** — replace full `pubkey_uri` values with display names only, or truncated IDs. The full public key of a connected user is already public, but there is no reason to persist it in a file that may be read by monitoring tools.

2. **Separate admin socket** — replace the file-based status mechanism with a Unix domain socket (or named pipe on Windows) that only the operator's process can connect to. Status commands connect to the socket rather than reading a file. This eliminates the file entirely and makes the data available only to processes with the right OS credentials.

3. **Encrypted state file** — encrypt the state file with a key derived from the node keypair passphrase. Only the operator who can unlock the keypair can read the state file. Adds meaningful protection on shared infrastructure without requiring the admin socket approach.

For Phase 1, file permissions are the sufficient and correct mitigation. The planned improvements are recorded here so they are not forgotten when Phase 2 deployment hardening is scoped.

---

## D-027 — CLI Observability Commands: Phase 1 Scope Extension

**Date:** 2026-04-29
**Layer:** 0 (CLI design — Phase 1 scope extension)
**Spec reference:** Ch4 section 4.16

The original Phase 1 definition of done (spec 3.7.11, IMPLEMENTATION_GUIDE_ph1.md Layer 10) specifies the smoke test as the completion criterion. It does not specify a CLI interface beyond what is needed to drive the smoke test.

The following commands are added to Phase 1 scope as a deliberate extension:

**xgen-node:** `status`, `connections`, `spaces`, `peers`, `identity list`
**xgen-client:** `status`, `spaces`, `whoami`

**Rationale:** the smoke test proves the library works in-process. Runnable binaries need to be observable — an operator running two Nodes on localhost needs to see that they are alive, that clients are connected, and that federation is active. Without these commands, the only evidence the system works is log output. These commands transform log output into structured, queryable state.

All observability commands read `xgen-node_state.json` or `xgen-client_state.json` (D-026) — they do not open a new network connection to the running process. This keeps them instant and dependency-free.

**These commands are NOT Phase 2 work.** They are Phase 1 CLI completeness. Phase 2 will replace or supplement them with a GUI dashboard. The state file mechanism (D-026) persists into Phase 2 as the data source for that dashboard.

**What is explicitly NOT in Phase 1 CLI scope:**
- Admin operations that modify Node state (ban identity, force-disconnect peer, etc.) — Phase 2
- Real-time streaming output (live event feed, live connection monitor) — Phase 2
- Auth Module management commands — Phase 2
- Multi-node management (controlling a remote Node) — Phase 2

---

## D-028 — `--help` Built-in: clap Derive Macros, Section 4.16 as Authoritative Source

**Date:** 2026-04-29
**Layer:** 0 (CLI design)
**Spec reference:** Ch4 section 4.16

`clap` with derive macros generates `--help` output automatically from doc comments (`///`) on struct fields and command variants. The help text in the source code is therefore documentation — it must match section 4.16 of Ch4 exactly.

The authoring rule: write section 4.16 first. Copy the argument descriptions and examples from 4.16 into the Rust doc comments. Never write help text in the code first and retrofit it into 4.16 — the spec is the source of truth, the code is the implementation.

Both `xgen-node --help` and `xgen-client --help` (and all subcommand `--help` variants) are generated by clap at compile time from these doc comments. No hand-written help strings.

---

## D-030b — Runtime file placement: GetModuleFileNameW on Windows; data_dir from config path

**Date:** 2026-04-29
**Layer:** 0 (deployment / binary wiring)
**Spec reference:** D-025 (file naming and placement)

### Problem

`xgen-node init` must create its runtime files (keypair, config, identities DB, state file) in a deterministic, predictable location. The natural choice is the directory that contains the running executable. Rust's `std::env::current_exe()` is sufficient on Linux/macOS but has documented edge cases on Windows: Windows Defender, UAC elevation, App Compatibility shims, and some third-party security products can run a process from a shadow copy at a temp path, causing `current_exe()` to return the temp location rather than the original binary location.

Additionally, Phase 1 requires running two Node instances simultaneously for testing (Node A on port 8080, Node B on 8081). When both nodes share the same binary, a single `exe_dir()` would cause Tier-1 file collisions between instances.

### Decision

**1 — `exe_dir()` on Windows uses `GetModuleFileNameW` directly.**

`GetModuleFileNameW(NULL, ...)` (Win32 API, `windows-sys` crate, Windows-only dependency) returns the full path of the module loaded into the calling process. This is the definitive answer to "where does this executable live" — it is immune to CWD, PATH lookup order, symlinks, shell wrappers, and any launcher that might shadow-copy the binary. The function is called with a growing buffer starting at `MAX_PATH` (260) and doubling until the path fits, ensuring correctness for paths beyond `MAX_PATH` (e.g., with `\\?\` extended-length prefix). On non-Windows the standard library's `current_exe()` is used unchanged.

`exe_dir()` panics rather than falling back to `"."` (the CWD). Silent fallback to CWD was the original failure mode — files appeared in a "random" working directory instead of next to the executable. A panic with a clear message is strictly better: it tells the operator exactly what is wrong rather than silently polluting the working directory.

**2 — `data_dir` is derived from the config file path.**

All Tier-1 runtime files are placed in the parent directory of the config file in use:

```
data_dir = config_path.parent()
```

- **Without `--config`:** `config_path` defaults to `exe_dir()/xgen-node_config.toml`, so `data_dir = exe_dir()`. Tier-1 files are co-located with the binary — matches spec D-025.
- **With `--config /path/to/config.toml`:** `data_dir = /path/to/`. This allows multiple Node instances to run from the same binary with fully isolated data directories, by giving each instance its own config file in its own directory.

This rule is simple, explicit, and composable: operators who need multi-instance deployments create one directory per instance and specify `--config`. Operators who run a single instance (the common case) run `xgen-node init` with no flags and get everything in the binary's directory, as expected.

**3 — `xgen-node init` accepts `--passphrase` flag.**

`init` calls `rpassword::prompt_password()` to read the passphrase interactively. This blocks automated setup (CI, scripted deployments, smoke-test harnesses). The `--passphrase` flag provides the passphrase directly without prompting. It is intentionally undocumented in `--help` (hidden flag) — it is not intended for interactive human use, only for scripting. Passing an empty string produces a keypair encrypted with empty passphrase (Phase 1 Local Node mode).

### Files affected

- `xgen-node/src/main.rs` — `exe_dir()`, `main()`, `cmd_init()`, `run_node()`, all observability commands
- `xgen-node/Cargo.toml` — `windows-sys = { version = "0.59", features = ["Win32_System_LibraryLoader"] }` as `[target.'cfg(windows)'.dependencies]`

---

## D-031b — Phase 1 Node configuration reference (xgen-node_config.toml)

**Date:** 2026-04-29
**Layer:** 0 (deployment / reference)
**Spec reference:** Ch4 section 4.8.1

`xgen-node init` generates a default `xgen-node_config.toml` in the data directory. Below is the canonical Phase 1 reference config with all fields documented.

```toml
# XGen Protocol Node — Phase 1 configuration
# Generated by: xgen-node init
# All paths are absolute. Relative paths resolve from the working directory
# at startup, which may differ from the binary location — use absolute paths.

[node]
# WebSocket endpoint this Node listens on.
# Phase 1: ws:// (plain TCP, localhost only).
# Phase 2: wss:// (TLS, public endpoint).
listen = "ws://127.0.0.1:8080/xgen"

# Local Node mode: skip signature verification on incoming events.
# TRUE for Phase 1 development. FALSE for any production or multi-operator setup.
local_mode = true

[paths]
# Ed25519 signing keypair, encrypted at rest (ChaCha20-Poly1305 + Argon2id).
# Phase 1: encrypted with empty passphrase. Phase 2: OS keystore or HSM redirect.
# This is the ONLY mandatory path. The Node will not start without it.
keypair_path = "C:\\XGen\\NodeA\\xgen-node_keypair.enc"

# Optional: redirect log output. Omit to suppress file logging (stderr only).
# log_path = "C:\\XGen\\NodeA\\xgen-node.log"

# Optional: directory for per-space DAG stores. Omit to use in-memory only.
# spaces_dir = "C:\\XGen\\NodeA\\spaces"
```

### Field reference

| Field | Required | Default if omitted | Phase 2 change |
|---|---|---|---|
| `node.listen` | yes | `ws://127.0.0.1:8080/xgen` | Change to `wss://` with real hostname |
| `node.local_mode` | yes | `true` | Set to `false` for production |
| `paths.keypair_path` | yes | — (Node refuses to start) | May redirect to HSM path |
| `paths.log_path` | no | no file logging | Route to syslog aggregator |
| `paths.spaces_dir` | no | in-memory only | Persistent DAG store directory |

### Multi-instance setup (Phase 1 testing)

To run two Nodes on the same machine:

```
E:\XGen\NodeA\xgen-node.exe --config E:\XGen\NodeA\xgen-node_config.toml init
E:\XGen\NodeB\xgen-node.exe --config E:\XGen\NodeB\xgen-node_config.toml init
```

Edit Node B's config to use port 8081. Each instance has its own keypair, identity registry, and state file — no collisions.

---

## D-029 — xgen-client depends on xgen-node lib for Phase 1 binary wiring

**Date:** 2026-04-29  
**Layer:** 0 (binary wiring)  
**Spec reference:** D-022 (xgen-core crate split, Phase 2)  
**Resolved by:** D-044 (2026-05-13)

`xgen-client` depends directly on the `xgen-node` library crate for Phase 1 binary wiring. This gives the client access to the transport layer (`Connection`, `connect_url`), wire types (`Event`, `IdentityMessage`, etc.), federation handshake, identity registration protocol, event building, and crypto — without duplicating ~2 000 lines of code.

The "circular" concern mentioned earlier was conceptual (two binaries sharing a library), not a Cargo constraint. `xgen-client → xgen-node-lib` is a valid, acyclic dependency.

In Phase 2, D-022 (xgen-core crate) extracts the shared protocol logic from `xgen-node` into a new `xgen-core` library. Both `xgen-node` and `xgen-client` will depend on `xgen-core` instead. The direct `xgen-client → xgen-node` dependency is replaced at that point.

---

## D-037b — Node deployment model: systray singleton with detachable admin window

**Date:** 2026-05-07  
**Layer:** 6 (UI / deployment)  
**Spec reference:** Ch6 §6.1, §6.4  

`xgen-node.exe` is a singleton process — it starts once and runs permanently. The UI is not the lifecycle host; the process is.

**Desktop deployment (normal launch):**
- Node starts → sits in system tray as a minimal persistent icon
- Systray icon reflects Node health at a glance (green = healthy, amber = warning, red = error)
- Double-click or right-click → Open Dashboard opens the full Tauri admin window
- Closing the admin window does not stop the Node — Node continues running in the tray
- Right-click context menu: Open Dashboard, View Logs, Stop Node

**Server/headless deployment:**
- `--service` flag or OS service wrapper (Windows Service, systemd, launchd)
- No systray, no window — process runs fully headless
- Managed via OS service tooling; logs routed to system aggregator

**One binary, two personalities.** No separate service executable. Launch mode determines behaviour.

**Architectural horizon (not scheduled):** long-term, Node administration via privileged client identity — the operator manages their Node through the XGen client itself as a protocol-native admin surface. This is philosophically aligned with XGen's identity-first model but requires a stable client first and has a bootstrapping challenge. Noted for post-Phase 2 consideration.

---

## D-038b — Tier badge placement: Node property, not member property

**Date:** 2026-05-07  
**Layer:** 6 (UI)  
**Spec reference:** Ch6 §6.11.4, Appendix E  

The Auth tier is a property of the **Node**, not of an individual member or message. It describes what authentication level the Node requires and enforces for the current session. A user authenticated at Tier 1 on one Node may be Tier 2 on another — the tier is session-scoped, not identity-scoped.

**Displaying tier badges on individual messages or member list entries is architecturally incorrect.** It implies tier is a permanent attribute of the person, which it is not.

**Correct placements:**
- Console status bar: `Joe / @joe [T1] · Space › #Room` — reflects the current session's auth level on the connected Node
- Node status panel in client sidebar — describes the connected Node's tier requirement
- Node admin dashboard — the Node's own tier displayed prominently

**Removed placements:**
- `room.message.decorator` slot in messages — removed
- Member list entries — removed
- Navigation footer local user identity — removed

The `room.message.decorator` slot remains in place as the module injection point. Tier badge removal does not affect the slot structure.

---

## D-039b — Application shutdown model: × to systray, explicit exit only

**Date:** 2026-05-07  
**Layer:** 6 (UI / deployment)  
**Spec reference:** Ch6 §6.11, Appendix E, D-037b  

Closing the window with × does not exit either application. Both applications minimize to the system tray. Explicit exit is always a deliberate user action.

**× button behaviour (both apps):**
- Hides the window, process continues running
- Client: stays connected, session live, logs flowing
- Node: keeps serving clients and federation peers, no change
- Consistent with D-037b (Node window is detachable from Node process)

**Exit paths — phased implementation:**

**Phase 2 skeleton (immediate):**
- In-app exit button in the nav footer alongside the user identity / Node health indicator
- Client nav footer: Disconnect button (drops connection, stays in window) + Exit button (CLOSING → flush → disconnect → process exits)
- Node nav footer: Restart button + Stop Node button (CLOSING → drain sessions → session footer → process exits)
- × button: disabled or no-op until systray is implemented
- This is the only exit path in the skeleton phase

**Phase 2 (systray implementation):**
- × minimizes to systray properly
- Systray right-click context menu → Exit (Client) / Stop Node (Node)
- Systray is the safety net when UI is unresponsive but process is alive

**Phase 3:**
- `xgen-client.exe --stop` and `xgen-node.exe --stop` CLI flags
- Sends graceful shutdown signal via local socket or PID file
- Works even when both UI and systray are unresponsive
- Built on the same IPC channel as Ch6 §6.9 Console input protocol
- Last resort before Task Manager kill (which produces no session footer)

**Graceful shutdown sequence (both apps, all exit paths):**
1. Enter `CLOSING` state — logged, status indicator updates
2. Flush outbound Event queue (max 2s grace period, then force-close)
3. Send `transport.close` to connected Node(s)
4. Write session footer to log
5. Archive session log
6. Process exits

**Appendix E clarification:** “Window close” in Appendix E means explicit exit action, not the × button. The × button triggers minimize-to-systray, not CLOSING state. CLOSING is only entered via an explicit exit action (in-app button, systray menu, or `--stop` flag).

**Nav footer button placement (from JozefN review, 2026-05-07):**
Two compact action buttons sit in the nav footer alongside the identity/health indicator — always visible, always reachable, deliberate enough not to be hit accidentally.
- Client: Disconnect + Exit
- Node: Restart + Stop Node

---

## D-040 — Idle presence state: social signal and resource hint

**Date:** 2026-05-07  
**Layer:** 3 (Specification) / 6 (UI)  
**Spec reference:** Ch3 §3.6 (Identity), Ch3 §3.9 (State resolution), Appendix E  

Idle is a presence state indicating a connected member has produced no non-keepalive protocol activity for a configurable period. It has two distinct roles: a social signal visible to other room members, and an internal resource hint used by the Node.

**What idle is:**
- A runtime presence state: `online` → `idle` → `online`
- A federated presence signal — propagated to federated Nodes so their members see correct presence
- An internal Node resource hint — the Node may deprioritize idle clients (e.g. lower Event delivery queue priority, reduced in-memory session cache) without exposing those decisions externally

**What idle is not:**
- A DAG Event — presence is ephemeral, not historical protocol state
- A log entry at INFO level — idle/wake transitions are DEBUG at most, or not logged
- A lifecycle state in Appendix E — idle does not interrupt the client’s READY state
- A kickout mechanism — idle clients are never disconnected for inactivity (D-039b)

**Trigger — what counts as activity:**
From the Node’s perspective, activity is any non-keepalive message received from the client — sending a message, issuing a command, joining a room. Pure Event delivery (Node pushing to client) does not reset the idle timer. The Node cannot observe client-side UI interactions.

**Timeout configuration:**

| Setting | Location | Default | Notes |
|---|---|---|---|
| `idle_timeout_ms` | `client_config.toml` | 900000 (15 min) | User preference |
| `idle_timeout_max_ms` | Node config | 1800000 (30 min) | Operator ceiling — takes precedence if stricter than client setting |

If the Node operator sets a maximum idle timeout, the effective timeout is `min(client_setting, node_max)`. If the client sets no preference, the Node default applies.

**Wake-up:** any non-keepalive message from the client immediately returns presence to `online`. No explicit wake command required.

**Federation:** idle/online presence state is federated — other Nodes propagate it to their members so cross-Node room participants see correct presence. The Node’s internal resource management decisions (cache eviction, queue deprioritization) are local and never cross federation.

**Keepalive logging:** ping/pong keepalive entries are logged at `DEBUG` level, not `INFO`. Over a 5-hour idle session this prevents hundreds of identical INFO entries burying meaningful protocol events. Only the initial connection, state transitions, and significant events are logged at INFO.

**Admin actions remain separate:** idle state has no relationship to `membership.kick` or `membership.ban`. Those are admin-initiated protocol Events for disturbance, not inactivity. An idle user is still a full member.

**Phase 2 note:** the presence signal mechanism — how idle/online state is communicated between Node and client, and across federation — requires a Ch3 Phase 2 specification entry. The EventType or message type for presence updates is not yet defined. This decision records the intent and constraints; the wire format is a Phase 2 spec task.

---

## D-048 — Layer 14 DM Space Promotion: DmProposal in NodeRuntime, not SpaceState

**Date:** 2026-05-14
**Layer:** 14 (DM Space Promotion)
**Spec reference:** Spec 3.16.1–3.16.4

### Context

The promotion proposal is in-memory state — the proposer sends `dm.promote_propose`, the Node stores the proposal, the other member confirms or rejects. The spec says proposals are not DAG events.

### Decision 1 — Proposal storage location

The proposal is stored in `NodeRuntime::dm_proposals: HashMap<String, DmProposal>` (keyed by space_id), not in `SpaceState`. `SpaceState` is replayed from the DAG on restart; proposals do not survive restart. `NodeRuntime` holds the ephemeral operational state that lives only during a running Node session.

### Decision 2 — dm_constraints_active flag on SpaceState

`SpaceState` gains `dm_constraints_active: bool` (true for DM spaces, set to false when `state.dm_promote` is applied). The constraint checks live in `apply_invite`, `apply_room_create`, and `apply_federation_add`. This makes constraints enforced at the DAG-apply layer — replay of the event log correctly lifts constraints when `state.dm_promote` is encountered.

### Decision 3 — state.dm_promote signed by Node keypair

Per spec 3.16.3 Step 4: `state.dm_promote` is produced and signed by the Node, not by either member. `handle_confirm` in `dm_promotion.rs` takes `node_key: &SigningKey` and calls `sign_event`. The sender field is the Node's identity_id. Test `promote_signed_by_node_not_member` verifies this.

### Scope

`dm_promotion.rs` provides pure handler functions — no WebSocket I/O. Delivery of `dm.promote_propose` to the other member and delivery of `state.dm_promote` to both members is the Node runtime's responsibility (xgen-node wiring, not implemented in Phase 2 library). The handlers return `deliver_to` identity IDs so the caller knows who to notify.

---

## D-047 — Layer 13 Pending Event Timeout: drain_timed_out takes explicit now parameter

**Date:** 2026-05-14
**Layer:** 13 (Pending Event Timeout)
**Spec reference:** Spec 3.9.6, WD-08 (30-second timeout)

### Context

Spec 3.9.6 requires pending events (those awaiting unknown prev_events) to be discarded after a timeout, emitting error 4002 (predecessor_timeout). The question was how to drive the timeout check: a monotonic clock dependency inside `PendingBuffer`, or an explicit parameter at the call site.

### Decision

`drain_timed_out` accepts an explicit `now: std::time::Instant` parameter rather than calling `Instant::now()` internally.

**Reason:** an explicit `now` makes the function testable without sleeping or mocking. Tests pass `Instant::now() + Duration::from_secs(31)` to trigger the timeout instantly. The background task in xgen-node passes `std::time::Instant::now()` in production — one extra token, no testability cost.

The timeout constant is `PENDING_TIMEOUT_SECS: u64 = 30` — a named `pub const` in `dag/pending.rs` so the value is tunable from one place (WD-08).

### Sweep task wiring

A background tokio task in `xgen-node/src/main.rs` calls `drain_timed_out(Instant::now())` on every Space's `PendingBuffer` every 5 seconds. For each discarded entry it logs at `WARN` with `event_id`, `missing_predecessors`, and `error_code = 4002`.

---

## D-046 — Layer 12 State Resolution: identity_home_nodes parameter and Layer 3 scope restriction

**Date:** 2026-05-14
**Layer:** 12 (State Resolution Algorithm)
**Spec reference:** Spec 3.9.3 (seven-layer resolution stack), 3.9.8 (error codes)

### Context

The Layer 12 `resolve()` function implements the seven-layer priority stack (spec 3.9.3). Two decisions beyond spec prescription are recorded here.

### Decision 1 — identity_home_nodes as explicit parameter

`IMPLEMENTATION_GUIDE_ph2.md` specifies `resolve(conflicts, space_state)`. The guide's two-parameter signature is insufficient to implement Layers 3, 5a, and 5b, all of which require knowing which home Node each identity is registered on. `SpaceState` does not hold this mapping (it holds federation_nodes, which is a different concept — the set of Nodes a Space has federated with, not the registration point of each identity).

**Decision:** `resolve()` signature is:
```rust
pub fn resolve<'a>(
    conflicts: &'a [Event],
    space_state: &SpaceState,
    identity_home_nodes: &HashMap<String, String>,
) -> Result<&'a Event, ResolutionError>
```

The caller (Node's message handler) provides `identity_home_nodes` from the identity registry. This keeps `resolve()` a pure function with no registry I/O inside the algorithm itself.

### Decision 2 — Layer 3 restricted to membership and key-rotation events

Spec 3.9.3 Layer 3 description: "Home Node assertion for Identity's own state." The phrase "Identity's own state" was narrowly interpreted: Layer 3 applies only to events whose state key is in the membership or system.key_rotation category.

Without this restriction, Layer 3 incorrectly fires for events like `state.room_update` — two concurrent room updates by two admins from different Nodes would be resolved by Layer 3 (which would pick the event from whichever Node happens to be the "affected identity's" home Node, a concept that doesn't apply to shared room state). Layer 3 must not fire for shared state — it is only meaningful when one specific identity's own record is in contention.

**Implementation:** `layer3_home_node_assertion` checks `is_membership_event(&first.event_type) || matches!(first.event_type, EventType::SystemKeyRotation)` before running. All other event types fall through to Layer 4.

### SpaceState extension

`SpaceState` gains `node_priority_order: Vec<String>` (populated by `state.node_priority` events via `apply_event`). This field is required by Layer 5a. Index 0 = highest priority Node.

### Outcome

- 226 tests pass (218 xgen-core + 8 xgen-node)
- All ten Layer 12 tests pass including Layer 5a `node_priority_respected`
- Layer 3 bug caught by test: applying to `StateRoomUpdate` gave a spurious early win before Layer 5a could run

---

## D-044 — xgen-core crate split executed

**Date:** 2026-05-13  
**Layer:** Phase 2 prerequisite  
**Spec reference:** D-022 (planned), D-029 (temporary arrangement, now resolved)

### Context

All shared protocol logic lived in `xgen-node/src/`. `xgen-client` depended directly on the `xgen-node` library crate (D-029 — intentional temporary arrangement). This was always planned to be resolved before Phase 2 protocol work began (D-022).

### Decision

Extracted all shared protocol logic from `xgen-node/src/` into a new `xgen-core` crate. `xgen-core` is GPL-2.0-or-later from day one — the public library that the XGen ecosystem builds on. `xgen-node` and `xgen-client` are now thin shells that depend on `xgen-core`.

**Module allocation after split:**

| Location | Contents |
|---|---|
| `xgen-core/src/` | `crypto/`, `wire/`, `dag/`, `transport/{auth,client,connection}`, `node/`, `federation/`, `identity/`, `space/`, `message/` |
| `xgen-node/src/` | `main.rs`, `lib.rs` (re-exports xgen-core), `lifecycle.rs`, `transport/server.rs`, `tests/` |
| `xgen-client/src/` | `main.rs`, `lib.rs`, `batch.rs`, `identity.rs`, `lifecycle.rs` |

**Adapter pattern in xgen-node transport:** `xgen-node/src/transport/mod.rs` declares `pub mod server` and re-exports `auth`, `client`, `connection` from `xgen_core::transport`. This means all `crate::transport::*` paths in `xgen-node`'s main.rs and tests continue to resolve correctly without modification.

**Test relocation:** inline tests in `federation/mod.rs` and `identity/mod.rs` that required `Server` (Node-specific) were moved to `xgen-node/src/tests/federation_integration.rs` and `xgen-node/src/tests/identity_integration.rs`. Pure unit tests that don't need a server were kept in xgen-core.

### Outcome

- 173 tests pass (`cargo test`) — zero behaviour change
- Release build clean (`cargo build --release`)
- D-022 resolved: xgen-core exists, GPL-licensed, all protocol logic lives there
- D-029 resolved: xgen-client no longer depends on xgen-node

---

## D-055b — Phase 2 server-side handler wiring: node_endpoint in Hello, identity replication routing

**Date:** 2026-05-14
**Layer:** Integration (server-side protocol handler gap closure)
**Spec reference:** 3.4.2 (federation.hello), 3.13.1–3.13.4 (identity replication), 3.3 (transport Inbound routing)

### Context

After Part A of integration testing (J-056), `xgen-node/src/main.rs` `process_inbound()` only handled `Inbound::Identity` and `Inbound::Event`. All Phase 2 Inbound variants added in M1 (`Inbound::IdentityReplicate`, `Inbound::DmControl`, `Inbound::Migration`, `Inbound::Bootstrap`, `Inbound::Reputation`, `Inbound::Mls`) hit `_ => {}` and were silently dropped.

The immediate blocker for smoke-test-ph2 Part B was step 22: the smoke test sends `identity.replicate` to Node B and expects `identity.replicate_ack`. Without a handler, Node B silently dropped the message and the test failed.

A deeper structural gap was also identified: `FederationRelationship` had no `peer_url` field, so after a federation handshake the Node had no stored return address for the peer. This made outbound identity replication (spec 3.13.1 — home Node pushes to replicas after registration) impossible.

### Decisions

**1. `node_endpoint` field added to `FederationMessage::Hello`**

Advisory field (excluded from canonical signature — not in `HELLO_FIELDS`). The initiating Node populates it from `self_url: Option<String>`, a new parameter to `run_initiating()`. The receiving Node extracts it as `peer_url` on the `FederationSession`. Rationale: the receiving Node has no other way to learn the peer's WebSocket URL after the handshake completes over an inbound TCP connection.

Backward compatible: `#[serde(skip_serializing_if = "Option::is_none")]` — old nodes receiving the new field ignore it; new nodes receiving old messages get `None`.

**2. `peer_url: Option<String>` added to `FederationSession` and `FederationRelationship`**

`FederationSession.peer_url` is populated by `run_receiving()` from the Hello's `node_endpoint`. `FederationRelationship.from_session()` copies it across. `NodeRuntime.peer_urls: HashMap<String, String>` (node_id → URL) gives the server a lookup table for outbound replication.

**3. `handle_identity_replicate_msg` added to `xgen-node/src/main.rs`**

Handles `Inbound::IdentityReplicate(Replicate)`: deserialises `identity_record: Value` → `IdentityRecord`, calls `handle_incoming_replicate()`, sends `ReplicateAck` on success or `transport.error` (code 3020) on version-stale rejection.

**4. `push_identity_to_peers` added to `xgen-node/src/main.rs`**

After a successful identity registration, spawns an async task per known peer URL: connect → authenticate → send `identity.replicate` → await `identity.replicate_ack` → record in `replica_registry`. Failures are logged but not fatal (registration already confirmed to the client).

**5. `run_initiating()` call sites updated**

All 4 call sites in `xgen-client/src/main.rs` and 3 in test files updated with the new `self_url` argument. The two federation steps in `smoke-test-ph2` pass the node_b URL; all other call sites pass `None`.

### Outcome

- 300/300 tests passing (292 xgen-core + 8 xgen-node)
- Step 22 blocker resolved: `identity.replicate` is now handled server-side
- Identity replication infrastructure complete per spec 3.13.1–3.13.4
- All other Phase 2 Inbound variants (`DmControl`, `Migration`, `Bootstrap`, `Reputation`, `Mls`) remain `_ => {}` — not required for smoke-test-ph2 steps (those steps use hardcoded `pass!()` or send content as DAG events)

---

## D-045 — Phase 2 wire type names: spec authoritative over implementation guide

**Date:** 2026-05-13
**Layer:** 11 (Wire Format Phase 2 Extensions)
**Spec reference:** 3.9–3.16

### Context

While implementing Layer 11, several wire type names in `IMPLEMENTATION_GUIDE_ph2.md` were found to diverge from the canonical wire strings in `docs/xgen_ch3_specification.md`. The spec is always authoritative.

### Discrepancies resolved

| Guide wire name | Spec wire name | Spec section |
|---|---|---|
| `migration.complete` | `migration.transfer_complete` | 3.12.5 |
| `migration.verify_ok` | `migration.verified` | 3.12.6 |
| `migration.verify_fail` | `migration.verification_failed` | 3.12.6 |
| `migration.tail_batch` | (not a separate type — tail uses `migration.event_batch`) | 3.12.5 |
| `migration.abort` | (not in spec type registry — state machine handles failure) | 3.12.3 |
| `bootstrap.node_register` | `bootstrap.register` | 3.14.3 |
| `bootstrap.node_register_ack` | `bootstrap.register_ack` | 3.14.3 |
| `bootstrap.node_lookup` | (not a wire type — directory lookup is HTTP GET) | 3.14.4 |
| `bootstrap.node_lookup_response` | (not a wire type — HTTP response, not WebSocket) | 3.14.4 |

### Types added beyond the guide (present in spec)

| Type | Spec section | Reason |
|---|---|---|
| `state.space_migrate` | 3.12.7 | Permanent DAG event recording completed migration |
| `migration.failed` | 3.12.3 | Source Node notifies owner of failure |
| `migration.batch_ack` | 3.12.4 | Destination acknowledges each batch |
| `migration.federation_notify` | 3.12.8 | Courtesy notification to federated peers |
| `bootstrap.keepalive` | 3.14.7 | Node refreshes directory TTL |
| `bootstrap.keepalive_ack` | 3.14.7 | Bootstrap Node acknowledges |
| `bootstrap.deregister` | 3.14.7 | Node explicitly removes itself |
| `mls.key_package_request` | 3.10.3 | Node requests KeyPackage from peer Node |
| `mls.key_package_response` | 3.10.3 | Node returns requested KeyPackage |

### Decision

All implementations use spec-authoritative wire names. The guide will be updated in a future documentation pass but the implementation does not wait for that. D-045 is the permanent record of the resolution.

---

## D-056b — Application Deployment Model: one binary per role, multi-mode dispatch

**Date:** 2026-05-16
**Layer:** Layer 6 (UI / deployment / packaging)
**Spec reference:** Ch2 — Application Deployment Model & Lifecycle States (Session 19); Appendix E — Application Lifecycle States (Session 4)

### Context

Earlier Ch2 wording described the deployment model as "one binary, two personalities" — desktop (with UI) versus service (`--service`, headless). That framing conflated two independent questions: (a) does the binary present a UI, and (b) is the invocation long-running or short-lived. The conflation became actively misleading when implementation work surfaced two facts:

- The Client side already has `--batch` (BATCH_FLAG_ph2.md, J-044a) — a short-lived, no-UI invocation that connects to a long-running instance via a named pipe (D-043), dispatches commands, and exits. This is neither "desktop personality" nor "service personality." It is a different category of invocation altogether.
- The current code carries `*-app.exe` build artifacts (`xgen-node-app.exe`, `xgen-client-app.exe`) as separate Tauri outputs alongside the CLI binaries. Two parallel `--batch` implementations exist on the Client side (one in `xgen-client/src/main.rs`, one in `xgen-client/src-tauri/src/batch.rs`). This is transitional scaffolding, not the target product shape — and it has no spec to point at because the previous Ch2 wording did not name what the target shape is.

This decision reframes the model cleanly and locks the target architecture so implementation can converge.

### Decision

**One binary per role.** The final product ships exactly two binaries:

- `xgen-node.exe` — the Node application
- `xgen-client.exe` — the Client application

No separate CLI build. No separate Tauri build. The `*-app.exe` outputs in the current repo are transitional and will be collapsed into the single product binaries.

**Two mode categories dispatched by flag.** Each binary detects flags at startup and dispatches into one of two mode categories:

- **Resident mode** — long-running. Owns the process lifecycle (the states defined in Appendix E). Hosts the protocol. Exposes a named-pipe server (D-043) at `\\.\pipe\xgen-{node|client}-{label}`. Two variants:
  - Desktop variant: default launch, with UI (systray + admin window for Node; Console for Client).
  - Headless variant: `--service` flag (primarily a Node concern, but available to either binary). No UI.
- **Control mode** — short-lived. Any flag that means "do something against the running instance, then exit." Process has no UI (no Tauri, no window, no systray). Optionally opens the named pipe of a resident instance, dispatches, reads the result, exits. Current examples: `--batch <file.xgb>`, `--init [--passphrase <p>]`. Future examples: `--stop`, `--reload-config`, `--export-log`, anything else that fits the shape.

"Control mode" is the canonical term. "Injection mode" is an acceptable informal synonym in conversation and journal entries.

**Shared command layer.** All input channels — Tauri UI button clicks, Console typed commands, `--batch` piped commands, future control-mode flags — dispatch through the same command layer defined in the library crate (`xgen-node/src/lib.rs`, `xgen-client/src/lib.rs`). One clap parser, one set of command implementations. No duplicate command code between CLI and UI paths. Adding a new command means defining it once; it becomes available to every input channel simultaneously.

**`--instance <label>` recommended on every resident launch.** The named pipe is derived deterministically from the instance label (D-043). Launching a resident instance without `--instance` produces the unnamed pipe (`\\.\pipe\xgen-{node|client}`), which works but is not the recommended deployment posture for anything beyond casual single-machine use. The recommendation: even when running a single Node or single Client on a machine, launch it with an explicit `--instance` label so control-mode invocations have a named target ready. Cost is zero; benefit is that any future diagnostic, scripted operation, or tooling not yet conceived has a stable address to target.

**Lifecycle scope clarified.** The lifecycle states defined in Appendix E (Node: `INITIALISING`, `READY`, `DEGRADED_*`, `MAINTENANCE`, `CLOSING`; Client: 11 states including `SETUP`, `CONNECTING`, `AUTHENTICATING`, etc.) describe **resident-mode** processes only. Control-mode invocations are outside the lifecycle: they open the pipe, dispatch, and exit. The resident instance does not change state when a control-mode command arrives — it simply processes one more command through its existing command layer.

### Implementation implications

These follow from the decision. They are not part of D-056b itself; they are tasks pulling current code into compliance:

1. **Node-side `--batch` implementation.** J-037 deferred this when the Client-side `--batch` was written. The spec target is now explicit. Port BATCH_FLAG_ph2.md's pattern to the Node side using the same library-first rule, same pipe-naming convention, same clap dispatch shape, with the Node's own command set.
2. **Collapse `*-app.exe` into the single product binaries.** Merge `xgen-{node,client}/src/main.rs` with `xgen-{node,client}/src-tauri/src/main.rs` into one entry point per role. Extract shared resident-mode logic (`run_node_server` / `start_client_session`) into the library crate so the single binary can dispatch any mode without code duplication. Eliminate the two parallel `--batch` implementations on the Client side.
3. **Pipe server in resident mode for both binaries.** Currently only the Client's Tauri variant hosts a pipe server. The Node Tauri shell's `--service` mode emits lifecycle events but binds no WebSocket server and no pipe server. Bring it into compliance with the new model: every resident-mode invocation hosts the pipe server.

These implementation tasks are tracked separately. D-056b locks the architectural target they converge on.

### Relationship to other decisions

| Decision | Relationship |
|---|---|
| D-043 | Pipe naming convention `\\.\pipe\xgen-{node\|client}-{label}`. D-056b generalises it: every resident instance, every control-mode invocation. |
| D-037b | Node deployment personality (now resident mode variants). Architectural horizon — protocol-native Node admin via privileged client Identity — survives unchanged. |
| D-039b | Shutdown model. `×` minimises to tray; `CLOSING` only entered via explicit exit action or a future `--stop` control-mode flag. Consistent with D-056b. |
| J-037 | Node `--batch` design discussion. Now has an explicit spec target to point at. |
| J-044a | Client `--batch` implementation (BATCH_FLAG_ph2.md). The principal worked example of the control-mode pattern D-056b generalises.

### Spec status

- Ch2 §Application Deployment Model — rewritten in Session 19 (2026-05-16) to match this decision.
- Appendix E — Design Principles section opened with a paragraph clarifying that lifecycle states describe resident mode only. Session 4 entry added.

---

## D-062 — Tauri inclusion model: compiled into product binary, runtime dispatch chooses UI

**Date:** 2026-05-16
**Layer:** Layer 6 (deployment / packaging)
**Spec reference:** D-056b (one binary per role, multi-mode dispatch). M1 task file `tasks/BINARY_CONSOLIDATION_M1.md` Phase 2.

### Context

D-056b named the deployment target — one binary per role, dispatched at startup. The implementation question that follows: when both binaries link in Tauri (for the desktop variant of resident mode), is the Tauri dependency a build-time variant (Cargo feature flag `tauri`) or always compiled in with runtime dispatch?

Two options surveyed:

- **(a) Feature flag.** `xgen-node`/`xgen-client` build with `--features tauri` for the desktop product; headless deployments build without. Smaller server-shape binary, faster server-shape build, CI can build two variants and classify breakages by side.
- **(b) Always compiled in.** Both binaries always contain Tauri. Runtime dispatch (presence of `--service`, presence of a subcommand, presence of a read-only control flag) decides whether to initialise the UI. Larger binary, longer build, but no packaging variant to mismanage.

### Decision

**Option (b) — always compiled in, runtime-dispatched.** The merged binaries link Tauri unconditionally. The CLI dispatcher in `main.rs` decides at startup whether to call `desktop::run()` (Tauri initialisation) or `app::run_node()` (headless WS server) or a one-shot control handler. The Tauri runtime is paid for in binary size and build time regardless of how the binary will be invoked.

### Rationale

**Fewer error classes.** Under option (a), a packager forgetting `--features tauri` ships a GUI-less binary to a desktop user. That is a real packaging-mistake category, and it can survive smoke-testing if the packager only exercises CLI commands. Option (b) removes this class entirely: every binary can always do everything.

**Honest trade-off.** Acknowledged costs of (b):
- Server-shape deployment carries the Tauri/WebView2 runtime dependency even though it never invokes the UI. Disk footprint grows; for embedded or container deployments this matters.
- `cargo build` time grows with the UI rather than just the protocol. CI cycle time increases.
- CI runs one build instead of two, so a break cannot be independently classified "UI-side broke" vs "protocol-side broke" by build behaviour alone — that classification has to come from the diff.

All accepted. The simpler operational story (one artefact per role, always works in any mode) is worth the build-time and binary-size cost. Revisiting in the other direction is straightforward if those costs become acute — `#[cfg(feature = "tauri")]` gates can be added retrofitting (b) into (a) without rewriting code.

### Implementation note

This decision is the literal Rust expression of D-056b's "one binary per role, multi-mode dispatch." Without D-062, D-056b has no Rust-level commitment; with D-062, the merge in M1 Phase 2 has a clean target shape:
- `xgen-node/Cargo.toml` and `xgen-client/Cargo.toml` carry `tauri`, `tauri-plugin-process`, and `tauri-build` (build-dependency) unconditionally.
- Each product crate's root holds `tauri.conf.json` + `build.rs` + `capabilities/` + `icons/` (formerly under `src-tauri/`).
- The Tauri shell code moved to library modules (`xgen-node-lib::desktop`, `xgen-client-lib::desktop`) so the binary's `main.rs` stays thin.

The `*-app.exe` build targets are removed from the workspace. Build artefacts after M1 Phase 2a: exactly `xgen-node.exe` and `xgen-client.exe`.

### Relationship to other decisions

| Decision | Relationship |
|---|---|
| D-056b | Architectural target. D-062 is the implementation-level commitment of how Tauri lives inside that target. |
| D-063 | Companion decision: where the resident-mode logic lives (library crate, not `main.rs`). Required by D-062's runtime-dispatch model — the dispatch target must be a library function any entry point can call. |

---

## D-063 — Resident-mode logic lives in the library crate

**Date:** 2026-05-16
**Layer:** Layer 6 (architecture)
**Spec reference:** D-056b (shared command layer requirement). M1 task file `tasks/BINARY_CONSOLIDATION_M1.md` Phase 1.

### Context

D-056b requires a shared command layer that every input channel (Tauri UI button clicks, Console typed commands, `--batch` piped commands, control-mode flags) dispatches through. For that requirement to be satisfied, the command layer has to live somewhere that all entry points can call — which means it cannot live in `main.rs` (only one `main.rs` exists per binary; library code, Tauri callbacks, and the binary's CLI dispatcher cannot all call into it from there).

The existing layout violated this. `run_node` (the Node's resident-mode entry point), the entire CLI subcommand set (`cmd_init`, `cmd_status`, `cmd_connections`, etc.), and the Client's batch-line dispatcher all lived in `main.rs` to varying degrees. The Tauri shell duplicated functionality (lifecycle scaffold) rather than calling shared code.

### Decision

**Resident-mode logic and the full command surface move to the library crate.** After this decision lands:

- `xgen-node-lib` (`xgen-node/src/lib.rs`) exposes `app::run_node`, `app::cmd_*` for every subcommand, `app::RunNodeOpts`, and `desktop::run` (the Tauri shell entry point, calling `app::run_node` internally).
- `xgen-client-lib` (`xgen-client/src/lib.rs`) exposes `app::cmd_*` for every subcommand, `app::run_batch_file`, the full `Cli` / `ClientCommand` clap structs, `batch::start_pipe_server`, `batch::dispatch_line`, `batch::pipe_name`, `batch::run_batch_client`, and `desktop::run`.
- Each binary's `main.rs` is a thin dispatcher: parse flags, decide mode, call the corresponding library function. No business logic in `main.rs`. The Node main.rs ends up around 270 lines (most of that clap definitions); the Client main.rs around 200 lines (most of that clap dispatch).
- The Client's `Cli` / `ClientCommand` clap structs live in `xgen-client-lib::app` rather than `main.rs` because the batch-file executor (`run_batch_file`) re-parses sub-CLI invocations per `.xgb` line, and that executor lives in the library.

### Rationale

This is the library-first architecture rule from `CLAUDE.md`, applied consistently across the merged binary structure. The rule already existed for Layer 1–10 code (everything below `transport`); D-063 extends it to the dispatch layer that sits between input channels and command implementations.

Without D-063, D-056b's "shared command layer" is impossible to express in code: the desktop shell would either duplicate command implementations (drift inevitable, J-067's two-`get_dag_tips` problem multiplied) or call back into `main.rs` somehow (Rust doesn't permit that cleanly). The library extraction is the unblock.

### Implementation note

The implementation pass lives in M1 Phase 1. After it ships:
- `grep "pub async fn get_dag_tips"` returns exactly one match in `xgen-client/src/batch.rs:239`. The duplicate from `xgen-client/src/main.rs` is gone. Closes F-003 / F-004 from J-067 permanently — that was the loudest visible symptom of the library-extraction gap.
- All `cmd_*` functions live in `app.rs` (per crate). `main.rs` calls them via `app::cmd_foo(...)`.
- `desktop::run()` calls `app::run_node()` with `RunNodeOpts { init_logging: false, ... }` so logging init is owned by the desktop module (since Tauri is already up by the time `run_node` runs). The bool flag is the seam.

### Relationship to other decisions

| Decision | Relationship |
|---|---|
| D-056b | Architectural target. D-063 makes the shared command layer physically possible. |
| D-062 | Sibling decision: where Tauri lives (compiled in always). D-063 says where the protocol logic lives (library crate). Together they define the merged-binary architecture. |
| J-067 F-003 / F-004 | The duplicate `get_dag_tips` bug was the visible symptom. D-063 is the structural fix that prevents the bug class. |

---

## D-064 — M3 AI operator role: distinct role, fall-upward resolution, AI-owned-Space prohibition

**Date**: 2026-05-17  
**Layer**: protocol (spec 3.6.10.6) + Space state derivation (xgen-core/src/space/state.rs) + event acceptance pipeline (xgen-core/src/message/exchange.rs) + Client CLI surface (xgen-client)  
**Spec reference**: 3.6.10.6 (rewritten in M3); 3.6.10.10 (3041 wire-name widened)

### Decision

Operator is a distinct role inside a Space, scoped per-(AI, Space) — not Space-wide privileges (those remain admin's and owner's). The system always knows who the operator is even with no explicit delegation Event ever signed: the resolution function falls upward through stored state — stored delegation → AI's inviter → Space owner — and transparently skips entries pointing at Identities who are no longer members. AI Identities are prohibited from being Space owners (any `state.space_create` / `state.dm_space_create` from an `is_ai = true` sender is rejected with 3041, superseding the D-059 `dm_initiate` capability gate for those events). The protocol records who the operator is and surfaces resolution; it does **not** grant the operator any protocol-level event-signing privileges in this version — those layer on top in future milestones.

### Locked principles

**Operator is its own role.** Member < Operator (scoped to AI-X) < Admin < Owner in privilege scope, not in role hierarchy. The same human can be operator of one AI in one Space, a plain member in another, and an admin in a third.

**Delegation flow.** Admin or owner picks a current Space member, signs `state.ai_operator_delegate(ai_identity_id, new_operator_identity_id)`. The previous operator's consent is not required. Operator never signs in their operator capacity in this version — operator-signed events arrive in future milestones layered on the resolution function.

**Fall-upward resolution.** `resolve_operator(space, ai_id)`:
1. If a stored delegation exists for `ai_id` AND the named delegate is a current member: return the delegate.
2. Else if the AI's recorded inviter (sender of the original `membership.invite`) is a current member: return the inviter.
3. Else: return the Space owner (always a member of a live Space).

No orphan state is reachable. The stored delegation map is honoured only when its target is still a member — left/kicked delegates auto-skip without requiring an explicit revoke. Revoke explicitly clears the stored entry, collapsing resolution to step 2 or 3.

**Inviter-as-operator is computed, not stored.** No separate "initial operator" record. When an AI joins with no delegation yet, resolution returns the inviter — identical to how the operator is resolved at any other time.

**AI-owned Space rejected.** Pragmatic deferral, not architectural impossibility — revisit when a real use case appears.

**No protocol-enforced operator privileges in v1.** The operator role is a declaration of responsibility recorded in the DAG. Practical privileges (DM command surface, audit access, AI silencing, capability override) emerge from real usage and future capabilities, layered on top — they will be "is this signer the current *resolved* operator?" checks, not "did this signer sign a delegate event?" checks.

### Implementation surface

| Surface | Shape |
|---|---|
| `SpaceMember.invited_by: Option<String>` | `None` for owner and pre-M3 replayed members; `Some(sender)` for members admitted via `membership.invite`. Captured in `apply_invite` (carried through `pending_invites`) and consumed by `resolve_operator` step 2. |
| `SpaceState.ai_operator_delegations: HashMap<String, String>` | Key = `ai_identity_id`; value = delegated operator's identity_id. Absence means "no explicit delegation; resolution falls through." |
| `SpaceState::resolve_operator(&self, ai_id) -> Option<String>` | Three-case fall-upward algorithm. `None` only for non-member `ai_id` or structurally-impossible no-owner state. |
| `state.ai_operator_delegate` / `state.ai_operator_revoke` | New `apply_event` arms (defence-in-depth signer check); validation in `exchange.rs::check_ai_operator_targets` (signer + target membership + `is_ai` flag). |
| `check_ai_capability` extension | Rejects `state.space_create` / `state.dm_space_create` from any AI sender with 3041, ahead of the D-059 `dm_initiate` 3042 path. The 3042 path remains in code as a framework for future re-enablement. |
| `can_delegate_ai_operator(role) -> bool` | New permission helper; `*role >= Admin`. |
| Wire-name 3041 widened | Was `ai_flag_immutable`; now `ai_role_violation`. Umbrella covers `is_ai` immutability **and** the M3 role validations. Wire **code** unchanged; wire **name** broadens. Spec table updated in §3.6.10.10. |

### CLI surface (M3 minimum, testability only)

- `xgen-client init --ai [--cap key=value]` — writes `[ai]` section to `xgen-client_config.toml`. Default capability values are `dm_initiate=false`, `spontaneous_post=false`; `--cap` flags override. `init --ai` re-run upserts the section without clobbering other config fields.
- `xgen-client register` — reads `[ai]`, builds `is_ai=true` + capabilities for AI registration via the existing `build_register_with_ai`.
- `xgen-client ai delegate --space <id> --ai <id> --to <member-id>` — signs and sends `state.ai_operator_delegate`.
- `xgen-client ai revoke --space <id> --ai <id>` — signs and sends `state.ai_operator_revoke`.
- `xgen-client ai status --space <id> --ai <id>` — connects via WS, replays the Space's DAG locally, runs `resolve_operator`, prints the result with provenance (stored delegation / inviter fallback / owner fallback). Returns the **queried Node's converged view**; call against each Node to verify federation propagation.

`whoami` and `status` remain offline-local-introspection (intentionally — operator-resolution is a network-resident dynamic property and deserves its own honest verb).

### Out of scope (deferred to future milestones)

- AI Client *binary* — a long-running daemon that registers as an AI, joins Spaces, receives events via `run_ws_loop`, responds under pacing rules. This decision lands the protocol primitives; the consuming binary is a separate milestone.
- Protocol-enforced operator privileges (DM command surface, audit access, AI silencing, capability override). Per the locked principles above, these layer on top when real features need them.
- `spontaneous_post` Node-side enforcement — Phase 2 leaves this unenforced (3.6.10.4); no change in M3.
- Operator self-transfer (operator signs over to next operator without admin/owner involvement). Not in M3's signer model.
- Cross-Space operator inheritance. Operator is strictly per-(AI, Space).
- Pacing / temperature plugin math (still plugin-owned per D-060/D-061).

### Why this shape rather than alternatives

The hard architectural question was whether the operator's existence and identity should be stored explicitly (initial operator written into a `SpaceMember.operator_of` field on AI admission) or resolved dynamically. The dynamic-resolution shape wins because:

1. **No special-case bootstrap.** "Inviter-is-operator when no delegate exists" is identical to how the operator is resolved at any other time — single algorithm, no separate code path for "first operator".
2. **Self-healing on member churn.** When a delegate leaves or is kicked, the system silently reverts to the inviter (or owner) without anyone having to sign a revoke. Compare to a stored-only model where every delegate departure requires explicit cleanup or leaves the Space in a broken state.
3. **No orphan state.** The fall-upward chain ends at the owner, who is always present in a live Space. There is no reachable state where "the operator is undefined".
4. **Clear delegation semantics.** Delegate writes a new entry; revoke clears the entry. Both are local point operations — no need to track "the previous operator" or "the operator-of-operator" or any chain.

The alternatives considered and rejected:

- **AI-as-owner permitted.** Rejected pragmatically — no clear use case in M3 and several open questions about what "an AI signs a space.update" means for trust attribution. Not architecturally impossible; revisitable when a real driver appears.
- **Operator-signed delegation (transfer-of-trust by the previous operator).** Rejected because it complicates the signer model and adds nothing the admin/owner-signed flow doesn't already cover. Admin/owner is already the locus of authority over the Space; operator authority over the AI is a subset.
- **Finer-grained error codes (3043 / 3044 for the new validation failures).** Rejected — wire-code granularity adds reading load without adding semantic value when the role family is already covered by 3041. The `ai_role_violation` umbrella catches structural role rules (3041) and capability flags (3042 — separate domain).
- **Cache `whoami` / `status` resolved operator into `xgen-client_state.json`.** Rejected — guaranteed-stale on every cross-Node action; pretending offline-cached state reflects federation truth is worse than a clear "this command is a network query" verb (`ai status`).

### Why now

M3 ships the protocol primitives that the AI Client binary milestone will consume. Landing the operator role, validation, and resolution function before the binary means the binary lands as a thin consumer of well-tested primitives rather than discovering the role-model gaps mid-flight.

### Spec reference

- 3.6.10.6 rewritten — operator role definition, signer rules, fall-upward algorithm, AI-owned-Space prohibition, "no protocol-enforced operator privileges in v1".
- 3.6.10.10 — 3041 wire name widened from `ai_flag_immutable` to `ai_role_violation`; same code.

### Code reference

| File | Surface |
|---|---|
| `xgen-core/src/space/state.rs` | `SpaceMember.invited_by`, `PendingInvite`, `SpaceState.ai_operator_delegations`, `resolve_operator`, `apply_ai_operator_delegate`, `apply_ai_operator_revoke`, `build_state_ai_operator_{delegate,revoke}_event` |
| `xgen-core/src/space/membership.rs` | `can_delegate_ai_operator` |
| `xgen-core/src/message/exchange.rs` | `ExchangeError::AiRoleViolation` → wire `(3041, "ai_role_violation")`; `check_ai_capability` extended; `check_ai_operator_targets` added; `check_permission` arms for delegate/revoke |
| `xgen-core/src/identity/registration.rs` | `AiFlagImmutable.to_registration_code()` returns `(3041, "ai_role_violation")` — wire-name widening |
| `xgen-client/src/app.rs` | `AiSection` in `ClientConfig`; `--ai` / `--cap` on `InitArgs`; `Ai(AiArgs)` subcommand group; `cmd_ai_delegate` / `cmd_ai_revoke` / `cmd_ai_status` |
| `xgen-client/src/main.rs`, `xgen-client/src/batch.rs` | Dispatch for the new subcommand group |

### Relationship to other decisions

| Decision | Relationship |
|---|---|
| D-059 | M3 builds on D-059's `is_ai` / `ai_capabilities` wire shape. The `dm_initiate` capability mechanism remains in code; the structural 3041 rule in M3 fires before it for `state.dm_space_create` from any AI, making D-059's 3042 path unreachable for that event in M3 but preserving the capability framework for future re-enablement. |
| D-060, D-061 | Adjacent AI-related protocol surfaces (pacing, temperature). Not touched by M3 directly but consumed by the same population of AI Identities. |

---

## D-065 — M4 AI Client: resident mode of xgen-client + plugin model + "honest behaviour over polite behaviour"

**Date**: 2026-05-17  
**Layer**: Client (xgen-client/src/ai_service.rs, ai_behavior.rs) + Configuration (xgen-client_config.toml schema) + Documentation (Ch6 §6.15)  
**Spec reference**: Ch6 §6.15 (new section); Ch3 §3.6.10 (cross-link)

### Decision

The AI Client is **a mode of `xgen-client`**, not a separate binary. `xgen-client --ai-mode --service` dispatches a long-running resident with a plugin-based behaviour model: the runtime owns connection, replay, pacing, mute, and pipe-server I/O; the `AiBehavior` trait owns the decision "should I reply, and what should I say." M4 ships exactly one plugin (`EchoPlugin`, config key `"echo"`) as the reference implementation — its job is to prove the loop end-to-end, not to be useful. Real LLM hookups and sophisticated dialog policies layer on the trait in future milestones.

This decision also names a recurring XGen design principle that has been implicit in earlier protocol choices: **honest behaviour over polite behaviour.** When a system can choose between behaviour that misrepresents its current state (polite — "I'll deliver this thought eventually" / queueing) and behaviour that honestly reflects its current state (honest — "I can't say this right now and the moment passed" / dropping), XGen picks honest.

### Locked architecture

**Binary identity.** Two binaries total: `xgen-node`, `xgen-client`. Three modes for `xgen-client`:

| Invocation | Role |
|---|---|
| `xgen-client <subcommand>` | One-shot human Client |
| `xgen-client --service` | Long-running human-Client resident |
| `xgen-client --ai-mode --service` | Long-running AI-Client resident |

The `--ai-mode` flag is meaningful only with `--service` (clap enforces). Existing pipe naming convention `\\.\pipe\xgen-client[-<instance>]` is unchanged; AI residents bind to the same pipe space and distinguish themselves via the `mode=` field in `__HEALTH__`.

**Why a mode and not a separate binary.** The Node's headless mode is `--service`, not a separate `xgen-node-service` binary. By symmetry, an AI Client is a client — same Identity registration, same Space membership, same event emission, same `[ai]` config staging — just with behaviour coming from a plugin instead of a keyboard. Consistency with the resident/control pattern wins. M1 collapsed binaries that shared identical code; xgen-client and the AI Client share the same library and dispatch through one entry point per mode. Three binaries (the rejected alternative) would have put M4 in conflict with the D-056b consolidation direction it should be following.

**Plugin model.** `AiBehavior` trait in `xgen-client-lib::ai_behavior`:

```rust
pub trait AiBehavior: Send {
    fn on_event(&mut self, ctx: &EventContext) -> Option<String>;
    fn name(&self) -> &'static str;
}
```

The plugin receives one inbound `Event` at a time and returns `Some(text)` to reply (as `message.text`) or `None` for silence. Plugins MUST be fast and non-blocking — long-running work is future-plugin design territory. The runtime handles pacing, mute, prev_events chaining, and WebSocket I/O.

**Reference plugin: `EchoPlugin`** (config key `"echo"`). Replies to mentions in `message.text` with the deterministic line `[echo-plugin] received mention from <last-12-chars-of-sender-id>`. Reply text is fixed — not configurable in M4. Rationale: smoke tests need to grep for the reply; nobody should mistake the artefact for a real reply during early demos.

**Mention detection: two-rail OR**, both case-sensitive:

1. **Rail A (always-on):** substring match for the AI's full `identity_id` URI in `content.text`.
2. **Rail B (optional):** substring match for a `mention_token` (e.g. `"@bob"`) read from `[ai.behavior]`. Default `None`.

Rails are **OR'd, not sequenced** — either match counts. The implementation MUST NOT interpret "always + optionally" as "fall through to optional if always-rail misses."

**Lifecycle.** Long-running daemon under `xgen-client --ai-mode --service`. Reuses the M2 pipe-server pattern for control commands (`__PING__` / `__HEALTH__` / `__STOP__` / `__RELOAD_CONFIG__`). `__HEALTH__` reply for an AI-mode resident extended to `HEALTHY pid=<pid> mode=ai operator_known=<N>/<M>` (where N = Spaces with resolvable operator, M = Spaces the AI is a member of). Coarse signal — the structured per-Space operator map stays on `xgen-client status`.

**Configuration shape.** Single config file `xgen-client_config.toml`. M4 adds two pieces to the existing `[ai]` section from M3:

```toml
[ai]
is_ai = true
plugin = "echo"            # which plugin

[ai.capabilities]
dm_initiate = false
spontaneous_post = false

[ai.behavior]              # plugin's own config; each plugin owns its keys
mention_token = "@bob"
```

The split between `plugin = "..."` (in `[ai]`) and `[ai.behavior]` is deliberate: "which plugin" is a single-line toggle; "how that plugin is tuned" lives in its own namespace. Open-enum on plugin name — unknown values pass config parsing but the runtime loader rejects them at startup with a clear error.

**Pacing — drop, don't queue.** The AI runtime maintains per-Space `last_send_at_ms`. Before emitting a reply, it checks `now - last_send_at_ms >= ai_pacing_ms`. If not, the reply is **dropped** (not queued). Drops are logged at WARN with the literal phrase `dropping reply — pacing cap not yet elapsed (honest behaviour over polite behaviour)` so the principle is greppable in production logs. The same path enforces mute (`active_mutes` in SpaceState).

**Join behaviour: manual, not auto.** The AI Client does NOT auto-join Spaces on startup. Joins are operator-driven via `xgen-client --instance <ai-label> join --space <id>`. Auto-join would make an AI Identity's first observable behaviour in a Space config-driven rather than chosen, muddying the trust model. Manual join keeps presence as an explicit, auditable event in the DAG.

**Operator control plane and temperature surfacing: out of scope for M4.** The protocol-level operator-signed event surface (DM commands, audit access, etc.) does not yet exist — designing M4 around it would load weight on something unbuilt. Temperature is conversational-dynamics design that needs its own conversation. Both layer on the M4 runtime in future milestones.

### The named recurring principle: honest behaviour over polite behaviour

When a system can choose between behaviour that misrepresents its current state and behaviour that honestly reflects it, XGen picks honest. Other places this principle is already operating, named here so future design conversations can invoke it explicitly:

- **Fall-upward operator resolution (D-064).** Returns the *currently-resolvable* operator (skipping stored entries that no longer point at members) rather than serving a stale stored value as if it were live.
- **Node event-acceptance pipeline.** Rejects events that fail validation rather than queueing them for retry; the rejection is the answer.
- **Mute semantics (Ch3 §3.7.8).** A mute is a wall, not a delay. The muted member's events are dropped, not queued for delivery after the cooldown.
- **`cmd_create_space` ack handling** (carry-over from M3, noted in J-075). Currently the Client says "Space created" optimistically; the M4 work surfaced this as a UX bug because optimistic reporting misrepresents the Node's actual decision. Future fix will adopt the honest "wait for ack, then report" pattern.
- **M4 AI Client pacing.** Drops replies that the cap rejects, rather than queueing them. The conversation has moved on; a queued reply now misrepresents the AI's current state.

The principle is not a prescription — sometimes politeness is correct (a Client retrying a transient network error is appropriate; pretending the send already succeeded is not). The naming exists so design conversations can articulate the trade-off cleanly: "this is polite-but-misleading; is that what we want?" and reach for "no, drop / fail / surface the truth" as the default.

### Implementation surface

| File | Shape |
|---|---|
| `xgen-client/src/ai_behavior.rs` | `AiBehavior` trait, `EventContext` struct, `EchoPlugin` impl with case-sensitive two-rail mention detection. |
| `xgen-client/src/ai_service.rs` | `pub fn run()` entry, `run_ai_loop` async fn, `AiPacingTracker` (drop-on-throttle, separate from PacingManager's queue-on-throttle), plugin loader (`load_plugin("echo") -> Box<dyn AiBehavior>`). |
| `xgen-client/src/batch.rs` | New `ResidentHealthState` struct (mode label + optional operator-known count). New `start_pipe_server_with_health` takes shared `Arc<Mutex<ResidentHealthState>>`; existing `start_pipe_server` becomes a default-state wrapper. `__HEALTH__` handler reads from the shared state. |
| `xgen-client/src/main.rs` | Dispatch adds AI-mode branch: `if cli.service { if cli.ai_mode { ai_service::run() } else { service::run() } }`. |
| `xgen-client/src/app.rs` | `AiSection` extended with `plugin: Option<String>` and `behavior: Option<AiBehaviorSection>`. New `AiBehaviorSection` struct (config sub-table for plugin-specific keys; M4's only key is `mention_token`). `cmd_init --ai` defaults `plugin = "echo"`. |
| `xgen-client/src/lib.rs` | `pub mod ai_behavior;` + `pub mod ai_service;`. |
| `docs/xgen_ch6_client_design.md` | New §6.15 "AI Client (resident mode)" — 10 subsections covering mode selection, config, trait, reference plugin, mention detection, runtime loop, pacing/mute, lifecycle/control, manual join, out-of-scope/forward-references. |
| `docs/xgen_ch3_specification.md` | §3.6.10 cross-reference list extended to include D-064 (M3 operator role), D-065 (M4 reference implementation), and Ch6 §6.15 (forward link to client-side surface). |

### Out of scope (deferred)

- **Real LLM hookups.** Future plugins as additional `AiBehavior` impls.
- **Multiple plugins / config-time plugin selection logic.** M4 ships one plugin; the loader matches the configured name to the only available impl. Phase 2+ adds the loader.
- **Operator command surface (DM commands, audit access, AI silencing through operator authority).** Separate protocol-level design conversation.
- **Temperature surfacing / room-temperature reaction by the AI.** Conversational-dynamics design; defer.
- **Auto-join of Spaces by invite.** Locked manual; testing convenience preserved by smoke-script CLI helper.
- **Cross-Space coordination, multi-device AI Client, Tauri / UI surface.** Future milestones.

### Why this shape rather than alternatives

The hard architectural question was *binary identity* — should the AI Client be a separate `xgen-ai` binary or a mode of `xgen-client`? The v0.1 draft of this decision proposed a separate binary; the v0.1→v0.2 review pass amended it to "mode of xgen-client" with reasoning that the M2 precedent (Node's `--service` mode rather than `xgen-node-service` separate binary) and the D-056b consolidation direction (one binary per role) both point the same way. AI Client is a client; the runtime loop differs from the human Client's loop but everything around it (config loading, connection, pipe server, lifecycle) is identical scaffolding. A separate binary would have duplicated that scaffolding for no clear gain.

The plugin trait is locked now rather than deferred. The trait surface is small enough that getting it wrong now is cheap; getting it wrong after a real LLM plugin exists is expensive — the future plugin would either accept the inherited shape or force a breaking-change rework of every consumer. Locking the shape during M4, before any real plugins exist, costs nothing extra and stabilises the interface.

Drop-late-replies is locked because queueing produces stale replies — by the time the cooldown expires, the conversation has moved on. The locked behaviour also is the simpler implementation, but the simplicity follows from the correctness, not the other way around: the honest design is also the lighter design here.

Manual join is locked because the trust model loses something when an AI Identity's first observable behaviour in a Space is config-driven rather than chosen. Auto-join would make the AI's presence implicit; manual join keeps it explicit and auditable through the standard `membership.join` event flow.

### Why now

M4 implementation began at v0.3 task-file lock (J-076) after D-056b consolidation was confirmed closed. The Client lifecycle conventions (PID file, pipe server, session header, log rotation) are stable from M1/M2; the protocol primitives the AI Client consumes are stable from M3. M4 is the first milestone that exercises all of them together in a long-running process and surfaces "what does this look like end-to-end" for the first time. The recurring honest-vs-polite principle was already implicitly operating across earlier decisions; naming it here makes future design conversations more efficient.

### Spec reference

- New section: Ch6 §6.15 "AI Client (resident mode)" — 10 subsections.
- Cross-references added in Ch3 §3.6.10 — pointing forward to §6.15 and back-referencing D-064, D-065.

### Code reference

| Component | File / surface |
|---|---|
| `AiBehavior` trait + `EchoPlugin` | `xgen-client/src/ai_behavior.rs` |
| AI runtime loop + plugin loader + pacing tracker | `xgen-client/src/ai_service.rs` |
| Pipe-server shared health state | `xgen-client/src/batch.rs::ResidentHealthState` + `start_pipe_server_with_health` |
| `--ai-mode` flag + dispatch | `xgen-client/src/app.rs::Cli::ai_mode`; `xgen-client/src/main.rs` mode-selection branch |
| Config schema | `xgen-client/src/app.rs::AiSection` + `AiBehaviorSection` |
| `init --ai` defaults | `xgen-client/src/app.rs::cmd_init` |

### Relationship to other decisions

| Decision | Relationship |
|---|---|
| D-056b | M4 is a mode of xgen-client per the locked "one binary per role" direction. D-056b closed first (J-076); M4 implementation followed. |
| D-059 | M4 consumes D-059's `is_ai` registration shape via the existing M3 `register` flow; no new wire shape needed. |
| D-060 | M4 reuses D-060's `ai_pacing_ms` field via a simpler drop-on-throttle tracker (sibling of `PacingManager` rather than wrapper, because the policies differ — queue vs drop). |
| D-061 | M4 is a passive recipient of temperature meta_atts; does not emit temperature, does not react to thresholds. |
| D-062 | M4 does NOT use Tauri — explicitly headless. |
| D-063 | M4 follows library-first per D-063: trait + runtime loop in `xgen-client-lib`, binary is thin dispatch. |
| D-064 | M4 surfaces M3's `resolve_operator` result on `__HEALTH__` (operator_known count). |

---

## D-066 — Split `--batch` legacy surface from `--aicontrol` AI surface; the latter is reference-implementation, not protocol

**Date**: 2026-05-17  
**Layer**: Reference implementation control plane (xgen-client / xgen-node binaries) — NOT XGen Protocol  
**Spec reference**: none (this decision is explicitly out-of-protocol). Cross-reference: `tasks/BATCH_FLAG_review.md` (Clair's review of `--batch`) and the Chat Claude addendum appended to the same file (2026-05-17).

### Decision

The `xgen-client` binary will expose **two distinct control surfaces** with different audiences and different design constraints:

| Flag | Audience | Shape | Format | Status under this decision |
|---|---|---|---|---|
| `--batch <file.xgb>` | Humans and human-readable automation (CI shell scripts, ops runbooks) | Fire-and-forget script runner. One command per line. | Plain text `.xgb` files; replies are `OK\n` / `ERROR: ...\n`. | **Frozen as-is.** Continues to behave exactly as it does today. |
| `--aicontrol` | AI drivers (Claude Code, future MCP servers, in-Space AI moderators, scripted multi-step agents) | Persistent control session. Long-lived connection, multiple commands, real-time event observation. | Newline-delimited JSON (JSONL) over a sister pipe. | **New surface.** Design and implementation scoped under this decision; details in `tasks/BATCH_FLAG_review.md` Chat Claude addendum. |

Both surfaces dispatch through a **shared command-implementation layer** (`xgen-client-lib::ops::*`) parameterised by execution context (one-shot connection vs persistent session). This extends the D-063 library-first principle one level deeper to eliminate the existing `cmd_*` / `exec_*` drift surface that produced F-003 / F-004 in J-067.

### The protocol-vs-implementation boundary (locked)

**`--aicontrol` is NOT part of the XGen Protocol.** The XGen Protocol is what travels on the wire between XGen participants — between a Client and its home Node, between two federated Nodes, between MLS group members. `--aicontrol` is none of these. It is a local control channel between an AI driver and a specific `xgen-client.exe` instance running on the same machine, carried on a Windows named pipe. It never reaches any XGen wire. A different XGen client implementation in a different language could ship a different control surface (gRPC, REST, MCP server, raw stdin/stdout, or no AI-control surface at all) and remain fully protocol-compliant. A proprietary XGen client built by a third party may take a completely different approach to AI automation — that is their implementation choice, not a protocol question. The XGen Protocol does not constrain how a Node operator or Client vendor builds their local automation surface.

This is structurally identical to how Matrix treats its Client-Server API as a reference convention while only the Federation API is protocol; or how MLS (RFC 9420) defines the cryptographic ratchet but says nothing about how clients UI it.

**Implication for the documentation tree.** When `--aicontrol` lands:

- It does NOT appear in Ch3 (Specification).
- It does NOT appear in Appendix I (Data Structures).
- It DOES appear in Ch4 (Implementation) or a new dedicated Appendix — explicitly marked "reference implementation control surface; not part of the XGen Protocol".
- Appendix F (CLI Reference) lists `--aicontrol` as a non-fundamental Client flag (per the §F.0 axis added in M4 documentation sweep) with a forward link to the dedicated design document.

### Locked principles

**`--batch` is preserved verbatim.** No behavioural change, no format change, no deprecation timeline. The human-readability properties of the current `--batch` were a deliberate design goal at its original spec time, and replacing them with JSONL would have been a regression. Two surfaces is the honest answer; one surface trying to serve both audiences was always a tension.

**`--aicontrol` is a persistent session, not a script runner.** The natural shape is `xgen-client --aicontrol` opens a long-lived control session on a dedicated pipe; the driver writes JSONL commands and reads JSONL replies and events. Scripts can be fed via shell redirection but there is no in-protocol "load a file" notion. The session lives as long as the connection lives.

**The shared `ops::*` layer ships first, independent of `--aicontrol` design.** The duplicate `cmd_*` vs `exec_*` problem is independent of which CLI flag invokes them. The refactor benefits both `--batch` and `--aicontrol` and unblocks both surfaces. Sequencing this first means the multiparty baseline pass exercises the unified handlers, not the drift-prone duplicates.

**The flag name was locked by Joe explicitly.** `--aicontrol` over alternatives (`--control`, `--session`, `--ctl`, `--aibatch`) because it makes the audience visible in the flag name. Future readers immediately see what category of driver this surface serves.

### Out of scope for this decision

- All technical details of the `--aicontrol` protocol (JSONL field shapes, command verbs, event subscription model, named bindings, lifecycle-aware error codes, pipe naming, concurrency model). These are in the Chat Claude addendum to `tasks/BATCH_FLAG_review.md` and are explicitly delegated to Chat Claude + Clair without per-decision approval from Joe — see the addendum preamble.
- The Node-side equivalent. Whether `xgen-node --aicontrol` also lands is a question for the design phase, not this decision.
- The cross-platform story. Windows-first; cross-platform pipe abstractions remain Phase 3+.
- Authentication and authorisation of the `--aicontrol` pipe (security model for multi-user MCP deployments). Flagged as a known deferred concern in the addendum.

### Why this shape rather than alternatives

**Alternative 1: Make `--batch` itself more capable.** Rejected. Adding JSONL reply mode, persistent sessions, and event observation to `--batch` would either break the human-readability contract or require a flag-on-a-flag dance (`--batch --reply-format=jsonl --persistent --subscribe=events`) that is harder to use than two cleanly separate flags. The current `--batch` is already at its design limit; trying to make it serve both audiences would degrade both.

**Alternative 2: Single flag with version negotiation.** Rejected. A `--batch --protocol=v2` style would put the version selection inside the wire data rather than at the CLI surface. CLI flags are the right place for major behavioural mode selection — it is visible in shell history, scriptable, and discoverable via `--help`.

**Alternative 3: External AI-driver process (MCP server) that translates between AI commands and `--batch`.** Rejected for this milestone. A future MCP server consuming `--aicontrol` is exactly the intended deployment shape, but it must consume a surface designed for AI drivers — layering it on top of the human-readable `--batch` would push every issue identified in Clair's review (per-command WS churn, log-scraping for return values, no real-time observation) into the MCP server as workarounds. The right architectural primitive is `--aicontrol`; MCP servers and other AI integrations sit above it.

**Alternative 4: gRPC or REST on a localhost port.** Rejected for the same reason named pipes were chosen for `--batch` originally: Windows-first, no port-allocation concerns, no firewall pop-ups, no TLS-on-localhost dance, no second listener to secure. Named pipes are the right primitive on Windows and remain so.

### Why now

The M4 documentation sweep surfaced that the CLI reference (Appendix F) and the canonical state of the system finally agree on what exists today. The next major piece of work (multiparty test suite redesign — paused since M1) cannot proceed honestly under the present `--batch` for the reasons in Clair's review: real-time fan-out is unmeasurable, latency metrics are uncapturable, and two-pass log-scraping ID substitution is structurally fragile. The multiparty A/B metrics protocol Clair specified requires a control surface that captures the metrics; the present `--batch` cannot. Therefore `--aicontrol` is a prerequisite for credible multiparty work, not an optional improvement.

Further: `--aicontrol` is the foundation for the future Claude-driven MCP server and any in-Space AI moderator agent. Designing it now — once — saves designing it later under feature pressure from those consumers.

### Relationship to other decisions

| Decision | Relationship |
|---|---|
| D-028 | The canonical-source rule ("Rust doc comments MUST match Appendix F") applies to `--aicontrol` once it lands. The detailed protocol document is the canonical source; Appendix F summarises and links to it. |
| D-035 | `--aicontrol` data lives under the same convention-derived directory layout as everything else — no new configurable paths. |
| D-043 | The pipe naming convention is extended (sister pipe `\\.\pipe\xgen-client[-<label>].aicontrol` alongside the existing legacy pipe). |
| D-056b | `--aicontrol` is a new dispatch mode on the existing `xgen-client` binary, consistent with the locked one-binary-per-role + multi-mode dispatch model. Not a new binary. |
| D-063 | The shared `ops::*` layer extends library-first one level deeper than D-063 originally specified — D-063 moved dispatch out of `main.rs`, this decision moves command implementations into a single shared layer below it. |
| D-065 | `--aicontrol` is the operator command surface that D-065 said would "layer on top in future milestones." This decision schedules it. |
| Clair's `BATCH_FLAG_review.md` | Diagnostic; this decision is the architectural response. Detailed technical decisions are appended to that file as the Chat Claude addendum. |

### Canonical home (added 2026-05-17)

The technical specification for `--aicontrol` lives in **`docs/xgen_aicontrol_implementation.md`** as of 2026-05-17. That document supersedes the Chat Claude addendum inside `tasks/BATCH_FLAG_review.md` (which remains in place as a historical predecessor) and extends the design to cover both binaries (`xgen-client` and `xgen-node`) rather than Client only. D-069 names the canonical-document discipline that this move implements. Future edits to the `--aicontrol` design land in the canonical document, not in DECISIONS.md notes or in `tasks/` addenda.

---

## D-067 — Single source of truth for xgen-client command implementations (`ops::*`); M7 prerequisite met

**Date**: 2026-05-17
**Layer**: xgen-client crate (structural)
**Spec reference**: `tasks/M5_OPS_REFACTOR.md`; D-066 (the `--aicontrol` split that M5 unblocks); D-063 (library-first principle that M5 extends one level deeper); J-067 (F-003 / F-004 background).

### Decision

Every xgen-client command implementation lives in exactly one place: `xgen-client-lib::ops::<verb>`. Every dispatcher — the CLI arm in `main.rs`, the CLI batch driver `app::run_batch_file`, the named-pipe dispatcher `batch::dispatch_line`, and any future Tauri-command / `--aicontrol` arm — calls into the same `ops::<verb>` function. Each dispatcher owns its own output format (CLI shim formats for stdout, pipe arm formats `OK\n` / `ERROR: …\n` per the D-066 freeze, M7's `--aicontrol` arm will format as JSONL); the data extraction lives in exactly one place per verb.

`SessionState` (per-invocation session bundle) and `ClientIdentity` (loaded keypair + cached `identity_id`) are the helpers that make `ops::*` parameterisable across execution contexts. `SessionState::ensure_identity` and `SessionState::ensure_connected` are idempotent so both M5 one-shot dispatchers and M7 persistent-session dispatchers reuse the same code paths.

The M5 type signatures include M7 extension fields (`SessionState.bindings`, `SessionState.spaces`) present-but-empty so the type signature is M7-stable; no shape changes will be needed between M5 and M7.

### Why this matters

**Drift surface eliminated.** Before M5 (and even after J-068's partial dedup), command implementations could diverge across dispatchers — the F-003 / F-004 pair in J-067 was a concrete instance where one `get_dag_tips` copy got a Space-filter fix and the other silently kept the bug. After M5, there is exactly one user-facing implementation per verb; a second copy cannot be introduced without being noticed.

**M7 (`--aicontrol` v1) prerequisite met.** D-066 deferred all `--aicontrol` technical details on the explicit assumption that a shared command layer would land first; designing `--aicontrol` against today's drift-prone duplicates would either inherit the F-003 / F-004 class or force the refactor under feature pressure. M5 ships that prerequisite cleanly.

**M6 (multiparty baseline pass with present `--batch`) benefits too.** The "A" baseline measurements in M6 exercise unified handlers rather than the drift-prone duplicates that existed before M5. Measurements done against M5's `ops::*` are directly comparable to the "B" measurements that M7's `--aicontrol` will produce.

### Out of scope for this decision

- The full M7 `--aicontrol` protocol detail (JSONL field shapes, command verbs, event subscription model, named bindings, lifecycle-aware error codes). Those are D-066's scope, designed in the next milestone.
- Tauri commands for the 13 protocol verbs. The current Tauri shell registers only `get_state` / `get_pacing_state` / `quit`; verb-level Tauri commands are a future milestone (likely alongside the long-lived Tauri resident or alongside `--aicontrol`). When they land they will naturally call `ops::*`.
- The flag-vs-config precedence bug in `xgen-node --port` (surfaced during M5 smoke setup). Not xgen-client; carry-over flagged in J-078.

### Relationship to other decisions

| Decision | Relationship |
|---|---|
| D-063 | M5 extends library-first one level deeper than D-063 originally specified — D-063 moved dispatch into the library; D-067 moves command *implementations* into a single shared layer below dispatch. |
| D-066 | D-066 split `--batch` (frozen) from `--aicontrol` (new) and stipulated that the shared `ops::*` layer ships first. D-067 is that ship. |
| J-067 (F-003 / F-004) | Concrete drift instances that motivated M5. D-067 closes the drift surface architecturally; the smoke run in J-078 confirms it closed by behaviour as well as by structure. |

---

## D-068 — CLI flag precedence over config file (locked)

**Date**: 2026-05-17  
**Layer**: Cross-cutting (both binaries) — implementation rule, not protocol  
**Spec reference**: Appendix F §F.0 (Flag model). Cross-reference: J-078 (M5 close-out) for the known violation that surfaced this decision; D-035 (convention-derived paths) for the related rule on path resolution.

### Decision

For any setting that can be specified both as a CLI flag and as a field in a TOML config file, the **CLI flag takes precedence**. No exceptions. The full precedence order is:

1. **CLI flag** (highest priority — most recent operator intent, visible in shell history and automation)
2. **Config file field** (persisted operator intent from `init` or manual edit)
3. **Default value** (the binary's built-in fallback)

This rule applies uniformly to both `xgen-node` and `xgen-client`, to every flag in Appendix F §F.0.1 (fundamental) and §F.0.3 (non-fundamental) that has a config equivalent, and to any future flag added to either binary that shadows a config field.

### Why this rule must be explicit

The rule has been implicit since Phase 1 and is documented per-flag in Appendix F descriptions (e.g. `--node` on Client: "Overrides config"). What was missing is a single citable architectural decision saying *all flags follow this pattern*. The M5 smoke setup (J-078) surfaced a violation of the rule on `xgen-node --port`, which suggests the rule was not universally enforced in implementation.

Three reasons the rule is structural, not stylistic:

**1. CLI is the most-recent intent.** The config file was written at some past time (init, manual edit, possibly stale). The CLI flag is what the operator typed *right now* when starting this process. Right-now intent must beat persisted intent. Anything else surprises the operator.

**2. CLI is visible; config is hidden.** A `--port 8081` in a command line appears in shell history, in scripts, in process listings, in `ps`/`Get-Process` output. A `listen = "..."` deep in a TOML file is invisible from the operational command surface. Visibility matters for diagnosis and audit; the most-visible source must be the authoritative one.

**3. The testing model depends on it.** Every smoke test, stress test, and multiparty scenario sets ports, modes, and instances via CLI flags so a single set of config files can serve many test invocations. The whole testing model assumes flag override is reliable. If a flag silently falls back to config, every test that depends on that flag is unreliable — silently wrong, not loudly broken.

Reason 3 is the operational urgency. M6 (multiparty baseline pass with present `--batch`) and every subsequent test milestone will fire many invocations against different ports, modes, and instance labels. If `--port` is broken on `xgen-node`, every test that varies the port produces results that may or may not reflect actual flag-override behaviour. The smoke-test ground truth degrades.

### Known violation

`xgen-node --port <port>` did not override the `listen` field in `xgen-node_config.toml` on the first invocation during M5 smoke setup (J-078, 2026-05-17). Observed behaviour: Node attempted to bind the *config-file* port (`8080`) rather than the *CLI flag* port (`8081`), failed on conflict with another Node already on `8080`, exited with `os error 10048`. The same command on second invocation succeeded — mechanism unclear (possibly OS-level port release timing, possibly delayed flag-application code path that catches up on retry).

The bug is in `xgen-node`, not in `xgen-client`. It is not M5 scope (M5 was a Client refactor). It is also not blocking M6 in the narrow sense — the workaround is to either match config to intended port at init time, or invoke twice. But the workaround is exactly the kind of silent-test-pollution this decision rules out.

### Audit task scheduled

**Priority: must be resolved before M6 starts.** M6 runs the multiparty test suite against the present `--batch` shape with metrics captured per Clair's protocol (`BATCH_FLAG_review.md`). The metrics protocol depends on flag overrides being reliable. Running M6 against a binary with broken flag precedence would produce metrics whose meaning is ambiguous (did flag X apply, or did config silently win?).

The audit task covers:

1. **`xgen-node --port`** — fix the observed violation. Root-cause the mechanism (full flag-vs-config code path inspection, not just empirical retry).
2. **All other CLI flags with config equivalents on both binaries** — written confirmation per flag that flag overrides config:
   - `--config <path>` (both binaries) vs default search path
   - `--node <endpoint>` (Client) vs `[client].node`
   - `--log-level <lvl>` (both) vs `[logging].level` and `XGEN_LOG` env
   - `--instance <label>` (both) vs implicit default-instance behaviour
   - `--service` (both) vs lifecycle default (Tauri shell)
   - `--local` (Node) vs `[node].local_mode`
   - `--quiet` (both) vs default banner behaviour
   - `--ai-mode` (Client) vs `[ai].is_ai` config
3. **Tests** — each flag-with-config-equivalent gets a focused test that locks the precedence: flag set, config conflicts, assert flag wins.
4. **A short Appendix F clarification** linking flag-by-flag to this decision (already added — §F.0.6).

Task file: `tasks/CLI_PRECEDENCE_AUDIT.md` (to be written before M6 task file is finalised).

**Completed in J-079 (2026-05-17).** The audit shipped in 5 atomic commits (helper + Node `--port` plumbing + four-site subscriber-init convergence + integration tests + doc sync). Empirical verification surfaced four additional violations beyond the named `--port` defect — four parallel subscriber-init blocks were silently dropping `[logging].level` and falling back to a hardcoded `"debug"` literal. Helpers `resolve_setting` and `resolve_log_level` shipped in `xgen-common::precedence`. Test count rose from 435 to 463 (+10 unit precedence + 5 URL-rewrite + 6 Node integration + 7 Client integration). The drift surface is architecturally eliminated — same shape as M5/D-067 eliminated drift in `xgen-client-lib::ops`. See J-079 for the full audit record.

### Out of scope for this decision

- Environment variable precedence (`XGEN_LOG` etc.) above or below config — the only env var currently in use is `XGEN_LOG`, whose precedence vs the `--log-level` flag is documented in Appendix F (flag wins). If more environment variables are introduced later, this decision can be extended.
- The `init` flow's interactive prompts — those are separate (they ask the user for values that go *into* the config file; they are not flag-vs-config comparisons).
- Default-value selection — covered by per-flag documentation in Appendix F; not in scope here beyond confirming defaults are the lowest-priority source.

### Relationship to other decisions

| Decision | Relationship |
|---|---|
| D-028 | Canonical-source rule says Rust doc comments must match Appendix F. D-068 is now a load-bearing rule in Appendix F; doc comments referencing flag-vs-config behaviour must align with it. |
| D-035 | Convention-derived paths rule established that data paths are derived from working directory, not configurable. D-068 is the dual: flags override what *is* configurable. Both decisions are about taking operator-intent volatility out of unexpected places. |
| D-043 | The named-pipe naming convention is partly driven by `--instance <label>` — a flag. D-068 confirms `--instance` is authoritative for pipe naming when present. |
| D-066 | `--aicontrol` (when shipped in M7) is itself a CLI flag opening a control surface. Its presence-vs-absence is by definition flag-driven, not config-driven; D-068 confirms the pattern. |

---

## D-069 — Delegated technical design discipline (locked)

**Date**: 2026-05-17  
**Layer**: Project management / roadmap discipline — not protocol  
**Spec reference**: none (rule about how the roadmap is sequenced and how delegated work is treated). Cross-references: D-066 (the original `--aicontrol` delegation grant); D-068 (the CLI Precedence Audit, which is the model that worked); the M6 descope of 2026-05-17 (the worked example that motivated this decision).

### Decision

When a milestone's technical design is delegated — typically to Chat Claude and Clair operating under a grant like D-066 ("all technical details ... explicitly delegated ... without per-decision approval from Joe") — the implementing milestone MUST NOT be declared ACTIVE in CLAUDE.md until two complementary conditions are met:

**1. Joe-lock on the architectural commitment.** The major shape — the split, the flag name, the binary boundary, the layer placement — comes from Joe and is recorded as a numbered decision in DECISIONS.md. D-066 is the model: a short, named, dated, citable architectural commitment that scopes what the delegation covers.

**2. Self-aware open-item flagging in the delegated detail.** The delegated technical document MUST explicitly list (a) what's been decided, (b) what's open, and (c) which open items can be resolved by Chat Claude/Clair in the design phase vs which need Joe input. The Chat Claude addendum §12 inside `tasks/BATCH_FLAG_review.md` is the model: a numbered list of "Open items for the design phase" that names exactly what hasn't been settled and signals when escalation is needed.

Additionally:

**3. Canonical-document rule.** Each major implementation surface that spans both binaries (or has the potential to) gets exactly one canonical document. Binary-specific implementation detail lives in sections of that document, not scattered across `tasks/`, addenda inside other documents, or DECISIONS.md notes. The canonical document is the single authoritative source; cross-references from CLAUDE.md, DECISIONS.md, and Appendix F point at it, not at the original scattered locations.

### Why this rule must be explicit

Delegation is necessary. Joe cannot review every JSONL field name, every error code string, every pipe-naming detail — the project would never ship. The 2026-05-17 framing in D-066 ("to avoid per-detail approval bottlenecks") is correct: delegation is how work proceeds at sustainable pace.

But delegated drafts that haven't been Joe-locked are structurally indistinguishable from locked specifications when written down in a `tasks/` file or an addendum. A reader (next-session Clair, future Chat Claude, a future contributor) cannot tell from looking at a file whether its contents represent (a) Joe's binding architectural decision, (b) Joe-conversation-locked detail recorded in writing, (c) Chat Claude's delegated draft awaiting refinement, or (d) Clair's working sketch.

The failure mode this decision prevents: a delegated draft gets scheduled as a milestone implementation target without anyone realising parts of it were assumed rather than decided. The implementation session opens, Clair starts execution, design questions surface as gate-questions partway in, and the milestone has to be paused or descoped. **M6 (multiparty baseline pass with present `--batch`) is the worked example.** The metric protocol in `tasks/BATCH_FLAG_review.md` was Joe-conversation-locked on 2026-05-16, but its *application* in the two MULTIPARTY task files was never reconfirmed after J-079 changed the binary shape. M6 was about to start against a delegated runbook whose anchoring assumptions had silently drifted.

Three reasons the rule is structural, not stylistic:

**Reason 1 — The lock step is a session in itself.** Joe-locking a delegated design is not a side task to bundle with implementation start. The implementation session reads a Joe-locked design; the lock session reads a delegated draft and produces a locked design. Conflating the two means implementation starts before the design is settled.

**Reason 2 — Open-item flagging surfaces drift.** The Chat Claude addendum §12 named six open items: full `cmd` verb set, control-surface error codes, subscription filter grammar, `state` command output schema, per-command timeout values, whether Node-side `--aicontrol` is in scope. This list made the design's boundaries visible. Compare: the metric protocol in the same file did not flag "is this still applicable after J-079?" as an open item, so its applicability was assumed when M6 was scheduled. Self-aware flagging is what prevents this.

**Reason 3 — Canonical-document discipline prevents the same lesson recurring.** When design content is scattered (e.g. `--aicontrol` design today lives in D-066, in the Chat Claude addendum inside `BATCH_FLAG_review.md`, in mentions in `tasks/CLI_PRECEDENCE_AUDIT.md`), no single reader can verify the design is complete and locked. Anyone trying to assess shovel-readiness has to reassemble it from three places, and the boundary between locked vs delegated content gets lost in the seams. One canonical document per surface is the structural fix.

### The two states a delegated design can be in

- **Drafted** — exists in `tasks/`, in an addendum, or in working notes. Useful for forward planning. NOT sufficient to schedule the implementing milestone as ACTIVE. May contain open items that haven't been escalated.
- **Joe-locked** — Joe has read the draft, asked questions, and either confirmed it or directed revisions that are now incorporated. The draft is annotated as locked (status header flipped, or a "Locked YYYY-MM-DD" line added, or the content has been promoted into the canonical document). Implementation milestone may now be declared ACTIVE.

### Known instances at time of decision

| Instance | Status as of 2026-05-17 | What needs to happen |
|---|---|---|
| D-068 → CLI Precedence Audit (J-079) | **Worked correctly.** D-068 was Joe-locked before `tasks/CLI_PRECEDENCE_AUDIT.md` was written. The task file enumerated open items per section; Clair gated on Joe approval at each section boundary. M5→audit→M6-or-equivalent ran cleanly. This is the model. | Nothing. Reference for future delegations. |
| D-066 → M7 (`--aicontrol` v1) | **Canonical home created 2026-05-17.** D-066 locks the architectural commitment. The canonical document `docs/xgen_aicontrol_implementation.md` now exists, covering both binaries; its §12 (Open items for design phases) carries forward the six items from the original Chat Claude addendum plus the additions surfaced when extending to both binaries. The addendum inside `tasks/BATCH_FLAG_review.md` remains as a historical predecessor. | M7 design phase resolves the §12 open items in the canonical document; Joe-locks the result; only then M7 goes ACTIVE. |
| `tasks/BATCH_FLAG_review.md` §"Baseline metrics protocol" → M9 (Multiparty Redesign) | **Joe-conversation-locked (2026-05-16) but applicability uncertain.** The metric set itself is sound. What's open: whether the same set applies to a both-binaries `--batch` / `--aicontrol` A/B framing, and whether the post-J-079 binary shifts any captures. | M9 design phase reconfirms or revises the metric set; promotes it into a canonical home (likely `docs/tests/MULTIPARTY_metrics_protocol.md` or similar); Joe-locks the result; only then M9 goes ACTIVE. |
| M6 (original multiparty baseline) | **Descoped 2026-05-17 — the worked example for this decision.** | Replaced by M9 in the roadmap. |
| M6 (new — Node admin write path) | **PENDING.** Architectural commitment locked in this session's CLAUDE.md edit (Node needs read-write admin surface symmetric to Client). Verb-set design is delegated and not yet drafted. | Open a design discussion on the verb set per category; produce `tasks/NODE_ADMIN_WRITE_PATH.md` with explicit open-item flagging à la addendum §12; Joe-lock the result; only then M6 (new) goes ACTIVE. |

### Out of scope for this decision

- Decisions Joe writes directly (D-035, D-061, D-063, D-068, etc.) — these are Joe-locked by definition; no separate lock step needed.
- Implementation-detail decisions inside a Joe-locked design (e.g. the helper signature in `CLI_PRECEDENCE_AUDIT.md` §5 was Clair's proposal, Joe-approved at the §5 gate). The lock is at the design level, not at every internal choice.
- Per-flag, per-verb, per-field micro-decisions that the design phase is explicitly authorised to settle. The rule is about the boundary between delegated draft and locked spec, not about preventing all delegation.
- Joe's discretion to override this rule for a specific milestone if velocity demands it — the rule is the default discipline, not an absolute prohibition. Overrides should be recorded as a note on the affected milestone block.

### Relationship to other decisions

| Decision | Relationship |
|---|---|
| D-066 | The original delegation grant for `--aicontrol`. D-069 adds the gates that delegation must pass before its implementing milestone goes ACTIVE. D-066's text remains valid; D-069 supplies the discipline around it. |
| D-068 | The CLI Precedence Audit is the pattern D-069 generalises. D-068 was Joe-locked before the audit task file was written; the task file flagged open items section-by-section; Clair gated on Joe approval at each gate. D-069 names this pattern and makes it the default for all future delegated milestones. |
| D-067 | M5's `ops::*` refactor architecturally eliminated drift between parallel implementations. D-069 is the discipline analogue: it eliminates drift between delegated drafts and locked specifications by requiring the canonical document and open-item flagging. Both decisions are about taking implicit gaps out of the system. |
| D-035 | Convention-derived paths took operator-intent volatility out of unexpected places. D-069 takes design-state volatility out of unexpected places (the gap between "drafted" and "locked"). Both decisions are forms of the same principle: make implicit state explicit. |

---

## D-070 — Two events of equal importance, opposite direction (named protocol principle)

**Date**: 2026-05-18  
**Layer**: Protocol — specifically wire-message symmetry for outcome signalling.  
**Spec reference**: `docs/xgen_node_admin_ops_design.md` §9 (original draft, preserved as historical record); `docs/xgen_propagation_reliability.md` §5 (J-081 audit finding that produced the corrected framing); `docs/xgen_federation_propagation_design.md` F-4 (the rejection sites this principle operates over).

### Decision

Wherever the XGen Protocol exposes a signal from one party to another about the outcome of an action, both directions of outcome — acceptance and rejection — MUST be exposed with equal first-class status. "Equal first-class status" means: same layer, same lifecycle, same correlation surface. When an action references a specific protocol object (an event, a registration, a federation request), both the acceptance signal AND the rejection signal MUST carry the identifier of that object so the originator can correlate the signal to the action it sent.

The principle has two halves and both are load-bearing:

1. **Both directions exist.** If the protocol exposes rejection (e.g. `TransportMessage::Error`), it MUST also expose acceptance (e.g. `TransportMessage::EventAccepted`). The originator must be able to learn either outcome through a first-class wire signal, not through inference from silence.
2. **Both directions carry the correlation identifier.** The envelope-level `event_id: Option<String>` field on `TransportMessage` (or the equivalent identifier for whatever protocol object the signal pertains to) MUST be populated on both the acceptance and rejection paths. Without correlation, the signal exists but the originator can't tell which of their in-flight actions it pertains to — making the signal hollow at scale.

Joe's verbatim framing, recorded across two moments in M6 Phase 0 Pass 3:

> *"Acceptance and rejection are two events of equal importance, just opposite direction."*

> *"The accept signal's importance warrants its own wire shape, not a side effect of an unrelated mechanism."*

The second quote is why the rejected M6 alternatives (C1 server-side self-fanout, C3 DAG-layer ack EventType) were rejected: neither treated the accept signal as a first-class concern. The first quote names the underlying principle.

### Why the corrected framing matters (vs the M6 §9 draft)

The original draft in `docs/xgen_node_admin_ops_design.md` §9 framed D-070 as "EventAccepted exists, symmetric to Error." That framing is necessary but not sufficient. Post-audit, J-081 §5 found that `TransportMessage::Error`'s wire shape lacked an `event_id` field at all — meaning even with both Error and a future EventAccepted, the originator could not correlate either signal back to a specific event. A driver with multiple in-flight events sees "Error" or "EventAccepted" arrive but has no way to know which event the signal is about.

The corrected framing makes both halves explicit: existence AND correlation. Without (2), (1) is hollow. M6 (new) Phase 2 ships both halves coordinated: the envelope-level `event_id` addition to TransportMessage, the new EventAccepted variant, and the wiring of Error's emit sites in `process_inbound` to populate the new field on every rejection path. F-4 of the Federation Event Propagation milestone produces the rejection sites consistently across all three event families; M6 Phase 2 wires them to the wire-layer signal under D-070.

### Why this is structural, not stylistic

**Reason 1 — It prevents structural-by-accident asymmetry.** The accept-signal gap existed because nobody designed an accept signal; it was a consequence of the event-streaming model (events flow one way; the response is fan-out, not a per-event reply). The Error-lacks-event_id gap existed because nobody designed Error to be correlatable; it was a consequence of Error originally being a generic transport-error signal rather than an event-rejection signal. Both gaps arose from "we didn't think about it" rather than "we deliberately chose this." Asymmetries that arise that way produce silent correctness bugs in the layers above. Naming the principle catches future instances at design time rather than at deployment time.

**Reason 2 — It pairs with D-065 cleanly.** D-065 binds the *content* of signals (don't lie about state). D-070 binds the *existence and correlation surface* of signals (when you can speak in one direction, you can speak in the other, and both directions name what they're about). Together they constrain the protocol to behaviour that is honest, complete, and correlatable. A protocol with only a rejection signal forces consumers to fake acceptance via heuristics (silence-equals-success); a protocol with both signals but no correlation forces consumers to fake correlation via timing (the next signal must be about the last action I sent). D-065 + D-070 together close both gaps.

**Reason 3 — It is reusable across future protocol design.** Any future XGen protocol addition (a new transport message family, a new federation request shape, a new bootstrap interaction, an Auth-Module verb response) inherits the principle. When a future design conversation asks "should this only signal failure, or should it also signal success?", the principle gives a default: yes, both, equal weight, both correlated. Departures from the default require explicit justification.

### Worked instances at promotion

- **`TransportMessage::Error`** — existing variant; gains envelope-level `event_id: Option<String>` in M6 (new) Phase 2. The five event-rejection sites in `process_inbound` ([`xgen-node/src/app.rs:846-851`](xgen-node/src/app.rs:846), [`855-858`](xgen-node/src/app.rs:855), [`885-897`](xgen-node/src/app.rs:885), [`913-921`](xgen-node/src/app.rs:913), [`926-934`](xgen-node/src/app.rs:926)) are wired to emit Error with `event_id: Some(...)` populated.
- **`TransportMessage::EventAccepted`** — new variant in M6 (new) Phase 2. Sent after the inbound event clears validation and is durably persisted, before local fan-out begins (the G2 boundary documented in `docs/xgen_node_admin_ops_design.md` §3.2).
- **Coordination with Federation Event Propagation milestone:** F-4 (validation pipeline unification) produces the rejection sites consistently across all three event families (today Paths B and C reject inline; after F-4 they reject through the dispatcher's `Rejected` return). M6 Phase 2 then wires those rejection sites to the wire-layer signal with envelope `event_id`. Both halves of D-070 land in coordinated milestones; the symmetry is realised at the moment both ship.

### Out of scope for this decision

- **Asymmetries where one direction genuinely doesn't apply.** `TransportMessage::Goodbye` has no `Greetings` counterpart because connection establishment is asymmetric by nature (the WebSocket handshake itself is the greeting). The principle does not force false symmetries where the underlying interaction is genuinely one-directional.
- **Asymmetries internal to the reference implementation.** A binary's CLI surface having a `--start` flag with no `--stop` flag, an admin verb that's WRITE-only with no READ counterpart, etc. The principle is about protocol-level signals, not implementation-internal control flow. The `--aicontrol` JSONL protocol (M7) inherits D-070 because that surface IS protocol-shaped between AI driver and reference implementation; raw CLI flag pairs are not.
- **The propagation reliability question.** That is a separate concern (§4 of the M6 design doc) addressed by the Propagation Reliability Audit milestone (J-081) and the Federation Event Propagation completion milestone. D-070 governs the signalling layer; D-071 governs the discipline of verifying the propagation layer underneath it. Two different concerns, two different decisions.
- **Backward compatibility migration.** Pre-M6 clients that don't recognise `EventAccepted` ignore it gracefully via existing match-arm fallbacks; post-M6 clients talking to pre-M6 Nodes handle the absence of both `EventAccepted` AND `Error` with a bounded timeout fallback documented in M6 design doc §3.6. D-070 lands the principle; the M6 milestone handles the migration mechanics.

### Relationship to other decisions

| Decision | Relationship |
|---|---|
| D-065 | Sibling protocol-design principle. D-065 binds the *content* of signals (don't misrepresent state). D-070 binds the *existence and correlation surface* of signals (when you can speak in one direction, you can speak in the other; both directions name what they're about). Together they make protocol signalling honest, complete, and correlatable. |
| D-066 | The `--aicontrol` JSONL protocol (M7) inherits D-070. Every JSONL reply shape will carry both `result` and `error` paths at equal first-class status with correlation identifiers, mirroring the `Error` / `EventAccepted` symmetry M6 establishes at the wire-message layer. |
| D-067 | M5's `ops::*` refactor architecturally eliminated drift between parallel command implementations. D-070 is the protocol-layer analogue: it eliminates drift between parallel outcome paths (acceptance and rejection) by requiring symmetric first-class signalling with correlation. Both decisions take implicit gaps out of the system architecturally rather than by discipline. |
| D-069 | D-070 was Joe-framed during M6 Phase 0 Pass 3 (a delegated design phase) per D-069 discipline. The corrected post-audit framing surfaced during the J-081 audit close. Promotion to DECISIONS.md follows the D-069 canonical-document rule: the M6 design doc §9 draft remains as historical record; this DECISIONS.md entry is the canonical authoritative form. |
| M6 (new) `docs/xgen_node_admin_ops_design.md` §9 | The original D-070 draft. Preserved as historical record of the principle's framing at M6 Phase 0 Pass 3. The corrected framing in this entry supersedes §9's text for canonical reference. |
| `docs/xgen_federation_propagation_design.md` F-4 | Produces the rejection sites that M6 (new) Phase 2 wires under D-070. The two milestones coordinate at the rejection-signal interface: F-4 ensures rejection paths exist consistently across all three event families; M6 Phase 2 wires them to the wire-layer signal with envelope `event_id`. |
| J-081 (Propagation Reliability Audit) | Produced the audit finding (§5) that the M6 §9 draft's framing was necessary but not sufficient. The corrected framing in this DECISIONS.md entry incorporates the audit's insight. |

---

## D-071 — Subsystem audits precede dependent milestones (project-management principle)

**Date**: 2026-05-18  
**Layer**: Project management / roadmap discipline — not protocol.  
**Spec reference**: none (rule about how milestones are sequenced and what their design phases must include). Cross-references: D-069 (Joe-locked design phase + open-item flagging + canonical-document rule); D-065 (sibling principle — honest behaviour over polite behaviour); J-081 (the Propagation Reliability Audit, where the pattern emerged).

### Decision

Every future milestone whose correctness depends on a load-bearing subsystem MUST include a subsystem audit as part of its Phase 0 (design phase). The audit runs before design decisions are locked, produces a code-grounded canonical document, and surfaces gaps that may need to close as preconditions of the milestone rather than as parallel work.

"Load-bearing subsystem" means a piece of infrastructure that the milestone's deliverables claim to operate against — a propagation pipeline, a validation pipeline, a federation registry, a transport surface, an event-store mechanism, the Auth Module dispatch. If the milestone's promises depend on the subsystem working as specified, the subsystem's actual working state must be verified, not assumed.

The audit's outputs:

1. A canonical document (`docs/xgen_<subsystem>_<audit-type>.md` shape) recording findings with code-grounded evidence — file paths, line numbers, function names, behavioural traces.
2. A severity-classified gap list (HIGH / MEDIUM / LOW / INFORMATIONAL, with explicit criteria for each level given the milestone's context).
3. An explicit statement of which gaps are preconditions of the dependent milestone vs which are parallel work vs which are recorded for future milestones.

The audit is sized to fit; it is not a re-architecture project. The Propagation Reliability Audit (J-081) shipped in one session and verified five stages of the propagation lifecycle.

### Why this rule must be explicit

The pattern emerged organically during the Propagation Reliability Audit. Two observations established it:

**Observation 1 — Audit findings consistently exceeded the audit's nominal scope.** J-081 was opened to verify Stage 6 federation propagation reliability. It returned HIGH-severity findings in four of five sections — not just Stage 6. The audit found what it was opened to find AND surfaced multiple substantial unexpected gaps (validation asymmetry in `process_inbound`, Error wire shape lacking `event_id`, `sync_complete` gap masking premature catch-up termination, pagination gap allowing unbounded responses). Without the audit, those gaps would have surfaced under feature pressure during M6 (new) or Federation Event Propagation implementation, producing emergency descope or hotfix work.

**Observation 2 — The audit became the precondition input for the dependent milestone's design phase.** Federation Event Propagation Phase 0 took J-081 as Pass 1 input rather than running its own audit. The audit work paid for itself across two milestones (M6 Phase 0, which originally motivated it, and Federation Event Propagation Phase 0, which inherited it). One audit, two downstream design phases consume it.

Three reasons the rule is structural, not stylistic:

**Reason 1 — Subsystem reality drifts from documentation.** The audit found that `docs/xgen_ch4_implementation.md` §4.11.3 + §4.12.3 and `docs/xgen_node_admin_ops_design.md` §4.2 described federation propagation mechanisms that did not exist in code. Without an audit, the dependent milestone's design phase would have inherited the documented-but-absent behaviour as a working baseline. The longer a documented-but-absent behaviour goes unaudited, the more downstream design accumulates against it. Audits make documentation drift visible at the moment it costs least to fix.

**Reason 2 — Multi-session design conversations accumulate assumptions.** The J-080 framing that `TransportMessage::Error` was the rejection signal for event acceptance had been operating across multiple sessions. Direct code trace at audit close refuted it: `Error` had no `event_id` field, and none of the five event-rejection sites in `process_inbound` emitted it — they all just logged via `tracing::error!` + `trace_local(RejectEvent)`. The assumption had been confident, consistent, and wrong. Audits force the moment of "actually look at the code" that long-running design conversations defer indefinitely.

**Reason 3 — The pattern is naturally one-time per subsystem.** Once J-081 audited propagation, the canonical document is durable. Future milestones touching propagation read the audit doc rather than re-discovering its findings. The audit's cost amortises across all dependent work. This is the same shape as D-069's canonical-document rule applied at the verification layer: one authoritative source per subsystem state, others point at it.

### Sequencing with D-069

D-071 extends D-069 backward by one phase. D-069 governs the design phase: Joe-lock + open-item flagging + canonical document. D-071 governs the phase before the design phase: the audit phase. The full sequence for a milestone touching a load-bearing subsystem is:

```
Audit phase (D-071)  →  Design phase (D-069)  →  Implementation phase
     |                       |                          |
  Audit doc        Joe-locked design doc          Runbook + commits
   (canonical)        (canonical)                    (Clair work)
```

Each phase produces a canonical artefact. The audit doc feeds the design doc; the design doc feeds the runbook. A milestone that skips the audit phase produces a design phase whose Pass 1 input is documentation rather than code, and the documentation may be drift. A milestone that skips the design phase produces an implementation phase against decisions never Joe-locked, per D-069.

Both disciplines together: every dependent milestone gets verified reality (D-071) AND locked design (D-069) before code is written.

### Known instances at promotion

- **M6 (new) Phase 0 → Propagation Reliability Audit (J-081, 2026-05-18).** Originally motivated by the J-080 carry-over (`cmd_create_space` optimistic-ack UX) escalating to a missing protocol primitive (no positive accept signal exists today). Audit closed in one session, produced `docs/xgen_propagation_reliability.md`, surfaced four HIGH findings across five stage sections.
- **Federation Event Propagation Phase 0 (Pass 2 + Pass 3, 2026-05-18) → inherits J-081.** No re-audit; the audit's outputs are Pass 1 of the design phase. Design phase Pass 2 produced ten framework decisions; Pass 3 produced the canonical design document and implementation runbook for Clair.

This instance pattern — one audit feeding two design phases — is the worked example for the reasoning above (audits pay for themselves across dependent work).

### Out of scope for this decision

- **Audits as standalone milestones detached from dependent work.** The discipline is about coupling audits to milestones that need them, not creating audit-for-its-own-sake work. The audit's value is its consumption by a dependent design phase; an audit with no dependent milestone scheduled is paperwork.
- **Re-auditing already-audited subsystems on every dependent milestone.** Once audited, subsequent milestones read the canonical audit doc; re-audit only if the subsystem has materially changed since the canonical doc shipped. The decision to re-audit is itself a design-phase Pass 1 question for the new milestone, not a routine ritual.
- **Audits of fully-stable subsystems where the dependent milestone has no exposure to gaps.** Crypto primitives (`ed25519-dalek`, ChaCha20-Poly1305 from `chacha20poly1305`, Argon2id from `argon2`) are not re-audited per XGen milestone — that is the upstream maintainers' work, consumed via crates.io. Settled wire formats whose semantics haven't changed in many milestones are similarly out of scope. The principle applies where there is realistic risk of drift between specification and implementation, not as a blanket requirement for every dependency.
- **The audit's exact methodology, severity-classification thresholds, or document template.** The J-081 audit shape (five-stage walk + per-section verdict + drift surface tally + canonical-doc output) is a precedent, not a prescription. Future audits adapt their methodology to the subsystem they verify. What the principle requires is that the audit *exists*, produces a canonical artefact, and feeds the dependent design phase; how it gets there is the auditor's call.

### Relationship to other decisions

| Decision | Relationship |
|---|---|
| D-065 | Sibling principle (honest behaviour over polite behaviour). D-065 is protocol-design; D-071 is project-management. Both decisions take implicit gaps out of the system: D-065 makes the protocol honest about its state at runtime; D-071 makes the project honest about its subsystems' state before locking design. The shared theme: don't let assumed-state substitute for verified-state. |
| D-067 | M5's `ops::*` refactor eliminated drift between parallel command implementations architecturally. D-071 eliminates drift between documented-vs-actual subsystem behaviour by requiring code-grounded audits. Both decisions are about taking implicit gaps out of the system — D-067 at the implementation layer, D-071 at the verification layer. |
| D-069 | The two disciplines pair: D-069 governs the design phase (Joe-lock + open-item flagging + canonical document), D-071 governs the audit phase before it. D-071 extends D-069's logic backward: design must be locked before implementation, and verification must precede design. Both decisions enforce that earlier discovery prevents implementation-time surprises. |
| D-070 | Sibling decision shipped earlier the same day. D-070 is protocol-design; D-071 is project-management. The two were both surfaced during the Propagation Reliability Audit's close-out: D-070 from the audit's §5 finding about Error wire shape; D-071 from the audit's §6.2 pattern observation about drift surfaces across multiple sections. Both were originally drafted in the M6 design doc and Federation Event Propagation work; both promoted to DECISIONS.md in coordinated post-Pass-3 work. |
| J-081 (Propagation Reliability Audit) | The audit that established the pattern. D-071 names the discipline that J-081 retroactively instantiates. Future audits inherit the J-081 shape (one session, code-grounded, severity-classified, canonical-document output) as a precedent but are not bound to its exact methodology. |
| M6 Phase 0 + Federation Event Propagation Phase 0 | The two milestones whose design phases consumed J-081's output. Pattern: subsystem audit → dependent milestone's Phase 0 design uses audit as Pass 1 input → Phase 0 produces design doc → implementation runbook → implementation. Both are worked examples of D-071 + D-069 operating together. |

---

## D-072 — XGID Adoption v1 (named identifier type discipline)

**Date**: 2026-05-20  
**Layer**: Cross-cutting — vocabulary + type discipline spanning every crate (`xgen-common`, `xgen-core`, `xgen-node`, `xgen-client`) and every documentation surface (Ch3, Ch4, Ch6, Appendix F, Appendix I, Appendix J, `docs/xgen_aicontrol_implementation.md`).  
**Spec reference**: `docs/xgen_appendix_j_en.md` (canonical expository document — taxonomy, construction, wire-invariance promise, immutability framing, worked rejection examples); `docs/xgen_ch3_specification.md` §3.X (terse normative section). Cross-references: D-073 (field-name-vs-type discipline that underwrites how XGID flavours compose with field names at use sites); D-069 (canonical-document rule — Appendix J is the canonical home, others point at it); D-065 (sibling principle — wire-format honesty over local convenience).

### Decision

The XGen Protocol adopts **XGID** as the canonical name and type discipline for all first-class identifiers in the protocol. Six XGID flavours ship at v1: **Event**, **Space**, **Room**, **TrustAssertion** (hash-anchored family) and **Node**, **Identity** (principal family). The Rust type representation is a layered newtype: a base `Xgid(String)` plus six flavour wrappers (`EventXgid`, `SpaceXgid`, `RoomXgid`, `TrustAssertionXgid`, `NodeXgid`, `IdentityXgid`), each `Deref<Target = Xgid>`, all serde-transparent as plain strings so wire format is untouched. All six wrappers, the base type, and the `XgidLike` trait ship in `xgen-common` at v1.

The principle wording, locked at walkthrough close 2026-05-20 and reproduced verbatim in Appendix J:

> *"XGID Adoption v1 ships the types and adopts them in new code. Retrofitting existing XGID-string fields is staged into subsystem-scoped retrofit milestones. The codebase MAY carry mixed discipline transitionally; every new field, new signature, and new trace event field MUST use XGID types from this milestone forward."*

Five **wire-format invariances** are guaranteed under D-072 across both wire crossings the protocol exposes (federation wire AND AI control / batch JSONL wire):

1. **Field names** — the JSON field name carrying an XGID does not change between v1 and any future retrofit pass.
2. **Field types** — the on-wire JSON type is `string`, regardless of which Rust newtype wraps it.
3. **Canonical form** — the string contents of any XGID are byte-identical when produced from the same inputs anywhere in the federation.
4. **URI grammar** — the structural shape of XGID strings (prefix, separators, length characteristics, character class) is fixed at v1 and does not change under retrofit.
5. **String-equality semantics** — two XGIDs are equal iff their string contents are equal. No flavour-aware comparison; no normalisation hooks.

The **second wire crossing** is named explicitly: the AI control / batch JSONL wire format (`docs/xgen_aicontrol_implementation.md`, Appendix F's batch reply schemas, Ch6 §6.15) inherits the five invariances. Any boundary where XGID strings cross a process is bound by the same rules; the protocol does not get to be sloppy at the implementation-protocol seam.

Adoption discipline is **Shape γ + ASAP** — staged retrofit milestones (XGID Retrofit Passes 1–5) land in ROADMAP.md Near future immediately after v1 ships, not Far future. The five passes are subsystem-scoped: Pass 1 retypes `xgen-common` core types and Appendix I Part I; Pass 2 retypes `xgen-core` validation/dispatch/pending-buffer surfaces; Pass 3 retypes `xgen-node` federation/fanout/app surfaces and Appendix F Node-side sections; Pass 4 retypes `xgen-client` ops/AI-behaviour/batch surfaces and the AI-control documentation; Pass 5 retypes test fixtures, helpers, trace events, and any remaining surfaces. After Pass 5 closes, the "mixed discipline transitionally" clause of the principle wording no longer applies.

### What XGID is and is not

**XGID is** the canonical name for first-class protocol identifiers — things that name a durable protocol object that other protocol objects reference by identity. The six flavours are exhaustive at v1. Sub-flavours (e.g. ephemeral `session_id` as an Event-XGID sub-axis) are taxonomic refinements within Appendix J, not new top-level flavours.

**XGID is not**:

- **Wire-envelope correlation handles.** M6 (new) Phase 2's `event_id: Option<String>` field on `TransportMessage` is a transport-layer correlation handle, distinct from the Event XGID it correlates to. The two are equal at the string level by construction but live at different protocol layers and have different lifecycles.
- **Error codes.** Numeric or string-tagged error codes (`4002`, `4006`, `4007`, etc.) are not XGIDs.
- **Config field names** or in-memory handle types like `FederationPeerSenders` keys (even though the keys' string values are XGIDs — the *map structure* isn't an XGID).
- **File paths, log line tokens, debug formatters.** XGID types may *appear* in these via `Display` / `Debug`, but the paths/tokens themselves are not XGIDs.
- **Bootstrap discovery URIs.** Discussed during Q1 walkthrough and explicitly excluded — these are operational addresses, not protocol-object identifiers.

### Why this discipline must be explicit

**Reason 1 — Field-typed-as-String hides protocol-object semantics.** A Rust function signature `fn foo(a: String, b: String, c: String)` carries no information about which argument is which protocol object. A reader has to consult the call site, the field name, and ideally a doc comment to recover the role each `String` plays. Layered newtypes recover that information in the type system: `fn foo(event_id: EventXgid, sender: IdentityXgid, room_id: RoomXgid)` cannot be miscalled. The protocol has eight years of identifier discipline ahead of it; a String-typed identifier surface accumulates miscalls and misroutings at a rate that retrofits cannot keep up with.

**Reason 2 — Without a canonical name, vocabulary fragments.** Before this decision, the project used "event ID", "event id", "sender pubkey", "node URI", "space ID", "room ID", "identity URI", "trust assertion ID" across documentation and code interchangeably and inconsistently. Different docs used different framings; different code used different field names for the same protocol object. "XGID" provides one umbrella name; six flavours provide the discriminators; all parts of the protocol that need to name an identifier reach for the same vocabulary. The discipline pays off most heavily in design conversations: "is this an XGID?" becomes a tractable question with a yes-or-no answer, where "is this an identifier?" was an open framing question every time.

**Reason 3 — Wire-invariance must be the default, not an aspiration.** A protocol whose identifiers can drift in field name, field type, canonical form, URI grammar, or equality semantics between releases produces federation-breakage at every release boundary. The five-invariance promise sets the default to "no drift"; departures from the default require explicit protocol-version negotiation, not silent change. The naming of the invariances at the wire-format layer (rather than as Rust-type-system properties) means the same promise binds non-Rust implementations: any future XGen client, written in any language, gets the same wire-level guarantees.

**Reason 4 — Staged retrofit is honest about the cost of perfection.** A "retype everything in one milestone" approach would either delay v1 by months or ship a v1 with cut corners. Shape γ + ASAP retrofit acknowledges the cost honestly: v1 ships the types and the discipline; existing String fields convert pass-by-pass over the subsequent retrofit milestones; the codebase carries mixed discipline transitionally and explicitly. This is the same shape as D-065's principle (honest behaviour over polite behaviour) applied to a project-management surface: the protocol does not pretend to be perfectly typed during the transition; it states the transition as a real and named project phase.

### Worked instances at promotion

- **Phase 7.5 `SpaceLocalMetadata.introducer_node_id`** — the v1 inaugural production use of an XGID flavour. The field was named with future-XGID-typing in mind during Phase 7.5 design (per §5.6 of `tasks/FEDERATION_PROPAGATION_PHASE_7_5_DESIGN.md`); the v1 implementation runbook retypes it from `Option<String>` to `Option<NodeXgid>` in Commit 2.
- **Phase 9 integration test infrastructure** — the test code Phase 9 ships uses XGID types from the start rather than being retrofitted later. This is why XGID Adoption is sequenced between Phase 7.5 closure and Phase 9 Commit 3 resumption: the test surface is touched once, with the right types.
- **`xgen-common` v1 — the type definitions themselves.** Six flavour wrappers, base `Xgid`, `XgidLike` trait, flavour-specific constructors (e.g. `EventXgid::from_event`, `NodeXgid::from_pubkey`), flavour-specific methods (e.g. `IdentityXgid::pubkey() -> VerifyingKey` on principal flavours; content-derived helpers on hash-anchored flavours), `Deref<Target = Xgid>` on each wrapper, serde-transparent string serialisation, full `Display` / `Debug` / `Eq` / `Hash` / `Clone` derives.
- **Pass 1–5 worked subsystems** — the five retrofit passes are themselves worked instances of the staged-retrofit discipline. Pass 1 (`xgen-common` core types) starts immediately after Phase 9 closes and the Federation Event Propagation milestone flips DONE.

### Out of scope for this decision

- **Future XGID flavour additions.** If a new protocol object surfaces that warrants first-class identifier status (and isn't a sub-axis of an existing flavour), the addition is a future decision, not a parameter of D-072. The taxonomy at v1 is the six-flavour set; growth requires explicit promotion through a future decision entry.
- **Cross-flavour conversion semantics.** Whether (e.g.) a `NodeXgid` can be converted to an `IdentityXgid` is a use-site question answered by use-site logic, not a type-system feature. The flavour wrappers are deliberately not interconvertible at the type level; converting one to another requires extracting the base `Xgid` (via `Deref`) and constructing the target flavour explicitly. This is a feature, not a limitation: silent flavour drift at use sites is what the newtype discipline exists to prevent.
- **Normalisation, case-folding, or whitespace-tolerance.** Invariance 5 (string-equality semantics) is strict: two XGIDs are equal iff their bytes are equal. No normalisation hooks at v1; if normalisation becomes necessary later, it's a protocol-version-bumped change, not a quiet upgrade.
- **Implementation language coupling.** XGID is a protocol-layer concept; the Rust layered-newtype implementation is the v1 *reference* implementation. Future XGen clients in other languages implement the same vocabulary, the same flavours, and the same five wire invariances; they MAY implement the type discipline differently (or not at all, if their type system can't express it cleanly). The invariances bind the wire; the types bind the reference implementation.
- **Wire-format protocol-version negotiation.** D-072 says identifiers don't drift at v1; it does not say there can never be a future protocol version with different identifier semantics. Future versions are explicit version bumps with explicit migration paths, not silent retrofits.

### Relationship to other decisions

| Decision | Relationship |
|---|---|
| D-073 | Coordinated output of the XGID Adoption design walkthrough (same session 2026-05-20). D-073 names the field-name-vs-type discipline that XGID's layered newtype design depends on at use sites: a field named `introducer_node_id` typed as `NodeXgid` says "this is the introducer's identity, and it is a Node XGID" — the name carries the role, the type carries the contract. D-072 establishes the types; D-073 establishes how those types compose with field names. Both decisions land in the same Phase 1 canonical sources commit. |
| D-065 | Sibling principle (honest behaviour over polite behaviour). D-065 is the protocol-design analogue; D-072's adoption discipline is the project-management analogue. Where D-065 requires the protocol to be honest about runtime state, D-072 requires the project to be honest about adoption state: "mixed discipline transitionally" is explicit, named, and bounded by the Pass 5 closure point. Both decisions take implicit gaps out of the system: D-065 from the protocol's behaviour, D-072 from the project's identifier vocabulary. |
| D-069 | The canonical-document rule applies here: Appendix J is the canonical home for XGID concepts; Ch3 §3.X carries the terse normative form; DECISIONS.md D-072 is the architectural commitment; all three reference each other and do not duplicate. The Phase 1 canonical sources commit is itself a worked example of D-069 discipline: a multi-surface concept gets one authoritative document (Appendix J) with downstream sources pointing at it, not scattered. |
| D-070 | Coordinated relationship at the protocol layer. D-070's `event_id: Option<String>` envelope-level correlation handle is *not* itself an XGID (per the "what XGID is not" section above), but its string value is byte-equal to the corresponding Event XGID by construction. The relationship is documented at the use site, not encoded in the type system: D-072's flavours bind protocol-object identifiers; D-070's envelope field is a transport-layer correlation handle that happens to carry an XGID-shaped string. Keeping the two separate at the type level prevents miscalls between protocol-layer and transport-layer surfaces. |
| D-071 | Sibling project-management principle. D-071 governs the audit phase before milestone design (verify reality before locking design). D-072 governs adoption discipline across the whole project (commit to vocabulary + types; stage retrofit honestly). Both decisions take implicit gaps out of the project's shape: D-071 between documentation and code; D-072 between identifier vocabulary in design conversations and identifier types in implementation. The shared pattern: make implicit state explicit. |
| Phase 7.5 design (`tasks/FEDERATION_PROPAGATION_PHASE_7_5_DESIGN.md`) | The originating precedent for XGID-aware field design. The `introducer_node_id` field's naming was Joe-locked at §5.6 with explicit future-XGID-typing intent. D-072 promotes that one-off forward-aware decision into a project-wide discipline; D-073 names the field-name-vs-type principle the §5.6 decision instantiated. Phase 7.5's implementation runbook retypes the field as the v1 inaugural production use. |

---

## D-073 — Field-name-vs-type discipline (project-wide naming principle)

**Date**: 2026-05-20  
**Layer**: Cross-cutting — naming and typing discipline at every Rust struct field, function parameter, trace event field, and JSON wire field across all four crates and all documentation surfaces describing them.  
**Spec reference**: `tasks/FEDERATION_PROPAGATION_PHASE_7_5_DESIGN.md` §5.6 (originating precedent — the `introducer_node_id: NodeXgid` worked example). Cross-references: D-072 (the XGID type vocabulary this principle composes with at use sites); D-069 (canonical-document rule — Appendix J's introduction echoes this principle in one sentence pointing here); D-065 / D-070 / D-071 (sibling architectural principles that take implicit gaps out of the system).

### Decision

**The field name carries the role; the type carries the contract.**

Every Rust struct field, function parameter, trace event field, and JSON wire field that names a protocol object obeys this composition rule:

- **The field name** identifies the *role* the protocol object plays at this use site — what *this particular instance* is doing here. Examples: `introducer_node_id` (the Node that introduced this Space to us); `sender` (the Identity that signed this event); `room_id` (the Room this message belongs to); `peer_node_id` (the Node on the other side of this federation session); `delegated_to` (the Identity an operator role was delegated to).
- **The field type** identifies the *contract* — what kind of protocol object this field can ever hold. Examples: `NodeXgid` (always a Node XGID, never an Identity XGID); `IdentityXgid` (always an Identity XGID); `RoomXgid` (always a Room XGID).

The two pieces of information are orthogonal and both are load-bearing. A field name without type discipline tells you the role but not the contract — a reader has to consult the field's documentation to know that `introducer_node_id` is always a Node XGID, never something else. A type without role discipline tells you the contract but not the role — a function signature `fn foo(a: NodeXgid, b: NodeXgid, c: NodeXgid)` cannot be miscalled at the type level, but a reader has no way to know which Node is which without consulting the docs. Both pieces together produce code that is self-documenting at the use site: `fn foo(introducer: NodeXgid, peer: NodeXgid, owner: NodeXgid)`.

The principle applies to all four surfaces:

1. **Rust struct fields** — `pub introducer_node_id: Option<NodeXgid>`, not `pub introducer: Option<NodeXgid>` (role lost) and not `pub introducer_node_id: Option<String>` (contract lost).
2. **Function parameters** — `fn drain_pending_by_federation_relationship(peer: NodeXgid, space: SpaceXgid)`, not `fn drain_pending_by_federation_relationship(a: String, b: String)`.
3. **Trace event fields** — when a structured trace event carries an XGID, the field name in the event matches the use-site role (e.g. `originator_identity` vs `recipient_identity`) AND the field is typed as the appropriate XGID flavour (not bare `String`).
4. **JSON wire fields** — same rule applied through serde-transparent serialisation: the wire field name carries the role, the underlying Rust type carries the contract. Wire readers see strings (per D-072 invariance 2), but the surrounding field name still names the role.

### Why this discipline must be explicit

**Reason 1 — The discipline emerged organically and was about to be lost in transition.** The originating precedent (Phase 7.5 §5.6, `introducer_node_id`) was Joe-locked mid-design as a forward-looking naming decision: the field was named with a future XGID-typing pass in mind, and the §5.6 inline note explained the reasoning. Without promotion to a DECISIONS.md entry, the rationale would have lived only in a Phase 7.5 design file — which becomes archived once Phase 7.5 ships, and whose authority decays with it. The next person designing a new field would either re-derive the principle from scratch, miss it, or invent a different one. Naming the discipline makes it durable across milestones.

**Reason 2 — Field-name-only discipline produces accidental String typing.** Without the type half of the discipline, a well-intentioned designer who names a field correctly (`introducer_node_id`) is free to type it as `String` because "the name says what it is." That works for one field by one designer in one PR. It fails when the field is used at five call sites, or when a second designer adds a sibling field (`peer_node_id`) and chooses a different type, or when a JSON-decoded value flows into the field without the surrounding type guard. The type half of the discipline is what makes the name-half load-bearing: the compiler enforces what the name claims.

**Reason 3 — Type-only discipline produces opaque use sites.** Without the name half, a function signature like `fn handshake(a: NodeXgid, b: NodeXgid)` is type-safe but unreadable. Which Node is `a`? Which is `b`? A reader has to consult the function body or doc comment to learn that `a` is the local Node and `b` is the remote Node. Naming the role at the field-name level pushes that information to the first place a reader looks, which is the signature itself.

**Reason 4 — The principle generalises beyond XGID.** While XGID is the v1 worked example, the field-name-vs-type discipline applies to every type the project uses for protocol-object identifiers, capabilities, or roles. A future field carrying a capability set (`pub required_capabilities: CapabilitySet`) follows the same rule: name says role (`required_capabilities`, distinct from `granted_capabilities`); type says contract (`CapabilitySet`, not bare `Vec<String>`). Naming the discipline as a standalone decision (not as a footnote to D-072) signals that it operates wherever a typed field carries a role-bearing semantic, not only for identifiers.

### Worked instances at promotion

- **`SpaceLocalMetadata.introducer_node_id: Option<NodeXgid>`** — the originating precedent, locked at Phase 7.5 §5.6 and realised in XGID Adoption v1 Commit 2. The Phase 7.5 design walkthrough explicitly chose this name over candidates like `introducer` (role-only, lost the "Node XGID" contract signal) and `introducer_id` (ambiguous about which kind of ID — could be Node, Identity, or Space at a glance). The locked name encodes both halves: the role (introducer) and the contract (a Node ID).
- **`peer_node_id` / `space_id` / `room_id` / `identity_id` as established naming convention.** The four idiomatic field names already widely used across the codebase obey the discipline at the name level; XGID Adoption v1 and the subsequent retrofit passes complete the discipline at the type level.
- **Forward-looking application to AI-control and admin-ops surfaces.** When M7 (`--aicontrol`) and M6 (new) ship their JSONL reply schemas, each XGID-carrying field obeys both halves: role-bearing names (`accepted_event_id`, `rejected_event_id`, `target_room_id`, `delegated_to_identity`) with XGID-flavour types underneath.

### Out of scope for this decision

- **Acceptable role-bearing field names.** The principle requires that field names carry a role; it does not prescribe a closed vocabulary of role names. `introducer_node_id` vs `bootstrapping_node_id` vs `origin_node_id` are all acceptable role-bearing names for similar concepts; the choice between them is a use-site-design question, not a D-073 question.
- **Non-XGID typed fields.** The principle is a *general* composition rule; this decision documents it via XGID worked examples because XGID is the v1 surface where it most heavily applies. Application to other typed-field surfaces (capabilities, error codes, event-type discriminators) is implicit in the principle's generality and does not require enumerating every future case here.
- **Naming-only docs (e.g. JSON wire docs where Rust types are not visible).** A JSON-only document like `docs/xgen_aicontrol_implementation.md` cannot show Rust types directly. The principle still applies through transitivity: the JSON field name carries the role, the documented type contract ("this field is an Event XGID") carries the contract, and the implementation enforces both halves through serde-transparent typed Rust fields.
- **Internal-only helper functions.** Discipline is meaningful at API boundaries (public structs, function signatures consumers see, trace events external observers consume, JSON wire fields). Truly local helpers (`fn parse_inner(s: &str) -> Result<...>`) are not bound to use role-bearing parameter names if the role is obvious from one call site over a five-line function. The principle is about preventing miscalls at scale, not about adding ceremony to trivial code.

### Relationship to other decisions

| Decision | Relationship |
|---|---|
| D-072 | Coordinated output of the XGID Adoption design walkthrough (same session 2026-05-20). D-072 establishes the type vocabulary (six XGID flavours, layered newtype, wire invariances); D-073 names the discipline that XGID's layered newtype design relies on at use sites — every use of an XGID type pairs with a role-bearing field name. Without D-073, D-072's type discipline could still be undermined by opaque field names; without D-072, D-073's role-bearing names would have no type system to enforce contracts against. Both decisions land in the same Phase 1 canonical sources commit. Appendix J's introduction carries a one-sentence echo of D-073 pointing here. |
| D-065 | Sibling principle (honest behaviour over polite behaviour) at the naming layer. D-065 requires the protocol to be honest about state; D-073 requires field names and types to be honest about what they hold. A field named `node_id` typed as `String` is dishonest in the same architectural sense: it claims (through the name) to hold a Node ID but cannot enforce (through the type) what kind of ID. The discipline takes that dishonesty out of the use site by structural means. |
| D-069 | The canonical-document rule applies: D-073's authoritative home is DECISIONS.md; Appendix J's introduction carries a one-sentence echo with a pointer here; `tasks/FEDERATION_PROPAGATION_PHASE_7_5_DESIGN.md` §5.6 remains as historical originating-precedent record. Three surfaces, one authority, explicit forwards. |
| D-070 | The envelope `event_id: Option<String>` field on `TransportMessage` is the worked counter-example: it deliberately departs from the field-name-vs-type discipline (the name carries the role "event ID" but the type is bare `String`, not `EventXgid`). The departure is intentional and documented in D-072's "what XGID is not" section: `event_id` is a transport-layer correlation handle, NOT itself an XGID, and the type-level separation prevents miscalls between protocol-layer and transport-layer surfaces. D-073 thus tolerates documented exceptions where the architectural reasoning supports them. |
| D-071 | Sibling project-management principle. Both decisions take implicit gaps out of the project: D-071 between documented and actual subsystem behaviour; D-073 between named roles and enforced contracts at field-level granularity. The shared pattern across D-065 / D-069 / D-070 / D-071 / D-072 / D-073 is the same: make implicit state explicit at the layer where the implicitness produces drift. |
| Phase 7.5 §5.6 (originating precedent) | The Joe-locked naming decision that produced the principle. §5.6 of `tasks/FEDERATION_PROPAGATION_PHASE_7_5_DESIGN.md` chose `introducer_node_id` over candidates that lost either the role half (`introducer`) or the contract half (`introducer_id`). The §5.6 inline reasoning is preserved as historical originating-precedent record; D-073 promotes the underlying principle into a project-wide discipline. |

---

## D-075 — `state.federation_add` is one party's act; `federation_nodes` is a vantage-aware derived projection

**Date**: 2026-05-21  
**Layer**: Protocol (event-model semantics for relationship-shaped events; cross-cutting with the applier discipline that derives state from events).  
**Spec reference**: `tasks/FEDERATION_BIDIRECTIONAL_NODES_DESIGN.md` §3 (Q1 + Shape A + A.1 locks); `tasks/FEDERATION_BIDIRECTIONAL_NODES_AUDIT.md` (canonical record of the finding that surfaced the design question); `docs/xgen_federation_propagation_design.md` §6.4.2 (canonical summary, added by implementation runbook Commit 1). Cross-references: D-070 (different shape of "two events" principle — D-070 is about acceptance vs. rejection signal pair on `TransportMessage`; D-075 explicitly rejected an analogous two-events framing at the event-model layer); D-072 / D-073 (XGID type discipline at the field-name-vs-type layer; D-075 is the analogous discipline at the event-semantic layer); D-065 (sibling principle at the protocol-behaviour layer).

### Decision

**`state.federation_add` records one party's act, not a two-sided relationship object.** The event has one signer (the asserter), one signature, one DAG slot, one logical assertion: "A declares that A federates with B for Space S." The asymmetry between `event.sender` (the asserter) and `event.content.node_id` (the other party) is the event's structure, not a redundancy.

**`SpaceState.federation_nodes[S]` is a derived projection over `state.federation_add` events with a vantage-aware derivation rule:** for each such event E, my local entry adds the **other party** — `event.sender` if I am `event.content.node_id`, else `event.content.node_id`. A and B's `federation_nodes[S]` end up as mirrors as a *consequence* of correct application, not as a structural property of the event store.

The principle generalises beyond `state.federation_add`. Any future protocol event recording a relationship between two parties follows the same pattern: one event per assertion, vantage-aware derivation at the applier. The protocol does NOT use "two events forming one relationship" as an event-model shape.

### Why the principle must be explicit

**Reason 1 — Precedent fit with the rest of the event registry.** Every relationship-shaped event in the current registry follows the one-party-assertion + derived-projection pattern: `membership.invite`, `membership.join`, `state.dm_promote`, `state.space_create`, `state.room_create`, `state.ai_operator_delegate`, `state.ai_operator_revoke`, `membership.kick`, `membership.ban`, `membership.mute`. The event records what one party did; the resulting data object (members list, room state, federation_nodes, pending_invites, ai_operator_delegations, banned set, active_mutes) is a derived projection. The applier may need vantage-awareness when interpreting the event (D-075's contribution), but the events themselves are always one-party assertions.

Without naming this principle, a future contributor adding a new relationship-shaped event would have no architectural guidance on whether to follow the one-event-per-assertion pattern or to invent a new two-events-per-relationship pattern. The bidirectional `federation_nodes` audit walked through exactly this question for `state.federation_add` and chose the one-event pattern; D-075 promotes that walkthrough's conclusion into a project-wide principle so future contributors don't re-derive it from scratch (or worse, miss it).

**Reason 2 — The bug that surfaced the principle was a derived-state error, not an event-shape error.** Phase 9 Commit 3a's Scenario 1 diagnostic run found that B's `federation_nodes[S]` ended up containing B's own Node ID instead of A's. The event itself was correctly constructed (A's `sender`, B's `content.node_id`); the bug was the applier reading `content.node_id` verbatim from every vantage. Fixing the applier (Shape A) restored correctness without any wire-format change. The principle D-075 names is what the fix instantiates: derived state can be vantage-aware, events do not need to be.

**Reason 3 — The rejected alternative (two events per relationship) would have been a real departure.** Reading (ii) of the design walkthrough (preserved in `tasks/FEDERATION_BIDIRECTIONAL_NODES_DESIGN.md` §3.1's reasoning) would have introduced the first event family in the protocol whose semantic completeness requires two signed assertions of the same type, one per party. The protocol could have legitimately gone there — federation is the only relationship between Nodes rather than between Identities, and Nodes are infrastructure peers with no natural asymmetry — but the precedent cost and the operational complexity (intermediate "half-federated" states, replay edge cases where one side's event is missing, reciprocal-mint timing concerns) tipped the lock the other way. D-075 makes the conservative choice explicit so future revisits do not silently drift toward the rejected alternative.

**Reason 4 — Distinct from D-070's "two events" principle.** D-070 ("two events of equal importance, opposite direction") is about the **acceptance vs. rejection signal pair** on `TransportMessage`. It applies at the transport layer to outcome-of-action signals. D-075 is about the **event-model semantics** at the protocol-event layer — what `state.federation_add` (and analogous future events) ARE as DAG events. The two principles operate at different layers and address different questions; D-075 carries an explicit cross-reference to D-070 in §5.5 of the bidirectional-fix design task file to prevent confusion.

### What this principle commits the protocol to

- **For every existing relationship-shaped event:** the event records one party's act; the receiver applies it with vantage-awareness if the relationship has a sender-vs-other-party asymmetry. The current registry already follows this; D-075 codifies the discipline.
- **For every future relationship-shaped event:** same pattern. A future event family that genuinely cannot be modelled this way (a relationship that truly requires two co-equal signed assertions to be semantically complete) requires an explicit future decision to depart from D-075. The departure would itself be a named protocol-design decision, not a silent precedent break.
- **Applier discipline:** the applier may legitimately depend on local Node context (e.g., `my_node_id`) when deriving state from events. This is not a violation of "apply is a pure function of the event" — it is "apply is a pure function of (event, applier context)", where applier context names well-defined local state that does not depend on history. Future appliers MAY follow this pattern when the event has a sender-vs-other-party asymmetry.

### Worked instances at promotion

- **`SpaceState::apply_federation_add` with `my_node_id` parameter** — the originating instance, shipped in implementation Commit 2 per `tasks/FEDERATION_BIDIRECTIONAL_NODES_IMPL.md` §4. The verbatim code-comment block at the applier site documents the design lock at the use site so future readers can trace from applier to principle.
- **`SpaceState::apply_event` plumbing** — the parameter threaded through the dispatch site, ignored by all other arms. This is the precedent-establishing shape for how future relationship-shaped events with sender-vs-other-party asymmetry would plumb their context.
- **The two-vantages-mirror unit test** (`apply_federation_add_two_vantages_mirror`) — the regression lock at unit-test level. The Phase 9 Scenario 1 resurrection in implementation Commit 3 is the regression lock at integration-test level.

### Out of scope for this decision

- **Whether a specific future event SHOULD use the one-event-per-assertion pattern.** D-075 is a default and a principle, not a prescription that overrides per-case design. A future event-shape design walkthrough may legitimately conclude that two-events-per-relationship is the right shape for that case; D-075 just requires that conclusion to be explicit and named rather than silent.
- **Applier-context parameter naming or threading shape.** Whether the local Node ID is plumbed as `&str`, `&NodeXgid`, a context struct, or a builder pattern is a use-site question subject to D-073 (field-name-vs-type) discipline at the parameter level. D-075 only specifies that vantage-aware appliers are the legitimate pattern; the parameter shape is below the principle.
- **`SpaceState` persistence model.** A.1 (re-derive on load) was locked specifically because `SpaceState` is currently non-persisted (verified against `xgen-core/src/node/runtime.rs` at design close); the fix lands and self-heals on next Node start. If a future milestone introduces `SpaceState` snapshotting for fast-start, the persistence model would need to be re-walked — D-075's principle would still hold (events are truth, derived state is cache) but the rebuild trigger might shift from "always on load" to "on load when snapshot is stale."
- **Other protocol events with sender-vs-other-party asymmetry that may benefit from explicit vantage-aware appliers.** D-075 sets the discipline; per-applier audits to identify candidate sites are future work. Not every event with a sender field needs a vantage-aware applier — only those where the receiver-side interpretation differs from the asserter-side interpretation in a load-bearing way. `state.federation_add` is currently the only confirmed instance.

### Relationship to other decisions

| Decision | Relationship |
|---|---|
| D-070 | Coordinated-but-distinct "two events" principle at a different layer. D-070 governs the **transport-layer signal pair** (acceptance vs. rejection on `TransportMessage` with envelope-level `event_id` correlation). D-075 governs the **event-model layer** (what a `state.federation_add` IS as a DAG event). The two principles do not overlap and do not conflict: D-070's "two events" is about outcome-of-action signals at the transport layer; D-075 explicitly rejected an analogous two-events framing at the event-model layer (Reading (ii) of the design walkthrough). Future readers seeing "two events" in either decision should consult the layer named in the decision title to disambiguate. |
| D-065 | Sibling principle at the protocol-behaviour layer (honest behaviour over polite behaviour). D-065 requires the protocol to be honest about runtime state; D-075 requires the event-model to be honest about what each event IS (one party's act, not a relationship object). Both decisions take implicit framings out of the protocol's design surface and replace them with explicit named principles. The shared theme: don't let convention substitute for explicit specification. |
| D-069 | The canonical-document rule applies: D-075's authoritative home is DECISIONS.md; the originating audit doc, design task file, and implementation runbook each forward-reference D-075 at the appropriate place; the canonical design doc's §6.4.2 summary names the principle and points here for the full reasoning. Four surfaces, one authority, explicit forwards. |
| D-071 | Sibling project-management principle (audit-precedes-dependent-design). D-071 produced the audit doc that surfaced the question D-075 answers. The pattern: audit (D-071) → design walkthrough (D-069) → locked principle (D-075) → implementation. Each step produces a canonical artefact; the principle decision is the load-bearing output of the chain. |
| D-072 / D-073 | XGID type discipline + field-name-vs-type discipline operate at the data-model layer (what types fields carry, what names fields use); D-075 operates at the event-semantic layer (what an event IS as a protocol-DAG primitive). The disciplines compose: a future relationship-shaped event would carry XGID-flavoured fields (D-072), with role-bearing names (D-073), and would record one party's act per D-075. The three decisions form a stack — types, names, semantics — that together specify the structural shape of new protocol events. |
| D-074 | The same-commit + JOURNAL-mandatory discipline (milestone-close commits include JOURNAL.md) applies to the implementation runbook's Commit 4 that ships this decision into production code. D-075 is not the canonical home for D-074's discipline; the two decisions just happen to coordinate in this milestone-close commit's shape. |
| Bidirectional `federation_nodes` audit + design task file + implementation runbook | The three-document chain that produced this decision. The audit doc grounded the gap in code at file:line granularity; the design task file walked the option space and locked the chosen reading; the implementation runbook ships the fix that instantiates the principle. D-075's authority is above the chain — the chain is the historical record of how D-075 came to be locked, not the canonical home for the principle itself. |

---

## D-089 — Federation event propagation is pairwise; no transitive relay (received-via-federation events are terminal)

**Date**: 2026-06-05  
**Layer**: Protocol federation propagation (ch3 §3.4.8; the federation authority model)  
**Spec reference**: ch3 §3.4.8; `docs/xgen_federation_propagation_design.md` §8 (F-5, JOE-LOCKED May 2026); M8.5-A (`tasks/M8_5_A_F5_COHERENCE.md`).  

### Decision

Federation propagates DAG Events **pairwise**. A Node pushes each Event it accepts into a Space's log directly to every Node with which it holds an ACTIVE federation relationship for that Space — one hop from origin. A Node MUST NOT re-forward onward an Event it received *via* federation: received-via-federation Events are terminal (delivered to local clients via fan-out and applied to Space state, but not re-pushed to other peers). Convergence among a Space's participating Nodes therefore requires direct relationships — in the general case a full mesh. This is distinct from Announcement Propagation (ch3 §3.5.5), where Node-discovery announcements MAY be relayed transitively. A future revision MAY add opt-in per-relationship transitive relay (default off) — forward-compatible, not v1.

### Why

This promotes the F-5 decision (federation propagation design §8.4, Option 1, JOE-LOCKED May 2026) from a design-doc record to a first-class cross-cutting invariant, and synchronizes it across all canonical surfaces: ch3 §3.4.8 (spec), this entry (DECISIONS), and the design doc §8 (rationale + the Option-3 v2 path). The decision was already implemented (`federation_session.rs:268` origin guard) and tested (`f5_anti_transitivity_*`, `phase9_three_node_anti_transitivity`); M8.5-A found that ch3 had never absorbed it — a D-069 canonical-document gap. Transitive relay is rejected for v1 because it would extend an Event's authority chain through Nodes the receiver has no direct relationship with, weakening the per-Space / per-peer relationship check (ch3 §3.4.5) — trust in a federation relationship is not transitive. Pairwise is also the easiest position to relax later (opt-in v2) without a protocol break.

### Relationship to other decisions

| Decision | Relationship |
|---|---|
| D-069 | Canonical-document discipline — D-089's authoritative home is DECISIONS.md + ch3 §3.4.8; the federation propagation design doc §8 is the originating record. M8.5-A closed the gap where the decision lived only in the design doc. |
| D-065 | Honest behaviour — the M8 finding / audit M85-A6 mis-stated this as an open fork plus a phantom ch3 §3.2 contradiction; M8.5-A corrected the record rather than silently designing against it. |

---

## D-091 — A DM's federation set is exactly its parties' home nodes (named protocol/privacy invariant)

**Date**: 2026-06-09  
**Layer**: Protocol federation propagation — direct-message Spaces (ch3 §3.16.1 DM privacy; the federation authority model)  
**Spec reference**: `tasks/MP_F1B_DM_FEDERATION_DESIGN.md` (Option-2 lock J-332 + the v1.1 §9 Design-Z amendment); MP-F1b shipped `9b4ab8b` (J-333); refines D-075.  

### Decision

A direct-message Space's `federation_nodes` is exactly the home nodes of its **parties** — its current **members ∪ pending invitees** — derived at membership-apply by a NodeRuntime helper (`repopulate_dm_federation_nodes`) and re-derived when an involved identity's record replicates. No other node ever receives DM content. For a DM `pending_invites` holds exactly the one seeded counterparty (`from_dm_space_create`; further invites rejected by `apply_invite`), so the set is exactly the two parties. A party whose `home_node` is not yet resolvable on this node is **omitted** (no crash, no guess, no fabricated home) — the omission is the harness/production boundary, closed for a late-arriving record by the identity-replicate re-derive + the existing F-3 drain. `apply_federation_add` stays rejected for DMs (`DmFederationNotAllowed`) — population is never via that path. F-3 (the inbound federation-relationship gate) stays the guard against a third-party federated join: a non-party's node is never in the set, so its pushed join is held — **no F-3 skip, no hole** (proven by `mp_f1b_third_party_dm_join_via_federation_blocked_by_f3`).

### Why

MP-F1b's (iii) closes cross-node DM convergence without weakening DM privacy: a DM federates **only** to its own parties, both directions, and never to a third party. Deriving the set from parties (not members-only) is what lets the counterparty's home be present **from create** — so the bootstrap `membership.join` passes F-3 with no admission-gate loosening, and the creator's pre-join message pushes via the existing path. This refines **D-075** (a relationship-shaped field is a vantage-aware derived projection) for the DM case: the projection's source is parties × registry, recomputed at every membership-apply (so a leave shrinks the set naturally). The premise that the populate alone sufficed ("no new send code", design §3.2) was **falsified** by the live two-node witness (J-333); the receiving-side F-3 bootstrap + the replication race required the identity-replicate re-derive/drain hook — which reuses the convergence-proven `drain_pending_by_federation_relationship` verbatim, so D-076 is discharged by inheritance. **Production convergence to a not-yet-discoverable counterparty is out of scope and deferred** behind the routed "production identity→home-node discovery" arc (F1B-D5): this invariant governs how the set is derived from *known* parties, not how a stranger's identity enters the registry. MP-C-07 cross-node is therefore witnessed **harness-green-with-boundary** (G-6 pre-seeds resolution); no production witness is claimed (test-integrity, D-065).

### Relationship to other decisions

| Decision | Relationship |
|---|---|
| D-075 | D-091 refines D-075 for the DM case — `federation_nodes` is the vantage-aware derived projection; D-091 fixes its source (parties × registry) and derivation point (membership-apply + identity-replicate) for DMs. |
| D-076 | The new identity-replicate trigger drains via `drain_pending_by_federation_relationship` verbatim (the same hook `state.federation_add` fires; `peer_node_id=None`) — one more trigger, no new ordering decision. Discharged by inheritance. |
| D-077 | `DmFederationNotAllowed` stays intact (no third-party `federation_add`); F-3 stays intact (no skip); regular-Space federation untouched (DM-only helper). No backward-coherence regression. |
| D-065 / D-069 | Honest-by-construction boundary (omit unresolvable; no production witness claimed); promoted at close per D-069 once the parties-rule held across the arc (Design Z, witness green). |
| F1B-D5 (routed) | Production identity→home-node discovery is a separate arc; D-091 governs derivation from known parties, not stranger discovery. |

---

## D-094 — Canonical-record archiving: relocate superseded content to a frozen ARCHIVED sibling with a forward pointer; never rewrite history

**Date:** 2026-06-17 · **Layer:** project-management discipline · **Spec ref:** none (process) · **Lineage:** D-074 (atomic canonical records) + the append-only JOURNAL and no-retroactive-rewrite conventions.

**Decision.** The four canonical operational records — `CLAUDE.md`, `JOURNAL.md`, `docs/ROADMAP.md`, `DECISIONS.md` — keep a small *live working head* and relocate superseded/historical content to a frozen `*_HISTORY` / `archive` sibling marked `Status: ARCHIVED`, leaving a forward pointer from the live file. Archiving is a **move, never a rewrite**: relocated text is byte-preserved; the live file's Rule-0 read cost stays bounded as the project grows.

**Constraints.** (1) No retroactive rewrite — archived content is verbatim; only its container's Status reads ARCHIVED. (2) Forward pointer mandatory — the live file names its archive sibling so a session can reach history on demand. (3) D-074 atomicity holds — an archiving move and its canonical-record updates travel in one commit. (4) The live head is the only PLAY block a fresh session must read on open; deeper history is opt-in via the pointer.

**First exercised:** the documentation-optimization phase (J-391) — CLAUDE.md's ~2,200-line superseded PLAY stack lifted into `CLAUDE_HISTORY.md` (ARCHIVED). Subsequent doc-opt sub-arcs apply the same pattern to `tasks/` (DO-2); it was applied to `JOURNAL.md` at DO-5 (J-396) — J-375 and older (358 entries) relocated to `JOURNAL_ARCHIVE.md` (ARCHIVED), live window J-395 … J-376.

---

## D-099 — Text-processor architecture: two engines × four rule-kinds; build kind 1, codify all four

**Date:** 2026-06-30 · **Layer:** UI reference library (`$common` substrate) · **Spec ref:** `tasks/M_RP4_0_PROCESSOR_ENGINE.md` (v1.1, design-locked); N-056; consumers reserved since N-029/N-032/N-038/N-040. · **Lineage:** discharges the D-065 reserved-seam (no empty machinery); reuses the N-024 DEV-hook idiom.

**Decision.** The UI text-processor is **not one engine**. It is **two engines × four rule-kinds**, orthogonal:

- **Two engines (by side):** an **edit-side** engine (a forwarded Svelte 5 *attachment* on an input/textarea, with the caret/re-entrancy plumbing) and a **render-side** engine (the deferred `use:render`, with the allowlist + sanitiser). The markup/sanitiser sink is **not** built this arc.
- **Four rule-kinds (the canonical taxonomy):** **1 transformer** `string→string` (live on `input`); **2 converter** `string↔T` (`toString`/`fromString`); **3 filter/guard** `T→T` (idempotent, on `change`/blur); **4 renderer** `string→safeHTML`. The §0.1 table in the runbook (mirrored in N-056) is the canonical reference.
- **kind ⟂ engine; kind 2 is the bridge** — its `fromString`/mask runs edit-side, its `toString`/format render-side. Never read as "four engines."

**P-1a — the edit seam is a forwarded attachment, not a `use:` action.** A `use:` action attaches only to elements in the component that writes them, so a consumer cannot forward one onto an atomic's internal element. The engine therefore ships as an attachment (`processor(rules)` → `createAttachmentKey()`-keyed prop); the atomic spreads `{...rest}` onto its root and carries **no** processing logic. The consumer lands it via `<Textarea {...processor(rules)} />`.

**P-2 — pure core split from the wrapper.** `transform.ts` (`string + rules → string`, DOM-free, framework-free — the `logic.ts` posture) is the kind-1 core; the render-side engine reuses the *idea*, not this file.

**P-3 — two provenance tiers.** **Tier-1** (trusted `common` code configs): full power. **Tier-2** (user/settings data): **serializable literal `{find, replace}` pairs only** — caps (count, length) + a **convergence lint** (reject a pair whose `replace` re-contains its `find`; the engine re-runs the whole value each keystroke, so it would loop). Untrusted **regex is rejected** (not representable as a literal pair); a regex rule-kind + its ReDoS guard are reserved behind an explicit advanced opt-in. Tier-2 rules persist as a section of the app's existing global settings file (reserved) — **no bespoke rules file**; the engine stays source-agnostic.

**P-4 — caret-preserving value sink.** On `input`: recompute; if changed, write `node.value`, restore the caret to the transformed-prefix length (= old caret + net length-delta of replacements before it), then dispatch a re-entrancy-guarded synthetic `input` so `bind:value` syncs.

**Scope (D-065 honest): build kind 1, codify all four.** Kind 1 (transformer) + the `textarea` host are **built** (M-RP4.0). Kinds 2/3/4 are **records-only** — declared in this taxonomy with named reserved consumers (kind 2 → number/date/phone field, needs a decoupled text field, `toString` may delegate to `Intl`; kind 3 → `number` min/max clamp, M-RP4.1; kind 4 → `paragraph` inline marks, the `use:render` arc) — **no runtime, no stubs**. **Forward-clean naming:** the kind-1 type is `TransformRule`; the future union `ProcessorRule = TransformRule | ConvertRule | ClampRule | RenderRule` is documented here but **not declared in code** until those kinds land. `TransformRule.reversible` is declared, not implemented.

**AMENDMENT (M-RP4.1, 2026-07-04).** Kind 3 (filter/guard) is now **built** — `ClampRule {min?,max?}` + `applyClamp` (pure, total, idempotent, `number|null` pass-through) in `transform.ts`, plus the change-triggered `clamp.ts` attachment (sibling of the input-shaped `processor.ts`); first consumer `number` (clamp-host via `{...rest}`). **Two of four kinds built** (1 transformer M-RP4.0, 3 filter/guard M-RP4.1); the `ProcessorRule` union stays codified-not-declared. Next: kind 2 (converter/bridge, `Intl`), kind 4 (`use:render`, deferred). See N-069.

**AMENDMENT (M-RP4.5, 2026-07-04).** Kind 2 (converter/bridge, `string ↔ T`) is now **built** — and it is the one kind that is a **component, not an attachment**: two representations of different type coexist (a formatted display string + a typed bound value), which a single forwarded `bind:value` cannot carry. Pure DOM-free contract in `transform.ts` (`PARSE_FAILED` sentinel symbol + `Converter<T> {toString, fromString, toEditable?}` + first concrete `intlNumber(opts?, locale?)` over `Intl.NumberFormat`, with a `formatToParts`-derived parser since Intl has no parse); host = new di atomic `converter-field.svelte` (`<script generics="T">`, root `<input type=text>`, owns `value`/`text`/`invalid`; the component is kind 2's sole framework touch + the DEV `__XGEN_CONVERT__` hook). Timing: parse on `change`/`blur`, `focus` shows the raw `toEditable` form, nothing on `input` (decoupled → no caret-restore). Parse-failure = **reject-and-mark** (`[data-invalid]`, value unchanged); empty commit = no-op revert. **Provenance = Tier-1 only** — a converter is code-supplied LOGIC, not a user string, so no caps/lint (contrast kind 1). **Three of four kinds built** (1 transformer M-RP4.0, 3 filter/guard M-RP4.1, 2 converter M-RP4.5); the `ProcessorRule` union stays codified-not-declared. Next: kind 4 (`use:render`, deferred) → dd-components. See N-070.

**Why.** Founds three things: (a) the edit seam is a forwarded attachment — the first time the library forwards behaviour from a consumer onto an atomic's inner element without the atomic carrying logic (the resolution of "a consumer simply layers it on", which `use:` could not satisfy); (b) the durable mental model is *two engines, four kinds, kind 2 the bridge*; (c) provenance tiers gate safety — the literal-only Tier-2 subset is what makes runtime-editable rules safe (literals can't ReDoS; the convergence lint stops the one loop a literal can cause). It discharges the longest-standing reserved UI seam (N-029) exactly as D-065 intended: built when a consumer is in hand, codified so growth is bounded.

**Relationship to other decisions:** D-065 (built-when-consumed / no empty machinery — the build/codify line is D-065 applied); D-095/D-097 (`$common` substrate / sampler test-bed — the engine is `common` infra, no catalogue row; verified in the sampler); N-056 (the implementing note + the taxonomy table). Arc-local kinds-2/3/4 details stay in the task doc/N-056 until built (D-069).

**AMENDMENT (M-RP-PROCESSOR-WIRE — where the attachment may land, 2026-07-28). The claim that the plug points are structurally gated is WITHDRAWN.**

🔒 **Default OFF everywhere; plug in only where it matters** (Joe-locked J-560). The processor attachment is never present by default. **The consumer opts in per call site, and that is the entire mechanism.**

🔑 **THE DISCRIMINATOR IS THE CONTEXT OF THE CALL SITE, NOT THE COMPONENT TYPE.**

- **Composing** — prose written in flow, where the transformation is visible and correctable as it happens ⇒ **plug in**.
- **Configuring** — a value stored and reused, where the rewrite lands silently and nobody is watching ⇒ **do not**.
- **Byte-exact** — values that must round-trip unchanged ⇒ **never**; a correctness rule, not a preference. **Two sources, both binding:** byte-exact **because of what the value is** (XGIDs, tokens, passwords) and byte-exact **because of what it is for** (a setting whose meaning is an exact letter combination — the substitution rule text is the live instance).

⚠️ **WITHDRAWN — *"`password-field` and `textfield` do not forward `{...rest}`, so processing them is structurally impossible."*** That describes what five components happen to do today, not a property of the design. `{...rest}` forwarding is the **capability** by which the attachment lands; a component that does not forward simply cannot host it *yet*, and a text field that needed the processor would be made to forward, exactly as `textarea` does. ⇒ **COMPONENT TYPE IS NOT A GATE.**

🔑 **THE PROOF IS ON ONE COMPONENT.** `composer-panel.svelte:113` and `substitutions-editor.svelte:147` both import the same file — `$core/components/data-independent/textarea.svelte`. The first lands `{...processor(substitutions.rules)}`; the second must **never** — processing the textarea where the rules are authored is a feedback loop, and typing `:)` into the rule list rewrites the rule being written. **Same component, opposite treatment, decided entirely at the call site.**

⚠️ **AND THERE IS NO STRUCTURAL BACKSTOP.** The design **does** trust every future call site, because **the call site is the only place the context is known.** A value silently rewritten by a user's own rule is a real harm and nothing below the call site will catch it. ⇒ **EVERY NEW TEXT-INPUT CALL SITE IS A DECISION, NOT A DEFAULT** — which is why it is written here rather than left to the pattern.

📌 **Measured 2026-07-28:** the client+common tree still has exactly two text-input call sites, unchanged since J-560. `converter-field`, `number` and `textarea` forward `{...rest}`; `password-field` and `textfield` do not — **recorded as current state, not as a rule.**

---

## D-100 — Substitution pairs: the ` | `/first-space grammar, a single-string TOML home, and a source-agnostic rule store

**Date:** 2026-07-01 · **Layer:** UI reference library (`$common` substrate) + `xgen-client` config · **Spec ref:** `tasks/M_RP4_2_SUBSTITUTIONS.md` (v0.1); N-057; executes decision 9 of the M-RP4.0 runbook. · **Lineage:** extends D-099 (the kind-1 transformer engine) with its first user-owned rule source; the engine itself is unchanged.

**Decision.** The kind-1 transformer's rules come from **one user-owned list of `{find, replace}` pairs**, not named code presets. Three parts:

- **Grammar (locked, literal, regex-free).** The whole list is one string; pairs separated by the literal **` | `** (space-pipe-space); within a pair, split on the **first space** → `find` = before, `replace` = the rest. `find` = any run with no whitespace; `replace` = any string (multi-char, emoji, internal spaces, a lone `|`). The only forbidden token substring is ` | ` itself; blank pairs are skipped. The simplest grammar that survives the actual tokens (`-->`, `<--`, `:)`, `|`) without regex — the literal engine stays literal.
- **TOML home = a single string.** A `[substitutions]` section in `xgen-client_config.toml` with ONE string field `rules`, so it mirrors the future one-textarea editor 1:1 (M-RP4.3). NOT a TOML array — a single string, parsed by the UI. The Rust side (`SubstitutionsSection` / `load_substitutions_section`, the `[sync]` precedent) carries the raw string **verbatim**; all grammar parsing happens in the Svelte store (the engine stays source-agnostic).
- **Source-agnostic store (`$common`).** `parseRules(text) → TransformConfig` (pure, next to `applyRules`) + a reactive `substitutions` store whose `setRules(text)` parses, runs `assertSafeRules({trusted:false})`, and fails safe (empty + DEV warn) on rejection. The store decouples *where rules come from* (client TOML via a Tauri command / sampler literal / future editor) from *who consumes them* (every processor-host). This makes D-099 P-3 (source-agnostic engine) concrete.

**Provenance = Tier-2 for config data.** Config-file rules are user data → the caps + convergence lint (D-099 P-3) actually protect the user from a self-authored looping pair (e.g. `a aa`). `configs.ts` (the `arrowMorph`/`emojiMorph` presets) is **retired as the live source and deleted** — it was sample data, never architecture (D-099/N-056).

**First-run seed (J-438, Joe-locked).** A fresh client config ships a six-pair starter pack (`--> → | <-- ← | :) 🙂 | <3 ❤️ | :( 🙁 | -- ‒`), seeded **once at config-birth** (`cmd_init`), never resurrected after the user clears it. The seed lives hand-synced in two places (Rust `DEFAULT_SUBSTITUTIONS_SEED` + the sampler TS literal) — a documented seam closed by **M-RP4.4** (the sampler real config-load arc).

**AMENDMENT (M-RP4.3, 2026-07-04).** The seed's `-->`/`<--` pairs change to `->`/`<-`. During live typing the shorter `--` rule (`-- ‒`) **shadows** the longer `-->`: the engine rescans the whole field every keystroke, so the `--` prefix morphs to `‒` the instant it is typed — `-->` never completes (`‒>`, never `→`). `->`/`<-` carry no `--` substring, so `--` cannot shadow them. **New seed:** `-> → | <- ← | :) 🙂 | <3 ❤️ | :( 🙁 | -- ‒`. General rule (sibling to the convergence lint): *a rule whose `find` is a substring of a longer rule's `find` shadows the longer one during live keystroke-rescan — prefer collision-free tokens.* Applies to both hand-synced Rust seed consts (client `app.rs` + sampler host `main.rs`) and the sampler placeholder text. See N-068.

**⚠️ SECOND AMENDMENT (M-RP-PROCESSOR-SEED, 2026-07-21) — AND IT OPENS BY RECORDING THAT THE FIRST ONE WAS NEVER APPLIED.** The amendment above named **three sites**: the client `app.rs` const, the sampler host `main.rs` const, and the sampler placeholder text. `git log -S` proves **only the placeholder was changed** (commit `7872a77`); **neither Rust const has moved since `7ea70fa`**. So from 2026-07-04 this decision said one thing and both binaries did another, while the sampler's own placeholder instructed users to type tokens the shipped seed did not contain. On 2026-07-20 the defect was rediscovered by Joe typing, Chat proposed dropping `--`, and Joe re-derived the shortened arrows — **byte-identical to the seed already locked here sixteen days earlier** — which J-563 recorded as a novel improvement. 🔑 ***A decision that is not applied is indistinguishable from one that was never taken, and the record then credits the rediscovery.*** *This is the J-561 class inverted: not a remembered gate asserted as current, but a written, locked decision nobody read.* J-563 is corrected in place by pointer; no entry is rewritten.

**① THE SEED IS ALSO THE GRAMMAR'S ONLY WORKED EXAMPLE — A STATED PROPERTY, NOT A CONVENIENCE (Joe, 2026-07-21).** Joe's words: the pairs must be there from the first run *"also because they are examples how to define the next ones."* Nothing else in the UI documents the ` | ` separator or the first-space split. ⇒ **The seed may never be emptied, and its CONTENT must demonstrate the grammar's range rather than merely be useful** — the locked set does: a multi-char `find` (`->`), an emoji `replace` (`🙂`), a symbol `replace` (`‒`). *Recorded because a hardcoded literal whose second job is undocumented looks exactly like dead data to the next person who reads it.* ⚠️ **This does not weaken "no hardcoded pairs", it locates it:** verified by grep — `configs.ts` (the retired `arrowMorph`/`emojiMorph` presets) is **not on disk**, there are **zero** hardcoded rule arrays in live UI code, and every path into the store is `setRules(<string>)` from a config file or the user's own typing. **No pair is ever a live rule from code.** The seed is birth content for a file the user then owns — *owned defaults, not locked presets*, as this decision already says.

**② THE MIGRATION BOUND — THE FIRST TIME THIS PROJECT REWRITES A USER'S CONFIG ON UPGRADE. WRITTEN BEFORE THE CODE.** Inside `clean_slate_config`'s existing `Option` arm, a captured `rules` value that is **byte-identical to a listed historical seed** causes the re-inject to be **SKIPPED**, leaving the freshly written new seed standing. **The migration is a SKIP, not a rewrite** — `write_fresh_config` has already written the new value, so there is no upgrade path, no string surgery and no parser. **BOUND (normative): byte-identity only** — never a substring match, never a prefix/suffix test, never a normalisation, trim, parse-and-compare or similarity heuristic, and never a rewrite of a value differing by one character · **an explicit list of NAMED CONSTANTS**, with the old seed kept in the source **because the constant IS the evidence** — documented as historical, never used to seed anything · **a curated list is NEVER migrated**: anything not byte-identical rides across verbatim forever, and for those users the answer is the diagnostic (④), never a rewrite · **the list never grows silently** — appending a value is a decision recorded here on its own line.

**③ THE LIST IS S2 ONLY (Joe-locked). 🔑 S1 IS NOT DEFECTIVE, AND MIGRATING IT WOULD INSTALL THE BUG ON SOMEONE WHO DOES NOT HAVE IT.** Two historical seeds exist: **S1** `--> → | <-- ← | :) 🙂 | <3 ❤️ | :( 🙁` (`2cf494f`, five pairs, **no `--` rule**) and **S2** `--> → | <-- ← | :) 🙂 | <3 ❤️ | :( 🙁 | -- ‒` (`7ea70fa`, still shipping). Under S1 nothing is a proper prefix of `-->`, so typing `-->` correctly yields `→` — **the defect arrived with S2**, when `--` was added beside a rule it shadows. Migrating an S1 install would make `--` a rule and turn its working `-->` into `‒>`: ***the exact defect this milestone removes, newly installed, silently, in a file the user never touched.*** An S1 config therefore falls through to the user-authored arm and is **preserved verbatim**. *Cost was not the decider — S1 would have cost one const and one test. The bound's principle is that untouched defaults get migrated, but its PURPOSE is to repair a defect we shipped, and consistency is not a reason to change behaviour under someone.* (Chat recommended S1+S2 and **reversed it**: the first answer was right about cost and was answering the wrong question.)

**④ THE PREFIX-REACHABILITY CHECK IS A NON-THROWING DIAGNOSTIC AND MUST NEVER JOIN `assertSafeRules`.** The general rule is already stated above (a shorter `find` shadows a longer one during live keystroke-rescan); what is locked here is **where it may live**. `assertSafeRules` **throws**, and `store.svelte.ts:setRules` **fails safe to empty** on rejection — so folding reachability into the validator would take a user with five working rules and one broken one and leave them with **zero substitutions**, explained only by a `console.warn` stripped from release builds. 🔑 ***A check that blanks the whole rule set to report one unreachable pair is strictly worse than the bug it diagnoses.*** It is computed separately, **never gates Apply**, and is surfaced beside the offending rule in the settings pane — the V-C3b shape: *make it visible and let the user fix their own data.* A rule is unreachable when another rule's `find` is a **proper prefix** of its own; list order is irrelevant, **typing order decides**.

**⑤ AND A CONFLATION THE BOUND EXPOSED, REPAIRED IN THE SAME MILESTONE.** `SubstitutionsSection` derives `Default` and `rules` carries `#[serde(default)]`, so a config with **no `[substitutions]` section at all** parses to `Some("")` — **indistinguishable from *the user cleared their pairs***. `clean_slate_config` then re-injects the empty string over the freshly seeded pack, so ***a pre-M-RP4.2 config launching on a current build is blanked permanently and never receives the starter pack — including the grammar example ① says it exists to provide.*** Real, not hypothetical: **fourteen** configs on Joe's machine carry no `[substitutions]` section — `instances\lp-cli\xgen-client_config.toml` (234 bytes, 2026-06-16) plus thirteen fixtures under `bin/instances/m3-*`, `m4-*`, `bin/test_01*` and `test_runs/multiparty_s1_run*`. *(Count corrected at J-567: the implementer flagged the `lp-cli` example as non-existent, but the flag's own evidence was wrong — the file is there, at an absolute path under `%LOCALAPPDATA%`, measured twice. Her conclusion was right and understated. Example kept, count fixed.)* ⇒ `try_load_substitutions_section` must report **section absent** distinctly from **rules empty**, and `clean_slate_config` treats *absent* like a first run: skip the re-inject, let the pack stand. The reader `load_substitutions_section` is **unchanged** — collapsing both is correct for a reader and wrong exactly once, in the wipe. *J-562's §3.1 conflation one level up: two states that are the same value at the point of decision.*

→ `tasks/M_RP_PROCESSOR_SEED.md` v1.0 · JOURNAL J-566.

**Why a new decision, not a D-099 amendment.** The grammar is arc-local detail (it could live under D-099), but the **single-string TOML home** and the **source-agnostic store as the standing rule-delivery mechanism** are durable, cross-cutting choices every future config-backed component inherits — they earn their own decision. D-099 stays the *engine/taxonomy*; D-100 is the *first user-owned rule source and its delivery contract*.

**Relationship to other decisions:** D-099 (the engine this feeds; unchanged); D-097/D-098 (the sampler seeds a literal because it is a minimal host with no client config — the source duality); D-065 (the richer Tier-2 per-pair-partition UX deferred to M-RP4.3, built when the editor needs it); N-057 (the implementing note).

---

## D-101 — Clean-slate-on-start config discipline (phase-scoped) + the sampler real config-load path

**Date:** 2026-07-01 · **Layer:** binaries (`xgen-client`, `xgen-node`, `xgen-sampler` host) + UI reference library · **Spec ref:** `tasks/M_RP4_4_SAMPLER_CONFIG_LOAD.md`; N-057 (the two-hand-synced-seeds seam this closes). · **Lineage:** refines D-097/D-098 (the minimal host gains a subset-config read/write); **suspends** J-438 seed-once for this phase.

**Decision — clean-slate-on-start (phase-scoped).** During the current UI-development phase, **every binary** (`xgen-client`, `xgen-node`, and the `xgen-sampler` host) **wipes any config file it finds at launch, before reading it, then regenerates from seed**. Config is treated as **ephemeral and deprecatable** while the settings logic is still in development. Crash-safe + self-healing: no binary inherits a stale or another binary's file, and a corrupt config never wedges a launch.

**Interaction with J-438 seed-once (load-bearing — must be findable).** J-438 built `cmd_init` to seed the client's substitution starter pack **once at config-birth, never resurrected after the user clears pairs**. Clean-slate-on-start **suspends** that guarantee for the duration of this phase: because the config is deleted + regenerated from seed every launch, cleared pairs **do** reappear. This is intended now — there is no persistent user-owned settings surface yet, so nothing durable is lost. J-438 seed-once resumes at the exit condition. This interaction is stated at the delete site **in code** AND **here** — a future session finding a vanishing/resurrecting config must reach both the *why* and the *until-when*.

**Exit condition (written retirement).** Clean-slate-on-start is removed when the real client/node UIs are rewritten and settings become **persistent** (a user's edits survive relaunch). At that point: delete-on-start is deleted from all three binaries, and J-438 seed-once becomes real client behaviour again.

**🔒 AMENDMENT — SCOPE DISCRIMINATOR (Joe-locked 2026-07-20, J-562, M-RP-PROCESSOR-WIRE Leg C, code at `1932474`).** **D-101 wipes CONFIG. It does not wipe USER-OWNED CONTENT.** Those were never the same thing, and the project had already drawn this line in writing — for the UI-state store, at `xgen-client/src/app.rs:290–291`: *"NOT config: the UI-state store is the project's first deliberately persistent user-facing state, so it is NOT touched by D-101 clean-slate-on-start (which wipes `xgen-client_config.toml` only)."* **The discriminator there was never "this section is special"; it was PERSISTENT USER-FACING STATE vs. CONFIG.** `ClientConfig` holds five sections — `client`, `paths`, `ai`, `sync`, `substitutions` — and **`substitutions` is the only one whose content a human authors** (`app.rs:69`: *"user-owned text-substitution pairs"*). ⇒ **This is NOT an exemption from D-101. It is the correction of a mis-filing: user-owned content was filed into the config file, and D-101 wipes config files.** *The wording is load-bearing — it keeps D-101 crisp (config wiped whole and undiminished, all five sections regenerated from seed) and stops this becoming a precedent for arbitrary per-section exemptions later.* **Mechanically:** `clean_slate_config` captures `[substitutions].rules` before the wipe and re-injects it after regeneration; only the user's rule **text** rides across, so D-101's rationale survives intact — the regenerated file still has whatever new **shape** we want. **No new D-number was minted;** a standalone filing rule (*"where should new user data live in the first place"*) remains available later and promoting this reverses nothing. **Option E — moving `[substitutions]` out to its own store beside `xgen-client_uistate.json` — was NAMED AND NOT TAKEN**: identical user-visible outcome at meaningfully higher cost, and still available as the tidier home without reversing this.

**🔒 EXIT CONDITION — FIRST INSTALMENT PAID (same lock).** The retirement above is **partial, not begun-and-abandoned**: **J-438 seed-once is REAL CLIENT BEHAVIOUR AGAIN for `[substitutions]` and for that section only.** Pairs the user clears **stay cleared across a relaunch**. The other four sections remain ephemeral by design, and delete-on-start is **still present in all three binaries** — so the exit condition above stands, minus this one section. ⚠️ **`None` (could not read the old config) and `Some("")` (the user cleared their pairs) are NOT the same state** and must never be collapsed: the read-path helper `load_substitutions_section` collapses both to empty, which is right for a reader and **wrong exactly once**, in `clean_slate_config` — hence the fallible sibling `try_load_substitutions_section`. Blanking on an unreadable config would **silently destroy the freshly seeded starter pack on top of whatever the corruption already cost the user.** *This distinction was gotten WRONG in the Leg C runbook's §3.1 and caught only by the implementer reading it against the code (J-562, Rule-6 flag 1) — it is recorded here because it is the kind of thing a future reader will otherwise re-derive by breaking it.*

**Verified live, not asserted (J-562, Rule-5 re-drive, throwaway data root):** rules survive a relaunch verbatim while `[logging]` reverts from `trace` to `debug` — *the control proving the wipe genuinely ran* · a cleared rules string **stays cleared** · an invalid rules string **survives on disk** while the store fails safe to zero rules, keeping the raw text in `source` for the settings pane to surface. ⚠️ **That last one is the hazard this amendment CREATES and it is not yet mitigated:** under clean-slate a bad rules file could not persist; now it can, and a bad file **silently disables every substitution**, explained only by a `console.warn` stripped from release builds. **The mitigation is M-RP-PROCESSOR-WIRE Leg A (V-C3b): the settings pane must show that raw text WITH its warning.** If it does not, the hazard is unmitigated and that is a Joe decision, not a defect to paper over.

**The sampler real config-load path (the positive half).** Config-backed components run in the sampler through the **real** `generate → file → load → command → setRules` chain, not a hand-synced frontend literal — so a component drops into the rewritten client/node UIs with **zero reprogramming**. Fidelity is at the **contract shape, not code reuse** (D-098: the sampler host can't depend on `xgen-client`): the host reimplements a minimal read/write/delete of its config + a `get_substitutions` command; the stable contract is the component interface (`substitutions.setRules(string)`). A **direct-inject literal** stays the documented fallback for components where a real file is impractical.

**Sampler config = subset snippets.** The sampler generates only the sections it needs (e.g. `[substitutions]`), not the whole client/node config — the needed slice of what the real `.exe`s generate.

**Refines D-097/D-098.** The sampler host gains a tiny fs+toml capability for its subset config; still "minimal" (no protocol deps). Closes the two-hand-synced-seeds seam N-057 flagged (the sampler stops carrying a separate frontend literal; it loads from a generated file like the client does). The **seed const** itself remains hand-synced across the client + sampler host until a shared-const crate is justified — explicitly **out of scope** here (a third copy, documented, not resolved).

**Relationship to other decisions:** D-097/D-098 (minimal host — refined, not broken); J-438 (seed-once — suspended this phase); N-057 (the seam this closes); D-100 (the substitution source this arc plumbs through the real path); D-065 (build-when-consumed — the sampler-load path is built now that a config-backed component exists to prove it).

---

## D-102 — The `widget` tier: a Level-2 UI-plugin above the di/dd × atomic/composite grid

**Date:** 2026-07-04 · **Layer:** UI reference library (`ui/common` substrate) · **Spec ref:** `ui/docs/xgen-widget-tier.md` (v1.0, canonical); N-059 (concept-lock, J-445); N-067 (this promotion). · **Lineage:** promotes the N-059 concept-lock to a checkable specification; sits above the Level-1 component model (D-095 tier split, D-096 atomic criterion, N-054 composite-registration model).

**Decision.** The `widget` is a new **Level-2** tier — a **UI plugin**: the pluggable, behaviour-carrying assembly unit that sits *above* the Level-1 di/dd × atomic/composite grid (Level 0 substrate → Level 1 components → **Level 2 widget**), not a rung wedged into the arity axis. The Level-1 grid stays entirely passive; the widget is where state-ownership, lifecycle, and host I/O live. Home = `ui/common`. Canonical definition lives in the spec doc; this entry names the decision and its load-bearing choices.

**Discriminator (passive composite vs active widget).** A widget owns state with a **transition-lifecycle that persists across renders** (draft→dirty→saving→saved, load→loaded→error) — progress through a task. A passive composite's state is a pure function of props plus **at most a single momentary view toggle** (`open`/`revealed`/`hovered`/`dragging`). Gloss: remove it — lose a *behaviour* (widget) or a *layout* (composite)? This settles the N-063 correction (one UI flag ≠ widget).

**Plugin inheritance + the one divergence.** A widget is conceptually the **same mechanism as a protocol/auth plugin** (contract-not-hardcoded, capability + Phase declaration, one aggregate getter, clean mount/unmount, swappable behind the interface). The single divergence: a plugin is **invocation-shaped** (call→return, one-shot, request-scoped); a widget's data connection is **binding-shaped** — a **reactive `$common` store binding** (standing subscription, mount-lifetime, read + optional write-back). *Widget = the plugin contract with the invocation channel replaced by a reactive store binding.*

**Constraint set (checkable, W-1…W-11 — full text in the spec).** Composes-down-only (logic from `core`/substrate, never a logic-bearing raw native tag) · owns state+lifecycle · I/O only via declared seams (Tauri `invoke` + `$common` stores) · one aggregate getter publishing observable **task-state** (`{dirty,valid,phase}`) but **never payload/secret** · clean mount/unmount (0-orphans, no cross-widget coupling) · skin L2 only + pure/effect-separable · scoped home + a Phase (A/B/C, N-028) · honest phase-limits (e.g. session-only write-back under D-101) · **representation** (an ordinary Svelte component + a Level-2 `envelope` marker + a `widgets/` home; connection v1 = static import, a widget registry + dynamic mount **reserved** until dd-components give it a first consumer) · **plugin contract** · **dd-socket** (a widget MAY expose typed dd-slots — each a `$common` store handle (read + write-back) + a named mount point, source-agnostic per N-057; the dd-component binds to the store, never to widget internals; defined ahead of any dd-component so one plugs in with zero widget rework).

**I/O seam.** Store-mediated by default (the N-057/N-058 substitutions precedent — the widget touches only `$common` stores, backed by `invoke` in the real shell and a literal in the sampler); callback/prop injection for a genuinely imperative one-shot action; a DEV hook (N-056 `__XGEN_PROC__` precedent) for a pure-compute core. No new mechanism invented.

**Verify home — two layers.** The widget's defining trait (host I/O + integration) is the sampler's declared blind spot (D-097). So verification splits: the **pure/presentational layer** (I/O stubbed) verifies in the sampler (a 5th **WIDGET** tab, mounted-not-`{#if}` per N-053); the **effect layer** (real config read/write, command round-trip, session-vs-persistent behaviour) verifies in the **real shell** (client/node, CDP 9222/9322). One milestone, two verify homes — a widget is not done until both are green.

**First widgets.** First **buildable** = `substitutions-editor` (M-RP4.3, in-app `[substitutions]` TOML editor + write-back; composes core-di only, no dd dependency; Phase-B, session-only write-back under D-101) — it dogfoods this spec. `temperature-indicator` is the first **conceived** widget but is **dd-blocked** (nothing to plug into until a dd-component exists; it will bind its `temperature` state through a W-11 dd-socket).

**Provisional status (D-065).** The spec ships **v1.0, first-instance-provisional** — the constraint set is drawn against the six closed di composites, not a built widget. M-RP4.3 may surface a constraint needing amendment (the `tag-select`→N-064 precedent). The spec firms once an instance proves it.

**Why a new decision.** The widget is the first **behaviour-carrying** UI tier and every downstream milestone (M-RP4.3/4.1, kind-2, kind-4, dd-components) cites it — a durable, cross-cutting architectural choice that earns its own decision + a citable spec doc (the federation-design-doc precedent, not a scattered D-entry).

**Relationship to other decisions:** D-095 (the `ui/{...}` tier split this extends with a Level-2 storey); D-096/N-054 (the Level-1 passive model the widget sits above); D-097/D-098 (the sampler blind spot that forces the two-layer verify); N-028 (the A/B/C Phase axis the one-tier decision maps I/O onto); N-057/N-058 (the source-agnostic store the I/O seam + dd-socket reuse); N-056 (the DEV-hook precedent); D-101 (the session-only write-back the first widget honestly surfaces); D-065 (build-when-consumed + first-instance-provisional); N-059 (concept-lock) → this.

---

## D-103 — Region / dock model: every UI region is a widget (`system` | `custom`); one serializable layout descriptor for both renderers

**Decision.** The main client UI panel is a **layout of dockable regions**, and **every region is a widget** — there is no separate "region" concept. Widgets carry a `kind`: **`system`** (the built-in surfaces R1–R8: pre-installed, non-removable, but individually configurable + redockable like any widget) or **`custom`** (installable/removable; MAY also contribute a region). This extends D-102: a widget already plugs into a `$common` store (its *data* seam, W-11); it now MAY also contribute a dockable *surface* (its *layout* seam, W-12). The di/dd × atomic/composite grid stays the **content** tier; **widgets are the dockable surfaces that host content.**

**The shared contract.** A single **serializable layout descriptor** (a tree of `leaf`/`split`/`tabs` nodes referencing widgets by id) is read by **both** renderers: a lean **config-grid (A)** now (M-RP6.1+; `split`-only subset, edit-to-rearrange) and an owned **Maya-style dock engine (B)** later (M-RP7; full tree + drag-drop hover-to-plug-in + splitters + save/restore). Because both read one descriptor, the dock engine is a **renderer upgrade, not a region rewrite**. A **selection bus** (`{regionId, entity}`) is the shell primitive R8/R1/R2 + `entity-context-menu` share.

**Two constraint additions to the widget tier** (`xgen-widget-tier.md` v1.2): **W-12** — a widget owns exactly one region (promotes the earlier "MAY own a region" to the universal rule); **W-13** — `system` widgets are non-removable (always present in the default layout; may collapse/redock/retab/configure but never fully close — a user can't lose the Composer).

**Why a new decision.** All-regions-are-widgets is a durable, cross-cutting architectural choice that reframes the whole client UI as a plugin surface (a custom widget can ship a new dockable region) and is cited by every M-RP6/M-RP7 milestone — it earns its own decision + a citable spec doc (`ui/docs/xgen-region-dock-model.md` v1.0, the federation-design-doc precedent). All-widgets framing locked by Joe 2026-07-07.

**Relationship to other decisions:** D-102 (the `widget` tier this extends with a layout seam) · W-11 (the data-socket sibling of W-12) · D-095 (the `ui/{...}` tier split) · D-056 (one shared command layer — the dock engine is shell-level) · D-065 (build-when-consumed → renderer A before B); N-075 (`EntityDescriptor` = the selection-bus payload).

---

## D-104 — Dev CDP verification: a temporary "build + unit-verified, CDP-deferred" close is allowed while the WebView2 remote-debug harness is blocked (→ M-RP-CDP1)

**Date:** 2026-07-08 · **Layer:** UI infra (M-RP5.6 A / M-RP-CDP1) · **Ref:** D-097, D-098, N-044

**Context.** The sampler CDP harness (D-097: a component milestone isn't done until its sampler cells are CDP-verified at 9422) worked unchanged J-405→J-480. It is now BLOCKED by an environment change: the machine's WebView2 Evergreen runtime auto-updated to **150.0.4078.48** (Chromium ≥136), which enforces the Chromium-136 remote-debugging hardening — `--remote-debugging-port` is IGNORED unless accompanied by a **non-default `--user-data-dir`** on the browser command line. Confirmed with real output on the reliable PowerShell path (clean launch + visible/foregrounded window + ~90s poll): 9422 never opens. All launcher/env levers exhausted — port-only; port + `--user-data-dir` inside `AdditionalBrowserArguments` (disallowed by WebView2); port + `WEBVIEW2_USER_DATA_FOLDER` (overridden by wry's own data folder). Official fixed-version runtimes are published only for the latest two majors (both ≥136); the only pre-136 source is an untrusted third-party archive (declined — supply-chain risk on a dev machine).

**Decision.** While the harness is blocked, a `core` UI component MAY land with its **CDP-only DoD items** (registry `count===unique` / 0 orphans / getter-G readout / both-accents) **DEFERRED and honestly recorded** — NOT marked complete, NOT fabricated (Rule 5: registry counts come only from a live `ids().length`). The other verification legs still gate the land: a clean `vite build`, and — where the logic is pure — a real unit test. The deferred CDP legs are closed retroactively once the harness is restored. This is a **scoped, temporary exception** to the D-097 sampler-CDP DoD, not a repeal.

**Harness restore = M-RP-CDP1** (its own milestone). Preferred approach: an **in-repo host change** making the sampler's Tauri/wry webview use an explicit, controlled **non-default data directory**, so `--remote-debugging-port` + a non-default `--user-data-dir` satisfy the Chromium-136 guard (the documented Playwright combo, once wry stops overriding it). Fallback: rewrite `cdp-debug.ps1` to CDP-over-`--remote-debugging-pipe` (the guard does not gate the pipe). **No untrusted runtime downloads.** Universal: the same block will hit the real client (9222) + node (9322) — restoring it fixes all three.

**First application:** M-RP5.6 A (`message-stream` shell, J-482) — landed build-clean + a 20/20 pure unit test on `stream/grouping.ts`; CDP legs deferred to M-RP-CDP1.

**Relationship:** D-097 (the sampler-CDP DoD this scopes a temporary exception to) · D-098 (sampler runtime = the WebView2 sibling where the block lives) · D-065 (honest-over-polite: deferred-not-faked).

**Resolution (2026-07-09, J-483) — root cause CORRECTED, exception RETIRED.** The diagnosis above was **wrong**. The real `msedgewebview2.exe` child command line (captured in a normal shell) showed: (a) a non-default `--user-data-dir=…\com.alchemydump.xgensampler\EBWebView` **already present** — Tauri forces a non-default data dir on Windows (`manager/webview.rs` L534–545) — so the Chromium-136 guard's precondition was met all along; and (b) `--remote-debugging-port` **absent from every webview2 process**. The port never reached the browser because **wry overrides the `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` env var** with its own programmatic `AdditionalBrowserArguments`. The `dataDirectory` spikes were fair-but-null (Tauri honours only a **relative** config `dataDirectory`; absolute is ignored — `webview/mod.rs` L392–424). Fix = **D-105** (route the port through config `additionalBrowserArgs` via a dev-only `--config` overlay). **M-RP-CDP1 CLOSED (J-483):** harness restored + verified on sampler 9422 + client 9222 + node 9322; M-RP5.6 A's deferred legs are now CDP-verified (registry 219→262). This temporary CDP-deferred exception is **retired** — not a standing allowance. Latent: `cdp-debug.ps1 -Launch` (built-exe) still uses the dead env var (flagged, not the supported path).

---

## D-105 — CDP remote-debug port via a dev-only Tauri `--config` overlay (base config stays release-safe)

**Date:** 2026-07-09 · **Layer:** UI infra (M-RP-CDP1) · **Ref:** D-097, D-098, D-104

**Context.** WebView2 Evergreen ≥136 (runtime 150.0.4078.48) broke the CDP harness — not via the Chromium-136 `--user-data-dir` guard (a red herring; Tauri already sets a non-default dir) but because **wry overrides the `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` env var**, so the env-supplied `--remote-debugging-port` never reached the browser command line (D-104 resolution). The port must instead travel through wry's **programmatic** additional-args channel.

**Decision.** Deliver the remote-debug port through Tauri config **`additionalBrowserArgs`** (the programmatic channel wry honours), kept out of release by a **dev-only overlay**: a per-app `cdp.dev.conf.json` (a complete `app.windows[0]` object + `--remote-debugging-port=<9422 sampler | 9222 client | 9322 node>`) merged via `cargo tauri dev --config cdp.dev.conf.json`. The base `tauri.conf.json` carries **no port** → RELEASE never exposes CDP. `--config` replaces the `windows` array (RFC-7396), so the overlay must be a **full** window object (preserves geometry — confirmed, sampler 960×820 intact).

**Wiring.** `run-sampler.ps1` / `run-client.ps1` / `run-node.ps1` `-Debug` pass `--config cdp.dev.conf.json`; the harness attaches unchanged (`cdp-debug.ps1 -App <app> -Mode eval`). Verified on all three apps (J-483).

**Scope + limit (D-065).** Covers the **dev-session** (`tauri dev`) CDP path — the supported one. It does **not** cover `cdp-debug.ps1 -Launch` (a built exe takes no `--config`): that path's env-var route is dead under ≥136, flagged for a later decision (a debug build with the port baked, or dropping `-Launch` in favour of dev-session attach). Baking a port into a release-shipped config was **rejected** (would expose CDP in release).

**Relationship:** D-104 (the block this resolves — its root cause corrected) · D-097 (the sampler-CDP DoD this restores) · D-098 (sampler WebView2 runtime) · D-056 (per-role launch scripts, one binary per role).

---

## D-106 — Grouped message rows drop BOTH name and avatar; the group head carries both; the avatar column stays reserved

**Date:** 2026-07-09 · **Layer:** UI / dd-component (M-RP5.7) · **Ref:** M-RP5.5 B (J-479, the build note this corrects), phase0 §10, N-075

**Context.** M-RP5.5 B shipped `grouped` as “suppress the name header, keep the avatar” (stated rationale: reader orientation). In practice a run of same-author continuations then repeats the identical avatar on every row — the exact who-is-speaking noise grouping exists to remove — and grouping became visually indistinguishable in the sampler (Joe flagged it against the M-RP5.6 B screenshot: same “AN” avatar on every row, nothing dropped). The B behaviour was a build note, not a recorded decision, so it is corrected here rather than re-litigated later.

**Decision.** A **grouped** row (a same-author continuation; `grouped === true`, i.e. NOT the group head) renders **neither the name header NOR the avatar** — symmetric suppression. The **group head** (`!grouped`) carries avatar + name as today. This matches the dominant chat convention (Discord/Slack/iMessage): the head establishes the speaker; continuations are bare bodies aligned beneath it.

**Mechanism (locked).**
- **Element absent, not hidden** — a grouped row does not render the `entity-avatar` child at all (not `visibility:hidden`); so a grouped cell registers **neither `__avatar` NOR `__name`** (`__name` already dropped at B). Clean, honest registry.
- **Column reserved** — the message grid keeps its avatar track pinned to **`28px`** (`.message` → `28px 1fr`, `.message[data-own]` → `1fr 28px`; the `28px` is the list-avatar width, the `1fr` the content — an earlier draft illustrated this as `28px 288px`, but 288px was never literal in code); only the avatar **cell content** is empty, so continuation bodies stay aligned under the head (no left-shift). Applies symmetrically for `isOwn`.
- **Independent of content state** — `grouped` is positional; it suppresses avatar + name **regardless of `deleted` / `edited`** (a grouped + deleted row = no avatar, no name, tombstone body).

**Consequences.** The live registry **drops**: every grouped cell loses its `__avatar` entry — this ripples across ALL grouped cells (the M-RP5.5 B `grouped`/`grouped-edited` sampler cells too, not only `stream-scroll`). The true new total is **CDP-measured at build and recorded** (Rule 5); closed milestones’ historical counts are **not** retro-rewritten (they were true when recorded). Grouping now also becomes visually obvious — dropping the avatar is what *makes* it visible, so no `stream-scroll` seed change is needed to demonstrate it.

**Scope.** `message.svelte` grouped branch + `skin.css` (empty-gutter handling) only. **No** `MessageDescriptor` / `stream/grouping.ts` / `message-stream.svelte` change (`grouped` stays the stream-computed prop). Tighter inter-continuation vertical spacing (the usual chat compaction) is a **deferred** skin follow-up, out of scope here (D-065).

**Relationship:** M-RP5.5 B (corrects its “keep the avatar” build note) · phase0 §10 (the M-RP5.7 lock) · N-075 (dd-composite / `message` root) · D-065 (honest longer work over the papered-over asymmetry).

---

## D-107 — App frame (menu-bar + status-bar) = fixed chrome outside the dockable region layout; frame containers are `core`, window-effects are shell-wired

**Date:** 2026-07-09 · **Layer:** UI / client shell (M-RP6.1) · **Ref:** D-103 (region/dock model, extended), D-102 (widget tier), D-095 (`ui/{…}` tier split), D-056 (one shared command layer), D-096 (fold criterion), D-065

**Context.** The client UI panel arc (M-RP6.1) needs a menu-bar and a status-bar. The region/dock model (D-103) knew only the dockable region layout; it had no concept of fixed window chrome. Left unresolved, a menu-bar or status-bar could be mis-modelled as a dockable system widget — which would let a user dock the Composer's neighbour away, or (worse) lose the only exit affordance.

**Decision.** The client window is a BorderPane: a fixed **top pane** (menu-bar), a fixed **bottom pane** (status-bar), and a **center** that holds the dockable region layout. The **menu-bar and status-bar are fixed frame chrome — NOT dockable regions/widgets — and live OUTSIDE the `Layout` descriptor.** Only the center is subdivided by the descriptor; the dock engine (renderer B, M-RP7) mutates the center, the frame is stable. This makes File→Exit unconditionally reachable (outside the dock tree entirely — stronger than W-13, which only keeps *system widgets* present).

**Frame containers are `core`; window-effects are shell-wired.** The status-bar (and the menu family) are reusable `core` library components — the node app needs an un-minimalized status-bar too. Because `core` stays app-agnostic (imports no Tauri/protocol, per the `link`/`entity-avatar` rule), any real-window effect is exposed as a **seam** the consuming shell wires to its own Tauri call: the status-bar's resize-grip via `onResizeGrip?` (shell → `startResizeDragging`), the menu's Exit via a command callback (shell → the existing exit command). Client and node reuse the same component, each supplying the window effect.

**Consequences (new components/objects, all this arc).**
- `icon` (`core`) — its own component, NOT folded into `image`; cleared by **D-096 on two axes** (value-type: a shape/path definition vs a `src` reference; surface: tintable UI glyph vs raster content). Primary path inline `<svg>` (tintable), raster `<img>` secondary. Mirrors JavaFX `SVGPath`-vs-`ImageView`.
- `separator` (`core`) — orientation vertical | horizontal; shared by the status-bar (between cells) and the menu (`menu-separator`).
- `Accelerator` (`ui/common`) — one value-object, two projections from a single definition (`toDisplay()` for the menu hint, `matches(event)` for dispatch) → no display/dispatch drift; consumed by a lean shell-level keymap registry.
- `menu-bar` / `menu` / `menu-item` (`core`) — minimal (File→Exit) now; reuses the `entity-context-menu` W-2 behaviour machine; grows by accretion (separator / check-item / submenu-flyout deferred, D-065).
- `status-bar` (`core`) — side-stacking `sb-cell`s + `separator`s + our own always-visible SE resize-grip (via the `onResizeGrip?` seam); text defaults to `--fs-s1` (9px).
- Skin: `--fs-s1: 9px` / `--fs-s2: 8px` added below the existing `--fs-0: 10px` (additive, no rename of the shipped `--fs-0/1/2` scale).

**Scope.** Frame concept + the component prerequisites + the frame-first M-RP6.1 re-sequence. Design-only at lock (J-488); each component gets its own design lock + runbook when its sub-milestone opens. Verify graduates from the sampler to the real client app (the sampler cannot reach a node; D-097) — three-layer: pure unit / real-client-offline / real-client + node.

**Relationship:** D-103 (extends it with the frame concept — the frame is the non-dockable complement of the center layout) · D-102/W-12/W-13 (the frame is deliberately NOT a widget) · D-096 (clears `icon` as its own component) · D-095 (frame containers are `core`) · D-056 (window-effects shell-wired, above components) · D-065 (minimal-now, grow-by-accretion).

---

## D-108 — The glyph bank: a glyph is a SKIN TOKEN, not source code; `core` owns the NAME, the skin owns the SHAPE

**Date:** 2026-07-12 · **Layer:** UI appearance (M-RP-ICON-ADOPT / CSS layer model) · **Ref:** N-101, N-020, N-025, N-031, N-090, D-096, D-067, D-109 · **Canonical doc:** `ui/docs/xgen-css-layer-model.md` v1.0

**The problem, measured — not assumed.** 21 distinct glyphs lived in **four mechanisms across two layers**: 3 as `<path d>` strings in a TS registry (`icons.ts` → `icon.svelte`, fill, `--icon-tint`); 10 as `mask-image` data-URIs in `skin.css` (mostly stroke); 7 as `background-image` data-URIs in `skin.css` with the **colour baked into the URI**; 1 as an `.svg` file consumed by `src` — **and re-inlined a second time** as a data-URI in `app_sampler.svelte:402`.

**And every `skin.css` glyph token was declared inside its own component's class selector — none at `:root`.** `skin.css` stated the intent explicitly: *"icon-data vars scoped here (no global token)."* **Two measured consequences:**
1. **`--tri` / `--tri-open` are declared TWICE** — `.combobox` (1232-33) and `.section` (1829-30), where the section's own comment says *"REUSES combobox's masked glyphs"* **and then re-declares them**. *The loss this decision exists to prevent had already occurred.*
2. **A component-scoped custom property is a private variable, NOT a theme surface.** A theme author cannot redraw *"the eye"* — they must know which component scopes it and redefine each shared glyph N times. **Component-scoping half-defeated the skinnability it was chosen for.**

**Decision.** **A glyph is a skin token — the same species as `--accent2`.**

> **`core` owns the NAME (identity = content). The skin owns the SHAPE (geometry = appearance).**
>
> A component says *which* glyph. The skin says *what it looks like*. **A component never writes geometry, for the same reason it never writes a colour.** (N-025 / N-090, applied to glyphs.)

- **Source of truth (hand):** `ui/assets/icons/*.svg` (24×24, geometry only, no colour) + `ui/assets/icons/icons.manifest.json` (paint / stroke-width / **source + licence per glyph**). **The `.svg` files never ship** — they are authoring source.
- **Generated:** `ui/assets/glyphs.generated.css` → `:root { --glyph-gear: path('…'); --glyph-gear-url: url("data:…") }` — **the bank, and the runtime default**. And `ui/core/.../icons.generated.ts` → `type IconName = 'gear' | …` — **names only, no geometry**.
- **Two token forms, and they are NOT redundant.** `path()` is consumable **only** by the CSS `d:` property. `<select>` and `<input>` have **no child element to hang a `<path>` on**, and **N-020 forbids wrapping the root** — so native roots consume `--glyph-*-url` via `background-image` / `mask`.
- **Layering (D-108's structural half):** `glyphs.generated.css` loads at **L1.5** — after the normalizes, **before `skin.css`**. **`skin.css` + `glyphs.generated.css` are ONE layer** (the default skin), split by **who writes it** — you never mix a generated block into a 98 KB file a human edits live over HMR. A later `theme-*.css` overrides **any** token — colour or glyph — **by the cascade, with no second machinery**.
- **Component wiring:** `<Icon name="gear"/>` → `<path>` with **no `d` attribute** + inline `--g: var(--glyph-gear)` → **ONE** skin rule for the whole system: `.icon path { d: var(--g); fill: var(--icon-tint, currentColor) }`.

**Guards (each failure mode dies structurally, not by discipline).** Typo → build error (`IconName` union). Duplicate name → build error (generator). **Glyph with no licence → build error** (the BSL→GPL gate becomes structural, not a periodic audit). Missing token at runtime → empty `<path>` + DEV-warn, **the behaviour `icon.svelte` already ships** for an unknown name (the W-13 unknown-id-drop precedent) — no throw, no new failure mode. Re-drawing a glyph nobody knew existed → the **sampler glyph-grid** renders the whole bank; *you cannot redraw a glyph you can see.*

**Consequences that closed open questions (all measured; evidence table in the canonical doc §5).**
- **`var()` resolves inside a custom-property value** → **one** skin rule, not one per glyph. The `data-glyph` per-glyph-rule fallback is **dead**.
- **Multi-path glyphs carry per-path independent fills** → **multi-colour marks stay `icon`s. D-096 is NOT re-opened** (the palette glyph does not become an `image`).
- **Stroke-vs-fill is a skin property on `.icon path`** → **`icon` gains no new prop.** The deferred "stroke variant" question dissolves.
- **`icons.ts` retires as a geometry store.** Its name half survives as the generated `IconName` union — **Joe's Java `SvgGlyph.GEAR_ICON` enum, split along the line the project already draws.**

**❌ Explicitly rejected — the `d`-attribute fallback.** The probe showed CSS `d:` **overrides** a present `d` attribute, which made *"ship geometry as an attribute AND let the skin override it"* technically possible, and Chat recommended it for one turn. **Rejected: it is two defaults for one glyph — a second source of truth for geometry (D-067 drift), hedging against a browser that cannot occur (D-109).** **Geometry lives in the skin. Only in the skin.**

**Relationship:** D-109 (the platform dependency this rests on) · D-096 (cleared `icon` as its own component — **not re-opened**) · D-067 (the drift this eliminates, and the reason the fallback was rejected) · N-020 (atomic roots — why native roots need the `-url` form) · N-025 / N-031 / N-090 (skin owns appearance; this is that rule applied to geometry) · D-071 (the Phase-0 classification pass still gates implementation).

**Scope.** Model only. **No code moved at lock.** Implementation is M-RP-ICON-ADOPT, gated behind the frame arc; its Phase-0 classifies all 21 glyphs (fill / stroke / multi-colour / native-root) and licence-sources every one.

---

## D-109 — CSS `d:` geometry is a DELIBERATE Chromium/WebView2 platform dependency, recorded rather than assumed

**Date:** 2026-07-12 · **Layer:** UI platform (M-RP-ICON-ADOPT) · **Ref:** D-108, N-101, Ch6 §6.1 · **Canonical doc:** `ui/docs/xgen-css-layer-model.md` §5–§6

**The dependency.** D-108's glyph bank puts SVG path geometry in the skin via the CSS **`d:`** property (`.icon path { d: var(--g) }`). **`d:` is a Chromium property.** It is not supported by Firefox or Safari/WebKit. **D-108 does not work on a non-Chromium engine.**

**Why this is acceptable, and why it is nonetheless recorded.** Ch6 §6.1 locks **Tauri**, which renders in the OS-native webview — **WebView2 (Chromium) on Windows**, the Phase-1/Phase-2 target. The dependency is therefore satisfied by the shipped platform, today, on the only target that exists. **But it is a real constraint on a real axis** (Ch6 §6.1 also names macOS/Linux for Phase 2, where Tauri uses **WebKit** — *and there `d:` does not work*). A dependency this load-bearing must be **written down at the moment it is taken**, not discovered by a future macOS build.

**Decision.** **Take the dependency, name it, and do not hedge it.**
- The XGen client is a **Chromium-engine desktop application** for glyph-rendering purposes. This is stated, not implied.
- **No fallback is shipped** (see D-108's rejected `d`-attribute fallback). A fallback would cost a permanent second source of truth for geometry to insure against a case that does not currently exist — **and if the macOS/Linux WebKit port ever lands, the honest fix is the `-url` (mask) form, which the bank ALREADY EMITS for every glyph.** The insurance is already in the bank; it does not need to be in the DOM.

**🔑 That last point is what makes the dependency safe rather than reckless.** D-108's two-form emission (`path()` **and** `url()`) was adopted for the native-root cases — but it means **every glyph in the bank already has a Chromium-independent representation.** A WebKit port re-points `.icon path { d: … }` at a `mask` on a `<span>`; the **bank, the names, the manifest, the licences, and every call site stay identical.** The port is a renderer swap, not a rewrite — the same "one descriptor, two renderers" shape D-103 already uses.

**Measured, not assumed** (real client 9222, WebView2): a `<path>` with **no `d` attribute** styled by `d: path('M5 5h14v14H5z')` → `getBBox()` **14×14**, `getTotalLength()` **56** (= 4×14, the true perimeter — the geometry engine, not merely the computed string). Indirection (`d: var(--glyph-x)`) and theme override through it both resolve. Full evidence table: canonical doc §5.

**Relationship:** D-108 (the model that rests on this) · Ch6 §6.1 (Tauri + native webview; the source of the constraint **and** of the Phase-2 macOS/Linux exposure) · D-067 (why no in-DOM fallback) · D-103 (the one-source-two-renderers precedent that makes the WebKit exit cheap).

**Scope.** A recorded platform dependency. No code. **Re-open if and when a WebKit target becomes real** — the exit is specified above and costs no call-site churn.

---

## D-110 — Space-theme override subset: a Space may re-COLOUR; it may NOT re-DRAW and may NOT re-LAYOUT

**Date:** 2026-07-12 · **Layer:** Client trust surface / theming (Ch6 §6.3) · **Ref:** D-108, D-109, D-057, D-036, N-101, N-102, Ch6 §6.2 / §6.3.1 / §6.3.2 / §6.13 · **Journal:** J-505

**The question, and how old it is.** Ch6 §6.2 has said since Session 1 (April 2026) that *"only a defined subset of tokens may be overridden by a Space theme"*, and §6.3 filed the open question *"Which specific CSS tokens may a Space owner override?"* for the second pass. **It was never answered.** D-108 made it urgent.

**Why it is a TRUST decision, not a styling one.** Ch6 §6.3's theming cascade has three layers: XGen default → **application theme** (operator/user) → **Space theme**. Layers 1 and 2 are ours and the user's. **Layer 3 is not: it is declared by a Space OWNER and arrives over the wire in a `state.space_theme` Event.** Under **D-108** a theme can redraw **any glyph** (a glyph is now a skin token, and a theme overrides a glyph exactly as it overrides a colour). **So an unrestricted Layer 3 lets a Space owner redraw a lock, a warning, a verified mark, or the AI badge (§6.13)** — making a hostile Space look trustworthy, or a human member look like a bot. **Icon spoofing, served from the wire, in a protocol whose entire premise is verified identity.**

**Decision (Joe, 2026-07-12).**

> ### A Space may **re-COLOUR**. A Space may **not re-DRAW**, and may **not re-LAYOUT**.

| Token class | Space override | Why |
|---|---|---|
| **Colour** — `--accent*`, surface / text / border, **and the glyph tint** (`--icon-tint`) | **✅ PERMITTED** | Brand identity — what Space theming was *for*. A Space may re-tint a glyph freely: **the mark keeps its meaning; only its hue changes.** |
| **Geometry** — `--glyph-*`, `--glyph-*-url` (D-108) | **❌ BANNED** | **The mark IS the meaning.** Redrawing it is spoofing, not branding. |
| **Layout / metrics** — spacing, radius, type scale, sizes | **❌ BANNED** | Readability + accessibility (the original D-057 intent), and it forecloses displacement attacks (hiding or moving a control by resizing it). |
| **Anything not on the allowlist** | **❌ BANNED by default** | **Allowlist, never denylist.** A token added tomorrow is banned until someone decides otherwise. |

**🔑 Consequence 1 — the split constrains D-108's GENERATOR, normatively.** A data-URI with a colour **baked into it** fuses colour and geometry into a single token. Permitting a Space to change that token's colour would therefore *necessarily* permit it to redraw the glyph — **the ban would be unenforceable on exactly those glyphs.** **Therefore `--glyph-*-url` MUST be emitted colour-free** (a `currentColor` mask), with colour supplied by a **separate** colour token. *This retires the seven glyphs currently shipping with `%23e6e6e6` baked into the URI (the 5 `textfield[type=]` insets, the `select` arrow, `--ea-spark`) — the Phase-0 re-emit is now a **security requirement**, not a tidy-up.*

**🔑 Consequence 2 — A KEY ALLOWLIST ALONE IS THEATRE.** A Space theme is a key→value **map**, not a stylesheet (Ch6 §6.2's event shape: named keys, scalar values). *(Correcting an overstatement in J-504, which called it "attacker-supplied CSS": it is an attacker-supplied token **map**. The threat is narrower — and the mitigation is sharper.)* **But if the client builds a stylesheet by string concatenation, a malicious VALUE escapes its declaration and injects arbitrary CSS — defeating the key allowlist completely:**

```
"color_primary": "red; } :root { --glyph-lock: path('M0 0h24v24H0z'); } /*"
```

**Mandatory mitigation, both parts required:** apply each override via **`element.style.setProperty(key, value)`** — the CSSOM **cannot break out of a declaration** — **and** validate the value first (`CSS.supports('color', value)`), rejecting anything not well-formed for its type. **Never interpolate a wire-supplied value into a `<style>` text node.** Plus **scope**: Layer-3 overrides apply only within the active Space's subtree — never at `:root`, never to application chrome (menu-bar, status-bar, Space list).

**Enforcement is CLIENT-side.** A Node does not police theme content; a malicious client can ignore all of this, and that is acceptable — the attack this closes is a **Space owner attacking the Space's own members through a conforming client**. The three rules (allowlist the key · validate the value + CSSOM · scope the application) are a **conformance requirement on the client**, and **all three are required — any one alone is insufficient.**

**🔑 Locked BEFORE implementation, deliberately.** Grepped 2026-07-12: **`state.space_theme` appears in no Rust, no TypeScript, no Svelte.** The theming cascade is **specified and entirely unbuilt**. D-110 lands before the first line of it is written — the cheapest moment a trust boundary can ever be set, and a case where the project's *"subsystem audits precede dependent milestones"* discipline (D-071) paid out in advance rather than in arrears.

**Relationship:** D-108 (made this urgent — a glyph is a skin token, so a theme can redraw it; and D-110 in turn constrains D-108's generator) · **D-111 (the sibling boundary: a client must not fetch a host chosen by someone else — it closes the *category* this decision's records wrongly claimed `url()` belonged to)** · Ch6 §6.3 (the cascade this governs; §6.3.1/§6.3.2 written at this lock) · Ch6 §6.13 (the AI badge — a spoofable mark) · D-057 (the readability/accessibility intent behind the layout ban) · D-036 (module UI forms — a related third-party-content trust surface, **not** closed by this).

**Scope.** Specification + trust boundary. **No code.** Binding on any future Layer-3 applier and on D-108's generator. **Open and NOT closed here:** the exact colour-token allowlist (names + count — enumerated when the theme layer is built); whether a user may disable Space themes entirely (recommendation: yes, and it is cheap — Layer 3 is a scoped, droppable overlay by construction).

> **⚠️ AMENDED 2026-07-12 (same day, D-111 / J-506) — this decision's original "wider surface" paragraph was WRONG and is corrected.** It read: *"the wider question of what else Space-owner-supplied content can do (`url()` fetches, font substitution, module widgets under D-036)."* **`url()` fetches and font substitution do not belong on that list:** under **this very decision** the allowlist is colour-only and every value is validated (`CSS.supports('color', v)`), so **a `url()` cannot enter a Space theme at all** — and fonts are **bundled in the binary** (Ch6 §6.2, *"without runtime internet dependency"*). **A threat named without checking the tree, in the same document that forecloses it.** The real property became **D-111** (a client must not fetch a host chosen by someone else). **What genuinely remains open is D-036 module widgets** — third-party HTML in an isolated webview, CSP/sandboxing still an open Ch6 §6.8.8 question. **That one is bigger than glyphs and themes combined, and it is Ch6's.**

---

## D-111 — A client MUST NOT fetch a host chosen by someone else: outbound URL resolution (link previews) is NODE-side, never client-side

**Date:** 2026-07-12 · **Layer:** Client conformance / privacy boundary (Ch2 "Client decisions") · **Ref:** D-110, M12 (blob store), Ch2 §"Client decisions — implementation freedom", Ch6 §6.2 (bundled fonts), Ch3 §600 · **Journal:** J-506

**How this was found — by trying to justify a claim and failing.** D-110's records listed *"`url()` fetches"* among the still-open Space-owner trust surfaces. Challenged to defend it, the claim **collapsed**: under D-110 the Space-theme allowlist is colour-only and every value is validated (`CSS.supports('color', v)`), so `url(…)` **cannot enter**. It was a familiar-looking threat shape asserted without checking the tree — **the second such overclaim in two turns**, and it is retracted.

**But the grounding that killed it surfaced the real property, and the real property is worth a decision.**

**🔑 The invariant, stated:**

> **Any mechanism where the client fetches a URL chosen by someone else turns the client into a BEACON** — it discloses the reader's **IP address** and the **timing of their read** to a host of the sender's choosing.
>
> **XGen publishes your XGID by design. It does NOT publish your network location.** A fetch primitive silently adds a channel the protocol deliberately excludes — and it does so against a **Space owner or a message sender**, not a distant third party.

**🔑 The protocol already forecloses this almost everywhere — structurally, not by mitigation. That was deliberate and it should be named.**

- **`message.image` / `message.file` carry `xgen://hash/sha256:<64-hex>`** — a **content address, not a location** (`xgen-core/src/blob_store.rs`: `blob_ref = hash_uri(bytes)`, the same scheme as `event_id`; blobs are federation-native (M12, J-389), per-blob client-encrypted before upload (M12-D5), and the store is *"content-blind by construction"*). **A hash cannot name a host.** The beacon is not *blocked* here — it is **unsayable**. There is no field in which "over there" could be written.
- **Fonts are bundled in the binary** — Ch6 §6.2, explicitly *"without runtime internet dependency."*
- **No XGen crate carries an HTTP client** (no `reqwest` / `hyper` / `ureq` in xgen-client, xgen-core, xgen-node).
- **Space themes cannot carry a URL** (D-110: colour-only allowlist + value validation).

**⚠️ The one place it survives: `link previews`, listed in Ch2's "Client decisions — implementation freedom" table.** Nothing is built; it is an *example* of what the protocol deliberately does not dictate. **But it is precisely where a well-meaning client re-opens the channel** — and it would do so on **every message**, invisibly to the reader, in a system that content-addressed its blobs specifically to prevent it.

**Decision.**

> **Link previews — and any other rendering that resolves an outbound URL — are fetched NODE-SIDE, never client-side.**

The **Node already talks to the world; the Client deliberately does not.** The Node fetches, strips, caches and serves the preview. Consequences: **one fetch per link, not one per reader**; **the sender learns nothing about who read the message or when**; and the client keeps its property of never opening a connection it was not configured to open.

**Implementation freedom is preserved and the table is not weakened.** *Whether* to show previews, and *how* they look, remain entirely the client's business. **Only the fetch location is fixed** — because it is **not a rendering decision. It is a privacy boundary wearing a rendering decision's clothes.**

**Relationship:** D-110 (the sibling boundary — a Space owner may re-colour, not re-draw; **this one closes the *category* D-110 explicitly did not**) · M12 / `blob_store.rs` (content-addressing, the structural precedent this decision generalises) · Ch6 §6.2 (bundled fonts — the same instinct, taken earlier) · Ch2 "Thin Client Principle" (a thin client that phones arbitrary hosts is not thin) · **D-036** (module widgets in isolated webviews — **NOT closed by this**; CSP/sandboxing remains an open Ch6 §6.8.8 question, and a webview that can fetch is a bigger surface than any of the above).

**Scope.** A client-conformance rule + a named invariant. **No code.** Binding on any future link-preview or outbound-URL-resolving feature. **Open and NOT closed here:** module-widget webview sandboxing / CSP (D-036, Ch6 §6.8.8) — **the largest remaining instance of the same property, and it belongs to Ch6, not here.**

> **✅ CLOSED 2026-07-12 (same day, D-113 / J-507).** The module-widget sandbox question this decision left open is **answered**: the boundary is the **delivery** axis, not the widget shape, and a `packaged` module UI is a webview with **no network** (**D-113 S-1**). The beacon is **unsayable inside a module** for the same structural reason it is unsayable in a message.

---

## D-112 — The plugin taxonomy: ONE plugin, THREE axes (host · delivery · surface)

**Date:** 2026-07-12 · **Layer:** Plugin / module architecture (spans Ch6 §6.8, the UI widget tier, the region model) · **Ref:** D-036, D-071, D-102, D-103, D-107, SE-D2 · **Journal:** J-507 · **Phase-0:** `docs/xgen-plugin-taxonomy-phase0.md`

**The problem.** Two specs disagreed about what a "widget" is and where it is placed. **Ch6 §6.8.3** (D-036, April 2026): an HTML file in an **isolated webview**, talking to a module backend over a **local WebSocket**, placed by a **named slot** from a fixed inventory, shipped by a third party as a **package + manifest**. **D-102/D-103** (July 2026): a **Svelte component**, in-process, fed by a **`$common` store**, placed by a **dockable region** in the layout descriptor. `self-panel` and `inspector-panel` are the latter — no webview, no socket, no slot, no manifest. **A D-067 drift surface sitting in the SPECS, not the code.**

**🔑 What grounding found, and it reframed the whole question.**

- **`xgen-common/src/module.rs` (SE-D2) — the plugin spine ALREADY EXISTS**, and its own doc comment already carries Joe's frame verbatim: *"There is one unified handshake mechanism; the code term **`kind`** carries the system/ui distinction: a **module** is a *system* plugin (`host = node`), a **plugin** is a *ui* plugin (`host = client`)."* It also ships slot/impl identity (`ModuleKindId` / `ModuleImplId`, **UUIDv4, never `Xgid`** — a module GUID is local and never federates) and a trust posture: *"the descriptor is a **const in the plugin's own code** — there is **no manifest file** … metadata is **authoritative**, location is **never trusted**."* *(Honest limit: the shipped `Descriptor` struct is `{kind_id, impl_id, name, assurance}` — the **vocabulary** is shipped, the `host`/`kind` **fields are not**.)*
- **`xgen-core/src/auth/module_registry.rs` — a THIRD species neither spec listed.** An Auth Module is a **protocol principal**: `AuthModuleXgid` + `endpoint_url` + a trusted/revoked registry. **A remote service** — not a compiled plugin, not a UI widget.
- **Zero `xgen-module.json` exists anywhere.** No manifest loader, no `modules/` scan, no local WebSocket server in the client.

> ### 🔑 So the drift is not "Ch6 vs D-102". It is **Ch6 §6.8 vs everything that was actually built.**
> §6.8 predates the plugin spine, the Auth Module registry, the widget tier, the region model and `WidgetMount`. **It is the outlier, and it is the thing that moves.** *(The J-502 "first bird" shape a second time — a section named before every convention it would live among existed.)*

**Decision — one plugin, described by three orthogonal axes.**

| axis | values |
|---|---|
| **host** | `node` (the **system** area) · `client` (the **ui** area) |
| **delivery** | `compiled` · `service` · `packaged` |
| **surface** *(client only)* | `none` (headless) · `region` · `shelf` · `window` — at most one (W-12) |

**"Module" and "widget" are not two things** — they are `host = node` and `host = client` on **one plugin**, exactly as `module.rs` already said. **One plugin, one list, several UI forms** (Joe's frame, 2026-07-11).

**The `delivery` axis is the new one, and it is where trust lives:** **`compiled`** (const descriptor, linked into our binary — everything shipped) · **`service`** (own process, own XGID, an endpoint, speaks protocol Events — the Auth Module) · **`packaged`** (third-party code + manifest — **zero lines exist**, and the entire open trust surface; see **D-113**).

**🔑 Placement vs containment — the slot inventory does NOT retire, and it was never a rival.** Split Ch6's slot list against the shipped surfaces §3.2 clause (*content inside another widget is not a surface*): `node.dashboard.widget` / `room.sidebar.*` are **regions**; `room.toolbar` / `room.message.decorator` / `space.header` / `global.statusbar` are **content anchors inside a host widget**. **And the containment mechanism is ALREADY SHIPPED** — `message.svelte` takes `details: WidgetMount[]` + `bodyExtras`, resolves against a prop-injected registry and **drops unknown ids** (W-13, M-RP5.5). ***`room.message.decorator` is `message.details` under another name.***

> **→ ONE placement model** (the D-103 descriptor — a plugin that *is drawn as a place* takes a surface) **+ ONE containment model** (**host-declared** `WidgetMount[]` anchors — spends no surface). **Ch6's slot table is a stale, guessed inventory of mount points**, written against a Room view that does not exist; it is **regenerated from the widgets that actually exist** at M-RP7.4, never copied forward. **A slot is declared by the HOST, never requested by the guest** — that is the anti-drift property.

**Consequences.**
- **The manifest is reconciled, not merged:** a **compiled `Descriptor` is AUTHORITATIVE** (our code); a **`packaged` manifest is UNTRUSTED INPUT** — it *declares intent*, the **host enforces**. **They must never become one type.**
- **Ch6 §6.8.7 is corrected:** the Auth Module is **`delivery: service`**, **not** "the reference implementation of a Window-form module". It has no manifest, no package, no webview.
- **The plugin list (M-RP6.1l)** renders from the axes: `host` → system/ui · `delivery` → the **trust badge** (`built-in` / `service` / `installed`) · `surface` → where it lives. Remove/Disable semantics **fall out** rather than being special-cased: `kind: system` → no Remove (W-13) · `service` → **revoke** (block-only, the shipped `revoked` flag) · `packaged` → Remove.
- **Settings takes `surface: window`. NO `screen` kind is added** — the `window` form already exists and has a second consumer (a packaged plugin's Launch button). *Ch6 §6.8.5's "a screen of its own" is prose, not a surface kind.* **Foreclosed knowingly:** the Discord full-window overlay shape — that would be a fifth kind, and it must be a lock, never a drift.
- **Ch6 §6.8.8's *module permissions* and *module signing* questions resolve ON the delivery axis** (deny-by-default capabilities; signing **mandatory for `packaged`**, meaningless for `compiled`, already solved for `service` — an Auth Module **is** its key, AMR-D3). *Two of the five open questions dissolve the moment the axis exists — the sign the axis is real.*

**Scope.** Architecture + vocabulary. **No code.** Binding on the plugin list, the shelf arc, M-RP7.4, and any future module work. **Unblocks** surfaces §6 item ① and M-RP6.1i–l.

**Deliberately NOT decided:** the **`compiled` plugin LOADING mechanism** (`temperature.rs` itself leaves it open — *"dynamic libraries, WASM, external process"*). **⚠️ A dynamic library is not a sandbox:** if the loader ever becomes `dlopen`, that plugin has `compiled`-trust with **none** of `compiled`'s review. **Whichever loader is chosen must land on this axis — and if it admits third-party code it inherits D-113.**

**Relationship:** D-036 (module architecture — **this aligns it, does not replace it**) · D-102 (the widget tier — a widget is a plugin with `host = client`) · D-103 (the region model — the placement half) · D-107 (the frame — outside the descriptor, unaffected) · D-067 (the drift this closes) · D-071 (the audit that found it) · SE-D2 (the spine it extends) · **D-113** (the sandbox — **locked together with this; you cannot classify a thing while leaving open what it is allowed to do**).

---

## D-113 — The packaged-module-UI sandbox: the boundary is DELIVERY, not "widget" — and a module UI has NO network

**Date:** 2026-07-12 · **Layer:** Client trust boundary (Ch6 §6.8.8, open since Session 2 / April 2026) · **Ref:** D-036, D-110, D-111, D-112 · **Journal:** J-507 · **Phase-0:** `docs/xgen-plugin-taxonomy-phase0.md` §10

**🔑 The reframe, and it is the whole decision.** Ch6 §6.8.8 has asked since April: *"Widget sandboxing: what CSP and iframe sandboxing apply?"* **But `self-panel` needs no CSP, and a compiled Rust storage engine needs no CSP.** **Nothing about *being a widget* is dangerous. Being `delivery: packaged` is** (D-112). **The question was attached to the wrong noun for three months** — and the same floor must cover a packaged module's **window** exactly as its **widget**, which a widget-shaped rule could never do.

**Why it is the largest open surface in the project.** Every other content channel has a **structural foreclosure**, not a filter: **blobs are content-addressed — a hash cannot name a host** (D-111) · **a Space theme is a colour-only allowlist — a colour cannot be a `url()`** (D-110) · **glyphs are banned from Space override** (D-110) · **fonts are bundled** (Ch6 §6.2). **A `packaged` module UI has NONE of these.** It is arbitrary third-party markup and script **with a network stack** — **the only channel with no floor under it.**

**Decision — the floor. Foreclose; do not filter.**

> ### **S-1 — A packaged module UI is a webview with NO NETWORK.**
> Its **only** egress is the local IPC channel to its **own backend**, which runs on the Node/Client **we** ship.
> CSP: `default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self' data:; font-src 'self'; connect-src <its own local channel>; frame-ancestors 'none'; form-action 'none'`. **No `http:` / `https:` scheme is reachable at all.**
>
> **This makes D-111's beacon UNSAYABLE inside a module — exactly as it is unsayable in a message.** *The same structural property, taken a second time.*

- **S-2 — Assets are PACKAGED, never fetched.** Everything it renders ships in the package. *The bundled-fonts rule, generalised: a packaged asset cannot phone home.*
- **S-3 — Isolation.** Own webview, **own origin**; no `allow-same-origin` with the host; **no Tauri IPC exposure** (no `withGlobalTauri`, no `invoke` in its context); no access to the host DOM, the host stylesheet, or another module's webview.
- **S-4 — The module never holds the key.** Ch6 §6.8.4's `identity_mode: user` is **consent, not key handover**: the module **requests**, **our Rust signs, per event**, against declared capabilities. *A module that can sign as you at will is not a module — it is you.*
- **S-5 — A module UI may not draw trust chrome.** **D-110's lesson, generalised beyond CSS.** It renders inside a **bounded, attributed frame** and **may not occupy the identity / verified / AI-badge zone** (Ch6 §6.13). *Icon spoofing does not become acceptable because the spoofer arrived as a package instead of a theme.* **Corollary:** S-3's separate origin means it never sees our glyph tokens — the foreclosure is structural here too.
- **S-6 — Capabilities are declared and enforced HOST-SIDE, DENY-BY-DEFAULT.** **Allowlist, never denylist** (D-110's rule). The manifest **declares**; the host **enforces** (D-112). *This is also Ch6 §6.8.8's "module permissions" question, answered.*
- **S-7 — The sequencing lock. NO `delivery: packaged` plugin may load until S-1…S-6 ship.** Until then the plugin list contains **`compiled` + `service` species only** — which is **exactly what exists today**, so **the lock costs nothing and forecloses everything.**

**🔑 Why lock it now, against zero code.** **Because that is the cheapest moment a trust boundary can ever be set.** `state.space_theme` was locked before a line of it existed (D-110); `delivery: packaged` is in the same position **today** — no manifest, no loader, no webview. **The first packaged module that ever loads will load into a floor that already exists.** *(D-071 paying out in advance rather than in arrears — again.)*

**Scope.** A client trust boundary. **No code.** **Binding:** no milestone may load third-party module UI code, and none may ship a module host that does not implement S-1…S-6 in full. **CLOSES Ch6 §6.8.8's widget-sandboxing question** (open since April 2026) and, with D-112, its *module permissions* and *module signing* questions.

**NOT closed here:** the **backend** half of a packaged module (Ch6 §6.8.1 — any language, WebSocket, `meta_atts`) is a **protocol participant**, and its risk is the Node's authorisation model, not the client's DOM. **The dangerous half is the UI half.**

**Relationship:** D-036 (module architecture — the widget/window forms this puts a floor under) · **D-111** (the beacon invariant — **this is the one channel D-111 explicitly could not close**; now closed) · **D-110** (colour-not-geometry; the allowlist-never-denylist rule, and the trust-chrome lesson) · **D-112** (the taxonomy — locked together) · D-071 (audits precede dependent milestones).

---

## D-114 — ONE UI-state store: geometry is TYPED in Rust, everything else is an OPAQUE blob

**Date:** 2026-07-12 · **Layer:** Client UI persistence · **Ref:** D-067, D-101, D-103, D-107 · **Journal:** J-510 · **Phase-0:** `docs/xgen-widget-surfaces-phase0.md` §4 (Joe-locked J-503) · **Milestone:** M-RP6.1k

**🔑 Two specs described the persistence of the client's UI, and they were never two objects.** `ui/docs/xgen-region-dock-model.md` §9 specified a **layout-only** store (`xgen-client_layout.json`, widening to `xgen-client_layouts.json` at M-RP7.6, five verbs `list/save/load/delete/rename_layout`). `docs/xgen-widget-surfaces-phase0.md` §4 specified a **UI-state** store (`xgen-client_uistate.json`) holding layout *and* geometry *and* shelf favourites *and* collapsed state *and* theme *and* the last open room, with two lifecycles (session / named). **§9 is simply the earlier and narrower draft** — written when the layout was the only thing anyone intended to persist, and before window geometry, the shelf, or a named-workspace concept existed.

**Decision: ONE store. `xgen-client_uistate.json`.** Two files would mean **two lifecycles, two clamps, two reconcile rules and two migration paths for one user-visible act** ("put my app back how I left it") — the D-067 drift surface this project exists to eliminate, self-inflicted and entirely avoidable. `docs/xgen-widget-surfaces-phase0.md` §8 had **already recorded the outcome in writing** at J-507 (*"M-RP-WINSTATE stays ⏸️ → absorbed by the §4 UI-state store at M-RP6.1k"*); this makes it binding.

**§9 is AMENDED, not overturned — and the amendment is a demotion, not a deletion.** What **survives verbatim** and becomes the *layout key's own* internal rule inside the one store: **`widgetId` is the durable identity** (the display name is a mutable label) · **drop nodes with unknown `widgetId`** · **re-inject missing `system` widgets** (W-13 — a saved state can never lose the Composer) · **`version` bump + migrate for schema changes only**. What is **superseded**: the two **filenames**, the **five layout verbs**, and *"the layout manager is itself a widget"* (under D-112/§3.2 that is **content inside Settings**, not a surface of its own).

**🔑 THE CARVE-OUT, AND IT IS THE REAL CONTENT OF THIS DECISION. §4 inherited J-499 D2's rule — *Rust persists an opaque blob and never learns the node shape* — and that rule CANNOT hold uniformly, because §4.2's clamp is MANDATORY.** Only Rust can read the **monitor work area**; only Rust can **apply a window rect before the webview exists** (apply it later and the window visibly jumps). **A clamp Rust cannot perform is not a clamp.**

**→ One file, two halves, and the line is principled rather than pragmatic:**

| half | form | why |
|---|---|---|
| **`geometry`** | a **typed Rust struct** (`x`, `y`, `width`, `height`, `maximized`) | **only Rust can do it** — the OS-window domain. Rust must *understand* it to clamp it. |
| **everything else** (`layout` today; shelf / collapsed / theme / room as their sources land) | an **opaque `serde_json::Value`**, round-tripped verbatim | the descriptor type stays in **exactly one place (TS)**. Rust never parses it → **zero D-067**. |

**The split is the *original* J-499 rule applied honestly, not an exception to it.** J-499 rejected `get_layout` because it would have forced Rust to **duplicate a type the webview owns**. Geometry is the **opposite case**: it is a type **Rust already owns** and the webview *cannot*. **Rust owns what only Rust can do; Rust stays blind to what the webview owns.** *(A rule that is stated once and applied without asking which side of it each thing falls on is how a rule becomes a superstition.)*

**Free consequence, and it is why the blob is the right shape:** an opaque value **preserves unknown keys**, so a key added by a later milestone survives a round-trip through an older binary, and every future key is **additive with no Rust change**.

**Verbs:** `get_ui_state` / `set_ui_state` — the shipped `get_substitutions` / `set_substitutions` shape. The frontend `loadLayout()` D2 seam swaps its **body**, never its call shape (which is exactly what it was written for at J-499).

**Binding:** no milestone may create a second UI-persistence file. **Geometry is UI state, never user config** — it does not enter `xgen-client_config.toml` (§4.4), and it is therefore **not subject to D-101's clean-slate-on-start** (which wipes the *config* only). *The UI-state store is the project's first deliberately persistent user-facing state — that is a feature, and it must be stated rather than discovered.*

**Relationship:** D-067 (the drift this prevents) · D-101 (clean-slate — config only; the store is out of scope, deliberately) · D-103 (the layout descriptor whose persistence this is) · D-107 (frame chrome outside the descriptor) · **D-115** (the unit) · M-RP-WINSTATE (**absorbed** — see D-115).

---

## D-115 — Window geometry is stored in PHYSICAL pixels, and the restored rect is clamped to the monitor work area

**Date:** 2026-07-12 · **Layer:** Client UI persistence · **Ref:** D-114 · **Journal:** J-510 (measured J-495, J-498 — N-092b) · **Milestone:** M-RP6.1k (**absorbs M-RP-WINSTATE**)

**M-RP-WINSTATE's deciding criterion fires here, and it was written down at J-498 precisely so nobody would re-argue it:** *"At kickoff, did the widget grid produce a persistent UI-state store? **YES → B** (own it: geometry becomes ~five keys in the existing store; no new dependency, one lifecycle). **NO → A** (`tauri-plugin-window-state`)."* **It did. → B.** **No `tauri-plugin-window-state`.** M-RP-WINSTATE **ceases to be a milestone** and becomes a facet of the D-114 store. *(The criterion worked exactly as designed: a question answered by evidence rather than re-litigated by whoever happened to be in the seat.)*

**The unit question, settled (N-092b, and it was measured twice, not reasoned):** the Tauri window config is applied in **physical** px — J-495 (900×600 config → 720×480 CSS) and J-498 (1240×1080 → 993×865 CSS at DPR 1.25).

**Decision: store PHYSICAL pixels.** Three reasons, and the first is decisive:

1. **A rect can only be compared to a monitor work area in the same unit**, and Tauri's `work_area()` is physical. **A logical rect makes the mandatory clamp unimplementable** — the clamp is not a nicety we could trade against tidiness.
2. The shipped `tauri.conf.json` `width`/`height` **already mean physical px**, so nothing changes meaning mid-flight.
3. Restore happens **before the webview exists**; there is no DPR to convert with at the moment it is needed.

**Mandatory, and both must be EXERCISED rather than asserted (the N-095 DoD shape):**

- **Clamp, don't refuse (§4.2):** if the saved rect **intersects no current monitor's work area** → **discard the geometry, fall back to default size + centre**. The unplugged-second-monitor case: a rect saved on an ultrawide must never throw the window off-screen on a laptop. **Verified by writing an off-screen rect into the store and launching** — not by reading the branch.
- **A missing / corrupt / schema-stale store falls back to defaults, never to a blank window and never to a blank centre.**

**⚠️ N-095's DoD MOVES from M-RP7.3 to M-RP6.1k, and this is not a scope grab.** N-095 pinned *"a missing/corrupt/schema-stale layout falls back to `DEFAULT_LAYOUT`, never a blank centre — exercised, not asserted"* to **M-RP7.3**, on the correct grounds that `loadLayout()` **could not return null** and the guard would have been an **unreachable branch in a closed milestone** (the same argument that kept the `tabs` branch out of renderer A). **M-RP6.1k is the milestone that makes it reachable** — it is where `loadLayout()` stops returning a constant and starts parsing a real file. **Leaving the DoD at 7.3 would mean 7.3 closing a hole 6.1k opened.** *A deferral is valid only as long as its premise holds; when the premise dies, the deferral dies with it — it does not quietly inherit a new one.*

**Consequence:** the `1240×1080` first-launch default is **now genuinely a first-launch default** — remembered geometry overrides it. Do not tune it.

**Relationship:** **D-114** (the store this is a key in) · N-092b (the measurement) · N-095 (the fallback DoD, relocated here) · M-RP-WINSTATE (**absorbed, ⬛ SUPERSEDED**) · M-RP8 (frameless + custom title-bar — changes *how* the window is dragged, **not** what is stored).

---

## D-116 — The dock rearranges; it never joins. A target tile is an ADDRESS, not a container

**Date:** 2026-07-13 (Joe-locked; designed at the dock-engine Phase-0 walk, 2026-07-12)
**Layer:** UI / client — the dock engine (renderer B), M-RP7 arc
**Reference:** `docs/xgen-dock-engine-phase0.md` §2/§3 · `ui/docs/xgen-region-dock-model.md` §3 (D-103) · J-514

### Joe's constraint

> *"to drag and drop one region on another is just for rearranging purpose. never mixing or joining."*

### The decision

**A region is NEVER put inside another region.** In a space-filling tree there is no empty space to drop *into* — every pixel already belongs to a tile. The tree's only vocabulary is **relative**: *above that one · below that one · left of that one · right of that one.*

> **The drag verb is not "put A into B". It is "insert A here, and *here* is expressed as an EDGE of B."**

**The target is a LOOKUP KEY, not a parent. It receives nothing.**

**This is why drag-to-dock exists at all:** fold changes one tile's size; the splitter changes two tiles' proportions; **neither can say *"I want the composer at the top."*** That sentence needs a **destination**, and in a tree the only destination that exists is *next to something*.

### What follows — and every consequence is SUBTRACTIVE

- **NO centre drop-zone.** Every drop-zone is an **edge band**. The centre of a tile is **inert**, not a target.
- **NO tabs are ever produced**, and no tab strip is built. **Fold is already the stacking mechanism, and it is the better one:** four folded stripes in a column is a tab strip lying on its side — except **several can be open at once** and **every label is always visible**. ***Tabs is what you reach for when you don't have fold. We have fold.***
- **NO docked/undocked mode.** A mode was only ever needed to discriminate split-vs-tab. No tab → no discriminator → no mode. *(And it was never a field anyway: **a region is docked iff it appears in the layout tree** — a **query**, not a flag. Two sources of truth for "is this docked" is a **D-067** drift surface.)*
- **NO `M-RP-ROVING` prerequisite.** A tab strip would have been the **5th** independent roving-tabindex implementation (D-069's four-recurrence bar already met). **It is not built, so the extraction stays filed and stays out of this arc.**

### The door stays SHUT, not LOCKED

`types.ts` **keeps typing `tabs`** and `resolve.ts` **keeps dropping it with a DEV warn**. **Zero cost, zero schema change if it is ever wanted.**

> **⚠️ If a future milestone reaches for tabs, it RE-OPENS THIS DECISION EXPLICITLY — it does not arrive as a rider on a drag milestone.**

### ⚠️ One argument in the Phase-0 was CORRECTED, and the decision was NOT

The Phase-0 §2 argued *"in a space-filling tree there is no empty space to drop into."* **At J-514 fold was found to create holes** (N-111), so that sentence is now only *mostly* true.

**D-116 is untouched by that finding, because its ground is Joe's constraint, not the geometry.** The rhetoric was corrected; the decision stands. **And the finding reinforces one clause:** **a hole is INERT — it is NOT a drop target.** A target tile is an **address**; **a hole has no address.** Want a tile there? **Drop on the EDGE of the tile above it.** *(If drops into holes were allowed, we would have quietly built free 2-D placement and retired the tree — which means retiring D-103's descriptor, not extending it.)*

### ⚠️ What this decision does NOT settle

**The fold axis.** M-RP7.1 shipped `collapsed` as a boolean with the axis **derived from the parent split**. **Joe superseded that design at J-515** — the axis becomes the **user's choice**. **The drafted `D-117` was NOT locked**, on Joe's word: ***"honestly i have to see it in practice."*** → **the fold decision enters this file only after `M-RP7.1b` is built and looked at.** *A decision locked for a design its author has said he needs to see first is a prediction wearing a decision's clothes.*

**Relationship:** **D-103** (the descriptor this constrains) · **D-067** (the drift surface a docked/undocked flag would create) · **D-069** (the roving bar this decision keeps from being met a 5th time) · **N-111** (the hole finding, which corrects §2's argument and not this decision) · **M-RP7.4 — drag to dock: grip, edge bands** (where this becomes code).

---

## D-118 — The plugin package: one zip, one root manifest — the universal distribution unit for all plugins

**Date:** 2026-07-16 · **Layer:** Plugin / module architecture, distribution + install path (spans node + client, all delivery kinds) · **Ref:** D-085, D-112, D-113, D-103, D-102, W-12/W-13 · **Journal:** J-531 · **Code:** none (architecture + naming convention; binding on the plugin manager, M-RP-SETTINGS, and the first `service`/`packaged` plugin).

**Decision.** Every out-of-tree plugin — for **either** app (`host = node` | `client`) and **any** delivery kind (`service` | `packaged`) — is distributed and installed as **one package: a `.zip` with a manifest at its root.** One package = one plugin. The delivery / host / surface / kind axes (D-112) are **fields in the manifest**, not separate on-disk shapes; the archive is the single uniform unit the plugin manager enumerates, trust-badges, installs, and removes.

**The manifest is the spine, and it is readable without executing or unpacking-to-live.** Enumerate, trust-badge, show-in-manager, and route-by-host all happen by reading the manifest entry *inside* the zip — nothing lands on disk as executable, and nothing runs, to know what a package is. The manifest declares `host`, `delivery`, `surface`, plugin id, semver, author, the trust-relevant fields, and **enumerates any secondary files** the plugin carries. “Works with secondary files” is not an exception — it is the manifest declaring them, and they travel **inside** the one archive, never loose beside it (containment, the D-112 shape: a plugin owns its files the way a host owns its `WidgetMount[]` — declared, not ambient).

**🔒 D-085 is UNCHANGED — a zip is transport, not a load path.** Packaging makes distribution uniform; it does **not** create a new way to run code in the key-holder's address space. A `service` package still **spawns out-of-process** and talks over the pipe; a `packaged` UI still runs in the **no-key sandbox** (and does not load until the S-1…S-6 floor ships — D-113/S-7). Nothing in a zip is ever `dlopen`ed into the node. **`compiled` plugins do NOT live in a package on disk** — they are statically registered in the binary (D-085), so every zip in `plugins\` is `kind: custom` by construction; a shipped `compiled` reference may be *presented* through the same manifest shape for uniformity, but it is baked in, not dropped in.

**Two-tier location** (from the app-vs-user lock, same session): `[app_root]\plugins\{client,node}\` = bundled / optional, read-only, updated via app update · `[userdata]\plugins\{client,node}\` (e.g. `%LOCALAPPDATA%\XGenProtocol\...`) = user-installed, writable — what the manager installs into. **Install = verified unpack into the plugin's OWN boundary** (a per-plugin subfolder `<plugin-id>\`, or read-on-demand from the zip — a mechanism choice deferred); one plugin's files never mix with another's.

**Discovery ≠ loading (the free half).** Scanning `plugins\` and enumerating from manifests touches no address space and is **not** gated by S-7. So the plugin manager (M-RP-SETTINGS) can show a real `plugins\` inventory — with enable / disable — *before* the sandboxed-loading half exists; actual loading of `packaged` bundles switches on when the S-1…S-6 floor ships.

### Naming convention (filenames are labels, not trust descriptors)

**`pg{c|n}_<plugin-id>_r<YYMMDD>.zip`**
- `pgc` = plugin · client · `pgn` = plugin · node (the `host` axis; also the routing hint for which folder it belongs in).
- `<plugin-id>` = kebab-case plugin id.
- `r` = release **channel** (single letter, deliberately — `d` dev / `b` beta fall out later with no redesign) · `<YYMMDD>` = release date, a human label.
- Example: `pgc_widget-grid-background_r260716.zip`.

**No ui-vs-system token in the filename** (Joe, this session): a `system` plugin (D-103 `kind`) is `compiled` + non-removable (W-13) and **never appears as a loose package**, so the package namespace is custom-only by construction — there is nothing to distinguish; and the host badge is already carried by `pgc`/`pgn`. **A filename is a human label + a routing hint — never a trust or capability descriptor** (anyone can rename a zip). Everything trust-relevant — `delivery`, `surface`, `kind`, semver, author, signature — lives in the **manifest inside the zip**, the authoritative verify-from source. The date in the name is a human label; the manifest's semver disambiguates two same-day releases.

**Scope.** Architecture + a distribution / naming convention. **No code.** Binding on the plugin manager, `M-RP-SETTINGS`, and the first `service`/`packaged` plugin (which is where the manifest schema itself gets proven — deliberately not specified here). **Unaffected:** the two `compiled` custom-widget exemplars (grid-background plate + connection-stats) touch no disk package at all.

**D-117 stays reserved** for the fold axis (drafted-not-locked, awaiting M-RP7.1b); this decision takes **D-118**.

**Relationship:** **D-085** (static-not-dynamic loading — this leaves it fully intact; zip is transport) · **D-112** (the three axes — this makes host / delivery / surface manifest *fields* over one package) · **D-113 / S-7** (the sandbox floor that gates `packaged` *loading*, not *discovery*) · **D-103 / W-12 / W-13** (system-vs-custom — why every package is custom) · **M-RP-SETTINGS** (the plugin manager that enumerates / installs / removes packages) · **M-RP-PLATE / connection-stats** (the `compiled` exemplars this does *not* touch).

## D-119 — The runtime install path for compiled custom widgets: a reactive registry, an available/installed split, and per-device install-state read before the layout

**Context.** M-RP-CONNSTATS (J-533) built the first `kind:'custom'` widget and, with it, the runtime **install → dock → uninstall** path that did not exist — the registry was a static `const` (`CLIENT_PLUGINS`) with no runtime mutation. This decision records the pattern the exemplar proved, because **M-RP-SETTINGS and every future custom install reuse it** (it recurs — it clears the D-069 arc-local bar).

**The path has four parts, and each is load-bearing:**

1. **A reactive registry, not a static const.** The active plugin set is `[...CLIENT_PLUGINS, ...installed customs]`, derived reactively from a `$common` `$state` installed-set (`installed.svelte.ts`). `widgetRegistry` / `bgWidgets` / titles became **pure builders** over that set (`buildWidgetRegistry` etc.), so install/uninstall re-derives them — one source, several readers (N-096). System rows stay non-removable (W-13); only customs install/remove. *(Svelte 5 caveat, N-realisation: a `$state` `Set` does not react to `.add`/`.delete` — reassign a fresh `Set`.)*

2. **An available/installed split.** `AVAILABLE_CUSTOM` (an in-tree catalogue of `compiled` customs) is distinct from `CLIENT_PLUGINS` (the system rows) **and** from the **installed set** (which subset is active). “Install” of a `compiled` widget is **register + inject a leaf**, NOT a code loader (**D-085 intact** — a zip is transport, `compiled` is baked in, nothing is `dlopen`ed); “uninstall” is **deregister + remove-leaf**, where **remove-without-blanking IS the existing collapse-degenerate** `move` already relies on (no new algebra — `insertLeaf` + a total `removeRegion` wrapper over `removeLeaf`).

3. **Boot-order: register before the layout resolves.** The installed-set is hydrated and its customs registered **BEFORE `loadLayout`** runs, so a persisted custom leaf finds its widget instead of being W-13-dropped. This ordering is not cosmetic — verified both ways: `droppedCount:0` when correctly hydrated, `droppedCount:1` (clean drop, no blank, shell present) when a custom leaf is unregistered. An unregistered leaf degrades honestly; it never blanks the grid.

4. **Per-device persistence in the SESSION bag.** Which customs are installed lives in `session.installed: string[]` in the UI-state store — **per-device** (the **J-503** truth: a `compiled`/`packaged` plugin's availability is per-device *arrangement*, not synced config), via the **N-107** per-key merge (geometry stays Rust's; writes even `[]`), **zero Rust** (the opaque-blob path — Rust never learns the shape, the `layout`/`locked` precedent). No `Layout` schema change (`version` stays 3 — an install is a session key, not a layout field).

**Scope.** The **`compiled`** custom path, proven end-to-end on `connection-stats`. The **`packaged`/`service`** install path (out-of-process, sandboxed) is a DIFFERENT mechanism — it rides **D-113 / S-7** (the sandbox floor gating `packaged` *loading*) and **D-118** (the package). This decision is the in-tree compiled register-and-inject path only.

**Relationship:** **D-085** (static-not-dynamic loading — fully intact; install = register+inject, not dlopen) · **D-103 / W-12 / W-13** (system non-removable, custom install/removable; unknown leaves drop, never blank) · **D-112** (the three axes — host/delivery/surface) · **D-114 / N-107** (uistate per-key persistence) · **D-116** (the leaf primitives this exposes come from `move`, whose ground is Joe's address-not-container constraint) · **D-118** (the package — the packaged path this does not touch) · **M-RP-SETTINGS** (the plugin manager that reuses this path for the real install/uninstall UI — the `[Remove]` action row, M-RP6.1m).

## D-120 — The plugin settings mechanism: component-per-plugin, hosted in the Settings content pane

**Date:** 2026-07-17 · **Layer:** Plugin / settings UI (client; the widget-tier hosting pattern) · **Ref:** D-112, D-102 (widget tier), D-119, W-3 / W-12, surfaces §3.2, Ch6 §6.8.2 / §6.8.5 · **Journal:** J-534 (D-B locked) · J-539 (design-lock, D-120 reserved) · J-540 (built + verified) · **Code:** `5f4a6fe` + `8b7ca1a` (M-RP-SETTINGS Leg C).

**Decision.** A plugin that has settings ships **its own settings component** (`PluginDescriptor.settingsComponent?`); the Settings modal **hosts it in the content pane**. The declarative `settings_schema` auto-render (Ch6 §6.8.2 / §6.8.5 — *“rendered automatically in the module list settings panel”*, **zero lines ever written**) is **superseded, not built** — *“it does not need to be yet another widget system”* (Joe). This resolves the **J-513 settings-mechanism collision** in favour of **B** (component-per-plugin) over **A** (declarative schema). It is the shipped widget-tier pattern: a widget hosts other content (surfaces §3.2 — content in a host is not a surface), exactly as `substitutions-editor` (M-RP4.3, the first widget) already did.

**Realized (Leg C).** The Leg-B `[settings]` button, greyed-for-all by construction (`disabled: !p.settingsComponent`), lights itself the moment a descriptor carries a component — **no `plugin-list` change**. `settings-dialog` generalised its Leg-B drill-in (`detailId` → `drill = {id, mode: 'info' | 'settings'}`, the reuse locked at J-539), intercepts the `settings` verb **locally** (never forwarding it to the shell — `app_client.handlePluginAction` untouched), and generic-mounts `{@const C = plugin.settingsComponent}<C/>`. **W-3 forces the settings component into `$common`** and its live value into a `$common` store (the shell mirrors it into `backgroundLive` + persists it): a `$common` widget cannot import a shell store. `grid-plate` is the first tenant (its backdrop *is* its setting; the B2 painted value, D-B proved end-to-end).

**Scope.** The **client** settings-hosting mechanism. Whether a `host:'node'` module's settings surface the same way rides `M-RP-PLUGINS-NODE`. The `settings_schema` path is not merely unbuilt — it is **superseded**; a future declarative renderer would need a fresh decision reversing this.

**Relationship:** **D-112** (the three-axis plugin taxonomy — this answers the question D-112 never asked: how a plugin's settings get DRAWN) · **D-102** (the widget tier — settings-as-hosted-content is that pattern) · **D-119** (the install path — the `[uninstall]` sibling verb) · **W-3** (why the component + store are `$common`) · **surfaces §3.2** (content in a host is not a surface) · **Ch6 §6.8.2 / §6.8.5** (the superseded `settings_schema`) · **M-RP-BACKDROP** (the grid-plate backdrop's full type menu — the first tenant's follow-on; type 1 = solid/gradient, Joe 2026-07-17).

---

## D-121 — Every question and recommendation is examined through three named lenses first: user-visible impact, then tier consequence, then resource cost

**Date:** 2026-07-19 · **Amended:** 2026-07-26 (lens 3 added — see "THE THIRD LENS" below) · **Layer:** Project-wide working principle (applies to Chat Claude, Clair, every runbook, every design walk, every option list put to Joe) · **Ref:** D-065 (honest behaviour over polite), D6 (never say sent when it is not), N-091 (no invented data on screen), **D-093 (universal E2E · Retained-T4 durability floor · no shared copy across erasure-fate)**, "honest longer work over fast shortcuts" · **Journal:** J-559, **J-591** · **Code:** none — this is a discipline, not a mechanism.

**Decision (Joe, 2026-07-19; third lens added by Joe 2026-07-26).** Before a question is put to Joe, and before any recommendation is made, it is examined through **three additional lenses, in this order and stated explicitly**:

1. **User-visible impact** — what does a person using XGen actually see, feel, or come to believe as a result? Stated **per option**, not once for the question.
2. **Tier consequence** — what does it do under the **auth-tier / retention model**? See the four questions below.
3. **Resource cost** — what does it drain? Build time, bundle size, maintenance surface, test surface, future work created.

Other views — architectural elegance, symmetry, tidiness, implementation convenience — are **tertiary**. They are still real and still recorded; they simply do not lead.

---

### 🔑 THE THIRD LENS — TIER CONSEQUENCE (added 2026-07-26)

**Four questions, asked per option:**

1. **Does crypto-shred remain a real guarantee?** (D-093 clause 1 — universal E2E, no protocol escrow, the node content-blind.)
2. **Does a T4 durability floor survive this?** 🔑 ***A durability floor with one copy is not a floor.*** Retained (T4) means *do not drop these ciphertext bytes*; an arrangement that leaves exactly one copy on one machine does not deliver it.
3. **Whose tier governs — and is that decided DELIBERATELY or BY ACCIDENT?** Where two parties at different tiers share a record, something decides whose obligations apply. **If the answer is "whoever acted first", that is not a decision, it is a race** — and compliance obligations settled by click order are indefensible.
4. **Is one party's erasure-fate silently imposed on another?** This is **D-093 clause 3 generalised** — that clause forbids one physical blob copy shared across differing erasure-fates *precisely because one record's policy would silently override another's*. The same shape recurs at conversation, Space and deployment scale.

**⚠️ "NO TIER CONSEQUENCE" IS A LEGAL AND EXPECTED ANSWER**, exactly as "no user-facing impact" is. Most questions — tooling, probes, records, widget layout — have none. **Say so plainly and move to resource cost.** *A manufactured tier rationale is as bad as a manufactured UX one.*

**⚠️ WHY IT RANKS ABOVE RESOURCE COST.** A tier breach is **a constraint violation, not an expense** — it cannot be bought off with more build time, and it is often invisible until the moment it matters. **Constraints are checked before costs.** 📌 *The ordering within the three is Chat's reading of Joe's "among user view and resource drain"; the lens itself is Joe's. Flipping 2 and 3 is his to do and changes nothing else.*

**🔑 THE EVIDENCE THAT PROMPTED IT, AND IT IS EXACT (J-591).** Joe asked Chat to assert the four DM-hosting options (H1 race · H2 deterministic host · H3 race + discovery · H4 bilateral replication) **against T4**. The pass did three things no other lens had done:

- ⚠️ **It invalidated Chat's principal objection to H4.** Chat had costed bilateral replication as *"two operators see the content instead of one"* — **false under D-093 clause 1: the node is content-blind at every tier**, so the number of nodes holding ciphertext does not change who can read it. *The objection had been stated twice and weighted heavily.*
- **It supplied the decisive positive argument**, which neither user-impact nor resource cost could reach: **T4's durability floor needs more than one copy, and single-homing provides exactly one.**
- **It exposed that single-home settles a compliance regime by accident of click order** — under H1 the retained-vs-erasable question for a whole conversation is answered by whoever clicked first.

⇒ **H4 was locked on the strength of that pass.** *The user-impact and resource lenses had been applied and had produced a defensible but wrong lean; the tier lens reversed it.* 📌 **Same shape as J-559, one lens further out: the argument was not wrong, it was answering a question that ranks below the one that decides.**

---

**⚠️ THESE ARE ADDITIONAL LENSES, NOT A REPLACEMENT.** Every existing rule stands **unchanged and undiminished**: D-065, D6, N-091, D-110 / D-111, D-067, D-093, the no-anonymity core, GDPR / right-to-be-forgotten, wire-format discipline. **D-121 adds a mandatory pass; it does not create a trump card.**

**⚠️ A COLLISION IS DISCUSSED, NEVER SILENTLY TRADED.** Where the best answer by user-visible impact conflicts with an existing rule — above all anything touching **identity, the wire, or anything irreversible** — the collision goes to Joe **unresolved**, named as a collision. *A rule that can be outranked by an appeal to user experience is not a rule, and the trades this project exists to refuse are exactly the ones that would feel good to a user in the moment.*

**⚠️ "NO USER-FACING IMPACT" IS A LEGAL AND EXPECTED ANSWER.** Many questions are purely internal — tooling, harness, probe design, records. **Say so plainly and decide on resource cost.** *A manufactured user-experience rationale is worse than none, because it launders an internal preference as a user's interest.* At J-559 the dev-bridge question was exactly this: `import.meta.env.DEV`-guarded and verified absent from a production bundle, so no user ever sees it either way — and a UX story could easily have been invented for it.

**Why Joe asked for this, recorded because the reason governs the application.** Joe locks architecture but **cannot independently verify most of the technical claims put to him**. User-visible impact and resource cost are the two axes he **can** judge. Stating them explicitly is what makes a recommendation **checkable by Joe** rather than trusted blind. *The lens is a trust mechanism, not a philosophy of design.*

**🔑 THE EVIDENCE THAT PROMPTED IT, AND IT IS EXACT.** At J-559, Chat recommended the Leg-D2 composer read the selection bus for its room, on a sound architectural argument (*do not lift a deliberate component-local workaround into a shared store*). **Joe asked what it would do to the user's experience.** Reading `stream-panel.svelte` then showed **R5 latches the room** (`latchedRoomId`, `effectiveRoomId` with a stale-latch guard) — so a bus-reading composer would **grey itself out while the user was still looking at the conversation**, on ordinary navigation, with nothing on screen explaining why. **The recommendation flipped.** *The architectural argument was not wrong; it was answering a question that ranks below the one that decides.*

**Consequence for Phase-0 — the part with teeth.** The user-visible consequence must be **GROUNDED**, not reasoned: read the code, run the probe, state what the user sees. *A principle that says "decide by user impact" without a step that measures user impact merely relocates the guessing.* At J-559 the answer did not exist until `stream-panel.svelte` was actually read.

**Claude's honest boundary, stated so it is not mistaken for expertise.** **Resource cost is measurable** — module counts, bundle bytes, test and maintenance surface — and Claude should measure it rather than estimate it. **User experience is only ever INFERRED**: Claude never observes a user. Claude's user-impact claims are inferences from code and from this project's stated values, and **on many such calls Joe is the better judge** — which is precisely why they are stated for him rather than acted on silently.

**Relationship:** **D-065** (honest behaviour over polite — D-121 is its procedural form: the honest answer is found by asking what the user ends up believing) · **D6** / **N-091** (both are user-visible-truth arguments that predate the rule naming them) · **"honest longer work over fast shortcuts"** (which already subordinates resource cost, and is why cost ranks second rather than first) · **D-071** (subsystem audits precede dependent milestones — the grounding pass D-121 now also loads with a user-impact question).

## D-122 — "Window" is a loose umbrella; "modal area" and "separate window" are the fixed terms; and display form is decided in situ, never inherited

**Date:** 2026-07-22 · **Layer:** UI vocabulary + appearance/state architecture · **Ref:** D-112 (surface axis), W-12 (a widget has at most one surface), N-100 (locked dock vocabulary), D-A in `docs/xgen-settings-phase0.md` (Joe, 2026-07-16), Ch6 §6.8.3 (the April origin), locks #8 and #11 in `tasks/M_RP6_3_COMPOSER.md` §9.11.3 · **Journal:** J-571 · **Code:** none — this is a vocabulary and a deferral.

### The three terms

| term | status | means |
|---|---|---|
| **window** | **loose umbrella** | a new area presenting information not displayed before it opened. **Says NOTHING about mechanism.** |
| **modal area** | **fixed** | the in-DOM overlay — About, Plugins, Settings. **The default mechanism.** |
| **separate window** | **fixed** | its own OS-level window. **Deferred**; requires a named reason. |

**Both fixed terms are lifted from existing records, not invented.** *"standalone modal area"* is Joe's own D-A wording (2026-07-16); *"a full separate desktop window"* is Ch6 §6.8.3's own sentence (April 2026). ***Nothing migrates except the vocabulary table — the concepts were always right, only the shared word was overloaded.***

### ⚠️ THIS VOCABULARY NAMES MECHANISMS; IT DOES NOT ASSIGN THEM

**No surface's display form is locked by these terms.** Naming a mechanism is not choosing one. Settings being a modal area today is a fact **about Settings**, not a property of the word.

**🔒 Joe reserves the right to change the display form of any information at any time.** The UI environment is still crystallising; the shapes recorded here were not visible one or two months ago. *A vocabulary that outlives the things it names is doing its job; one that freezes them is doing the opposite.*

### 🔑 DISPLAY FORM IS DECIDED IN SITU, NOT READ OFF AN OLD RECORD

**When a surface is built, its display form is decided in front of the thing, against the UI as it exists that day. Minimum: re-open it. Never inherit it silently.** When the Auth plugin arrives, Ch6's *"separate desktop window"* is **history, not instruction.**

**Why this is forced rather than merely preferred:** a display-form decision is a **[👁️ PERCEPTION]** call, and those cannot be made from records at all. Proven twice on 2026-07-22 — lock #5 was unfalsifiable until someone looked at a real screen, and three typeface variants were judged invalid because the thing on screen was not the thing being judged (J-570). ***You cannot look at a document.***

**The record's REASONING survives even when its CONCLUSION does not.** Ch6 does not only say *separate window*; it says why — *a module whose UI is too substantial to be a widget*, wanting *its own independent lifecycle*. That reasoning is still a useful input years later even if the answer flips. ⇒ **Conclusions are re-opened; the WHY is inherited.** Otherwise every re-discussion starts from zero, which is expensive in the other direction.

**⚠️ SCOPE — THIS CLAUSE COVERS DISPLAY FORM AND NOTHING ELSE.** For the wire, identity, the no-anonymity core, GDPR/right-to-be-forgotten, and anything irreversible, **records remain BINDING** and re-deciding in the moment is precisely the failure mode this project exists to prevent. *"Decide in situ, not from old records" is a sentence that would be dangerous if it escaped its category.*

### Consequences for lock #11

**⇒ Lock #11 ("N windows, one device") reads at VIEW scope and is MET.** Any number of mounts — tiles, panels, modal areas — read one `$common` store and cannot disagree. It falls out of lock #2 exactly as written.

**⚠️ THE BOUNDARY, RECORDED BECAUSE IT IS THE PART THAT WOULD BITE SILENTLY.** The consistency claim **does not extend to a separate window**. Each Tauri webview is its own JS context, so module-level `$state` is per-window, and **lock #8 makes the echo session-mortal and never persisted**. Two separate windows would hold **two independent echo stores by construction**. 🔑 *Locks #8 and #11 are contradictory at separate-window scope, and nobody had put them side by side.* Sharing the echo across separate windows requires promoting the store **out of the webview** — a protocol/Rust arc, not a UI one.

### Why deferred rather than refused

The separate-window want is real (a room on a second monitor cannot be served by a modal area). It is deferred because **it is not the blocker for what was actually asked**: two rooms side by side does not need windows, it needs **per-view room binding** (`tasks/M_RP_VIEW_BINDING.md`, N-159). ***Nothing is wasted by waiting — the binding work is a prerequisite for the separate-window case anyway.***

### ⚠️ Still open, NOT decided here

- **A consistency rule.** If every surface picks its own mechanism, About / Plugins / Settings may diverge and feel like three applications. Proposed but not locked: **same mechanism unless a named reason**, so divergence is always a recorded decision and never an accident.
- **The Auth Module.** Ch6 names it the first customer for a separate window. ⚠️ **The identity reasoning may point the opposite way:** a separate window is *harder for a user to tie back to the app that spawned it*, which is exactly the property a credential-phishing surface would want. **An in-DOM modal area the user cannot detach may be safer.** Ch6 chose in April without that consideration on the table. **Identity-adjacent ⇒ Joe's, and per this decision it is re-opened in situ when the plugin is actually built.**

## D-123 — The seat division stated as a rule: Joe owns appearance and architecture, Chat owns technical execution and truth

**Date:** 2026-07-22 · **Layer:** Project-wide working principle · **Ref:** D-121 (two lenses), D-065 (honest behaviour over polite), D-074 (records travel together), D-122 (display form decided in situ, scoped) · **Journal:** J-573 · **Code:** none — this is a discipline, not a mechanism.

**Decision (Joe, 2026-07-22).** The division of seats, until now carried only in session kickoffs, is stated as a rule:

| | owns |
|---|---|
| **Joe** | **What it looks like, and what shape the system is.** Appearance, structure, taxonomy, naming, the no-anonymity core, what gets built and in what order. |
| **Chat Claude** | **How it gets done, and whether it is true.** Implementation, grounding, measurement, records, verification, cleanup, tooling. |

**⚠️ THE RULE ALREADY EXISTED AS FOLKLORE.** It lived in the session kickoff and nowhere else — *the same shape as the defects this project spent 2026-07-22 fixing*: `window` meaning two things across four documents, the J-491 fs-allow lesson filed against one consumer, the `ui/templates/` deprecation that lived only in a chat. **A rule that is only ever restated at session open is a rule that can be lost by a dropped round.**

### ① The line is about who DECIDES, not who NOTICES

Chat still surfaces appearance and architectural problems — **on 2026-07-22 that was most of what it found**: lock #5 renders Joe's full XGID on his own rows (J-569), the declared fonts had never loaded in any dev shell (J-570), W-12 blocks two rooms side by side (J-571). **None of those were Chat's to decide; all of them were Chat's to find and bring.** 🔑 ***Reporting is not encroaching, and over-reporting beats letting a finding sit because it fell on the other side of the line.***

### ② A technical decision that acquires appearance or structural consequences STOPS BEING CHAT'S

**⚠️ This is the edge that actually bites, and 2026-07-22 produced two examples in one session.** The `server.fs.allow` fix looked purely technical — it exposed that **dev and the built app were rendering different typefaces**, which is an architectural fact. The font choice looked purely technical — **the deciding argument turned out to be the reskinning surface**, which is Joe's, *and he overturned Chat's recommendation on exactly that ground and was right.* ⇒ **When a technical call carries appearance or structural consequence, it goes to Joe NAMED AS SUCH.** *A boundary that only holds while the categories stay clean is not a boundary.*

### ③ Chat proposes on Joe's side; proposing is not deciding

Per-view room binding, the vocabulary split, the `skin.css` split — all Chat's to argue, Joe's to lock. **He rejected two of those three on 2026-07-22 and was right both times** (one file is the reskinning surface; the loose-umbrella vocabulary is smaller than a five-document split). ***That is the mechanism working, not Chat overstepping.***

**⚠️ HELD HARDEST — unchanged and undiminished:** anything touching **identity, the wire, or an irreversible act** goes to Joe **UNRESOLVED and NAMED**, *even when it arrives dressed as a technical detail.* D-122's scope clause exists for this reason, and D-121's collision rule is not softened by this decision. **D-123 describes ownership; it creates no trump card and overrides nothing.**

## D-124 — Naming scope is an axis (local · per-space · global); the Self toggle is a VIEW PREFERENCE, not an identity edit

**Date:** 2026-07-23 · **Layer:** Identity vocabulary + client view state · **Ref:** D-121 (two lenses), D-122 (vocabulary precedent, scope clause), D-123 (seats), lock #5 in `tasks/M_RP6_3_COMPOSER.md` §9.11.3, `tasks/M_RP_LOCK_RECHECK.md` §11, spec 3.6.10.2 · **Journal:** J-576 · **Code:** none — this is a vocabulary and a deferral.

### Why it was needed

*"Customisable later"* (lock #5's fix, Joe 2026-07-22) was read as a UI affordance. Grounding at `7408056` says it is not: `identity.update` is a **real, signed, versioned wire message** (`xgen-core/src/identity/registration.rs` — `update_version`, `changes`, signature-verified, `is_ai` immutable, error 3041), `display_name` is validated (≤128, non-empty, no control chars) and **replicates**. ⚠️ **And the client cannot emit it** — `xgen-client/src/ops.rs` L392/404 set `display_name` at **registration only**, and `xgen-client` holds **zero** references to `sign_update`. All "rename" hits in the client are **room** renames.

🔑 *One phrase was carrying three different features, one of which is a protocol operation. That is the D-122 shape at the identity layer.*

### The three fixed terms

| term | means | state at `7408056` |
|---|---|---|
| **L — local override** | you rename **someone else**; only **you** see it | client-only, no wire. ⚠️ **zero lines exist today** |
| **S — per-space name** | you name **yourself**; **that space** sees it | ⚠️ **impossible today** — `SpaceMember` is `identity_id · role · joined_at · invited_by`, **no name field at all** |
| **G — global name** | you name **yourself**; **everyone** sees it | = `identity.update`. Protocol exists; **client cannot emit** |

🔑 **THE AXIS IS SCOPE — nobody · one space · everywhere — NOT "local vs replicated".** L is not a cheap G: **it points at a DIFFERENT PERSON.** ⚠️ And **erasure difficulty scales with scope**, which is the GDPR gradient.

### Rulings (Joe, 2026-07-23)

1. **The self surface's naming scope is G ONLY.** L belongs on the *other* person's card / member list, never on the self surface.
2. ⚠️ **"Self" is a TOGGLE, not a label and not a placeholder.** Joe: *"self will be optional. that what the setting needs. merge names in display to Self or stay various by the wish."*
   - **ON (DEFAULT, per lock #5)** — all your **own** names **merge** to *"Self"* in **your own view**, so different names in different contexts stop bothering you.
   - **OFF** — your actual name for that context renders instead (G today; S later, if S is ever built).
3. 🔒 **ONE GLOBAL PREFERENCE. NOT PER-SPACE.** Joe: *"globally is correct. if for all spaces, one could loose his mind."* 🔑 **The joke is the rationale** — the toggle exists to REDUCE the load of tracking your own names, so a per-space version would **reinvent the problem it solves.** ⚠️ Do not "generalise" it later; that is a regression, and this line is why.
4. ⇒ **It is L pointed at yourself: a VIEW PREFERENCE.** ZERO wire, no `identity.update`, no migration gate, **not an identity edit**.
5. ⚠️ **DO NOT build a name input into the self surface.** *"Customisable"* means **the toggle**, confirmed twice. Shipping a rename field without a ruling would silently choose a branch — a decision applied in code and never entered in the record.
6. **The styling verdict is ORTHOGONAL** — V3 + `#E5E5E5` (J-575) applies to the self name in **both** states.

### Where the preference lives

**PER-DEVICE** (Joe, 2026-07-23, on Chat's recommendation). It rides the mechanism that already holds view preferences — `uiStateStore`, which already persists `session.layout` across relaunch (M-RP7.5 Leg B).

**Why this and not identity-bound:** ① **no new persistence surface, no wire, and no GDPR surface for a checkbox** — D-124's *zero wire* claim stays true of the preference itself, not only of what it displays; ② the drift cost is near-zero in practice, because the default is ON, so a fresh device shows *"Self"* anyway, and a user who turned it OFF needs one click to correct it. ⚠️ **Identity-bound would have put a cosmetic preference into replicated user data**, which is the opposite of what the no-anonymity core needs to keep small.

### ⚠️ `merge`, never `collapse`

**`collapsed` is a persisted layout schema field** on leaves — including `widgetId: 'self'` — and has been **migrated twice** (`ui/client/src/layout-default.ts:91`: `version: 3`, leaf `collapsed` boolean → FoldAxis, upgraded by `migrateLayout`; `foldLeaf` / `handleFold(regionId, collapsed)` persist across relaunch).

⇒ *"collapse the self panel"* reads most literally as **fold the self dock tile**. **`merge` is the fixed term** (Joe's own word, from the ruling). 🔑 *Caught before it entered a decision record — `window` (D-122) and `self` were both caught after.*

### It is testable today, and lock #5 lands without resolving the fork

ON → *"Self"*; OFF → the registered `display_name`, which already resolves to *"Joe"* (`self-panel.svelte:43`). **Two real states, nothing new on the wire.** ⇒ **Lock #5 closes without deciding how G is EDITED.** That fork — local-cosmetic vs a real signed `identity.update` — is **later, and is not this milestone**.

### Auth uses keys only — `display_name` plays no part

`TrustAssertion` binds to `identity_id` (`xgen://pubkey/ed25519:`); `xgen-core/src/auth/**` holds **zero** references to `display_name`. ⚠️ T2–T4 claims carry `legal_name_verified: bool` — **a FLAG, not a NAME.** The protocol attests **that** a name was verified and **never carries the name**. Tiers: T1 cryptographic identity only; T2/T3/T4 TTL 365/180/90 days; space admission gated by `auth_tier` via `verify_tier` (error 3030 `tier_mismatch`).

### ⚠️ Still open, NOT decided here — both JOE'S

- **NAME TRUTHFULNESS BY TIER.** Joe's position: T1 = the name does not matter; T4 = real name required. **The scaffolding exists** (space `auth_tier` + tier claims); **the comparison does not** — nothing links `display_name` to any tier check, and the assertion carries no name to compare against. ⇒ enforcement needs **either** a new claim carrying the verified name **or** the module attesting the name rather than the fact. ⚠️ **That puts legal names on the wire** — GDPR territory, sitting on the no-anonymity core. **UNRESOLVED. Do not design around it; ask.**
- **DOES S SURVIVE NO-ANONYMITY?** One name in one space and another elsewhere is **soft pseudonymity** even with a fixed cryptographic XGID underneath: the XGID makes correlation **possible**, per-space names make it **effortful**. The question is whether no-anonymity means the identity is **verifiable** or the person is **recognisable**. **Not answered.** S is out of scope either way (ruling 1).

### ⚠️ AMENDED 2026-07-23 (J-577) — Ch2 ALREADY CARRIED THIS MODEL, AND CARRIES ONE LAYER MORE

**Found after D-124 was written**, by reading `docs/xgen_ch2_architecture.md` §*"User Representation — The Full Picture"* on Joe's pointer. **L/S/G was derived from the code; Ch2 had specified it months earlier — with a FOURTH layer and an override chain D-124 did not carry.**

| Ch2 layer | set by | seen by | lives in | D-124 |
|---|---|---|---|---|
| Global display name | B, about themselves | everyone | Public Identity record | **G** |
| Space nickname | B, about themselves, per Space | that Space | **Space membership record** | **S** |
| Contact alias | A, about B, privately | only A | Private Identity record | **L** |
| **Contact note** | A, about B, privately | only A | Private Identity record | ⚠️ **NO D-124 TERM** |

**① THE FOURTH LAYER.** The **contact note** is *"not a display name — it does not replace any label … supplementary context, displayed on demand"*. It is the field Ch2 says *makes a contact list a genuine personal address book rather than just a list of keys*. **D-124's three terms cover names only; the note is a fourth thing and is not renamed by any of them.**

**② THE OVERRIDE CHAIN, which D-124 omitted entirely.** Ch2 locks the precedence: **contact alias → Space nickname → global display name**, and *"the contact alias overrides everywhere, regardless of context."* ⇒ **L does not merely coexist with G and S — it OUTRANKS both, everywhere.** 🔑 This strengthens D-124's ruling ① rather than weakening it: L is powerful precisely because it is yours about someone else, which is why it has no place on the self surface.

**③ ⚠️ THE CORRECTION THAT MATTERS — S IS NOT "IMPOSSIBLE".** D-124 above reads, verbatim and retained:

> **S — per-space name** … ⚠️ **impossible today** — `SpaceMember` is `identity_id · role · joined_at · invited_by`, **no name field at all**

**The measurement was correct; the word was not.** Ch2 **assigns S a home** — the *Space membership record*. ⇒ **S is SPECIFIED-BUT-UNBUILT, not unimagined.** *"Impossible" describes the code and silently libels the architecture, which had answered the question already.* The open question in D-124 (*does S survive no-anonymity?*) is unchanged and still Joe's — Ch2 specifies the mechanism, not the ruling on whether to build it.

**④ CONSEQUENCE FOR `M-RP-INBOUND-NAME`.** It was filed as *XGID → display_name*. Ch2 makes it **the four-layer override chain**, with the address book as the **Private Identity record plus the resolver that walks it**. Same blocker, **materially larger scope**.

🔑 **THE LESSON, WHICH IS NOT ABOUT NAMES.** D-124 was grounded against **code** and got three layers. The **architecture document** had four and the precedence rule. ***Grounding against the code proves what IS; grounding against the spec proves what was DECIDED — and a decision record needs both.*** D-124's conclusions stand; its completeness did not.

## D-126 — Humane pubkey label: a display-only rendering of an XGID for convenience listings

**Date:** 2026-07-24 · **Layer:** client display utility (non-protocol) · **Ref:** D-088 (identity erasure = orphaned pubkey), D-121 (two lenses), D-123 (seats — appearance is Joe's), N-163 (positive control) · **Journal:** J-579 · **Code:** none yet — recorded as adopted intent, no runbook.

**Joe, 2026-07-24:** adopt a human-friendlier rendering of a pubkey/XGID for cases where listing full 65-char XGIDs is heavy and full precision is not needed — e.g. statistics listings. *"not profound but sufficient distinguishing."* Not for serious/identifying use.

### What it is

A **pure deterministic function of the XGID** — no registry, no storage, no network, nobody assigns or picks it. Same key always renders the same label, recomputed on read. It is a **different alphabet for the same number**, not a second identifier.

Two families, both in scope depending on the view:
- **Tail truncation** — last N chars of the XGID (entropy is in the tail; the `xgen://pubkey/ed25519:` prefix is constant). N=8 for a stat listing. Cheapest; already the pattern in `app.rs`/`ops.rs`.
- **Word rendering** — SHA-256 the XGID, slice into 11-bit chunks, index a fixed 2048-word list, join (`amber-falcon`). Hash-first for even spread. More readable in a table, needs a wordlist file.

### The hard limit — LABEL, NEVER IDENTIFIER

⚠️ Short and globally-unique are mutually exclusive (birthday bound: 2 words ~2k, 3 words ~90k, N=8 base32 ~millions before a likely collision). This is acceptable **only because it is a display aid read inside a bounded view** (a listing, a roster), where uniqueness need only hold within the view. If a view collides, escalate the colliding entries to a longer form.

**It MUST NOT be typeable, searchable, or usable to address anyone**, and the full XGID MUST stay reachable. An attacker can grind keys to render as any target label in seconds — so nothing may match, look up, or rest trust on the label (the PGP short-key-ID lesson).

### Open, and Joe's (appearance/structure — deferred, not blocking)

- **Canonical vs cosmetic** — one workspace-wide wordlist so every client renders a key identically (enables verbal reference: *"the amber-falcon one"*) vs client-local. Canonical makes it a protocol-adjacent concern; cosmetic keeps zero spec surface.
- **Wordlist language** — a single fixed list is English-by-default; phonetically-distinct invented syllables read cleanly across Slovak/English. A naming call.
- **Word count / form per surface** — 2 words suffices for stat listings.

These are recorded as open. Nothing is built; when a surface needs it, it is a small function plus (for the word form) a wordlist file, no protocol messages, no migration.

---

## D-125 — The utilities row is MIXED: utility buttons, toggles and indicators are different kinds and must stay distinguishable

**Date:** 2026-07-23 · **Layer:** UI vocabulary + component taxonomy · **Ref:** D-121 (two lenses), D-122 (vocabulary precedent), D-123 (seats), D-112/D-113 (plugin taxonomy), W-12, N-063 (owned popup) · **Journal:** J-576 · **Code:** none — this is a vocabulary.

**Joe, 2026-07-23:** *"utilities -> utility buttons for button forms"*, and on the self region: *"this will be epicentre / heart of whole client … all as just buttons [?]"*.

### The answer is no — and all four kinds are already shipped `core`

| kind | holds state? | clickable? | component | example |
|---|---|---|---|---|
| **utility button** | no | yes | `button` (64 ln) + `icon` (97 ln) | settings · address book · help |
| **toggle** | **yes, and shows it** | yes | `toggle` (36 ln) | the Self toggle · device-control class |
| **indicator** | reflects state | **NO** | `status-indicator` (73 ln) / `led` (58 ln) | the connection light |
| **split control** | opens a list | yes | `menu` (283 ln) or owned-popup (N-063) | a button with a chevron beside it |

All four are `use:envelope`-registered, so all four are CDP-readable. 🔑 **Nothing new enters `core`** — the utilities row is a **widget-tier composition**, not a component milestone.

### The rule, and why it is user-visible rather than architectural

**A toggle authored as a utility button loses its on/off affordance. An indicator authored as a utility button invites a click that does nothing.** (D-121 lens ①.) ⚠️ On the surface that represents the person to themselves, **a control that lies about whether it is a control** is the worst possible place for it.

⇒ **The kinds must remain distinguishable to the eye.** ⚠️ **HOW they are distinguished is APPEARANCE and therefore JOE'S** (D-123). *That* they must differ is the rule.

### Scope

**Vocabulary only.** It assigns no utility to any row, fixes no layout, and does not decide whether the row is fixed or plugin-extensible — that is FILED INTENT in `tasks/M_RP_SELF_GATE.md` §4, locking at **Phase-0**. ⚠️ If it becomes extensible it is a **plugin surface under D-112/D-113**, which is materially larger than a widget.

---

## D-127 — `identity.record` returns revoked Identities with a flag; `not_found` is reserved for erasure

**Date:** 2026-07-25 · **Layer:** protocol (client-facing identity lookup) · **Ref:** M13 Client Identity Lookup Widening, D-088 (identity erasure = orphaned pubkey), D-121 (two lenses), D-123 (seats) · **Journal:** J-584 · **Code:** none yet — M13 is PENDING.

⚠️ **PROVENANCE: Chat recommended, Joe DELEGATED ("by your recommendation", 2026-07-25).** This is a core-touching decision — it defines what the protocol says about a person — and was flagged as Joe's before he delegated it. **Recorded as delegated, not as a considered Joe-lock**, so that a later revisit reads the provenance correctly.

**Decision:** when M13 widens `identity.record`, a lookup of a **revoked** Identity returns the record **with `revoked` set**, not `identity.not_found`.

### Why

- **The DAG is permanent.** Every event that Identity signed still exists and still renders. Revocation cannot retract history; it constrains what the Identity may do *going forward*. Answering `not_found` makes all of that history **permanently unattributable** — in a protocol founded on knowing who said things.
- **"Revoked" and "never existed" are different facts.** Under `not_found` a revoked Identity, a mistyped key and a stranger's key are indistinguishable.
- **§5 revocation-on-encounter is otherwise unimplementable** — it is defined as marking a cached record on re-encounter, and there is nothing to mark if the record vanishes.

### The hard limit — DO NOT CONFLATE WITH ERASURE

🔑 **`not_found` is the correct answer for ERASURE, and must stay reserved for it.** Erasure removes a person; revocation kills a credential. If both produce `not_found`, the protocol loses the ability to distinguish a compromised key from a withdrawn human — and would be answering the project's open federated right-to-be-forgotten question by accident, in the wrong layer.

### Consequence for the address book

A cached `revoked = true` is a **historical fact that can never become wrong**. A cached `revoked = false` is only true *as of `last_seen`* ⇒ **staleness and absence must both render as UNKNOWN, never as fine.** This generalises the J-582 badge rule from one field to the whole book.

---

## D-128 — Tier-required claims: proofs by default, encryption only for a reader known at issuance, and a ceiling on what a tier may demand

**Date:** 2026-07-25 · **Layer:** protocol (Auth Module / Trust Assertion claims) · **Ref:** Appendix M §M.2 (`has_claim`, §3.8.5 check 7 required-claims gate), §M.3 `ModulePolicy`, D-121, D-123 · **Journal:** J-584 · **Code:** none — **M10 Auth Module Reference Set** territory, NOT M13.

⚠️ **PROVENANCE: Chat recommended, Joe DELEGATED ("by your recommendation", 2026-07-25),** on Joe's own concept — an Auth Tier may require specified details, carried on the visit card.

**Decision, three parts:**

1. **Tier-required claims are PROOFS, not disclosures.** The existing claim vocabulary already settles this: `tier_verified: bool`, `email_verified: Option<bool>`, and `email_hash` — *"Salted SHA-256 … plaintext never permitted"* (Appendix M §M.2). A tier may require that a property be **proven**; it may not require the **value** be published. A professional tier can demand `credential_verified: true` plus a hash — a reader learns the credential is real without learning the licence number.
2. **Encrypted claim values are permitted NARROWLY** — only where the intended reader is **known at issuance** (e.g. a regulator-readable field encrypted to one known public key). Mechanically this needs no wire change: `claims.extra` is `BTreeMap<String, Value>` and a base64 ciphertext is a `Value`.
   - ⚠️ **Per-recipient or dynamic-audience encryption is STRUCTURALLY IMPOSSIBLE here.** `claims.extra` rides **inside the signed canonical form** (Appendix M §M.3, deliberately — *"keeps it inside the signed bytes"*). Re-encrypting per reader changes those bytes and invalidates the Ed25519 signature. Selective disclosure and signed-in-place ciphertext are mutually exclusive by construction. **Do not attempt to solve this by re-wrapping.**
   - ⚠️ **Harvest-now-decrypt-later.** Identity records replicate across federated Nodes and persist. An encrypted value is published permanently with a countdown running on its cipher — a poor trade against a hash, which has no key to leak and nothing to decrypt later. Prefer hashing wherever the value need not be read back.
   - 📌 **A visible key leaks the schema.** `medical_licence` as a plaintext key is fine. `immigration_status` or `health_condition` are cases where the *existence* of the claim is itself the sensitive fact. **Not decided here** — no consumer exists yet; recorded as a known gap for M10.
3. **The protocol should BOUND what a tier may demand.** `claims.extra` is an **open namespace** — unknown keys are preserved and `has_claim` consults them — so absent a ceiling an Auth Module can make access conditional on ever-growing disclosure: *"disclose or lose your tier."* 🔑 **That is enshittification arriving through the Auth Module rather than through a platform** — the one vector the architecture does not currently close, reached by exactly the door tier-required claims opens. The ceiling's shape (proofs and hashes yes, plaintext personal data no, mirroring the `email_hash` rule) is **filed for M10**; imposing it before modules exist is far cheaper than after.

📌 **The optional/required tension resolves one level up:** choosing a tier is voluntary, so disclosure is optional at tier selection rather than per field. **A blank visit card stays legal at T1.**
---

## D-129 — `goodbye` poisons the session: a GATE on any arc that makes a session persistent across ops

**Date:** 2026-07-25 · **Layer:** client session/transport · **Ref:** J-586, D-121 (two lenses), D-071 (audits precede dependents) · **Code:** `xgen-client/src/session.rs` (`ensure_connected`), `xgen-client/src/ops.rs` (25 `goodbye` sites).

⚠️ **PROVENANCE: found by Chat's live orchestration pass (J-586); Joe delegated the TIMING ("it needs to be fixed but it is yours when").**

**The defect.** `ClientConnection::goodbye` closes the socket but leaves `SessionState.conn = Some(dead)`. `ensure_connected` reconnects **only when `conn` is `None`**, so it hands back the corpse and the next send fails with *"Sending after closing is not allowed"*. **Measured: `ops.rs` has 25 `goodbye` sites and exactly ONE `conn = None`** — the one inside `fill_from_space`.

**Why it has never bitten — measured, not assumed.** Every dispatcher builds a fresh session per command: M5/M6 CLI one-shot, and `aicontrol.rs:404` states it explicitly — *"Each arm builds its own session (matches the batch arm)"*. `fill_from_space` is the first verb **designed** to be called repeatedly on one session, so it is the first to step on the mine. It now clears on every exit path (J-586).

🔒 **DECISION — GATE, NOT A DATE.** This is **NOT scheduled now**: ① user-visible impact today is **zero** (nothing reuses a session; "no user-facing impact" is a legal answer under D-121) and ② the fix touches `session.rs` and every op's assumptions, so it needs its own arc and audit.

⚠️ **THE TRIGGER: any arc that makes a `SessionState` persistent across ops MUST fix this FIRST.** `ops.rs:30-32` names that as the intent (*"the same instance will be reused across commands in a persistent `--aicontrol` connection in M7"*) — so the trigger is live intent, not a hypothetical.

🔑 **A DATE WOULD BE ARBITRARY; A TRIGGER CANNOT BE MISSED.** Fixed early it is one function in `session.rs`. Discovered late it is 24 call sites failing intermittently, in an arc that will look like it broke something unrelated.

📌 **Shape of the eventual fix:** make `ensure_connected` detect a closed connection and reconnect, rather than trusting `Option::is_some` as a liveness proxy. Per-op `conn = None` is a workaround, correct only where it is written.

---

## D-131 — A citation proven broken is annotated at the site, never silently repaired, and investigated only when work reaches it

**Date:** 2026-07-28 · **Layer:** project-management discipline (canonical records) · **Ref:** J-607, D-065 (honest over polite), D-094 (archiving), the designation-suffix convention · **Lineage:** the annotation form was invented during M-DOC-ROADTREE Leg C and applied four times in `docs/ROADMAP.md` before it was named.

⚠️ **PROVENANCE: Joe proposed the rule; Chat measured the surface and recommended the form. DELEGATED.**

**Decision.** When a citation to a canonical record (`J-nnn`, `D-nnn`, `N-nnn`) is proven not to resolve, it is **annotated in place** with what is known and otherwise left intact. It is **not** silently repointed and it is **not** deleted. Working out what it should have said happens **when work next reaches that site**, not on discovery.

**Form — inline, at the site.** `· J-098 — never written, see J-603` · `D-030 — bare retired, see D-030a/D-030b` · `D-130 — never written`. The annotation carries **what is known**, never a guess.

🔑 **WHY ANNOTATE RATHER THAN REPAIR.** A broken citation is not an edit, it is a question — *what did this mean?* — and answering it needs the surrounding work. **Repairing on discovery turns a bounded documentation pass into an unbounded archaeology pass**, and the guess it produces is indistinguishable from a fact once written down.

🔑 **AND WHY NOT LEAVE IT SILENT.** An unresolvable citation reads exactly like a resolvable one: `D-130` looks like a decision until someone opens this file. **The annotation converts a silent false claim into a visible open question** — D-065 applied to the records themselves.

📌 **AN ANNOTATION IS SAFE TO FREEZE.** A repaired link inside a frozen archive is a claim nobody can re-check; an annotated one tells the future reader not to trust it. Archiving under **D-094 therefore does not gate on link repair.**

**The surface, measured at J-607 (2026-07-28).** 22,664 citation sites across 245 live-surface files; **474 unresolved — 2.09%** — across only **20 distinct designations**. The work is per-designation, not per-site, and it resolves to four knots: the **D-030…D-056 bare-retirement cluster** (7 designations, 110 sites, one historical renumbering pass) · **records never written** (`J-098`, `J-109`, `J-113` — 268 sites, **already investigated at J-603**) · the **J-044/J-045 collision** (12 sites, open as `M_DOC_ROADTREE.md` §8b) · **genuinely uninvestigated** (`N-092a`, `N-092b`, `N-095b`, `J-067`, `J-171`, `J-81` — 84 sites).

⚠️ **THIS DECISION DOES NOT AUTHORISE A RETROACTIVE ANNOTATION SWEEP.** Annotating 474 sites now is the same unbounded pass the rule exists to prevent. The register above **is** the record; sites are annotated as work reaches them.

📌 **D-130 IS SKIPPED, NOT AVAILABLE.** It is cited in the `CLAUDE.md` PLAY head and reserved for a decision whose wording is still open with Joe. This one takes **D-131**.
---

## D-132 — `INTERACTIVE — HANDS OFF`: driving a live xgen app is a custody transfer, requested and released, never assumed

**Date:** 2026-07-28 · **Layer:** working discipline (live verification) · **Ref:** J-609; first locked by Joe 2026-07-21 at J-568 Q1/Q2, first used in `tasks/M_RP_LOCK_RECHECK.md` §3 · **Lineage:** D-065 (honest over polite), D-123 (seats).

⚠️ **PROVENANCE: Joe's rule, locked 2026-07-21. Promoted here 2026-07-28 because it existed in four non-identical copies and none of them in a file read at session open.**

🔒 **THE DEFAULT THIS RULE INTERRUPTS.** When an xgen app is running, **Joe is in it** — exploring, checking the UI by hand. That is the normal state, not the exception. In Joe's words, the warning is *a warning for me that I have to leave full unreserved UI playground to you*. ⇒ **THE WARNING IS NOT A NOTIFICATION. IT IS A REQUEST FOR CUSTODY**, and the all-clear is the return half.

**Decision.** Any leg whose reading requires the UI untouched is marked **`INTERACTIVE — HANDS OFF`**. Before driving it, the driver posts in chat:

```
🛑 HANDS OFF — live measurement running
   App:     <client 9222 | node 9322 | sampler 9422>
   Reading: <what is being measured>
   Do not:  <the app is not to be touched at all>
   Expect:  <duration>
```

and **always**, including when the run dies or is abandoned:

```
✅ ALL CLEAR — measurement done, the app is yours again
```

🔑 **THE HANDOVER IS TOTAL, NOT ENUMERATED.** The original wording listed *click, scroll, focus the window, open dialogs*. ⚠️ **A list of prohibitions implies that anything unlisted is permitted**, which is the opposite of the rule. The app is handed over whole and handed back whole.

**Fires for:** registry counts (all seven axes, N-155) · computed style or geometry · scroll and focus legs · keystroke-by-keystroke legs · echo counts · anything with quiescence as a precondition.
**Does NOT fire for:** cargo · npm · svelte-check · git scope · files on disk. ⚠️ *Warning on those would train Joe to ignore the warning, which is worse than missing one.*

🔒 **NEVER A STANDING CONDITION.** A hands-off window is minutes, requested and released. **Forced, not preferred:** a total handover cannot be open-ended — an interruption with no stated end is indistinguishable from being locked out of your own app.

⚠️ **STATED LIMIT — this protects against Joe's hands, not against a background process.** Port checks and the quiescent-baseline rule remain the guard against everything else.

⚠️ **HARNESS LIMIT.** Dev ports are fixed (client 5173/9222), so **two clients cannot run at once**. Node and sampler are separate and can be measured while Joe is in the client. If this becomes real friction, a second dev port set is a small change — **file it, do not tolerate it**.

📌 **THE STANDING DEFAULT OUTRANKS THE CONVENTION: THE APP IS JOE'S.** If he is in it, the driver waits. *His walking through the app is the highest-yield verification this project has — three defects in one day that no automated leg caught.*

📌 **WHY IT NEEDED A NUMBER.** It lived in `tasks/M_RP_LOCK_RECHECK.md` §3 (COMPLETED — findable, not read), `CLAUDE.md`'s PLAY head (**which silently dropped both stated limits**), `docs/ROADMAP_ARCHIVE_2026-07-26.md` (ARCHIVED) and J-568. ⇒ **THE COPY MOST LIKELY TO BE READ NEXT WAS THE ONE MISSING THE LIMITS.**
---

## D-133 — The `Owes:` line: a parent doc stays ACTIVE only while it still owes the work it spawned

**Date:** 2026-07-28 · **Layer:** project-management discipline (task-doc headers) · **Ref:** J-609; first locked by Joe 2026-07-21 at J-568 Q1 · **Lineage:** D-074 (atomic canonical records); the five-status vocabulary.

⚠️ **PROVENANCE: Joe's rule, locked 2026-07-21 and quoted then as "very strict and unexpanded". Promoted here 2026-07-28; the meaning below is Joe's reading, confirmed against the record 2026-07-28.**

🔑 **THE QUESTION IT ANSWERS.** The sweep at J-568 found five task files reading `Status: ACTIVE` while the ROADMAP recorded their milestones closed — *a DoD signal that needs a human to interpret it has stopped being a signal*. Two were genuinely stale. **Two were real parents with real open work.** ⇒ `Owes:` answers *this doc's own milestone closed, so why is it still ACTIVE?* — **because it is a parent that still owes the work it spawned.**

**Decision — the format, strict and unexpanded.** A parent doc gains **one header line**: milestone IDs with short titles, separated by ` · `, **nothing else — no reasons, no dates, no parentheticals**.

```
> Owes: M-RP6.4 room-history backfill · M-RP6.7 resident pong timeout · M-RP6.8 view-latch persistence
```

- **One physical line.** *If it does not fit, the doc owns too much and should be split — the line's length is the signal.*
- **An item leaves when it closes.** The debt shrinks as the children close.
- **Leg runbooks never get one.** A leg spawns nothing; it is not a parent.
- **Not applied retroactively** — only to files a sweep touches.

🔒 **CORRECTION — AN EMPTY `Owes:` FLIPS `Status` TO `COMPLETED`, NOT `ARCHIVED`** (Joe, 2026-07-28). J-568 Q1 said ARCHIVED. ⚠️ **The clause had never fired** — all three live `Owes:` lines still carry items — so the error survived unexercised. Measured: **0 of 127 task docs are ARCHIVED** (109 COMPLETED · 9 PENDING · 8 ACTIVE · 1 DEPRECATED), the same sweep closed its own two stale docs as **COMPLETED** (J-508, J-509), and all 13 ARCHIVED files repo-wide are frozen historical records — `CLAUDE_HISTORY.md`, `JOURNAL_ARCHIVE.md`, the ROADMAP archive, `docs/backup/*`. ⇒ **A finished parent is not frozen-do-not-modify; it is COMPLETED.** *`Owes:` empty plus `ACTIVE` remains a contradiction anyone can spot* — only the flip target changes.

📌 **THE ID IS WHERE THE WORK NOW LIVES, WHICH IS USUALLY BUT NOT ALWAYS A CHILD.** `M-RP6.6-INGEST` and `M-RP-FONTS-WOFF2` are true children of their parents. **`M-RP-SKIN ConnStats row-swap` is a re-home** — the row-swap came out of M-RP6.6 and was routed to the separate appearance milestone. Both are the parent still owing the item; recorded so the re-home does not read as a mistake.

📌 **AN ACTIVE DOC WITHOUT AN `Owes:` LINE IS NOT A GAP.** Five of the eight ACTIVE task docs carry none, correctly: they are ACTIVE because **their own milestone is still in flight**, not because they are parents holding spawned work. Only a parent whose own milestone closed needs the line.

---

## D-134 — Designations are issued unique; a duplicate is repaired by lettered split, and the bare number is retired

**Date:** 2026-07-29 · **Layer:** project-wide records discipline (all designation families) · **Ref:** J-619; `M_DOC_ROADTREE.md` §4b, §4c; the `CLAUDE.md` standing-convention block of 2026-07-26 · **Lineage:** D-094 (windowing), D-074 (atomic canonical records), D-131 (broken citations are annotated, never silently repaired).

⚠️ **PROVENANCE.** §4b's collision rule is **Joe's, 2026-07-26**. Chat formalised it into a `CLAUDE.md` standing-convention block the same day, marked *⚠️ CONFIRM OR AMEND*, **and it stood unconfirmed for three days.** Joe **confirmed and amended it 2026-07-29** — the amendment is the third-and-beyond case and the framing in §1 below. **Both are his.** The evidence tables are Chat's, measured 2026-07-29.

### §1 — THE PRIMARY RULE, WHICH IS NOT THE SPLIT

🔒 **DESIGNATIONS ARE ISSUED UNIQUE. THAT IS THE RULE.** The lettered split below is **a repair mechanism for an accident**, applied **at a moment of revision** — when a duplicate is discovered by Joe or by an AI seat reviewing the record. ⚠️ **It is NOT a way of allocating designations, and nothing may be issued as `X-nnna` in the first instance.**

🔑 **STATED BECAUSE THE EARLIER DRAFT READ THE OTHER WAY.** Chat's formalisation described the notation without saying it was remedial, which invites a future reader to reach for `a`/`b` as a normal issuing path. **Joe's framing is what makes the record correct.**

### §2 — ① THE COLLISION SPLIT: THE BARE NUMBER IS RETIRED

🔒 **Where one designation was issued more than once for UNRELATED things, the copies are suffixed in the order they appear IN THE RECORD — `a`, then `b`, then `c`, and so on — and the bare number CEASES TO EXIST.** Applies identically to `J-` · `D-` · `N-` · `M-`.

📌 **THE THIRD-AND-BEYOND CASE IS JOE'S AMENDMENT (2026-07-29).** Every prior statement of this rule stopped at `a`/`b`. **Nothing guarantees a collision involves only two copies**, and a rule that stops at two would have to be re-decided the first time three appear.

🛑 **`a` MEANS FIRST IN THE RECORD, NOT FIRST IN THE FILE, AND IN A NEWEST-FIRST FILE THOSE ARE OPPOSITE.** `JOURNAL_ARCHIVE.md` runs newest-first ⇒ **the HIGHER line number is the EARLIER record** ⇒ it takes `a`. ⚠️ **Written explicitly because Chat misread exactly this on 2026-07-29** — reading *"order they appear in the record"* as file position, and manufacturing a contradiction with §4c that did not exist. **A reader who has to infer this will infer it wrong in half of all files.**

✅ **EVIDENCE — measured over `DECISIONS.md` headings, 2026-07-29. The rule was practised SEVEN times before it was ever written down, and it holds 7 of 7:**

| family | bare heading | `a` | `b` |
|---|---|---|---|
| `D-030` · `D-031` · `D-037` · `D-038` · `D-039` · `D-055` · `D-056` | **0 each** | 1 each | 1 each |

📌 **139 `D-` headings total, and the fourteen above are the ONLY lettered ones** — so the `D-` family contains no addendum case at all, and every letter in it is a collision split.

### §3 — ② THE ADDENDUM: THE BARE NUMBER SURVIVES

🔒 **A follow-on that EXTENDS OR CORRECTS an existing record takes the next free letter while the original KEEPS ITS BARE NUMBER.**

✅ **EVIDENCE — measured across `CLAUDE.md`, `CLAUDE_HISTORY.md`, `JOURNAL.md`, `JOURNAL_ARCHIVE.md`, `DECISIONS.md`, `docs/ROADMAP.md`, 2026-07-29:**

| family | bare | `a` | `b` |
|---|---|---|---|
| `N-124` | **14 refs, own block heading** | 9 | 6 |
| `M-RP2.30` | **9** | 7 | 0 |
| `M-RP2.31` | **6** | 5 | 0 |

⚠️ **EVIDENCE TYPE DIFFERS BETWEEN §2 AND §3 AND THE DIFFERENCE IS STATED RATHER THAN SMOOTHED.** §2 counts **headings** in one file, which is authoritative. §3 counts **references** across six files, because `N-` and `M-RP` designations have no single headings file; the bare `N-124` does carry its own block heading in `CLAUDE.md`, which is the strongest single piece of §3's evidence. **References are a proxy, not a census of definitions.**

### §4 — THE DISCRIMINATOR, AND WHY IT NEEDS NO NEW NOTATION

🔑 **ASK WHETHER THE BARE NUMBER STILL EXISTS.** Present ⇒ the letters are **addenda**. Absent ⇒ the letters are a **collision split**. ✅ **Measured above across all four families with no exceptions** ⇒ the two conventions coexist unambiguously and **no third mark is needed.**

🔒 **WHICH MAKES RETIRING THE BARE NUMBER NORMATIVE, NOT COSMETIC.** A collision split that leaves the bare number in place destroys the only discriminator there is.

### §5 — 🛑 NEVER RESOLVE A COLLISION BY RENUMBERING ONE SIDE TO A FREE NUMBER

⚠️ **Every existing citation of the old number would then silently point at the wrong record**, and citations are how this project's chronicle is navigated at all. **The split is the only permitted repair.** 📌 **Corollary: every surviving citation of a retired bare number must be re-pointed to `a` or `b` INDIVIDUALLY** — there is no mechanical way to tell which a bare citation meant. Measured for the two live cases: **12 citations** — 📌 **corrected 2026-07-30 (J-625); the earlier `28` was a REFERENCE count that never separated citations from discussion of the collision itself. 87 bare hits partition to 4 definition sites · 71 discussion · 12 citations** (per-hit table: `RUNBOOK_ROADTREE_LEGB_BIS.md` §2).

### §6 — WHAT THIS ENTRY DOES **NOT** CLAIM

🛑 **THE CAUSES OF THE KNOWN DUPLICATES ARE NOT ESTABLISHED, AND THIS RECORD DOES NOT ASSERT THEM.** `M_DOC_ROADTREE.md` §8b downgraded the `J-317`–`J-321` mechanism to *"most likely a write that emitted each entry twice — the better-supported reading, not established"*; **no cause is recorded anywhere for `J-044` / `J-045`.** ⇒ **This mechanism exists because collisions occur, not because anyone knows why they occur.** Establishing causes is separate measurement work and is not owed by this decision.

⚠️ **AND NOT EVERY DUPLICATE IS A COLLISION.** `J-317`–`J-321` are **byte-identical pairs** ⇒ **one copy is DELETED, not suffixed.** 🔑 **Suffixing them would enshrine an accident as two distinct events** — the opposite of what this rule is for. **The test is whether the bodies differ.**

### §7 — STANDING WORK THIS RULE GOVERNS

✅ **`M-DOC-ROADTREE` Leg B-bis EXECUTED 2026-07-30 (J-625)** — the only two known live collisions, under §8b's ruling that `JOURNAL_ARCHIVE.md` may be corrected: `J-044` → `J-044a` (the M1–M3 implementation) / `J-044b` (the review) · `J-045` → `J-045a` (the xgen-core crate split) / `J-045b` (the `--batch` AI-tool design note), **12 citations re-pointed individually** (📌 **not 28 — that was a reference count, superseded J-620**), both bare numbers retired. ⚠️ **The `a`/`b` assignment was settled by ARTEFACT EVIDENCE, not by sort direction** — four independent confirmations, recorded at J-625 and in `RUNBOOK_ROADTREE_LEGB_BIS.md` §1b.

📌 **The `CLAUDE.md` standing-convention block is reduced to a pointer at this entry** rather than deleted — the briefing keeps the signpost, the permanent record keeps the rule. **Rationale: `CLAUDE.md` is periodically drained (65 blocks archived at J-615); a normative rule must not live only in the file this project empties.**

---

## D-135 — A predicate is tested in BOTH directions, and an assertion built from the predicate it checks cannot fail

**Date:** 2026-07-30 · **Layer:** project-wide measurement discipline (any count, census, or parse of a repo document) · **Ref:** J-620 · J-621 · J-622 · J-627 · J-628 · J-629; `tasks/RUNBOOK_ROADTREE_LEGE.md` §1c, §3.3, §3.4, §4c; `tasks/M_DOC_ROADTREE.md` §8-E.

🔒 **PROVENANCE — LOCKED BY JOE 2026-07-31 (J-631): *“confirmed”*.** This entry is **no longer reversible**; amendment now requires a further Joe ruling.

📌 **HOW IT GOT HERE, KEPT NOT ERASED.** It was **minted by delegation** the previous day: Chat recommended minting it and offered to write it for Joe to rule on; Joe answered *“go as you recommend”* (2026-07-30), which delegated the **authorship** but not the ruling — the J-623 precedent. It therefore stood as **DELEGATED and REVERSIBLE for one day**, was applied in that state through P0-bis, P1 and P2 (J-630), and **found two wrong inherited verdicts while still provisional.** 🔑 **It is written to `DECISIONS.md` rather than left in a journal entry for one reason: the five preceding statements of this rule all lived in `JOURNAL.md`, and EACH WAS BROKEN BY THE NEXT PASS.**

### §1 — THE RULE

🔒 **A PREDICATE USED TO COUNT OR LOCATE ANYTHING IS TESTED IN BOTH DIRECTIONS BEFORE ITS OUTPUT IS USED:**
- **① FALSE NEGATIVES — what did it MISS?** Run a deliberately WIDER predicate and **read every extra hit individually.**
- **② FALSE POSITIVES — what did it WRONGLY ADMIT?** Test each hit against a property the real thing must have and **read every failure individually.**

🛑 **NEITHER DIRECTION SUBSTITUTES FOR THE OTHER, AND WIDENING CANNOT FIND AN OVER-MATCH.** Five successive sharpenings in this arc all pushed toward widening because every defect found up to that point had been a miss. **The sixth defect was an over-match, and the accumulated rule was structurally blind to it.**

### §2 — 🛑 THE SELF-REFERENTIAL ASSERTION, WHICH IS THE HARDER HALF

🔒 **AN ASSERTION THAT COMPARES A PREDICATE'S OUTPUT TO A NUMBER DERIVED FROM THAT SAME PREDICATE CANNOT FAIL FOR THE REASON IT APPEARS TO TEST.** It detects only **drift** — the artefact changing under the procedure. It cannot detect the predicate being **wrong**, because both sides move together.

⚠️ **THIS IS THE SAME DISEASE AS A SUMMATION THAT CLOSES BY CONSTRUCTION**, and the two appeared **one line apart** in the same runbook. §3.4 asked for a span-sum that consecutive differences always satisfy; §3.3 asked whether the head count equalled a census figure **produced by the same predicate**. 📌 **§3.4 was struck at J-628 and §3.3 was not — because the reader was looking for missing heads, and §3.3 does not look like arithmetic.**

🔑 **THE REPLACEMENT IS AN ASSERTION AGAINST A PROPERTY OF THE ARTEFACT, NOT AGAINST A PRIOR MEASUREMENT.** *First element at 0 · no duplicates · strictly ascending · every head follows a sentence terminator · the key sequence is monotonic* — **each of these can fail while the predicate is unchanged.**

### §3 — ✅ EVIDENCE: THE SECOND PREDICATE IS THE WHOLE DIFFERENCE

**Measured over the 124,299-char closure log in `CLAUDE.md`, 2026-07-30 (J-628, J-629):**

| region | independent cross-check it had | false heads it carried |
|---|---|---|
| **A** (20 heads) | **yes** — 20 `CLOSED (J-nnn)` marks against 1 + 19 heads | **0** |
| **B** (51 heads) | **yes** — J-keys descend **strictly monotonically** J-503 → J-445, no repeats | **0** |
| **C** (23 heads) | **none** | **3** |

🔑 **THE ONLY REGION WITHOUT A SECOND PREDICATE IS THE ONLY REGION THAT WAS WRONG, AND IT WAS WRONG THREE TIMES.** `**M-` matched bold *emphasis* on milestone IDs mid-sentence — `**M-RP5.5**` and `**M-RP5.6**` inside one clause at 115,900 / 115,973, and `**M-RP5.6 CLOSED**` after an em-dash at 119,871. 📌 **B's anchor is a STRUCTURAL delimiter (`) / `); C's and A's is TYPOGRAPHIC emphasis. Typographic anchors admit over-matches; structural ones do not.**

### §4 — THE COST OF NOT DOING THIS, MEASURED

⚠️ **The same line has now had SIX published shapes.** `≈ 82 records` → `95 / 91` → `97 / 93` → **`94 / 90`**. 🛑 **Three consecutive sessions planned against a number that later moved**, and at J-628 a pass was declared **CLEARED** on assertions that all passed while three of its heads were not heads.

⚠️ **AND THE COLLISION SET INHERITED IT.** `§4a`'s *“eleven collisions measured at J-622”* was measured inside part two's two C sub-windows (`110,516–111,445` and `122,026–123,912`) and **never examined the 10,581 chars between them.** The all-pairs test found **sixteen**; the five extra — J-478 · J-479 · J-480 · J-483 · J-485 — **all sit in that gap**, and grounding part three positively asserted they were *not* in the collision set.

### §5 — COROLLARY: A CORRECTION IS NOT APPLIED UNTIL THE WHOLE RECORD SET IS SEARCHED

🔒 **When a measurement is corrected, EVERY LIVE RESTATEMENT OF IT IS FOUND AND ANNOTATED — not the section the reader happened to be in, not even the file.** ⚠️ **J-627 corrected the runbook and `§11` but not `§8-E`'s census; J-628 corrected `§1a`/`§1b`/`§3.4` but not `§5`/`§6`.** 📌 **Before the J-629 sweep, `J-627` and `J-628` appeared EXACTLY ONCE in the entire governing task doc.**

🛑 **DATED RECORDS ARE EXEMPT AND MUST NOT BE BACK-EDITED.** Prior `CLAUDE.md` PLAY blocks and prior `JOURNAL.md` entries are contemporaneous accounts of what was known then; rewriting them destroys the only evidence of how a figure moved. **The corollary binds LIVE claims only — checklists, grounding tables, task-doc state, runbook assertions.** Superseded figures are **annotated, never deleted** (D-131 family).

### §5a — AMENDMENT DRAFTED 2026-07-31 (J-632): SWEEP FOR THE REPLACEMENT, NOT ONLY THE SUPERSEDED FIGURE

⚠️ **STATUS: DELEGATED AND REVERSIBLE, AWAITING JOE'S RULING.** `D-135` was locked at J-631, so this clause is **drafted, not adopted**; Joe answered *“go on 1), as you recommend”* to the repair, and Chat is not amending a locked decision on its own authority. 📌 **Same J-623/J-629 pattern Joe confirmed the previous day.**

🔒 **PROPOSED CLAUSE — §5's SWEEP RUNS ON BOTH THE OLD FIGURE AND THE NEW ONE.** When a measurement is corrected, the sweep must find every live restatement of the **superseded** value **and every live restatement of the replacement**, because the replacement will itself be superseded if the measurement moves again. 🔑 **AN ANNOTATION ASSERTING A CORRECTED VALUE IS A LIVE CLAIM AND DECAYS EXACTLY LIKE THE CLAIM IT CORRECTS.**

✅ **EVIDENCE — SEVEN LIVE CLAIMS, ALL OF THEM WRITTEN BY THE PREVIOUS SWEEP.** The J-629 sweep hunted `13`, `24` and `95 / 91`, annotated them correctly, and wrote **`97 / 93`** as the corrected value. **Four hours later J-630 moved it to `94 / 90` and nothing re-swept the annotations.** Found at J-632, all seven still standing: `RUNBOOK §6` DoD (**the checklist Leg E is measured by**) · `RUNBOOK §5` L218 and its own correction note · `RUNBOOK §1a` row C · `M_DOC` §8-E × 3.

🛑 **A LINE CAN BE ANNOTATED AND STILL BE FALSE.** A line-scoped detector reported `ann=True` on `§5` L218 — the block **does** carry a correction, **the wrong one**. ⇒ **Presence of an annotation is not evidence that the CURRENT figure is annotated.**

🔑 **AND THE CARRIER MATTERS: §6 HAS NOW BEEN WRONG AT THREE CONSECUTIVE PASSES** (J-629 found `24`, J-632 found `97`). **It sits past the working sections and is never the section in hand.** ⇒ **A figure change is swept across the record set MECHANICALLY, never from memory of which sections carry it.**

### §6 — 🛑 WHAT THIS ENTRY DOES **NOT** CLAIM

⚠️ **It does not claim the head set is now final.** C's 23 heads rest on **three hand verdicts** (the two mid-clause emphases and the em-dash continuation) plus **one genuinely ambiguous boundary at 116,587**, which opens after a **semicolon** rather than a sentence end and was admitted on the strength of its head form. **A hand verdict is a judgement, and it is recorded as one.**

⚠️ **It does not claim the key-extraction method is sound.** For C's **forward-looking, non-closure** heads the first `(J-nnn)` in the head window is a **citation, not an identity**. Four such heads carry a key on that basis. **It produced no false collision here — all sixteen were read on both sides — but the method must not be reused blind.**

⚠️ **It does not claim a probe is trustworthy because it ran.** Two probes in this arc returned **internally inconsistent** output — one measured the same slice three times under array flattening (J-627), one returned a head count of **1** for a 94-head input (J-629). 📌 **Both tells were internal inconsistency, not implausibility. A probe that reports one number for many different inputs is reporting on itself.**

---

## D-136 — A completed sweep is not a standing rule; a convention that is not enforced regresses silently in the code written after it

**Date:** 2026-08-01 · **Layer:** project-wide convention durability (any retrofit, migration, rename, or type-discipline pass) · **Ref:** J-645; XGID Retrofit Passes 1–5 (J-122–J-148); `M-RP-ADDRESS-BOOK` Leg D (J-586); `M-RP-XGID-SLOT-RETYPE`

🔒 **MINTED BY DELEGATION AND LOCKED BY JOE 2026-08-01.** Chat recommended minting it and named the alternative — hold it as a J-645 observation until a second instance proved durability under `D-077` — and Joe answered *"2) + 3) as you recommend"*, then *"all locked"*. **Recorded as delegated per the `D-135` precedent, so the provenance is not lost.**

### §1 — THE RULE

🔒 **A PASS THAT SWEEPS A CONVENTION ACROSS AN ENUMERATED SET OF SURFACES HAS CHANGED THOSE SURFACES. IT HAS NOT CREATED A RULE.** Unless the convention is separately enforced — by a type that makes the wrong form unrepresentable, by a test, by a lint, or by a written standing rule that new work is read against — **code written after the pass closes will regress to the pre-pass form, and it will compile.**

🔑 **THE MECHANISM IS THAT THE SWEEP'S OWN COMPLETENESS IS WHAT HIDES THE REGRESSION.** A pass closes by demonstrating that every enumerated surface was converted. That demonstration is true, and it is about **the surfaces that existed on the day it ran**. It says nothing about the next file, and the closing record reads as though the subject were the codebase rather than a snapshot of it.

⇒ **the completion claim is narrower than the thing it appears to describe** — the recurring species of this project, here at the level of a milestone's own close.

### §2 — ✅ THE MEASURED INSTANCE THAT MINTED IT

**The XGID Retrofit's five-pass arc closed 2026-05-29 (`7ed4e30`).** Pass 4 retyped the whole of `xgen-client` under a locked classification: **identifier slots retype to a typed XGID flavour; descriptive slots stay `String`** (design doc §4.1.a — 31 mechanical identifier retypes, 12 descriptive stays, 3 borderline locks).

| struct | first appeared | identifier slot |
|---|---|---|
| `MemberEntry` | **2026-06-01** — 3 days after the arc closed | ✅ `IdentityXgid` |
| `FetchedIdentity` · `SeenRecord` · `FillReport` | **2026-07-25** — two months after | ❌ `String` |

🔑 **`MemberEntry` and `FetchedIdentity` are in the SAME FILE (`xgen-client/src/ops.rs`), chose oppositely, and the one that broke the rule is the later one.** All three July structs arrived on **one commit** (`a0c8b4c`, `M-RP-ADDRESS-BOOK` Leg D, J-586).

**Three independent corroborations:**
- 🛑 **`SeenRecord.home_node: String` (`address_book.rs:89`) contradicts a Pass 4 borderline lock BY NAME** — §4.1.a reads *"2 NodeXgid for `home_node` ×3"*. The slot was ruled typed, then re-introduced as `String`.
- 🛑 **`address_book.rs` contains ZERO occurrences of `IdentityXgid`**; `ops.rs` contains 39. The type was not weighed and rejected in that file — **it was never in the room.**
- 🛑 **The downgrade at `ops.rs:2734` / `:2742`** (`e.sender.as_str().to_string()`, `m.identity_id.as_str().to_string()`) **reads as a deliberate seam and is not one.** It exists only to feed a `BTreeMap<String, SeenRecord>` that should not have been `String`-keyed. **Remove the regression and the seam disappears rather than moving.**

⚠️ **`String` COMPILES.** There is no failing build, no failing test, and no reviewer prompt. **The regression is invisible by construction**, which is why it survived two months and was found only when a third party — Joe — recalled the retrofit from memory and asked for the check to be re-run deeper.

### §3 — WHAT THE RULE REQUIRES IN PRACTICE

🔒 **A pass that closes MUST state, in its own closing record, how the convention will be enforced after it — or state explicitly that it will not be, so the gap is known rather than assumed.** Acceptable enforcement, strongest first: **① make the wrong form unrepresentable** (the typed newtype at the API boundary, so `String` does not compile) · **② a test that fails on the wrong form** · **③ a lint or grep gate** · **④ a written standing rule in `CLAUDE.md` that new work is read against.**

📌 **④ IS THE WEAKEST AND IT IS OFTEN THE ONLY AVAILABLE ONE. That is fine — what is not fine is silence.** The failure here was not choosing a weak enforcement; it was choosing none and not noticing that none had been chosen.

### §4 — 🛑 WHAT THIS ENTRY DOES **NOT** CLAIM

⚠️ **It does not claim the XGID regression's full extent is known.** Only the structs on the address-book fill path were measured. **No sweep has been run for other post-2026-05-29 structs that regressed the same way**, and if the mechanism is real there is no reason to expect this commit was the only one. **That sweep is `M-RP-XGID-SLOT-RETYPE`'s Phase-0 job (`D-071`), and the milestone is filed 🟡 PENDING, not started.**

⚠️ **It does not claim the retrofit passes were done badly.** They converted what they enumerated, and they enumerated honestly. **The defect is in what a close means, not in the work.**

⚠️ **It does not retroactively fault `M-RP-ADDRESS-BOOK` Leg D.** With no enforcement in place, `String` was the locally reasonable choice and nothing signalled otherwise. 🔑 **A convention nobody can see is not a convention anyone can follow — that is the whole point of this entry.**

📌 **RELATION TO THE EXISTING FAMILY.** Sibling to **a trigger that has fired is a defect**: both concern a completed action being mistaken for a durable property. Distinct from `D-071` (audits precede dependent milestones), which governs **order**; this governs **what a completed pass leaves behind**.