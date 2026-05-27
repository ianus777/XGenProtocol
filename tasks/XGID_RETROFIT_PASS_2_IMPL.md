# XGID Retrofit Pass 2 — Implementation Runbook
> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-27 (J-125 — Commit 1 doc-pass SHIPPED. Five-file atomic per D-074 (twenty-second instance): design doc Status ACTIVE → COMPLETED v1.0 → v1.1 + new §6.7 Shape α entry; ROADMAP v1.34 → v1.35 + visual tree + Present + Past; CLAUDE PLAY flip; JOURNAL J-125 header chain entry; this runbook header chain entry. Pre-Clair audit CLEAN across six dimensions (file paths / type shapes at named anchors / Pass 1 carry-overs Borrow<str> + 33 inline markers / contingency surfaces / parallel-milestone drift = none / test baseline 489 = 34 lib + 8 invariance + 447 xgen-core matches J-122). Joe-lock checkpoint #1 fires after this commit; four drift-detection points confirmed. Joe-lock checkpoint #2 fires next (pre-Commit-2 verbatim surface list from design doc §2). No state transitions: Pass 2 milestone stays PLAY; Commit 2 production code is next-active for Clair. Per Rule 0 + D-065 + D-067 + D-069 + D-071 + D-074 + D-077 + D-078.) Previous 2026-05-27: runbook NEW ACTIVE v1.0 at J-124.  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 Framing

### §1.1 What this document is

Clair's build instructions for **XGID Retrofit Pass 2** — the second of five staged retrofit passes per the Shape γ + ASAP discipline locked in D-072. Pass 2 retypes the **xgen-core algorithm-bearing functions** that consume the data structures Pass 1 retyped: `validate_event` + `ValidationOutcome`, `NodeRuntime::dispatch_event` + `DispatchOutcome`, `PendingBuffer` arrival hooks, `FederationRegistry` / `IdentityRegistry` APIs, and `accept_message`.

This runbook is the canonical artefact for Pass 2's scope and commit shape. The authoritative architectural decisions are recorded at `tasks/XGID_RETROFIT_PASS_2_DESIGN.md` (COMPLETED v1.0 at J-123). Where this runbook makes implementation choices that go beyond the design doc, the choice is recorded here and stays here.

**Inherited from Pass 1 (J-122 close):**

- `Borrow<str>` additive API on `Xgid` + flavour wrappers — load-bearing for Pass 2's design principle (§3 projection mechanism). Pass 1's Commit 4 implementation-kickoff Joe-lock means `HashMap<NodeXgid, V>::get(&str)` and sibling Borrow-driven lookups work without per-query wrapper allocation. Pass 2 builds on this directly.
- All Pass 2 surfaces already carry inline `// Pass 2 widens this method to take typed XGIDs; the wrap collapses then` code-comment flags written during Pass 1 implementation. These flags pre-locate Pass 2's work without an audit phase.
- Pass 1's `xgen-core` lib stayed clean at workspace test count 489 (xgen-common 34 lib + 8 invariance + xgen-core 447). `cargo build --workspace` is deliberately broken per Path A — Pass 2 inherits the same broken-workspace posture; verification is package-scoped to `xgen-common` + `xgen-core`.

### §1.2 Precedent-departure self-defense

This runbook targets ~40-50 KB across eight sections, substantially lighter than the trilogy precedent's ~80-100 KB range and lighter than Pass 1's ~65 KB. The departure is honest per D-065:

1. **Pass 2's design surface is smaller than Pass 1's.** Pass 1 retyped the full data-structure surface across two crates with two appendices. Pass 2 retypes five algorithm-bearing function surfaces all in xgen-core (no appendix updates — Appx D is Pass 3, Appx F is Pass 4).
2. **The architectural work is already done.** Pass 1's pre-walk reconnaissance flagged every Pass 2 surface with inline code-comment markers. Pass 2's design phase (J-123) produced one principle + two architectural decisions; the rest is mechanical per the principle.
3. **Contingent-split posture is pre-locked.** Pass 2's split into Commit 2 + Commit 2a (test-fixture sweep) is a contingent path with an explicit trigger criterion, not an emergent surprise. Pass 1's Commit 4 → Commit 4 + Commit 4a split surfaced mid-implementation when ~296 fixture errors appeared; Pass 2 pre-locates the same shape with the trigger named upfront.

The three reasons map onto J-123's Pass-internal-consistency-over-trilogy-internal-consistency framing (design doc §7.7). Pass 2 sits inside the five-Pass XGID Retrofit arc whose closer sibling is Pass 1 (also lighter than trilogy at runbook authoring per J-114-J-122 honest framing).

### §1.3 Latitude

Latitude during implementation belongs to Clair within the bounds of:
- The design principle (design doc §3) is load-bearing — call-site-boundary projection via `Borrow<str>` is the Pass 2 mechanism; no `Deref<Target = str>` shortcuts.
- The two architectural decisions (design doc §4 — Q2.8 partial / Q5 deprecation) are Joe-locked.
- Mechanical locks (design doc §2 Q-tables) follow the principle uniformly. Clair has latitude on test-fixture details, error-message rephrasing for typed payloads, and minor projection wrapper placement so long as the projection happens at the call-site boundary (not the function body interior).
- Three Joe-lock checkpoints (§2.3) are mandatory STOP points.

Any surfacing that contradicts the design principle is a Rule 3 STOP condition — surface for Joe-lock per established discipline.

---

## §2 Sequence overview

### §2.1 Three-commit base + contingent Commit 2a

| Commit | Status | Scope | Files (approx) |
|---|---|---|---|
| Commit 1 | 🟡 | Doc-pass — design doc Status flip + ROADMAP + CLAUDE PLAY flip + new design-doc §15 sub-section recording Pass 2 entry | 4-5 |
| Commit 2 | 🟡 | xgen-core algorithm-bearing retypes (all five surfaces atomic) + per-surface unit tests | 6-10 |
| Commit 2a | 🟡 [contingent] | Test-fixture projection sweep — fires at Joe-lock checkpoint #3 if test-fixture error count from Commit 2 exceeds ~50 | varies |
| Commit 3 | 🟡 | Milestone close per D-074 (Status flips, ROADMAP, JOURNAL, CLAUDE PLAY flip, freeze J-NNN placeholders) | 6-8 |

**Why three-commit base and not four-or-five.**

- **One-commit-for-all-five-surfaces at Commit 2** is the right shape because all five surfaces share one mechanic (the Borrow<str> projection) and have lower internal coupling than Pass 1's data-structure surface. Splitting into per-surface commits would inflate the commit count without improving reviewability — each surface's diff is mostly signature widening + body-internal projection adjustments, and the surfaces compile-depend on each other (validate_event consumes types from runtime; pending consumes types from both; registry consumes types from common). A single Commit 2 keeps the type cascade coherent in one atomic.
- **Contingent Commit 2a** rather than always-split — honest per D-065. The single-commit shape is the expected-default; the split fires only if the test-fixture surface produces error count exceeding the trigger threshold. Pre-locking the split posture without forcing it acknowledges that the expected-default is achievable while preserving the discipline to split if reality says otherwise.
- **Commit 3 isolates the milestone close** per the trilogy precedent. Pass 1's Commit 6 + persistence-amendment's Commit 4 + topo-sort's Commit 4 all isolate cross-doc updates from substantive code change.

### §2.2 Five Pass 2 surfaces (Commit 2 scope)

Per design doc §2, walked in dependency order (bottom-of-stack first):

| # | Surface | Location |
|---|---|---|
| 1 | `validate_event` + `ValidationOutcome` + `validate_steps_8_13` (deprecated) | `xgen-core/src/message/exchange.rs` |
| 2 | `NodeRuntime::dispatch_event` + `DispatchOutcome` + `NodeRuntime` struct (partial) | `xgen-core/src/node/runtime.rs` |
| 3 | `PendingBuffer` arrival hooks (`add`, `resolve`, `resolve_identity`, `resolve_federation_relationship`) | `xgen-core/src/dag/pending.rs` |
| 4 | `IdentityRegistry` + `FederationRegistry` method APIs | `xgen-core/src/identity/registry.rs` + `xgen-core/src/federation/registry.rs` |
| 5 | `accept_message` signature | `xgen-core/src/node/runtime.rs` |

### §2.3 Three Joe-lock checkpoints

Mandatory STOP points for Clair. Each checkpoint requires explicit Joe approval before proceeding to the next phase.

**Checkpoint #1 — Post-Commit-1 doc-pass drift check.**

Trigger: after Commit 1 lands. Clair pauses and asks Joe to confirm:
1. Design doc Status ACTIVE → COMPLETED + v1.0 → v1.1 + Last-updated chain entry.
2. ROADMAP version bumped + Pass 2 row gains 🟢 implementation sub-bullet.
3. CLAUDE PLAY block flipped from "runbook authoring next-active for Chat Claude + Joe" to "Pass 2 implementation ACTIVE — Clair pickup at runbook §4 Commit 2".
4. New §15-equivalent sub-section in design doc recording Pass 2 entry (canonical-doc surface for the milestone). Sibling-shape to topo-sort design doc §15 row freeze pattern but at Pass 2's design doc since Pass 2 doesn't add a new D-NNN at the no-drift-surface family layer.

Drift-detection question to Joe: does any of this disagree with what was intended at runbook authoring (J-124, this commit)? If yes, surface and stop.

**Checkpoint #2 — Pre-Commit-2 verbatim surface list.**

Trigger: before Clair touches any production code in Commit 2. Clair extracts and surfaces to Joe a verbatim list of the five surfaces from design doc §2, with the locked Q-decision tables verbatim, and asks Joe to lock the surface list by name before any code lands.

Lock format:
```
Surface #1 — validate_event + ValidationOutcome + validate_steps_8_13 [DEPRECATED]
  Q1: missing_predecessors: Vec<EventXgid>, missing_identity: Option<IdentityXgid>
  Q2: HeldPending(Vec<EventXgid>), NotARoomMember(RoomXgid); descriptive stays String
  Q3: bind as typed references; project at call-site boundary
  Q4: fed_add_via_federation: bool — no change
  Q5: deprecate + minimally retype validate_steps_8_13 + accept_event per §4.2 Q5.b

Surface #2 — NodeRuntime::dispatch_event + DispatchOutcome + NodeRuntime (partial)
  Q2.1: DispatchOutcome::Accepted.new_joiner: Option<IdentityXgid>
  Q2.2: DispatchOutcome::Rejected(String) — no change
  Q2.3: additional_persisted: Vec<Event> — no change
  Q2.4: EventOrigin — no change
  Q2.5: NodeRuntime.node_id: NodeXgid
  Q2.6: peer_urls: HashMap<NodeXgid, String>
  Q2.7: internal projections bind as typed references
  Q2.8: partial — node_id + peer_urls retype now; six per-space HashMap keys defer to Pass 3

Surface #3 — PendingBuffer arrival hooks
  Q3.1-Q3.4: method signatures retype per design §2.3
  Q3.5: drop Xgid::new(...) wraps inside method bodies

Surface #4 — IdentityRegistry + FederationRegistry
  Q4.1-Q4.7: method signatures retype + drop bridge wraps + storage already typed

Surface #5 — accept_message
  Q5.1-Q5.4: signature retypes + body internal projections + deprecation-audit flag
```

Joe approves the surface list verbatim. Any divergence from design doc §2 surfaces at this checkpoint, not later.

**Checkpoint #3 — Post-Commit-2 split-trigger decision.**

Trigger: after Commit 2 lib retypes land and `cargo test -p xgen-common -p xgen-core --lib` is verified clean. Clair runs `cargo test -p xgen-common -p xgen-core --tests` (including test-fixture files) and reports the error count to Joe.

Joe-lock decision per the trigger criterion:
- **Error count ≤ ~50:** Absorb the test-fixture sweep into Commit 2 as a follow-up edit before Commit 2 lands. Single-Commit-2 shape wins.
- **Error count > ~50:** Ship Commit 2 as lib-clean (test-fixture errors stay broken), then ship Commit 2a as the test-fixture projection sweep atomic. Sibling-shape to Pass 1's Commit 4 + Commit 4a split.

The ~50 threshold is honest per D-065 — at Pass 1, ~296 errors made the split obvious; at smaller counts the split's overhead exceeds its benefit. Joe holds final discretion; the threshold is a heuristic, not a hard rule.

### §2.4 Verification rigour

Pass 2's verification target at Commit 2 (or Commit 2 + Commit 2a if split fires):

- **5 isolated runs** of `cargo test -p xgen-common -p xgen-core` (with `cargo clean` between each, to ensure no incremental-build false-positives).
- **3 workspace-scoped runs** of `cargo test -p xgen-common -p xgen-core` (without cargo clean, to verify under cargo's incremental cache).
- **Total: 8 green runs minimum at milestone-bearing commit.**

Sibling-shape to topo-sort J-101 + persistence-amendment J-108 + Pass 1 J-122 verification rigour. Pre-existing flakes (precedence env-var race; `reconnect_with_existing_tip_small_delta_delivered`) live in xgen-node and don't apply at xgen-core package scope — Pass 2's verification is naturally isolated from them.

Clippy verification at Commit 2 + Commit 3:
- `cargo clippy -p xgen-common -p xgen-core --lib --all-features -- -D warnings`: clean.
- `cargo clippy -p xgen-common -p xgen-core --tests -- -D warnings`: clean.
- `cargo build --workspace`: deliberately broken per Path A (sibling-shape to Pass 1 J-122 — xgen-node + xgen-client consume retyped types at Pass 3 + Pass 4; Pass 2 inherits the broken-workspace posture).

---

## §3 Commit 1 — Doc-pass

**Scope:** Documentation-only commit. No production code changes. Flips design doc Status, bumps ROADMAP, flips CLAUDE PLAY block, adds Pass 2 §15-equivalent canonical entry.

### §3.1 Files touched (4-5)

1. **`tasks/XGID_RETROFIT_PASS_2_DESIGN.md`** — Status ACTIVE → COMPLETED, Version 1.0 → 1.1, header `Last updated` chain entry recording J-NNN at Pass 2 implementation kickoff. New sub-section under §6 (Cross-references) — §6.7 "Pass 2 implementation entry" — recording the runbook reference and the J-NNN-placeholder freeze site (this design doc gains the milestone-event pointer per Pass 2 milestone-close convention).
2. **`docs/ROADMAP.md`** — version bump 1.34 → 1.35; visual structure tree's XGID Retrofit Pass 2 row gains 🟢 implementation sub-bullet under design ✅; Present section's Pass 2 entry updated from "runbook authoring next-active for Chat Claude + Joe" to "Pass 2 implementation ACTIVE — Clair pickup at runbook §4 Commit 2"; Past section gains a small "Pass 2 implementation runbook shipped" paragraph under the XGID Retrofit cluster; header chain entry.
3. **`CLAUDE.md`** — PLAY block flipped from "XGID Retrofit Pass 2 design ✅ at J-123; runbook authoring next-active for Chat Claude + Joe" to "XGID Retrofit Pass 2 implementation ACTIVE — Clair pickup at `tasks/XGID_RETROFIT_PASS_2_IMPL.md` §4 Commit 2"; header `Last updated` chain entry.
4. **`JOURNAL.md`** — new J-NNN entry recording runbook shipped + design close referenced + three Joe-lock checkpoint shape + contingent-split posture; header chain entry.
5. **`tasks/XGID_RETROFIT_PASS_2_IMPL.md`** (this file) — Status stays ACTIVE v1.0 at Commit 1; header `Last updated` chain entry recording Commit 1 landing.

### §3.2 What does NOT change in Commit 1

- No code touched in `xgen-core/src/` or anywhere else under `xgen-common/` / `xgen-core/`.
- No new test files added.
- No DECISIONS.md entries — Pass 2 doesn't promote a new D-NNN at the no-drift-surface discipline family (per design doc §6.3); the §4.1 sub-principle ("small-cardinality vs large-cardinality identifier-keyed maps retype in different Passes per call-site density") may surface for D-NNN promotion at Pass 3 milestone close once a third recurrence makes the pattern durable.

### §3.3 Verification at Commit 1

- `cargo test -p xgen-common -p xgen-core` clean (no behavioural change).
- `cargo clippy -p xgen-common -p xgen-core --lib --all-features -- -D warnings` clean.
- `cargo clippy -p xgen-common -p xgen-core --tests -- -D warnings` clean.
- `grep -rn 'J-NNN' . --include='*.rs' --include='docs/*.md' --include='tasks/*.md'` returns: ONE expected match (this runbook's milestone-close placeholder in §5 — freezes at Commit 3) and any sites this Commit 1 introduces with the J-NNN-placeholder convention.

### §3.4 Checkpoint #1 fires after Commit 1 lands

Clair pauses post-Commit-1 and surfaces the four drift-detection points per §2.3 to Joe. Joe approves or surfaces drift. Implementation does NOT proceed to Commit 2 until checkpoint #1 closes affirmatively.

---

## §4 Commit 2 — xgen-core algorithm-bearing retypes (all five surfaces atomic)

**Scope:** All five Pass 2 surfaces retyped per design doc §2 Q-tables, atomic. Adds per-surface unit tests where the retype changes observable behaviour (most surfaces have no behavioural change — type-only retype).

### §4.1 Pre-Commit-2 checkpoint #2

Before any code lands in Commit 2, Clair runs Checkpoint #2 per §2.3 — extracts the verbatim surface list from design doc §2 and gets explicit Joe approval. The checkpoint exists to catch any divergence between the design doc's locked surfaces and what Clair is about to write, before the writing starts.

### §4.2 Surface #1 — `validate_event` + `ValidationOutcome`

**Location:** `xgen-core/src/message/exchange.rs`

**Retypes per design doc §2.1 Q1-Q5:**

1. **`ValidationOutcome::HeldPending`** field types:
   - `missing_predecessors: Vec<String>` → `Vec<EventXgid>`
   - `missing_identity: Option<String>` → `Option<IdentityXgid>`

2. **`ExchangeError` variants**:
   - `HeldPending(Vec<String>)` → `HeldPending(Vec<EventXgid>)`
   - `NotARoomMember(String)` → `NotARoomMember(RoomXgid)`
   - **Stay unchanged** (descriptive-string slots): `DagError(String)`, `PermissionDenied(String)`, `AiCapabilityViolation(String)`, `AiRoleViolation(String)`. Per design doc §3 — descriptive-string slots stay `String`.

3. **`validate_event` body internal projections**: convert from `event.event_id.as_ref().map(|e| e.as_str())` pattern to typed-reference binding. The function takes a `&Event` parameter; `event.sender` is already `IdentityXgid` from Pass 1; internal variables now bind as `&IdentityXgid` etc.; `&str` projection happens at the call-site boundary where consumers like `IdentityRegistry::contains` accept either `&str` (legacy signature, soon to be retyped at Surface #4) or `&IdentityXgid` (post-retype).

4. **`fed_add_via_federation: bool` parameter**: no change. Flag, not identifier.

5. **`validate_steps_8_13` (deprecated legacy path) + `accept_event`**:
   - Add `#[deprecated(note = "Deprecated at XGID Retrofit Pass 2 per design §4.2 Q5.b; removal scheduled as own audit-design-impl arc per D-071. Production callers should migrate to `validate_event` (F-4 unified core). Test fixtures may continue to use during the deprecation window.")]` attribute on both functions.
   - Minimally retype: signatures' identifier-bearing parameters bind to typed XGID references at call-site boundary; function bodies update only the bindings that compile-block, not the bodies' internal logic.
   - Code-comment block at each function explaining the deprecation + scheduled removal + Pass 5 candidate-for-test-fixture-migration framing.

### §4.3 Surface #2 — `NodeRuntime::dispatch_event` + `DispatchOutcome` + `NodeRuntime` struct (partial)

**Location:** `xgen-core/src/node/runtime.rs`

**Retypes per design doc §2.2 Q2.1-Q2.8:**

1. **`DispatchOutcome::Accepted` field types**:
   - `new_joiner: Option<String>` → `Option<IdentityXgid>` (Q2.1)
   - `additional_persisted: Vec<Event>` — no change (Q2.3; already typed)

2. **`DispatchOutcome::Rejected(String)`** — no change (Q2.2; descriptive-string slot)

3. **`EventOrigin`** enum — no change (Q2.4; no identifiers)

4. **`NodeRuntime` struct fields (partial retype per design §4.1 Q2.8.c)**:
   - `node_id: String` → `node_id: NodeXgid` (Q2.5)
   - `peer_urls: HashMap<String, String>` → `HashMap<NodeXgid, String>` (Q2.6; key retypes, URL value stays `String`)
   - **Stay unchanged at Pass 2**: `spaces`, `stores`, `graphs`, `pending`, `dm_proposals`, `space_local_metadata` (all six per-space `HashMap<String, _>` maps). Per design doc §4.1 Q2.8.c — large-cardinality per-space maps defer to Pass 3 where xgen-node's call sites get touched anyway.
   - Code-comment block at the struct definition recording the Pass 2 partial-retype rationale + Pass 3 forward-flag for the six deferred maps. Sibling-shape to Pass 1's `Borrow<str>` lock comment at `xgen-common/src/xgid.rs`.

5. **`dispatch_event` body internal projections** (Q2.7): bind variables as typed references; project at call-site boundary. Particular attention to the drain helpers (`drain_pending_uniform`, `drain_pending_by_identity`, `drain_pending_by_federation_relationship`) — these were retyped at Surface #3 but called from `dispatch_event`'s body; the call-site projection rule applies.

6. **`peer_urls` insertion sites**: where production code inserts `(String, String)` pairs into `peer_urls`, the insert side needs an owned `NodeXgid` constructed from `Xgid::new(string)`. Borrow<str> lookups (`peer_urls.get(&str_id)`) continue to work via Pass 1's additive API.

### §4.4 Surface #3 — `PendingBuffer` arrival hooks

**Location:** `xgen-core/src/dag/pending.rs`

**Retypes per design doc §2.3 Q3.1-Q3.5:**

1. **`add()` signature** (Q3.1):
   - `missing_predecessors: &[String]` → `&[EventXgid]`
   - `missing_identity: Option<&str>` → `Option<&IdentityXgid>`
   - `missing_federation_relationship: Option<(&str, &str)>` → `Option<(NodeXgid, SpaceXgid)>` (or `Option<(&NodeXgid, &SpaceXgid)>` — Clair picks the cleaner shape at implementation per latitude; the internal storage is already keyed by `(NodeXgid, SpaceXgid)` so the input shape is whatever projects most cleanly to that storage)

2. **`resolve()` signature** (Q3.2): `resolved_id: &str` → `&EventXgid`

3. **`resolve_identity()` signature** (Q3.3): `resolved_identity_id: &str` → `&IdentityXgid`

4. **`resolve_federation_relationship()` signature** (Q3.4): `resolved_peer: &str, resolved_space: &str` → `&NodeXgid, &SpaceXgid`

5. **Drop bridge wraps in method bodies** (Q3.5): any `Xgid::new(...)` wraps inserted at Pass 1 as deferred bridges disappear. Method bodies now consume the typed references directly through the already-typed internal storage.

### §4.5 Surface #4 — `IdentityRegistry` + `FederationRegistry` method APIs

**Locations:** `xgen-core/src/identity/registry.rs` + `xgen-core/src/federation/registry.rs`

**Retypes per design doc §2.4 Q4.1-Q4.7:**

1. **`IdentityRegistry::get(identity_id: &str)`** → `get(identity_id: &IdentityXgid) -> Option<&IdentityRecord>` (Q4.1)

2. **`IdentityRegistry::contains(identity_id: &str)`** → `contains(identity_id: &IdentityXgid) -> bool` (Q4.2)

3. **`register()` + `upsert()`** — no signature change (Q4.3, Q4.4; already take typed `IdentityRecord`)

4. **`FederationRegistry` parallel methods** (Q4.5): apply same retype shape to identifier-bearing method parameters. Specific methods: `get_relationship`, `contains_relationship`, `mark_active`, `mark_lost`, `peer_record`, `verify_event_signature`, etc. — each retypes its `&str` identifier parameters to `&NodeXgid` / `&IdentityXgid` per the parameter's semantic role.

5. **Drop bridge wraps in method bodies** (Q4.6): any `Xgid::new(...)` wraps from Pass 1 deferrals disappear.

6. **Internal HashMap keys already typed** (Q4.7): no storage-layer changes needed.

### §4.6 Surface #5 — `accept_message` signature

**Location:** `xgen-core/src/node/runtime.rs`

**Retypes per design doc §2.5 Q5.1-Q5.4:**

1. **`accept_message` signature parameters** (Q5.1): identifier-bearing parameters retype to typed XGID references at the call-site boundary. Specific parameters to inspect: `space_id` (→ `&SpaceXgid`), and any others depending on the current signature. The `event: Event` parameter is already typed from Pass 1.

2. **Function body internal projections** (Q5.2): bind as typed references per the project-wide principle (design doc §3).

3. **`validate_steps_8_13` call propagation** (Q5.3): `accept_message` flows through the deprecated `validate_steps_8_13`; accept the propagated retype for compatibility (the deprecation attribute on `validate_steps_8_13` from Surface #1 applies regardless).

4. **Deprecation-audit flag** (Q5.4): add a code-comment block at `accept_message` flagging it as candidate-for-deprecation-audit per design doc §4.2 audit-design-impl arc — kept active in Pass 2 because xgen-node clients call it directly, but may join the deprecation removal arc alongside `validate_steps_8_13` + `accept_event`.

### §4.7 Per-surface unit tests

Most surfaces are type-only retype with no behavioural change. Tests added selectively:

- **Surface #1 — `ValidationOutcome` payload retypes**: a roundtrip test confirming `HeldPending` constructed with a `Vec<EventXgid>` round-trips through serde correctly if `ValidationOutcome` is serializable (likely not — it's a runtime outcome, not a wire type — verify at implementation kickoff). If non-serializable, no test added.
- **Surface #2 — `DispatchOutcome::Accepted.new_joiner` retype**: a unit test constructing the variant with a typed `Option<IdentityXgid>` and asserting the field's type at pattern-match.
- **Surface #3 — `PendingBuffer` arrival hooks**: extend the existing `PendingBuffer` test module's coverage to call the retyped method signatures with typed references, asserting compile-time correctness. Behavioural tests already cover the resolution logic; signature retypes are a type-system change.
- **Surface #4 — `IdentityRegistry` + `FederationRegistry` method retypes**: add unit tests calling `contains(&IdentityXgid)` and `get(&IdentityXgid)` with typed references, confirming the post-retype API surface compiles and behaves correctly.
- **Surface #5 — `accept_message` signature**: no behavioural change at the function level. Unit test added at the call-site boundary if practical.

Clair has latitude on exact test names and placement; the rule is that each surface gets at minimum one test asserting the post-retype API surface compiles + behaves correctly. Sibling-shape to Pass 1's per-commit unit test discipline (J-122 Commit 3 + Commit 4 + Commit 4a + Commit 5).

### §4.8 Checkpoint #3 fires after Commit 2 lib retypes verify clean

After all five surfaces' lib-level retypes land and `cargo test -p xgen-common -p xgen-core --lib` is clean + `cargo clippy --lib` clean, Clair runs `cargo test -p xgen-common -p xgen-core --tests` to surface the test-fixture error count. Reports the count to Joe; Joe locks single-Commit-2 (absorb fixture sweep before Commit 2 lands) or split (ship Commit 2 lib-clean + ship Commit 2a fixture sweep).

### §4.9 Verification at Commit 2

At the Commit 2 milestone-bearing boundary (either single-Commit-2 or Commit-2-lib-clean-before-Commit-2a-fires):

- 5 isolated runs of `cargo test -p xgen-common -p xgen-core` (with `cargo clean` between each).
- 3 workspace-scoped runs of `cargo test -p xgen-common -p xgen-core`.
- Total: 8 green runs minimum.
- `cargo clippy -p xgen-common -p xgen-core --lib --all-features -- -D warnings` clean.
- `cargo clippy -p xgen-common -p xgen-core --tests -- -D warnings` clean (if single-Commit-2; deferred to Commit 2a if split).

---

## §4a Commit 2a — Test-fixture projection sweep [CONTINGENT]

**Scope:** Test-fixture files outside `xgen-core/src/` that consume retyped types and need projection updates. Fires only if Joe-lock checkpoint #3 (post-Commit-2 lib retypes verified clean) records test-fixture error count > ~50.

**If fired:**

- All test-fixture errors from Commit 2's `cargo test --tests` resolved.
- No lib code touched (Commit 2 owns lib retypes; Commit 2a is strictly test-fixture).
- Verification at Commit 2a milestone-bearing boundary inherits the same 5+3 = 8 green runs rigour from §2.4.

**If not fired** (error count ≤ ~50):

- Test-fixture sweep absorbed into Commit 2 as follow-up edits before Commit 2 lands.
- This §4a section becomes inert at runbook execution time but stays in the runbook as the locked posture for sibling-milestone reference.

**Sibling-shape posture for runbook completeness.** §4a is intentionally short — the trigger criterion (§2.3 checkpoint #3) and the verification rigour (§2.4) carry the load; the substantive scope (test-fixture-projection-only) is the inverse of Commit 2 (lib-only). Pass 1's Commit 4a sized ~550 LOC at ~296 fixture errors; Pass 2's Commit 2a (if fired) sizes proportionally smaller per the smaller surface count.

---

## §5 Commit 3 — Milestone close

**Scope:** Status flips + ROADMAP + JOURNAL + CLAUDE PLAY flip + freeze J-NNN placeholders per D-074 milestone-close + Lock #3 per-commit cadence.

### §5.1 Files touched (6-8)

1. **`tasks/XGID_RETROFIT_PASS_2_IMPL.md`** (this runbook) — Status ACTIVE → COMPLETED, Version 1.0 → 2.0 (or 1.x per Clair's session-close state), Last-updated chain entry recording milestone-close summary including test count at close + verification rigour confirmation + layered-B3 audit answer + the two Joe-lock checkpoints' outcomes.

2. **`tasks/XGID_RETROFIT_PASS_2_DESIGN.md`** — header `Last updated` chain entry only (already COMPLETED at v1.1 from Commit 1). New milestone-close pointer at design doc §6.7 (Pass 2 milestone-close reference).

3. **`docs/ROADMAP.md`** — version bump 1.35 → 1.36; visual structure tree's XGID Retrofit Pass 2 row flipped 🟢 → ✅ with full commit detail (Commit 1 hash + Commit 2 hash + Commit 2a hash if applicable + this Commit 3 milestone close); Present section's Pass 2 PLAY entry collapsed to ⬛-CLOSED pointer referencing Past; Past section gains the Pass 2 milestone closure paragraph under the XGID Retrofit cluster (sibling-shape to Pass 1 milestone closure paragraph at J-122 Past entry); Near future loses the now-shipped Pass 2 line; header chain entry.

4. **`CLAUDE.md`** — PLAY block flipped from "XGID Retrofit Pass 2 implementation ACTIVE — Clair pickup at runbook §4 Commit 2" to "standby for next-milestone selection — Pass 3 + M6 (new) both ready; sequencing is Joe's call" or equivalent; header `Last updated` chain entry recording milestone-close summary.

5. **`JOURNAL.md`** — new J-NNN entry recording milestone close (sub-sections per HANDOFF/precedent spec; freeze J-NNN placeholders below).

6. **`xgen-core/src/message/exchange.rs`** — freeze any J-NNN placeholders in deprecation code-comment blocks added at Commit 2 Surface #1 to the actual J-NNN milestone-close number.

7. **`xgen-core/src/node/runtime.rs`** — freeze any J-NNN placeholders in code-comment blocks added at Commit 2 Surface #2 + Surface #5 (deferred-maps comment block, deprecation-audit flag comment).

8. **`docs/ROADMAP.md`** Cross-cutting section — if the §4.1 sub-principle promoted to D-NNN at Pass 3 (per design doc §6.3 promotion-watch), update the cross-cutting principles section to reference. **Likely not at Pass 2 close** — pattern's three-instance threshold not yet met; flag stays at flagged-not-promoted.

### §5.2 J-NNN freeze guardrail

Per J-108 codified discipline:

```
grep -rn 'J-NNN' . --include='*.rs' --include='docs/*.md' --include='tasks/*.md'
```

MUST return ZERO matches post-staging. Unconstrained `grep -rn 'J-NNN' .` returns expected hits in CLAUDE.md and JOURNAL.md narrative prose (historical-pointer use at authoring time, not freeze sites); the constrained grep is the authoritative verification.

### §5.3 Layered-B3 audit answer

Pass 2's milestone-close DoD includes the layered-B3 audit question per established discipline (introduced at topo-sort J-101; recurred at persistence-amendment J-108; Pass 1 J-122 found zero). The audit asks: did any sibling-layer encoding of an invariant Pass 2 retyped surface during implementation, requiring atomic closure within a Pass 2 commit?

**Expected answer at Pass 2:** zero. Sibling-shape to Pass 1's J-122 finding — Pass 2's scope is algorithm-bearing function shape, not validator-companion structural invariants; the principle's projection mechanism (Borrow<str>) handles type-projection at boundaries without secondary encoding surfaces.

If a layered-B3 surface emerges during Commit 2 implementation, Clair surfaces it at Joe-lock checkpoint #3 alongside the test-fixture error count; the surface may justify scope expansion of Commit 2 + Commit 2a per the topo-sort J-101 + persistence-amendment J-108 precedent.

### §5.4 Test count at close

Pass 2 Commit 3 test count expectation: ≥ 489 (Pass 1's J-122 baseline) + per-surface unit tests added at Commit 2 (§4.7). Specific counts emerge at implementation time; the milestone-close entry records the exact figure.

Workspace test count (`cargo test --workspace`) stays broken per Path A inherited from Pass 1; Pass 5 close restores the workspace count to ≥ 627 + Pass-arc additive tests.

### §5.5 Verification at Commit 3

- `cargo test -p xgen-common -p xgen-core` clean (count recorded above).
- `cargo clippy -p xgen-common -p xgen-core --lib --all-features -- -D warnings` clean.
- `cargo clippy -p xgen-common -p xgen-core --tests -- -D warnings` clean.
- `grep -rn 'J-NNN' . --include='*.rs' --include='docs/*.md' --include='tasks/*.md'` returns ZERO matches.

---

## §6 Verification gate + Definition of Done

### §6.1 Definition of Done (Pass 2 milestone)

Pass 2 milestone closes when ALL of the following hold:

1. ✅ All five surfaces per design doc §2 retyped per the locked Q-tables.
2. ✅ All deprecation attributes (`#[deprecated]`) applied to `validate_steps_8_13` + `accept_event` per design doc §4.2 Q5.b.
3. ✅ `NodeRuntime` partial retype per design doc §4.1 Q2.8.c (node_id + peer_urls retyped; six per-space HashMap keys explicitly deferred to Pass 3 with code-comment block recording the rationale).
4. ✅ All Pass-1-deferred `Xgid::new(...)` bridge wraps in retyped functions' bodies dropped.
5. ✅ All J-NNN placeholders introduced at Commit 1 or Commit 2 frozen at Commit 3 milestone-close J-NNN number.
6. ✅ Verification rigour 5 isolated + 3 workspace = 8 green runs minimum on `cargo test -p xgen-common -p xgen-core` at Commit 2 milestone-bearing boundary (and at Commit 2a if split fires).
7. ✅ `cargo clippy --lib --all-features -- -D warnings` clean at Commit 3.
8. ✅ `cargo clippy --tests -- -D warnings` clean at Commit 3.
9. ✅ `grep -rn 'J-NNN' . --include='*.rs' --include='docs/*.md' --include='tasks/*.md'` returns ZERO matches post-staging.
10. ✅ Layered-B3 audit answer recorded in Commit 3 milestone-close JOURNAL entry (expected: zero).
11. ✅ Three Joe-lock checkpoints (#1 post-Commit-1, #2 pre-Commit-2, #3 post-Commit-2 split-trigger) all closed affirmatively.

### §6.2 What is NOT in Pass 2's DoD

- **Workspace test count restoration.** Inherited broken-workspace posture from Pass 1 per Path A. Restoration is Pass 5's DoD.
- **Removal of `validate_steps_8_13` + `accept_event`.** Deprecation only at Pass 2; removal is its own future audit-design-impl arc per design doc §4.2 + D-071.
- **Six per-space HashMap key retypes.** Explicitly deferred to Pass 3 per design doc §4.1 Q2.8.c.
- **Test fixture builders / integration test helpers / trace event field types / log line formatters / Debug-Display impls.** All belong to Pass 5.
- **xgen-node + xgen-client surfaces.** Pass 3 + Pass 4.
- **Appendix D, Appendix F, Appendix G, `xgen_aicontrol_implementation.md`, Ch6 §6.15.** Each per its scheduled Pass.

### §6.3 Pre-existing flakes carry forward

Two known intermittent flakes live in xgen-node and don't apply at Pass 2's xgen-core package scope:
- precedence env-var race (from J-079, ~10–20% workspace runs).
- `reconnect_with_existing_tip_small_delta_delivered` (Phase 3 test surfacing under Phase 4 parallelism, ~10% workspace runs).

Pass 2's verification is naturally isolated from these — package-scoped to xgen-common + xgen-core, where neither flake fires.

---

## §7 Discipline notes

### §7.1 Precedent-departure self-defense (full reasoning at §1.2)

Pass 2's runbook targets ~40-50 KB, lighter than Pass 1's ~65 KB and the trilogy's ~80-100 KB. The departure is honest per D-065: Pass 2's design surface is smaller (five algorithm-bearing functions vs Pass 1's data-structure-plus-two-appendices); the architectural work was done at Pass 1 (code-comment flags pre-locate every Pass 2 surface); the contingent-split posture is pre-locked rather than emerging mid-implementation.

### §7.2 Pass-internal-consistency over trilogy-internal-consistency

Per design doc §7.7 + this runbook's §1.2: when the five-Pass XGID Retrofit arc and the audit-design-impl trilogy precedents conflict on shape, Pass-internal consistency wins. Pass 2 sits inside the XGID Retrofit arc; its closer sibling precedent is Pass 1 (also lighter than trilogy at runbook authoring). Pass 3 + Pass 4 + Pass 5 should evaluate their own walk-findings against Pass-internal precedent (Pass 1 + Pass 2) before defaulting to trilogy shape.

### §7.3 Honest framing of contingent-split posture

The single-Commit-2 shape is the expected-default. Commit 2a is contingent on Joe-lock checkpoint #3's trigger criterion (test-fixture error count > ~50). Honest per D-065: framing the split as contingent rather than forcing it pre-acknowledges that the expected-default is achievable, while preserving the discipline to split if reality says otherwise. Sibling-shape to Pass 1's Commit 4 + Commit 4a Joe-lock-at-implementation-time pattern, but pre-locked rather than mid-implementation surfaced.

### §7.4 Inline-lock pattern (fifth-Pass recurrence)

Pass 2's design phase closed all Q-decisions inline (Q1-Q5 + Q2.1-Q2.8 + Q3.1-Q3.5 + Q4.1-Q4.7 + Q5.1-Q5.4). The inline-lock pattern's fifth recurrence at the XGID-Retrofit-Pass-arc layer (Pass 1 design closed Q1-Q6 inline; Pass 2 closes Q1-Q5 + sub-questions inline). Pattern's durability now matches the trilogy's three-recurrence threshold at the federation-milestone layer.

### §7.5 `Borrow<str>` projection mechanism is load-bearing

Pass 2's design principle (design doc §3) relies on Pass 1's Commit 4 implementation-kickoff Joe-lock — the additive `Borrow<str>` API on `Xgid` + flavour wrappers. Without this API, every HashMap lookup at the call site would require wrapping the `&str` query into a typed `Xgid` newtype (allocation + construction cost per lookup). With it, `HashMap<NodeXgid, V>::get(&str)` works directly. Pass 2's retype is structurally cheap *because Pass 1 paid the structural cost up-front*; if Pass 1 had taken Q5.c (skip the Borrow<str> addition), Pass 2 would be substantially heavier and the deferred-maps decision (Q2.8.c) would tilt toward retype-now (Q2.8.b).

### §7.6 "Honest longer work over fast shortcuts" — Pass 2 count starts at zero

Per design doc §7.6 + the established framing (J-101 + J-108 + J-122 precedent): recurrences are counted at milestone-events, not design-events or runbook-authoring-events. Pass 2's count starts at zero at this runbook landing. The count increments if a recurrence fires during Commit 1 / Commit 2 / Commit 2a / Commit 3 implementation. Pass 1's milestone-scope final count was one (J-121 hygiene atom).

### §7.7 Layered-B3 audit answer — expected null

Per §5.3 + design doc §5.5: Pass 2 expects zero layered-B3 surfaces. Pass 1's J-122 found zero (validators consume typed fields as `&str` projections through `Borrow<str>` — naturally type-clean). Pass 2 inherits the absence: the algorithm-bearing functions consume typed fields through `Borrow<str>` exactly the way Pass 1's validators did; no secondary encoding of the same invariant is expected to surface.

If one does surface during Commit 2 implementation, the layered-B3 second-surface-closes-atomically discipline applies (topo-sort Commit 2a + persistence-amendment Commit 4 precedent) — the surface gets scoped into the same atomic that closes the primary fix.

### §7.8 D-069 audit-vs-design boundary at Pass 2

Pass 2 design doc §4.1 surfaces a candidate sub-principle ("small-cardinality vs large-cardinality identifier-keyed maps retype in different Passes per call-site density") flagged-not-promoted at one instance. Per D-069, this stays flagged at Pass 2 implementation; promotion-watch opens at Pass 3 milestone close if a third recurrence makes the pattern durable. Sibling-shape to candidate D-NNN "ingest path invariant encoding under bidirectional sustainability discipline" at persistence-amendment milestone (J-105/J-107) still flagged-not-promoted awaiting future-walk.

---

## §8 Cross-references

### §8.1 Pass 2 sources

- **Pass 2 design doc:** `tasks/XGID_RETROFIT_PASS_2_DESIGN.md` (COMPLETED v1.0 at J-123; flips to v1.1 at Commit 1).
- **Pass 2 design close:** J-123 (2026-05-27) — four-file atomic commit per D-074 (twentieth instance).

### §8.2 Pass 1 (closed)

- **Pass 1 implementation runbook:** `tasks/XGID_RETROFIT_PASS_1_IMPL.md` (COMPLETED v2.1 at J-122).
- **Pass 1 milestone close:** J-122 (2026-05-26) — seven atomic commits on main; +J-121 hygiene atom. Test count at close: 489.
- **Pass 1's `Borrow<str>` lock:** J-122 sub-section 2 records the Commit 4 implementation-kickoff Joe-lock; load-bearing for Pass 2's design principle (§7.5 in this runbook).

### §8.3 XGID Adoption v1 (closed)

- **DECISIONS.md D-072** — XGID Adoption v1 (canonical principle).
- **DECISIONS.md D-073** — Field-name-vs-type discipline.
- **`docs/xgen_appendix_j_en.md`** — XGID specification with §J.4 (immutability), §J.5 (per-call-site witness pattern), §J.9 (wire-format invariance with rejected-proposal examples), §J.10 (rename-a-Space worked example), §J.11 (Shape γ + ASAP staged retrofit principle).
- **`docs/xgen_ch3_specification.md` §3.0** — six normative subsections covering XGID adoption.

### §8.4 No-drift-surface discipline family

Pass 2 does NOT add a new D-NNN at the family layer. The family's current members:

- **DECISIONS.md D-067** — Code-organisation layer.
- **DECISIONS.md D-070** — Transport-layer correlation pair.
- **DECISIONS.md D-075** — Event-model layer.
- **DECISIONS.md D-076 v1.1** — Wire-format layer (causal-DAG-respecting-order amendment).
- **DECISIONS.md D-077** — Meta-layer bidirectional sustainability discipline.
- **DECISIONS.md D-078** — Protocol-test-layer production-grounded test enumeration.

The §4.1 sub-principle ("small-cardinality vs large-cardinality identifier-keyed maps retype in different Passes per call-site density") may surface for D-NNN promotion at Pass 3 milestone close once a third recurrence makes the pattern durable.

### §8.5 Trilogy precedent runbooks (for sibling-in-shape reference)

- **`tasks/FEDERATION_TOPOSORT_IMPL.md`** (COMPLETED v1.2, ~93 KB, eight sections + §11 re-walk amendment).
- **`tasks/FEDERATION_BIDIRECTIONAL_NODES_IMPL.md`** (COMPLETED v1.1, eight sections).
- **`tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_IMPL.md`** (COMPLETED v1.2, ~95 KB, eight sections).

Pass 2 runbook departs per §7.1 precedent-departure self-defense.

### §8.6 Code surfaces at Pass 2 scope

- `xgen-core/src/message/exchange.rs` — Surface #1 + Surface #5 consumer.
- `xgen-core/src/node/runtime.rs` — Surface #2 + Surface #5.
- `xgen-core/src/dag/pending.rs` — Surface #3.
- `xgen-core/src/identity/registry.rs` — Surface #4 (IdentityRegistry).
- `xgen-core/src/federation/registry.rs` — Surface #4 (FederationRegistry).

### §8.7 Forward references

- **Pass 3:** Will retype xgen-node call sites + Appendix D. Includes the six deferred per-space HashMap keys per Pass 2 design doc §4.1 Q2.8.c.
- **Pass 4:** Will retype xgen-client + AI control docs.
- **Pass 5:** Will retype test fixtures + helpers + trace event field types + workspace test count restoration.
- **Deprecation removal audit-design-impl arc per D-071:** Will remove `validate_steps_8_13` + `accept_event` (and candidate `accept_message` per design doc §2.5 Q5.4) after Pass 5 close.

---

**End of runbook.**
