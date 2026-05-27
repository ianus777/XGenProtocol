# XGID Retrofit Pass 2 — Design
> **Status**: COMPLETED  
> Version: 1.1  
> Date: May 2026  
> **Last updated**: 2026-05-27 (J-126 — Pass 2 milestone CLOSED. §6.7 J-NNN placeholder frozen to J-126. Three-commit Clair-facing sequence on main: Commit 1 `5892e9e` doc-pass (J-125); Commit 2 `22765a0` xgen-core algorithm-bearing retypes (all five surfaces atomic per the locked Q-tables, lib-clean per Path A); Commit 2a `58b94a5` test-fixture projection sweep (Joe-lock checkpoint #3 split-trigger fired at 93 errors > ~50 threshold; sibling-shape to Pass 1 Commit 4a precedent); this Commit 3 milestone-close commit. Test count at close: 491 (34 + 8 + 449); +2 vs J-122 baseline of 489 per per-surface tests from Commit 2. Layered-B3 audit answer: zero, as expected at design close §5.5 (sibling-shape to Pass 1 J-122 finding). All three Joe-lock checkpoints closed affirmatively. "Honest longer work over fast shortcuts" — Pass 2 milestone-scope final count zero recurrences (first project milestone to ship with zero recurrences since the framework was named). D-074: twenty-third instance + thirteenth milestone-close. Per Rule 0 + D-065 + D-067 + D-069 + D-071 + D-074 + D-077 + D-078.) Previous 2026-05-27 (J-125 — Status flipped ACTIVE → COMPLETED + v1.0 → v1.1 at Pass 2 implementation Commit 1 doc-pass per runbook §3.1. Implementation kickoff: pre-Clair audit clean across six dimensions (file paths / type shapes at named anchors / Pass 1 carry-overs `Borrow<str>` + 33 inline markers / contingency surfaces / parallel-milestone drift since J-124 = none / test-count baseline 489 = 34 lib + 8 invariance + 447 xgen-core). New §6.7 entry recording Pass 2 implementation entry per runbook §3.1 item 1 (Shape α — pointer-style, sibling to §6.6 forward-reference style). Body §1–§7 stays authoritative as historical record of design-time locks per the COMPLETED-with-amendments convention. Five-file atomic commit per D-074 (twenty-second instance) + Lock #3 per-commit cadence: (1) this design doc Status + version + header chain + new §6.7; (2) `docs/ROADMAP.md` v1.34 → v1.35 + visual tree Pass 2 row update + Present + Past + header chain; (3) `CLAUDE.md` PLAY block flip from "Pass 2 implementation ACTIVE — Clair pickup at runbook §3 Commit 1" to "Pass 2 implementation ACTIVE — Clair pickup at runbook §4 Commit 2 (Commit 1 doc-pass ✅ at J-125)" + header chain; (4) `JOURNAL.md` J-125 header chain entry; (5) `tasks/XGID_RETROFIT_PASS_2_IMPL.md` header chain entry. **No state transitions in this atom**: Pass 2 milestone stays PLAY (Commit 1 doc-pass is within-milestone); Pass 2 implementation has now begun (Commit 2 production code is next-active for Clair). Per Rule 0 + D-065 + D-067 + D-069 + D-071 + D-074 + D-077 + D-078 discipline.)  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 Framing

### §1.1 What this document is

The Pass 2 design closure for the XGID Retrofit milestone series. Pass 1 closed at J-122 (2026-05-26) with the core data structures in `xgen-common` + `xgen-core` retyped to typed XGID flavours. Pass 2 retypes the **xgen-core algorithm-bearing functions** that consume those data structures: `validate_event` + `ValidationOutcome`, `NodeRuntime::dispatch_event` + `DispatchOutcome`, `PendingBuffer` arrival hooks, `FederationRegistry` / `IdentityRegistry` APIs, and `accept_message`.

This is **Phase A** of Pass 2's audit → design → runbook → implementation → close arc. The audit phase was absorbed into Pass 1's pre-walk reconnaissance (every function in Pass 2's scope already carries `// Pass 2 widens this method to take typed XGIDs; the wrap collapses then` code-comment flags written during Pass 1 implementation) — so Pass 2 opens directly at design with the question set pre-surfaced. Per D-065 honest framing, this design doc is correspondingly lighter than the trilogy precedent (~25-30 KB target vs ~80-100 KB).

### §1.2 Precedent-departure self-defense

This design doc departs from the trilogy precedent (`tasks/FEDERATION_TOPOSORT_DESIGN.md` ~33 KB; `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_DESIGN.md` eleven sections) in three ways:

1. **Substantially lighter at ~25-30 KB target**. The trilogy design docs walked novel architectural questions. Pass 2's surfaces were all Pass-1-anticipated with code-comment flags marking their boundaries. Five surface walks produced one design principle, two architectural decisions, and zero novel layered surfaces. Padding to ~80-100 KB would obscure the honest finding: Pass 1's pre-walk reconnaissance did the architectural work; Pass 2 design phase is principle-locking + decision-recording.

2. **No §11 re-walk reservation**. The trilogy precedent reserved §11 for design-phase re-walk surfaces (topo-sort §11 amended at Step 2 J-099). Pass 2 walked five surfaces in sequence without re-walks firing; the re-walk pattern's value is preserving option to re-walk when later surfaces destabilise earlier locks; no such destabilisation surfaced here. If re-walk fires post-design-close, this document gains §11 amendment in place per topo-sort precedent.

3. **One sole governing principle vs trilogy's multiple Joe-locks**. Topo-sort had three Q-locks (Q3.ii + Q2 + Q1) instantiating D-076; persistence-amendment had four Q-locks (Q1 through Q4) across distinct sub-questions. Pass 2 surfaces decompose to a single principle (§3) applied uniformly with two scope-bounding architectural decisions (§4); the rest is mechanical application.

The three departures are honest reflections of Pass 2's surface, not corner-cutting. Trilogy-internal consistency outranked here by Pass-internal consistency: Pass 2 sits inside the five-Pass XGID Retrofit arc whose closer sibling is Pass 1 (also lighter-than-trilogy at runbook authoring per J-114-J-122 honest framing).

### §1.3 Reading order

For future Chat Claude + Joe sessions reading this document:

1. §3 design principle (the sole governing rule)
2. §4 architectural decisions (Q2.8 partial / Q5 deprecation — the only non-mechanical locks)
3. §2 five-surface walk (substantive content; locks per surface)
4. §5 honest-broadening discipline (which deferred surfaces belong to which Pass)
5. §6 cross-references

For Clair reading via the runbook (authored next session): runbook will reference this design doc by §-number for the locks; Clair reads runbook first, drops into this doc only at the cited §-numbers.

### §1.4 Latitude

Latitude during implementation belongs to Clair within the bounds of:
- The design principle (§3) is load-bearing — call-site-boundary projection via `Borrow<str>` is the Pass 2 mechanism; no `Deref<Target = str>` shortcuts.
- The two architectural decisions (§4) are Joe-locked.
- Mechanical locks (Q1-Q5, Q2.1-Q2.8, Q3.1-Q3.5, Q4.1-Q4.7, Q5.1-Q5.4) follow the principle uniformly; Clair has latitude on test-fixture details, error-message rephrasing for typed payloads, and minor projection wrapper placement so long as the projection happens at the call-site boundary (not the function body interior).

Any surfacing that contradicts the design principle is a Rule 3 STOP condition — surface for Joe-lock per established discipline.

---

## §2 Five-surface walk

The five Pass 2 surfaces per ROADMAP's Pass 2 row, walked in dependency order (bottom-of-stack first):

### §2.1 Surface #1 — `validate_event` + `ValidationOutcome`

**Location:** `xgen-core/src/message/exchange.rs` — F-4 unified validation core (design doc §7.4) and the legacy `validate_steps_8_13` sibling.

**Current state at Pass 2 open:**
- `ValidationOutcome::HeldPending` carries `missing_predecessors: Vec<String>` + `missing_identity: Option<String>`.
- `ValidationOutcome::Rejected(ExchangeError)` — `ExchangeError` carries multiple identifier-bearing payloads (`HeldPending(Vec<String>)`, `NotARoomMember(String)`) and multiple descriptive-string payloads (`DagError(String)`, `PermissionDenied(String)`, `AiCapabilityViolation(String)`, `AiRoleViolation(String)`).
- Function body internal projections bind variables as `&str` via `event.event_id.as_ref().map(|e| e.as_str())` pattern — explicit projection at boundary, no typed-reference binding.
- `validate_steps_8_13` (legacy pre-F-4 path) still exists and is called from `accept_event` + xgen-core test fixtures directly.

**Locked decisions:**

| Q | Decision |
|---|---|
| Q1 (`ValidationOutcome` payloads) | `missing_predecessors: Vec<EventXgid>`, `missing_identity: Option<IdentityXgid>` |
| Q2 (`ExchangeError` payloads) | Identifier variants retype: `HeldPending(Vec<EventXgid>)`, `NotARoomMember(RoomXgid)`. Descriptive-string variants stay `String`: `DagError`, `PermissionDenied`, `AiCapabilityViolation`, `AiRoleViolation`. |
| Q3 (internal projections) | Bind variables as `&EventXgid` / `&IdentityXgid`; project to `&str` at the call-site boundary (`Borrow<str>` consumers stay). |
| Q4 (`fed_add_via_federation: bool`) | No change — flag, not identifier. |
| Q5 (`validate_steps_8_13` lifecycle) | Deprecate + retype minimally for call-site compatibility + schedule removal as own future audit-design-impl arc per D-071. Applies symmetrically to `accept_event` (its sole non-test caller in production). |

**Q5 rationale (single architectural decision at this surface):** `validate_steps_8_13` is being kept alive primarily by tests, not by production callers (`accept_event` itself is used only in xgen-core test fixtures; production code flows through `validate_event` / F-4 unified core). Three options walked: full retype (Q5.a) costs Pass 2 effort on code structurally on its way out; skip entirely (Q5.c) leaves the function with typed `Event` inputs but `String`-semantics body — exactly the drift surface Pass 1 was designed to close; deprecate-and-minimally-retype (Q5.b) costs ~10 minutes more than skip in exchange for no drift surface and a clean "deprecated but coherent" state. Q5.b locked.

### §2.2 Surface #2 — `NodeRuntime::dispatch_event` + `DispatchOutcome`

**Location:** `xgen-core/src/node/runtime.rs` — F-4 unified post-validation pipeline + the `NodeRuntime` struct itself.

**Current state at Pass 2 open:**
- `DispatchOutcome::Accepted { new_joiner: Option<String>, additional_persisted: Vec<Event> }` — `new_joiner` carries identity_id per code-comment; `additional_persisted` carries fully-typed `Event` objects.
- `DispatchOutcome::Rejected(String)` carries a human-readable rejection reason.
- `EventOrigin` enum — pure runtime metadata, no identifiers.
- `NodeRuntime` struct: `node_id: String`, plus six `HashMap<String, _>` per-space maps (`spaces`, `stores`, `graphs`, `pending`, `dm_proposals`, `space_local_metadata`), plus `peer_urls: HashMap<String, String>` keyed by node_id.
- `ingest_event` body carries inline code-comment from Pass 1: *"Internal NodeRuntime maps stay keyed by String (NodeRuntime is Pass 2 algorithm-layer scope); project at boundary"* — i.e. Pass 1 explicitly flagged the HashMap retyping question for Pass 2 deliberation.

**Locked decisions:**

| Q | Decision |
|---|---|
| Q2.1 | `DispatchOutcome::Accepted.new_joiner: Option<IdentityXgid>` |
| Q2.2 | `DispatchOutcome::Rejected(String)` — no change (descriptive-string slot) |
| Q2.3 | `additional_persisted: Vec<Event>` — no change (`Event` already typed end-to-end at Pass 1) |
| Q2.4 | `EventOrigin` — no change (no identifiers) |
| Q2.5 | `NodeRuntime.node_id: NodeXgid` |
| Q2.6 | `peer_urls: HashMap<NodeXgid, String>` (key retypes; URL value stays `String`) |
| Q2.7 | Internal `&str` projections → bind as typed references; project at call-site |
| **Q2.8** | **Partial HashMap retyping** — see §4 architectural decision |

### §2.3 Surface #3 — `PendingBuffer` arrival hooks

**Location:** `xgen-core/src/dag/pending.rs` — three arrival-hook methods (`resolve`, `resolve_identity`, `resolve_federation_relationship`) plus `add()`.

**Current state at Pass 2 open:**
- Internal storage **already fully typed**: `events: HashMap<EventXgid, BufferedEntry>`, `waiting_for: HashMap<EventXgid, HashSet<EventXgid>>`, `waiting_for_identity: HashMap<IdentityXgid, HashSet<EventXgid>>`, `waiting_for_federation_relationship: HashMap<(NodeXgid, SpaceXgid), HashSet<EventXgid>>`.
- `TimedOut` struct **already fully typed**: `event_id: EventXgid`, `missing_predecessors: Vec<EventXgid>`, `missing_identity: Option<IdentityXgid>`, `missing_federation_relationship: Option<(NodeXgid, SpaceXgid)>`.
- Public method signatures **still `&str`-typed** with code-comments: *"Pass 2 widens this method to take `&EventXgid`; the wrap collapses then"*, etc.

**Locked decisions:**

| Q | Decision |
|---|---|
| Q3.1 | `add()`: `missing_predecessors: &[EventXgid]`, `missing_identity: Option<&IdentityXgid>`, `missing_federation_relationship: Option<(NodeXgid, SpaceXgid)>` |
| Q3.2 | `resolve()`: `resolved_id: &EventXgid` |
| Q3.3 | `resolve_identity()`: `resolved_identity_id: &IdentityXgid` |
| Q3.4 | `resolve_federation_relationship()`: `resolved_peer: &NodeXgid, resolved_space: &SpaceXgid` |
| Q3.5 | Drop all `Xgid::new(...)` wraps inside method bodies (Pass-1-deferred bridges) |

No architectural surprises at this surface — all mechanical per the design principle.

### §2.4 Surface #4 — `FederationRegistry` / `IdentityRegistry` APIs

**Location:** `xgen-core/src/identity/registry.rs` (IdentityRegistry) and the parallel FederationRegistry surface.

**Current state at Pass 2 open:**
- `IdentityRegistry.records: HashMap<IdentityXgid, IdentityRecord>` — internal storage already typed.
- `IdentityRecord.identity_id: IdentityXgid`, `home_node: NodeXgid` — record fields already typed at Pass 1.
- Method signatures still `&str`-typed: `get(identity_id: &str)`, `contains(identity_id: &str)` with Pass-2-flag code-comments.

**Locked decisions:**

| Q | Decision |
|---|---|
| Q4.1 | `IdentityRegistry::get(identity_id: &IdentityXgid) -> Option<&IdentityRecord>` |
| Q4.2 | `IdentityRegistry::contains(identity_id: &IdentityXgid) -> bool` |
| Q4.3 | `IdentityRegistry::register(record: IdentityRecord)` — no signature change (already takes typed `IdentityRecord`) |
| Q4.4 | `IdentityRegistry::upsert(record: IdentityRecord)` — no signature change |
| Q4.5 | `FederationRegistry` parallel methods — apply same retype to identifier-bearing method parameters |
| Q4.6 | Drop all `Xgid::new(...)` wraps inside method bodies (Pass-1-deferred bridges) |
| Q4.7 | Internal HashMap keys already typed correctly; no change needed at storage layer |

No architectural surprises at this surface — all mechanical.

### §2.5 Surface #5 — `accept_message` signature

**Location:** `xgen-core/src/node/runtime.rs` — the `NodeRuntime::accept_message` method (full 13-step pipeline via `validate_steps_8_13` + store).

**Current state at Pass 2 open:**
- Consumes a typed `Event` parameter (already at Pass 1).
- Routes through `validate_steps_8_13` (the legacy path locked under Q5.b as deprecated at Surface #1).
- Called directly from xgen-node clients per the module doc-comment: *"Used for: message.text events from authenticated clients."*

**Locked decisions:**

| Q | Decision |
|---|---|
| Q5.1 | `accept_message` signature parameters retype where identifier-bearing; `space_id`-typed parameters bind to `&SpaceXgid` at call boundary |
| Q5.2 | Function body internal projections bind as typed references per the project-wide principle |
| Q5.3 | `accept_message` flows through `validate_steps_8_13` (deprecated legacy path per Surface #1 Q5); accept the propagated retype for compatibility |
| Q5.4 | Surface `accept_message` itself as candidate-for-deprecation-audit at runbook authoring time — kept active in Pass 2 because xgen-node clients call it directly, but the deprecation-audit-arc per D-071 may include it alongside `validate_steps_8_13` + `accept_event` |

No architectural surprises at this surface beyond the deprecation-audit flag (which is documentation-only, not a code change).

---

## §3 Pass 2 design principle

The sole governing rule across all five surfaces:

> **Identifier slots retype to typed XGIDs. Descriptive-string slots stay `String`. Internal variables bind as typed references; `&str` projection happens at the call-site boundary via `Borrow<str>`.**

Three sub-properties:

1. **Identifier-vs-text discrimination.** A field/parameter/payload is an "identifier slot" if it carries a value that another part of the protocol uses to look up, match against, or correlate with a typed entity. Descriptive-string slots carry human-readable text (error messages, rule descriptions, event-type names) that the protocol does not use for lookup or matching. The same principle Pass 1 applied at the data-structure layer.

2. **Binding shape.** Local variables inside a function body that hold an identifier value bind as `&EventXgid` / `&IdentityXgid` / `&NodeXgid` / etc., not `&str`. The variable's identity is "this is a typed XGID"; the projection to `&str` is a comparison-site or HashMap-lookup-site detail, performed at the call.

3. **Projection mechanism.** Pass 1 shipped the additive `Borrow<str>` API on `Xgid` and flavour wrappers (per the Commit 4 implementation-kickoff Joe-lock at J-122). Pass 2's call sites use this directly: `id_registry.contains(typed_id)` works where `contains` signature accepts `&impl Borrow<str>` or accepts `&IdentityXgid` (Q4.2). The wrap-with-`Xgid::new(...)` shape that appeared in Pass 1's transitional code disappears entirely at Pass 2.

**No `Deref<Target = str>` added on the base `Xgid`.** Pass 1's posture stands: projection is visible in the call-site syntax (`.as_str()` where explicit; `Borrow<str>` where implicit through HashMap-style APIs), never silent via Deref coercion.

**Honest broadening posture for method signatures.** A method that today takes `&str` and would naturally take `&IdentityXgid` per the principle retypes its signature **iff** the method is in Pass 2's scope (one of the five surfaces). Methods outside Pass 2's scope keep their `&str` parameters with the code-comment flag preserved, deferred to the Pass that touches their primary call-site crate (xgen-node ⇒ Pass 3; xgen-client ⇒ Pass 4; test fixtures ⇒ Pass 5).

---

## §4 Architectural decisions

Two scope-bounding decisions surfaced during the walk; the rest is mechanical per §3.

### §4.1 Q2.8 — Partial HashMap key retyping

**The question:** Pass 1's `ingest_event` body carries an inline code-comment: *"Internal NodeRuntime maps stay keyed by String (NodeRuntime is Pass 2 algorithm-layer scope); project at boundary."* The note pre-flags the HashMap-key retyping question for Pass 2 deliberation — should `spaces: HashMap<String, SpaceState>` and its five siblings retype to `HashMap<SpaceXgid, SpaceState>`?

**Three options walked:**

- **Q2.8.a — Honor the inline-locked posture.** Keep all six per-space maps and `peer_urls` keyed by `String`; project at insertion / lookup boundary via `Borrow<str>`. Cheap, low-risk, preserves Pass 1's additive API working without extra effort.
- **Q2.8.b — Full retype.** All six per-space maps gain `SpaceXgid` keys; `node_id` + `peer_urls` gain `NodeXgid` keys. Honest about what the keys are; consistent with Pass 1's "data structures retype" pattern. Cost: every lookup site needs to construct a typed wrapper for the key OR rely on `Borrow<str>` (which works for `HashMap::get` but not `HashMap::insert` — insert requires owned `SpaceXgid` constructed from `Xgid::new(string)`).
- **Q2.8.c — Partial retype.** Retype `node_id` and `peer_urls` now (Node-identifier surfaces, small-cardinality maps). Defer per-space maps (`spaces`/`stores`/`graphs`/`pending`/`dm_proposals`/`space_local_metadata`) to Pass 3 where xgen-node's call sites get touched anyway and a single sweep is cleaner than two.

**Locked at Q2.8.c (partial).** Rationale:

1. **The existing inline comment predates Pass 1's Commit 4 implementation-kickoff Borrow<str> lock.** Pass 1's transitional thinking assumed wrap-with-`Xgid::new(...)` cost at every insert site; with `Borrow<str>` available, partial retype is cheap at the read-side and the write-side cost is constrained to the surfaces that already need to touch the map structure.

2. **Pass 3 is the natural locus for the per-space sweep.** Pass 3 retypes xgen-node + Appendix D, which is where the per-space maps' primary call sites live (federation_session, fanout, app handlers). A single Pass 3 sweep — touching the maps' definitions in xgen-core AND their call sites in xgen-node atomically — is cleaner than a Pass 2 partial-retype + Pass 3 call-site-sweep two-step.

3. **The asymmetry between Node-identifier surfaces and Space-identifier surfaces is honest.** Node-identifier surfaces (node_id, peer_urls) have small cardinality (~handfuls of peers, one self-node) and their key constructions concentrate in known sites (handshake registry population, replication targeting). Space-identifier surfaces (spaces, stores, graphs, pending, dm_proposals, space_local_metadata) have larger cardinality and their key constructions spread across event ingestion, dispatcher, persistence — all touched by Pass 3 substantively. Partial retype honors the asymmetry.

**Pass 2 design principle extension (codified):**

> Small-cardinality identifier-keyed maps retype in their own Pass. Large-cardinality per-space maps defer to the Pass that touches their primary call-site crate (xgen-node ⇒ Pass 3).

This sub-principle generalises beyond Pass 2 to Passes 3-5: each Pass retypes the maps whose primary call sites it owns, not the maps whose definitions sit in its crate-scope. Inverts the naive crate-boundary heuristic in favour of a call-site-density heuristic. Sibling-shape to D-067's no-drift-surface discipline at code-organisation layer — the heuristic prevents the drift of "Pass 2 retypes the map but Pass 3 has to re-touch every call site anyway."

### §4.2 Q5 — `validate_steps_8_13` + `accept_event` lifecycle

**The question:** `validate_steps_8_13` (the legacy pre-F-4 validation path) still exists in `xgen-core/src/message/exchange.rs` alongside the F-4 unified core `validate_event`. The module's own doc-comment frames `validate_event` as the replacement: *"Replaces the pre-F-4 asymmetry where Path A (messages) ran the full 13-step pipeline while Paths B (MembershipJoin) and C (other state events) skipped signature verification..."* But production code paths and the xgen-core test fixtures still call `validate_steps_8_13` directly (via the sibling `accept_event` helper). What's its lifecycle at Pass 2?

**Three options walked:**

- **Q5.a — Full retype.** Treat as live code; retype all signature parameters identically to `validate_event`. Costs Pass 2 effort on something structurally on its way out.
- **Q5.b — Deprecate, retype minimally, schedule removal.** Add `#[deprecated(note = "...")]`. Minimally retype to compile against typed inputs. Future audit-design-impl arc removes it.
- **Q5.c — Skip entirely.** Leave a code-comment flag. Implies removal is imminent. But leaves the function with typed `Event` inputs and `String`-semantics body — exactly the drift surface Pass 1 was designed to close.

**Locked at Q5.b.** Rationale per the four-paragraph analysis at the surface walk (§2.1): Q5.b costs ~10 minutes more than Q5.c in exchange for no drift surface and a clean "deprecated but coherent" state. Removing in a future arc per D-071 (own audit-design-impl phase) preserves the discipline that lifecycle-change-of-load-bearing-functions surfaces through its own milestone shape rather than as a side-effect of an adjacent Pass.

**Symmetric application to `accept_event`** (its sole non-test production caller pattern is parallel to `validate_steps_8_13`). Both functions get the `#[deprecated]` attribute, minimal retype, and a comment block flagging the future audit-design-impl arc for their removal.

**Audit-design-impl arc scope (forward-flag, not Pass 2 work):** Surveys all production callers of both functions; deprecates with `#[deprecated]` (this Pass 2 work); audits whether removal is safe at xgen-core test-fixture level (likely involves rewriting fixtures to construct events through `validate_event` directly); removes. Per D-071 own-arc discipline. The arc may also pick up `accept_message` itself per Surface #5 Q5.4 flag — runbook authoring sets the framing.

---

## §5 Honest broadening — deferred surfaces

Pass 2's locked principle (§3) and architectural decisions (§4) leave several adjacent surfaces deliberately untouched. Per D-065 honest framing, each gets named here with its Pass-of-record:

### §5.1 Deferred to Pass 3 (xgen-node + Appendix D)

- Six per-space `HashMap<String, _>` maps in `NodeRuntime` per Q2.8.c.
- `dispatch_event(peer_node_id: Option<&str>)` and parallel xgen-node-side `&str` parameters carrying `NodeXgid` semantics.
- `federation_session.rs` peer/space identifier parameters.
- `fanout.rs` peer/space identifier parameters.
- `app.rs` handler-level identifier parameters.
- Reconnect scheduler identifier parameters.

### §5.2 Deferred to Pass 4 (xgen-client + AI control docs)

- `ops::*` layer identifier parameters.
- `AiBehavior` trait identifier parameters.
- `AiPacingTracker` identifier parameters.
- Session state, batch dispatcher, CLI dispatcher identifier parameters.
- AI service / Tauri command identifier parameters.

### §5.3 Deferred to Pass 5 (test fixtures, helpers, remaining surfaces)

- Test fixture builders.
- Integration test helpers.
- Trace event field types.
- Log line formatters.
- Debug / Display impls.

### §5.4 Deferred to own future audit-design-impl arc per D-071

- `validate_steps_8_13` removal (deprecated at Pass 2 per §4.2 Q5.b).
- `accept_event` removal (deprecated at Pass 2 per §4.2 Q5.b symmetric application).
- Candidate inclusion: `accept_message` if Surface #5 Q5.4 audit confirms.

### §5.5 Layered-B3 audit answer

**No layered surfaces detected during the five-surface walk.** Pass 1's milestone close (J-122) recorded the same finding: *"no layered surface emerged — Pass 1's scope is data-structure shape, not algorithm validation; validators consume the typed fields as `&str` projections through `Borrow<str>`."* Pass 2 inherits the absence: the algorithm-bearing functions consume typed fields through `Borrow<str>` exactly the way Pass 1's validators did; no secondary encoding of the same invariant surfaced.

Pass 3 + Pass 4 should ask the layered-B3 audit question at their milestone-close DoD per established discipline. The pattern increments only at code-shipping events that surface layered structure; Pass 2 design-close adds zero instances to the running count (currently at two project-wide: topo-sort Commit 2a + persistence-amendment Commit 4).

---

## §6 Cross-references

### §6.1 Pass 1 (closed)

- **Pass 1 implementation runbook:** `tasks/XGID_RETROFIT_PASS_1_IMPL.md` (COMPLETED v2.1)
- **Pass 1 milestone close:** J-122 (2026-05-26) — seven atomic commits 403ef3f + 8a94dee + 75e81b4 + 774fe9d + 4895446 + 096162e + close; plus J-121 hygiene atom 1dd909e
- **Pass 1's `Borrow<str>` lock:** J-122 sub-section 2 records the Commit 4 implementation-kickoff Joe-lock; load-bearing for Pass 2's design principle (§3 projection mechanism)

### §6.2 XGID Adoption v1 (closed)

- **DECISIONS.md D-072** — XGID Adoption v1 (canonical principle)
- **DECISIONS.md D-073** — Field-name-vs-type discipline
- **`docs/xgen_appendix_j_en.md`** — XGID specification with §J.4 (immutability), §J.5 (per-call-site witness pattern), §J.9 (wire-format invariance with rejected-proposal examples), §J.10 (rename-a-Space worked example), §J.11 (Shape γ + ASAP staged retrofit principle)
- **`docs/xgen_ch3_specification.md` §3.0** — six normative subsections covering XGID adoption

### §6.3 No-drift-surface discipline family

- **DECISIONS.md D-067** — Code-organisation layer
- **DECISIONS.md D-070** — Transport-layer correlation pair
- **DECISIONS.md D-075** — Event-model layer
- **DECISIONS.md D-076 v1.1** — Wire-format layer (with causal-DAG-respecting-order amendment)
- **DECISIONS.md D-077** — Meta-layer bidirectional sustainability discipline
- **DECISIONS.md D-078** — Protocol-test-layer production-grounded test enumeration

Pass 2's design principle (§3) is consistent with the discipline family but doesn't add a new D-NNN at this design close. Candidate D-NNN promotion deferred per D-069 audit-vs-design boundary; the §4.1 sub-principle ("small-cardinality vs large-cardinality identifier-keyed maps retype in different Passes per call-site density") may surface for D-NNN promotion at Pass 3 milestone close once a third recurrence makes the pattern durable.

### §6.4 Trilogy precedent design docs

- **`tasks/FEDERATION_TOPOSORT_DESIGN.md`** (COMPLETED v1.1, ~33 KB, ten sections + §11 re-walk amendment)
- **`tasks/FEDERATION_BIDIRECTIONAL_NODES_DESIGN.md`** (COMPLETED v1.0, nine sections)
- **`tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_DESIGN.md`** (COMPLETED v1.2, eleven sections)

Pass 2 design doc departs from the trilogy shape per §1.2 self-defense; sibling-shape posture for Pass 3 + Pass 4 + Pass 5 design docs is open — each Pass surfaces its own walk, and the trilogy/lighter-shape choice depends on the walk's findings.

### §6.5 Code surfaces at Pass 2 scope

- `xgen-core/src/message/exchange.rs` — Surface #1 `validate_event` + `ValidationOutcome` + Surface #5 `accept_message` consumer
- `xgen-core/src/node/runtime.rs` — Surface #2 `dispatch_event` + `DispatchOutcome` + Surface #5 `accept_message`
- `xgen-core/src/dag/pending.rs` — Surface #3 `PendingBuffer` arrival hooks
- `xgen-core/src/identity/registry.rs` — Surface #4 `IdentityRegistry`
- `xgen-core/src/federation/` — Surface #4 `FederationRegistry`

### §6.6 Forward references

- **Pass 2 implementation runbook:** `tasks/XGID_RETROFIT_PASS_2_IMPL.md` (Status ACTIVE v1.0 at J-124, 2026-05-27; three-commit base + contingent Commit 2a shape; three Joe-lock checkpoints)
- **Pass 2 milestone close:** Single-commit-or-split shape locked at runbook authoring per design close; contingent Commit 2a split-trigger at Joe-lock checkpoint #3 if test-fixture sweep error count > ~50

### §6.7 Pass 2 implementation entry

- **Runbook:** `tasks/XGID_RETROFIT_PASS_2_IMPL.md` (ACTIVE v1.0 at J-124, 2026-05-27).
- **Implementation kickoff:** J-125 (2026-05-27) — Commit 1 doc-pass shipped; pre-Clair audit clean across six dimensions (file paths / type shapes / Pass 1 carry-overs `Borrow<str>` + 33 inline markers / contingency surfaces / parallel-milestone drift = none / test baseline 489).
- **Milestone close:** J-126 (2026-05-27) — Pass 2 milestone CLOSED at three-commit Clair-facing sequence (J-125 Commit 1 doc-pass `5892e9e` + Commit 2 xgen-core algorithm-bearing retypes `22765a0` + Commit 2a test-fixture sweep `58b94a5` per split-trigger at Joe-lock checkpoint #3 over the ~50 threshold + this Commit 3 milestone close). Test count at close 491 (xgen-common 34 lib + 8 invariance + xgen-core 449). Layered-B3 audit answer zero, as expected per §5.5. "Honest longer work over fast shortcuts" Pass 2 milestone-scope final count zero recurrences. All three Joe-lock checkpoints closed affirmatively.

---

## §7 Discipline notes

### §7.1 Precedent-departure self-defense (full reasoning at §1.2)

This design doc is ~25-30 KB target, substantially lighter than the trilogy precedent's ~80-100 KB range. The departure is honest per D-065: Pass 1's pre-walk reconnaissance did the architectural work; Pass 2 design phase is principle-locking + decision-recording, not novel-question deliberation. Padding to trilogy size would obscure the honest finding.

### §7.2 Inline-lock pattern (fourth-Pass recurrence)

All five surface walks closed with inline-locks (Q1-Q5 + Q2.1-Q2.8 + Q3.1-Q3.5 + Q4.1-Q4.7 + Q5.1-Q5.4) rather than open-questions-for-design-walkthrough. This is the inline-lock pattern's fourth recurrence at the XGID Retrofit Pass-arc layer (Pass 1 design walkthrough closed Q1-Q6 inline; Pass 2 closes Q1-Q5 + sub-questions inline; Pass 3 + Pass 4 + Pass 5 will likely follow the same shape). Pattern's durability now matches the trilogy's three-recurrence durability at the federation-milestone layer.

### §7.3 Honest framing of "all mechanical"

Four of the five surface walks recorded *"No architectural surprises at this surface — all mechanical"*. The framing is load-bearing per D-065: it signals the design phase produced a thin output because the architectural work happened earlier (Pass 1 implementation-kickoff Joe-locks + Pass 1 code-comment flags), not because the design phase shortcut anything. The framing's discipline cost is honest recording at each surface; the framing's benefit is preventing future "why was Pass 2 so light?" misreadings.

### §7.4 Pass 2's runbook will be lighter than Pass 1's runbook

Pass 1's runbook was ~65 KB across six commits with an unforeseen mid-implementation Commit 4 → Commit 4a split. Pass 2's design surface is smaller (5 algorithm-bearing functions vs Pass 1's full data-structure surface); the contingent Commit 2a split-trigger is pre-locked at design close rather than emerging mid-implementation; the principle is one rule applied uniformly. Runbook authoring target: ~40-50 KB; explicit per-surface implementation pattern blocks; Joe-lock checkpoints #1 (post-Commit-1 drift) + #2 (pre-Commit-2 verbatim surface list) + #3 (post-Commit-2 split-trigger decision).

### §7.5 Honest-broadening discipline at Pass 2

The §5 deferred-surfaces enumeration is itself a Pass-2-specific discipline application: every adjacent surface that's tempting-to-retype-here gets named with its Pass-of-record, preventing Pass-2-scope-creep at runbook authoring. Sibling-shape to Pass 1's Commit 2 honest-broadening discipline (J-095) where six adjacent String-typed XGID fields were deliberately left untouched with their Passes named.

### §7.6 "Honest longer work over fast shortcuts" — design-close not a recurrence

Per the established framing (J-101 + J-108 precedent), recurrences are counted at milestone-events, not design-events. This design close is *inside* the Pass 2 milestone scope; the count starts incrementing if a recurrence fires during runbook authoring or implementation. Pass 1 milestone-scope final count was one (the J-121 hygiene atom); Pass 2 starts at zero.

### §7.7 Pass-internal-consistency over trilogy-internal-consistency

Per the trilogy-internal vs Pass-internal consistency question at §1.2.3: when the five-Pass arc and the audit-design-impl trilogy precedents conflict on shape, Pass-internal consistency wins. Pass 1 was lighter than trilogy at runbook authoring (per J-114 honest framing); Pass 2 is lighter than trilogy at design authoring. Pass 3 + Pass 4 + Pass 5 should evaluate their own walk-findings against Pass-internal precedent (Pass 1 + Pass 2) before defaulting to trilogy shape.

### §7.8 D-069 audit-vs-design boundary at Pass 2

Pass 2 surfaces a candidate sub-principle at §4.1 ("small-cardinality vs large-cardinality identifier-keyed maps retype in different Passes per call-site density") that may promote to D-NNN at Pass 3 milestone close if a third recurrence makes the pattern durable. Per D-069, this is flagged-not-promoted at the design phase; the audit-design-impl boundary discipline names which Pass surfaces it and which Pass commits to it. Sibling-shape to candidate D-NNN "ingest path invariant encoding under bidirectional sustainability discipline" at persistence-amendment milestone (J-105/J-107) still flagged-not-promoted awaiting future-walk.

---

**End of document.**
