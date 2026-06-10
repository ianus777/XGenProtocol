# XGen Protocol — Implementation Decisions
> **Status:** ACTIVE  
> **Last updated:** 2026-06-10  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  

Every decision that goes beyond spec prescription is recorded here before advancing to the next layer.
Format: title, date, layer, spec reference, decision narrative.

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

- Auth Tiers (D-037) — protocol carries the marker, Auth Module supplies the verification meaning
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
**Spec reference:** Ch1 (Human and Agent Operation); Ch3 §3.6 (Identity registration); Layer 15 / D-049 (identity replication); D-037 (Tier 1 = persistent accountable identity)

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

**Implication for accountability:** the same persistent-accountable-identity guarantee (D-037) applies. An AI cannot "reset" its identity to escape consequences any more than a human can. The keypair is the anchor.

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

## D-056 — recv() routing: sender-field check precedes all type-prefix checks

**Date:** 2026-05-14  
**Layer:** Transport (xgen-core/src/transport/connection.rs)  
**Spec reference:** Spec 3.3.4 (WebSocket framing); spec 3.1.2 (Event fields)

**Problem:** `recv()` dispatched incoming binary frames by matching `value["type"]` against type-string prefixes (`"mls."`, `"bootstrap."`, `"reputation."`, etc.). `Event.event_type` is serialised as `"type"` on the wire (via `#[serde(rename = "type")]`). DAG Events such as `mls.key_package`, `bootstrap.node_announce`, and `reputation.defederation_signal` therefore matched the control-message prefix check before the Event check was reached. Deserialization into the control enum failed because `Event` and the control types have different JSON shapes. The error propagated out of `recv()` as `Err`, which the node's connection loop caught as `Err(_) => break`, silently closing the connection.

**Decision:** Add `value.get("sender").is_some()` as the **first** branch in the `recv()` routing chain, before all type-prefix checks. Every `Event` struct has `pub sender: String` with no `skip_serializing_if`, so `"sender"` is always present in a serialised Event. No control message type (`TransportMessage`, `FederationMessage`, `IdentityMessage`, `MlsMessage`, `BootstrapMessage`, `ReputationMessage`, etc.) carries a `"sender"` field. The invariant is enforced by the type system: adding `sender` to a control message would require a structural change that would be immediately visible.

**Impact:** Any message carrying `"sender"` routes to `Inbound::Event` unconditionally. All other routing is unchanged. One-line change; no new allocations; no test changes required. 300/300 tests pass.

---

## D-055 — Server-side Phase 2 handler wiring: peer_url propagation and identity replication push

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
**Spec reference:** Spec 3.10.1–3.10.9; DECISIONS.md D-031 (MLS selected over Megolm)

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

## D-039 — Pending buffer wiring: NodeRuntime holds PendingBuffer directly

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

## D-038 — Client session header omits `identity_id` and `connected_node`

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

## D-037 — Tier 1 identity: precise definition of persistent accountable identity

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

## D-031 — End-to-End Encryption: MLS (RFC 9420) selected over Megolm

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

## D-030 — xgen-node will be packaged as a system service post-stabilisation

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

## D-030 — Runtime file placement: GetModuleFileNameW on Windows; data_dir from config path

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

## D-031 — Phase 1 Node configuration reference (xgen-node_config.toml)

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

## D-037 — Node deployment model: systray singleton with detachable admin window

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

## D-038 — Tier badge placement: Node property, not member property

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

## D-039 — Application shutdown model: × to systray, explicit exit only

**Date:** 2026-05-07  
**Layer:** 6 (UI / deployment)  
**Spec reference:** Ch6 §6.11, Appendix E, D-037  

Closing the window with × does not exit either application. Both applications minimize to the system tray. Explicit exit is always a deliberate user action.

**× button behaviour (both apps):**
- Hides the window, process continues running
- Client: stays connected, session live, logs flowing
- Node: keeps serving clients and federation peers, no change
- Consistent with D-037 (Node window is detachable from Node process)

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
- A kickout mechanism — idle clients are never disconnected for inactivity (D-039)

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

## D-055 — Phase 2 server-side handler wiring: node_endpoint in Hello, identity replication routing

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

## D-056 — Application Deployment Model: one binary per role, multi-mode dispatch

**Date:** 2026-05-16
**Layer:** Layer 6 (UI / deployment / packaging)
**Spec reference:** Ch2 — Application Deployment Model & Lifecycle States (Session 19); Appendix E — Application Lifecycle States (Session 4)

### Context

Earlier Ch2 wording described the deployment model as "one binary, two personalities" — desktop (with UI) versus service (`--service`, headless). That framing conflated two independent questions: (a) does the binary present a UI, and (b) is the invocation long-running or short-lived. The conflation became actively misleading when implementation work surfaced two facts:

- The Client side already has `--batch` (BATCH_FLAG_ph2.md, J-044) — a short-lived, no-UI invocation that connects to a long-running instance via a named pipe (D-043), dispatches commands, and exits. This is neither "desktop personality" nor "service personality." It is a different category of invocation altogether.
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

These follow from the decision. They are not part of D-056 itself; they are tasks pulling current code into compliance:

1. **Node-side `--batch` implementation.** J-037 deferred this when the Client-side `--batch` was written. The spec target is now explicit. Port BATCH_FLAG_ph2.md's pattern to the Node side using the same library-first rule, same pipe-naming convention, same clap dispatch shape, with the Node's own command set.
2. **Collapse `*-app.exe` into the single product binaries.** Merge `xgen-{node,client}/src/main.rs` with `xgen-{node,client}/src-tauri/src/main.rs` into one entry point per role. Extract shared resident-mode logic (`run_node_server` / `start_client_session`) into the library crate so the single binary can dispatch any mode without code duplication. Eliminate the two parallel `--batch` implementations on the Client side.
3. **Pipe server in resident mode for both binaries.** Currently only the Client's Tauri variant hosts a pipe server. The Node Tauri shell's `--service` mode emits lifecycle events but binds no WebSocket server and no pipe server. Bring it into compliance with the new model: every resident-mode invocation hosts the pipe server.

These implementation tasks are tracked separately. D-056 locks the architectural target they converge on.

### Relationship to other decisions

| Decision | Relationship |
|---|---|
| D-043 | Pipe naming convention `\\.\pipe\xgen-{node\|client}-{label}`. D-056 generalises it: every resident instance, every control-mode invocation. |
| D-037 | Node deployment personality (now resident mode variants). Architectural horizon — protocol-native Node admin via privileged client Identity — survives unchanged. |
| D-039 | Shutdown model. `×` minimises to tray; `CLOSING` only entered via explicit exit action or a future `--stop` control-mode flag. Consistent with D-056. |
| J-037 | Node `--batch` design discussion. Now has an explicit spec target to point at. |
| J-044 | Client `--batch` implementation (BATCH_FLAG_ph2.md). The principal worked example of the control-mode pattern D-056 generalises.

### Spec status

- Ch2 §Application Deployment Model — rewritten in Session 19 (2026-05-16) to match this decision.
- Appendix E — Design Principles section opened with a paragraph clarifying that lifecycle states describe resident mode only. Session 4 entry added.

---

## D-062 — Tauri inclusion model: compiled into product binary, runtime dispatch chooses UI

**Date:** 2026-05-16
**Layer:** Layer 6 (deployment / packaging)
**Spec reference:** D-056 (one binary per role, multi-mode dispatch). M1 task file `tasks/BINARY_CONSOLIDATION_M1.md` Phase 2.

### Context

D-056 named the deployment target — one binary per role, dispatched at startup. The implementation question that follows: when both binaries link in Tauri (for the desktop variant of resident mode), is the Tauri dependency a build-time variant (Cargo feature flag `tauri`) or always compiled in with runtime dispatch?

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

This decision is the literal Rust expression of D-056's "one binary per role, multi-mode dispatch." Without D-062, D-056 has no Rust-level commitment; with D-062, the merge in M1 Phase 2 has a clean target shape:
- `xgen-node/Cargo.toml` and `xgen-client/Cargo.toml` carry `tauri`, `tauri-plugin-process`, and `tauri-build` (build-dependency) unconditionally.
- Each product crate's root holds `tauri.conf.json` + `build.rs` + `capabilities/` + `icons/` (formerly under `src-tauri/`).
- The Tauri shell code moved to library modules (`xgen-node-lib::desktop`, `xgen-client-lib::desktop`) so the binary's `main.rs` stays thin.

The `*-app.exe` build targets are removed from the workspace. Build artefacts after M1 Phase 2a: exactly `xgen-node.exe` and `xgen-client.exe`.

### Relationship to other decisions

| Decision | Relationship |
|---|---|
| D-056 | Architectural target. D-062 is the implementation-level commitment of how Tauri lives inside that target. |
| D-063 | Companion decision: where the resident-mode logic lives (library crate, not `main.rs`). Required by D-062's runtime-dispatch model — the dispatch target must be a library function any entry point can call. |

---

## D-063 — Resident-mode logic lives in the library crate

**Date:** 2026-05-16
**Layer:** Layer 6 (architecture)
**Spec reference:** D-056 (shared command layer requirement). M1 task file `tasks/BINARY_CONSOLIDATION_M1.md` Phase 1.

### Context

D-056 requires a shared command layer that every input channel (Tauri UI button clicks, Console typed commands, `--batch` piped commands, control-mode flags) dispatches through. For that requirement to be satisfied, the command layer has to live somewhere that all entry points can call — which means it cannot live in `main.rs` (only one `main.rs` exists per binary; library code, Tauri callbacks, and the binary's CLI dispatcher cannot all call into it from there).

The existing layout violated this. `run_node` (the Node's resident-mode entry point), the entire CLI subcommand set (`cmd_init`, `cmd_status`, `cmd_connections`, etc.), and the Client's batch-line dispatcher all lived in `main.rs` to varying degrees. The Tauri shell duplicated functionality (lifecycle scaffold) rather than calling shared code.

### Decision

**Resident-mode logic and the full command surface move to the library crate.** After this decision lands:

- `xgen-node-lib` (`xgen-node/src/lib.rs`) exposes `app::run_node`, `app::cmd_*` for every subcommand, `app::RunNodeOpts`, and `desktop::run` (the Tauri shell entry point, calling `app::run_node` internally).
- `xgen-client-lib` (`xgen-client/src/lib.rs`) exposes `app::cmd_*` for every subcommand, `app::run_batch_file`, the full `Cli` / `ClientCommand` clap structs, `batch::start_pipe_server`, `batch::dispatch_line`, `batch::pipe_name`, `batch::run_batch_client`, and `desktop::run`.
- Each binary's `main.rs` is a thin dispatcher: parse flags, decide mode, call the corresponding library function. No business logic in `main.rs`. The Node main.rs ends up around 270 lines (most of that clap definitions); the Client main.rs around 200 lines (most of that clap dispatch).
- The Client's `Cli` / `ClientCommand` clap structs live in `xgen-client-lib::app` rather than `main.rs` because the batch-file executor (`run_batch_file`) re-parses sub-CLI invocations per `.xgb` line, and that executor lives in the library.

### Rationale

This is the library-first architecture rule from `CLAUDE.md`, applied consistently across the merged binary structure. The rule already existed for Layer 1–10 code (everything below `transport`); D-063 extends it to the dispatch layer that sits between input channels and command implementations.

Without D-063, D-056's "shared command layer" is impossible to express in code: the desktop shell would either duplicate command implementations (drift inevitable, J-067's two-`get_dag_tips` problem multiplied) or call back into `main.rs` somehow (Rust doesn't permit that cleanly). The library extraction is the unblock.

### Implementation note

The implementation pass lives in M1 Phase 1. After it ships:
- `grep "pub async fn get_dag_tips"` returns exactly one match in `xgen-client/src/batch.rs:239`. The duplicate from `xgen-client/src/main.rs` is gone. Closes F-003 / F-004 from J-067 permanently — that was the loudest visible symptom of the library-extraction gap.
- All `cmd_*` functions live in `app.rs` (per crate). `main.rs` calls them via `app::cmd_foo(...)`.
- `desktop::run()` calls `app::run_node()` with `RunNodeOpts { init_logging: false, ... }` so logging init is owned by the desktop module (since Tauri is already up by the time `run_node` runs). The bool flag is the seam.

### Relationship to other decisions

| Decision | Relationship |
|---|---|
| D-056 | Architectural target. D-063 makes the shared command layer physically possible. |
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

**Why a mode and not a separate binary.** The Node's headless mode is `--service`, not a separate `xgen-node-service` binary. By symmetry, an AI Client is a client — same Identity registration, same Space membership, same event emission, same `[ai]` config staging — just with behaviour coming from a plugin instead of a keyboard. Consistency with the resident/control pattern wins. M1 collapsed binaries that shared identical code; xgen-client and the AI Client share the same library and dispatch through one entry point per mode. Three binaries (the rejected alternative) would have put M4 in conflict with the D-056 consolidation direction it should be following.

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

The hard architectural question was *binary identity* — should the AI Client be a separate `xgen-ai` binary or a mode of `xgen-client`? The v0.1 draft of this decision proposed a separate binary; the v0.1→v0.2 review pass amended it to "mode of xgen-client" with reasoning that the M2 precedent (Node's `--service` mode rather than `xgen-node-service` separate binary) and the D-056 consolidation direction (one binary per role) both point the same way. AI Client is a client; the runtime loop differs from the human Client's loop but everything around it (config loading, connection, pipe server, lifecycle) is identical scaffolding. A separate binary would have duplicated that scaffolding for no clear gain.

The plugin trait is locked now rather than deferred. The trait surface is small enough that getting it wrong now is cheap; getting it wrong after a real LLM plugin exists is expensive — the future plugin would either accept the inherited shape or force a breaking-change rework of every consumer. Locking the shape during M4, before any real plugins exist, costs nothing extra and stabilises the interface.

Drop-late-replies is locked because queueing produces stale replies — by the time the cooldown expires, the conversation has moved on. The locked behaviour also is the simpler implementation, but the simplicity follows from the correctness, not the other way around: the honest design is also the lighter design here.

Manual join is locked because the trust model loses something when an AI Identity's first observable behaviour in a Space is config-driven rather than chosen. Auto-join would make the AI's presence implicit; manual join keeps it explicit and auditable through the standard `membership.join` event flow.

### Why now

M4 implementation began at v0.3 task-file lock (J-076) after D-056 consolidation was confirmed closed. The Client lifecycle conventions (PID file, pipe server, session header, log rotation) are stable from M1/M2; the protocol primitives the AI Client consumes are stable from M3. M4 is the first milestone that exercises all of them together in a long-running process and surfaces "what does this look like end-to-end" for the first time. The recurring honest-vs-polite principle was already implicitly operating across earlier decisions; naming it here makes future design conversations more efficient.

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
| D-056 | M4 is a mode of xgen-client per the locked "one binary per role" direction. D-056 closed first (J-076); M4 implementation followed. |
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
| D-056 | `--aicontrol` is a new dispatch mode on the existing `xgen-client` binary, consistent with the locked one-binary-per-role + multi-mode dispatch model. Not a new binary. |
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

