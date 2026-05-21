# XGID Retrofit Pass 1 — Implementation Runbook
> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-21 (skeleton authored; Pass 1 title-vs-scope contradiction Joe-locked as Option B — Pass 1 retypes core data structures regardless of crate, spanning xgen-common and xgen-core; ROADMAP.md v1.4 → v1.5 in same commit set renames the Pass 1 section title accordingly. Four sub-questions framed openly per the session opener: (1) commit sequence shape, (2) `canonical_event_bytes` module-move ordering, (3) bridging strategy for downstream `&str`/`&String` consumers during the retype, (4) invariance test suite extension. Each is a Joe-lock pending; the runbook walkthrough sections (Code scope, Doc scope, Verification gate, Milestone close) are stubbed pending sub-question outcomes.)  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Purpose

This runbook is Clair's build instructions for **XGID Retrofit Pass 1** — the first of five staged retrofit passes per the Shape γ + ASAP discipline locked in D-072. Pass 1 retypes the foundational data structures the protocol surface is built on from `String`-typed XGID fields to flavour-typed XGID newtypes (`EventXgid`, `SpaceXgid`, `RoomXgid`, `TrustAssertionXgid`, `NodeXgid`, `IdentityXgid` — all shipped at XGID Adoption v1 in `xgen-common`).

This runbook is the canonical artefact for Pass 1's scope and commit shape. Where this runbook makes implementation choices that go beyond what the authoritative architectural sources say, the choice is recorded here and stays here.

The authoritative architectural sources Pass 1 inherits:

- `DECISIONS.md` D-072 — XGID Adoption v1 (the architectural commitment; types and adoption discipline).
- `DECISIONS.md` D-073 — Field-name-vs-type discipline (the composition rule: name carries the role, type carries the contract).
- `docs/xgen_appendix_j_en.md` — Canonical expository document (taxonomy, construction, wire-invariance, immutability, type representation, worked rejection examples). §J.11 covers adoption discipline (Shape γ + ASAP); §J.8 covers scope boundaries (what is and isn't an XGID).
- `docs/xgen_ch3_specification.md` §3.0 — Terse normative section.
- `tasks/XGID_ADOPTION_IMPL.md` — XGID Adoption v1 runbook (COMPLETED v1.1). Worth reading for runbook shape, the five invariance test names locked at v1, and the carry-over rationale that Pass 1 picks up.
- `tasks/XGID_DOC_SWEEP.md` — Phase 2 doc-tree sweep classification table (COMPLETED v1.2). The **canonical input** for Pass 1 runbook authoring: identifies which docs land in Pass 1 (Appx C + Appx I) and pins the **coordination flag** that code + Appx C + Appx I must land in one coordinated commit set.

---

## Cross-crate scope (load-bearing, read before scope statement)

Pass 1 deliberately spans **two crates**: `xgen-common` and `xgen-core`. The Phase 2 doc-tree sweep classified Appendix C + Appendix I as Pass 1 deliverables to be shipped in one coordinated commit set, and splitting the data structures across two Passes would split Appendix I documentation across the gap — which the canonical-document rule (D-069) prohibits.

Concretely:

- **xgen-common-resident data structures** Pass 1 retypes: `Event` (`xgen-common/src/wire.rs`); `SpaceLocalMetadata.space_id` (`xgen-common/src/space_local.rs` — the `introducer_node_id` half was already retyped at XGID Adoption v1 Commit 2, only `space_id` remains); observability structs in `xgen-common/src/state.rs` (`NodeState`, `ConnectedClient`, `FederatedPeer`, `HostedSpace`, `HostedRoom`).
- **xgen-core-resident data structures** Pass 1 retypes: `SpaceState` and `RoomState` (`xgen-core/src/space/state.rs`); `FederationRegistry` and `PeerOperationalRecord` keys (`xgen-core/src/federation/registry.rs`); `IdentityRegistry` keys (`xgen-core/src/identity/registry.rs`); `PendingBuffer` map keys (`xgen-core/src/dag/pending.rs`).

**The crate boundary is incidental at the data-structure layer.** Pass 1 owns retyping data-structure *fields*; Pass 2 will own retyping *algorithm-bearing functions in xgen-core* that consume those fields (`validate_event`, `NodeRuntime::dispatch_event`, registry method APIs, `accept_message`, etc.). Pass 1's natural seam is at the data/algorithm boundary, not the crate boundary.

---

## Scope and non-scope

### In scope at Pass 1

**xgen-common data structures:**
- `Event` struct field retypes (`xgen-common/src/wire.rs`): `event_id: Option<EventXgid>`, `sender: IdentityXgid`, `room_id: RoomXgid`, `space_id: SpaceXgid`, `prev_events: Vec<EventXgid>`. The `signature` field stays `Option<String>` (signature strings are not XGIDs — see D-072 "What XGID is not").
- Content structs in `xgen-common/src/wire.rs` that carry XGID-typed fields: `StateAiOperatorDelegateContent` (`space_id`, `ai_identity_id`, `new_operator_identity_id`), `StateAiOperatorRevokeContent` (`space_id`, `ai_identity_id`), `MembershipMuteContent` (`target_identity`), and any other content struct in `wire.rs` whose fields name protocol-object identifiers.
- `SpaceLocalMetadata.space_id` (`xgen-common/src/space_local.rs`): the only remaining `String`-typed XGID field on this struct after v1 Commit 2.
- Observability structs in `xgen-common/src/state.rs`: `NodeState.node_id`; `ConnectedClient.identity_id`; `FederatedPeer.node_id`, `session_id`, `shared_spaces`; `HostedSpace.space_id`; `HostedRoom.room_id`.

**xgen-core data structures:**
- `SpaceState` (`xgen-core/src/space/state.rs`): `space_id: SpaceXgid`, `owner_id: IdentityXgid`, `home_node: NodeXgid`, `members: HashMap<IdentityXgid, SpaceMember>`, `pending_invites: HashMap<IdentityXgid, PendingInvite>`, `ai_operator_delegations: HashMap<IdentityXgid, IdentityXgid>`, room maps keyed by `RoomXgid`, `federation_nodes: Vec<NodeXgid>` (or `HashSet<NodeXgid>` if the existing shape is set-like).
- `SpaceMember` (`xgen-core/src/space/state.rs`): `identity_id: IdentityXgid`, `invited_by: Option<IdentityXgid>`.
- `PendingInvite`: any identity-bearing fields.
- `RoomState`: `room_id: RoomXgid`, `space_id: SpaceXgid`, `members: HashSet<IdentityXgid>`.
- `FederationRegistry` (`xgen-core/src/federation/registry.rs`): keys retyped to `NodeXgid` for both `relationships` and `peer_records` maps. `PeerOperationalRecord.peer_node_id: NodeXgid`. `FederationRelationship` identifier-bearing fields.
- `IdentityRegistry` (`xgen-core/src/identity/registry.rs`): registry keyed by `IdentityXgid`; record-internal identifier fields.
- `PendingBuffer` (`xgen-core/src/dag/pending.rs`): map keys typed `EventXgid` for the primary index; `missing_predecessors: Vec<EventXgid>`; `missing_identity: Option<IdentityXgid>`; `missing_federation_relationship: Option<(NodeXgid, SpaceXgid)>` (or whatever the current shape is); `waiting_for_identity: HashMap<IdentityXgid, HashSet<EventXgid>>`.

**Carry-overs from XGID Adoption v1 milestone close (J-095) — coordinated with the retypes:**
- Move `canonical_event_bytes` (and the surrounding canonical-form helpers `canonical_event_json`, `canonical_object_json`, `canonical_value`) from `xgen-core/src/wire/canonical.rs` to a new `xgen-common/src/canonical.rs`. `xgen-core/src/wire/canonical.rs` becomes a thin re-export module (`pub use xgen_common::canonical::*;`) so existing xgen-core call sites continue to work without changes.
- Add the deferred hash-anchored convenience constructors on the v1 flavour wrappers in `xgen-common/src/xgid/flavours.rs`: `EventXgid::from_event(&Event)`, `SpaceXgid::from_space_create(&Event)`, `RoomXgid::from_room_create(&Event)`, `TrustAssertionXgid::from_assertion(&TrustAssertion)`. These were deferred at v1 because `canonical_event_bytes` lived in xgen-core; with the module move complete, they become implementable in xgen-common.

**Documentation:**
- `docs/xgen_appendix_c_en.md` — primitive schema field tables: every XGID-bearing field's Type column updated from `String` to the appropriate flavour-typed XGID (e.g. `EventXgid`, `IdentityXgid`). Field name unchanged (D-073 invariance 1 — name stays).
- `docs/xgen_appendix_i_en.md` — data structures field tables: same column-by-column treatment, applied to every struct table in Parts I through IX that carries XGID-bearing fields. The Wire-key column stays unchanged (D-072 invariance 1 — wire field names don't drift). The Req/Opt column stays unchanged. The Description column gets minimal edits where the description currently calls the field "a String" or similar type-naming language.

### Out of scope at Pass 1

- **xgen-core algorithm-bearing functions.** `validate_event`, `ValidationOutcome` variant fields, `NodeRuntime::dispatch_event`, `DispatchOutcome` variant fields, `PendingBuffer` arrival-hook signatures (`resolve`, `resolve_identity`, `resolve_federation_relationship`), `FederationRegistry` / `IdentityRegistry` method APIs (`mark_active`, `mark_lost`, `contains`, `get`, `verify_event_signature`, etc.), and `accept_message` signature. All belong to Pass 2.
- **xgen-node code surfaces.** `process_inbound`, `apply_fanout`, `apply_federation_push`, `stream_federation_delta`, `reconnect_scheduler`, etc. All belong to Pass 3.
- **xgen-client code surfaces.** `ops::*` layer, `AiBehavior`, `AiPacingTracker`, session state, batch dispatcher, CLI dispatcher. All belong to Pass 4.
- **Test fixture builders, integration test helpers, trace event field types, log line formatters, debug/Display impls.** All belong to Pass 5.
- **Appendix D, Appendix F, Appendix G, `xgen_aicontrol_implementation.md`, Ch6 §6.15.** Per the Phase 2 doc-tree sweep classification: Appx D → Pass 3; Appx F + AI control doc → Pass 4 (full per-section annotation); Appx G → Pass 5; Ch6 → no-pointer per Scope B but content retypes during Pass 4 when adjacent client code is touched.
- **JOURNAL.md, DECISIONS.md, CLAUDE.md, ROADMAP.md, design docs, audit docs, task files.** Per Phase 2 sweep Scope-B lock — non-normative surfaces take no pointer treatment; new entries naturally use XGID types once Pass 1 ships.

### Honest-broadening warning

At v1, the warning was "resist retyping nearby String fields that obviously could be retyped — they belong to a Pass, not v1." At Pass 1, the analogous discipline applies in the opposite direction: **resist retyping fields that belong to Pass 2 (or later) just because they sit next to a Pass 1 field in the same file.**

Specific places this discipline applies:

- `xgen-core/src/dag/pending.rs` — Pass 1 retypes `PendingBuffer`'s **map keys and stored-record fields**. Pass 2 retypes the **method signatures** (`resolve`, `resolve_identity`, `resolve_federation_relationship`, `drain_timed_out`). If Pass 1 finds itself needing to widen a method signature to make a key retype compile, that's a Pass 2 boundary leak — pause and use a `&str` projection via `Deref` (per sub-question 3 outcome) rather than widening the signature.
- `xgen-core/src/federation/registry.rs` — Pass 1 retypes the **fields** (`peer_records: HashMap<NodeXgid, PeerOperationalRecord>`, `PeerOperationalRecord.peer_node_id: NodeXgid`). Pass 2 retypes the **methods** (`mark_active(&NodeXgid)`, `peer_record(&NodeXgid)`, etc.). Same boundary discipline.
- `xgen-core/src/identity/registry.rs` — same: keys and stored-record fields belong to Pass 1; method signatures belong to Pass 2.
- `xgen-core/src/space/state.rs` — `SpaceState` field retypes belong to Pass 1; the `apply_*` methods on `SpaceState` (`apply_space_create`, `apply_join`, etc.) belong to Pass 2 because they're the algorithm layer.

If a Pass 1 field retype *forces* an adjacent method signature change because the cascading types must be consistent and `Deref` projection doesn't suffice, Clair flags this and pauses — that's a design question for Joe, not a unilateral broadening decision.

---

## Coordination with parallel work

Pass 1 runs in parallel with **Phase 9 Commit 3 onwards** (Clair's other active track — Federation Event Propagation deployment integration tests). File disjointness check:

- Phase 9 touches `xgen-node/src/tests/` (integration test files) and may touch `xgen-node/src/` minimally for observability or test hook reasons.
- Pass 1 touches `xgen-common/src/` (Event, space_local, state, new canonical module, xgid/flavours.rs), `xgen-core/src/wire/canonical.rs` (becomes thin re-export), `xgen-core/src/space/state.rs`, `xgen-core/src/federation/registry.rs`, `xgen-core/src/identity/registry.rs`, `xgen-core/src/dag/pending.rs`, plus `docs/xgen_appendix_c_en.md` and `docs/xgen_appendix_i_en.md`.

**No file overlap is expected.** Clair should verify before Pass 1 implementation kickoff by running `git diff --name-only main HEAD` on her Phase 9 branch state at that moment; if any of the above Pass 1 files appear in her Phase 9 diff, surface the overlap to Joe before proceeding (it would be a structural finding).

**Test-count baseline is captured at Pass 1 implementation kickoff, not pre-pinned to v1's 571.** Whatever Clair has shipped in Phase 9 between J-095 (the v1 milestone close) and Pass 1 implementation start has likely added integration tests; the Pass 1 verification gate inherits the count at the moment Clair picks up this runbook. Sub-question 4's outcome may pin specific new test names to add on top.

---

## Open Joe-locks (resolve before walkthrough sections fill in)

The runbook's substantive walkthrough sections (Code scope, Doc scope, Verification gate, Milestone close) are stubbed pending Joe-locks on four sub-questions. Each is framed openly with the trade-offs noted; the runbook author does not pre-decide.

### Sub-question 1 — Commit sequence shape

**Question.** Does Pass 1 ship as one large commit (all code retypes + canonical module move + convenience constructors + Appx C retype + Appx I retype atomically), or as a Clair-style multi-commit sequence with the milestone-close commit carrying the doc retypes?

**Trade-off framing.**

- *One large commit.* Maximum atomicity: code and docs land together with no intermediate state where a reader sees one but not the other. Cost: review surface is enormous (likely 20+ files across two crates plus two appendices), and a single revert undoes everything. Bisecting a regression that surfaces post-Pass-1 becomes harder because the commit blob is monolithic.
- *Multi-commit sequence.* Smaller atomic units per commit (e.g. canonical-form module move; convenience constructors; xgen-common data structure retypes; xgen-core data structure retypes; doc retypes; milestone close). Each commit is reviewable in isolation, bisectable, and revertable. Cost: between commits within the sequence, the codebase exists in intermediate states. The coordination flag's load-bearing promise is that *the commit set as a whole* keeps code and docs aligned — sequential commits in one PR satisfy that, but a reader checking out a mid-sequence commit sees a partial state.

The coordination flag from `tasks/XGID_DOC_SWEEP.md` says code + Appx C + Appx I in **ONE commit set**. "Commit set" is interpretable either way; the runbook author treats this as the question to lock.

**Downstream impact.** Sub-question 1's outcome determines the runbook's section structure. One-large-commit shape uses a single "What ships" section. Multi-commit shape uses per-commit sections with per-commit Definition of Done checklists, mirroring `tasks/XGID_ADOPTION_IMPL.md`'s two-commit shape.

**Status:** [JOE-LOCK pending]

---

### Sub-question 2 — `canonical_event_bytes` module-move ordering

**Question.** Does the `canonical_event_bytes` module move from `xgen-core/src/wire/canonical.rs` to `xgen-common/src/canonical.rs` (with `xgen-core` re-export) land as its own isolated commit at the head of the Pass 1 sequence, or fold into a larger commit alongside other Pass 1 work?

**Trade-off framing.**

- *Isolated head commit.* Module moves want git-history clarity — `git log --follow` works cleanly when the move is its own commit. The move is structurally orthogonal to the retypes (it doesn't change any field's type; it just changes a module's address). Isolating it means a future reader running `git log -- xgen-common/src/canonical.rs` sees the move as a clean event. Once the module move lands, the convenience constructors (`EventXgid::from_event`, etc.) become implementable and can ride in a follow-on commit.
- *Folded into a larger commit.* Reduces total commit count and keeps the Pass 1 sequence shorter. Cost: `git log --follow` may not detect the file move cleanly if other file changes happen in the same commit, depending on git's heuristic threshold.

**Dependency note.** This sub-question is meaningful only if sub-question 1 locks "multi-commit sequence." If sub-question 1 locks "one large commit", the module move is part of that commit regardless and this sub-question is moot.

**Status:** [JOE-LOCK pending — meaningful only if sub-question 1 → multi-commit]

---

### Sub-question 3 — Bridging strategy for downstream `&str`/`&String` consumers

**Question.** During Pass 1's retypes, every consumer of a now-retyped field (in xgen-core algorithm code, in xgen-node, in xgen-client, in tests) currently expects `&str` or `&String`. How does Pass 1 bridge these consumers without widening signatures (which would leak into Pass 2 / 3 / 4 scope)?

**Trade-off framing.**

- *Trust the `Deref` chain.* Q2 of the XGID Adoption design walkthrough locked `Deref<Target = Xgid>` on the flavour wrappers precisely so downstream consumers can transparently treat a typed XGID as if it were the base `Xgid`. Adding `impl Deref<Target = str>` on `Xgid` (or providing an `.as_str()` method already covered by v1) lets call sites that currently take `&str` continue to work via auto-deref or explicit `.as_str()` at the boundary. **Low diff churn, slightly opaque** at call sites that don't show the deref explicitly.
- *Explicit bridging at each call site.* Every call site that crosses the data-structure / algorithm boundary gets an explicit `.as_str()` projection added: `dispatch_event(peer_node_xgid.as_str(), ...)` instead of `dispatch_event(peer_node_xgid, ...)`. **High diff churn, fully explicit** — every type-boundary crossing is visible in the diff.
- *Mixed.* Trust `Deref` where the call site is in `xgen-core` algorithm code (because Pass 2 will widen those signatures soon anyway and the deref-shim is short-lived); explicit projection where the call site is in xgen-node or xgen-client (because Passes 3 and 4 are further out and the explicit shim is more honest about the temporary nature).

**Recommendation hedge.** Q2's `Deref` lock was deliberate; the runbook author's instinct is that trusting `Deref` is the design-intended path. But the question deserves an explicit Joe-lock because the implications ripple into every Pass 1 call site.

**Status:** [JOE-LOCK pending]

---

### Sub-question 4 — Invariance test suite extension

**Question.** XGID Adoption v1 shipped five required wire-format invariance tests pinned by name in `xgen-common/tests/xgid_invariance.rs`. Pass 1's retypes deepen the invariance promise from "the types preserve byte-equality" to "the types preserve byte-equality across the full Event struct shape and the full SpaceState persistence shape." Which new test names should Pass 1 pin as required, on top of the v1 set?

**Trade-off framing.**

The runbook author proposes three candidate test names; Joe locks the final set (may add, remove, or rename):

- `event_struct_full_xgid_roundtrip_through_canonical_form` — construct an Event with every XGID-typed field populated, compute canonical bytes, deserialize from those bytes back through the typed Event, assert byte-equal canonical re-derivation. Regression-locks Event-shape across the canonical-form module move + the retypes.
- `space_state_full_xgid_roundtrip_through_persisted_json` — meaningful only if `SpaceState` gains Serialize/Deserialize derives during Pass 1 (currently it's Debug/Clone only — persistence happens elsewhere via event replay). If SpaceState stays non-serde, this test name doesn't apply; replace with a test that exercises SpaceState construction + XGID-typed field access through the public API.
- `pending_buffer_keys_serialise_as_plain_strings_after_retype` — meaningful only if `PendingBuffer` has a serialised form (e.g. for state-file inclusion). Currently it's an in-memory structure; same caveat as the SpaceState one. May need to be renamed to a test that exercises the keys' string-equality semantics rather than wire serialisation.

**Structural finding to surface here.** Several of Pass 1's data structures (`SpaceState`, `RoomState`, `FederationRegistry`, `IdentityRegistry`, `PendingBuffer`) do NOT currently have Serialize/Deserialize derives — their persistence happens via event replay, not direct serialisation. The wire-format invariance promise (Appendix J §J.5 invariance 2) binds anything that crosses a wire; structures that never cross a wire don't have a direct invariance witness to write. The invariance witness for these structures is instead at the *event-source* layer (the Events that drive their construction) and the *observability* layer (`xgen-node_state.json` snapshots that mirror them). Both of those surfaces ARE serde-derived and DO get invariance tests at Pass 1.

This refines sub-question 4's framing: the new tests should target *Event canonical form* + *state.json observability shape* rather than direct serde roundtrips on the runtime structs themselves.

**Status:** [JOE-LOCK pending]

---

## Code scope walkthrough

**[STUB — populates after Joe-locks on sub-questions 1–4 land.]**

Walks the Pass 1 retypes file by file, in the order determined by sub-question 1's commit-sequence outcome. For each file, lists:

- The struct(s) being retyped.
- The specific field-by-field changes (current `String` type → new flavour-typed XGID).
- Call sites in the same file that need adjustment (per sub-question 3's bridging-strategy outcome).
- Any test fixtures co-located with the struct that need parallel updates.

Files in expected scope (full list pending sub-question 1's commit sequencing):

- `xgen-common/src/wire.rs` — `Event` struct + content structs (`StateAiOperatorDelegateContent`, `StateAiOperatorRevokeContent`, `MembershipMuteContent`, any other identity-bearing content struct).
- `xgen-common/src/space_local.rs` — `SpaceLocalMetadata.space_id`.
- `xgen-common/src/state.rs` — `NodeState`, `ConnectedClient`, `FederatedPeer`, `HostedSpace`, `HostedRoom`.
- `xgen-common/src/canonical.rs` — **new file** (target of `canonical_event_bytes` module move).
- `xgen-common/src/xgid/flavours.rs` — convenience constructors added (`EventXgid::from_event`, `SpaceXgid::from_space_create`, `RoomXgid::from_room_create`, `TrustAssertionXgid::from_assertion`); the deferred-carry-over doc comment can be removed once they land.
- `xgen-common/src/lib.rs` — re-exports updated to include the new `canonical` module.
- `xgen-core/src/wire/canonical.rs` — becomes thin re-export module (`pub use xgen_common::canonical::*;` plus a module-level doc comment explaining the move).
- `xgen-core/src/space/state.rs` — `SpaceState`, `RoomState`, `SpaceMember`, `PendingInvite`.
- `xgen-core/src/federation/registry.rs` — `FederationRegistry`, `PeerOperationalRecord`, `FederationRelationship`.
- `xgen-core/src/identity/registry.rs` — `IdentityRegistry` and its stored-record type(s).
- `xgen-core/src/dag/pending.rs` — `PendingBuffer` and its entry struct(s).

---

## Doc scope walkthrough

**[STUB — populates after Joe-locks on sub-questions 1–4 land.]**

Walks Appendix C and Appendix I field tables column-by-column. For each XGID-bearing field, the Type column updates from `String` to the appropriate flavour-typed XGID. Field-name column stays unchanged (invariance 1). Wire-key column stays unchanged (invariance 1 + invariance 2). Req/Opt column stays unchanged. Description column gets minimal edits only where the description currently calls the field "a String" or similar.

Expected scope (full list pending sub-question 1's commit sequencing):

- `docs/xgen_appendix_c_en.md` — primitive schema tables for every XGID-bearing structure documented in this appendix. Per the Phase 2 doc-tree sweep, Appx C contains 15 XGID hits across its schema field definitions.
- `docs/xgen_appendix_i_en.md` — data structure tables across all 9 Parts (Event Envelope through MLS). Per the Phase 2 doc-tree sweep, Appx I contains 122 XGID hits, the heaviest doc surface in Pass 1.

---

## Verification gate

**[STUB — populates after sub-question 4 locks the new test names.]**

Pass 1 verification gate (target shape, pending sub-question 4):

- [ ] `cargo test --workspace` clean. Test count baseline captured at Pass 1 implementation kickoff (≥ 571 from XGID Adoption v1 close, plus whatever Phase 9 has added in the parallel-track window).
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` clean.
- [ ] All v1 invariance tests (`xgid_serializes_as_plain_string`, `xgid_deserializes_from_plain_string`, `flavour_wrapper_is_serde_transparent`, `event_xgid_roundtrip_through_event_canonical_form`, `node_xgid_roundtrip_through_handshake_message`) still pass.
- [ ] New Pass 1 invariance tests (pinned by name in sub-question 4 outcome) all pass.
- [ ] Pre-existing flake handling: two known flakes (precedence env-var race from J-079; `reconnect_with_existing_tip_small_delta_delivered` from Phase 3) are NOT a Pass 1 regression signature. If either fires during verification, retry once in isolation per their established flake fix-shape.

---

## Milestone close

**[STUB — populates after sub-question 1 locks the commit sequence.]**

Pass 1 milestone close checklist (target shape):

- [ ] `tasks/XGID_RETROFIT_PASS_1_IMPL.md` Status flipped ACTIVE → COMPLETED with version bump and milestone-close note in the Last-updated field.
- [ ] CLAUDE.md PLAY block refreshed to reflect Pass 1 closure + Pass 2 becoming the next-active XGID retrofit slot.
- [ ] `docs/ROADMAP.md`: Past gains a Pass 1 closure entry; Present updated (Pass 1 RESUMED → DONE, Pass 2 PENDING → RESUMED if applicable); Near future loses the now-shipped Pass 1 line; Visual structure tree updated in same edit per the v1.4 guardrail; header version bumped.
- [ ] JOURNAL.md gains a Pass 1 close entry (next available J-number — pre-emptively included in the milestone-close commit's changed-files list per the D-074 candidate principle surfaced 2026-05-20).
- [ ] All workspace tests pass clean; test count delta recorded honestly in the JOURNAL entry.
- [ ] Workspace clippy clean.
- [ ] No DoD checklist item names "commit pushed" — the milestone-close commit is itself the push, and "commit pushed" is unflippable inside the commit that performs the push.

---

## Sequence and dependencies

- Pass 1 is sequenced **after** Federation Event Propagation Phase 9 ships and that milestone flips PLAY → DONE. The current parallel-eligibility window between Pass 1 runbook authoring (Chat Claude) and Phase 9 Commit 3+ (Clair) is acceptable; Pass 1 *implementation* waits for Phase 9 close.
- Pass 1 is sequenced **before** Pass 2 (xgen-core algorithm-bearing functions). Pass 2 consumes Pass 1's typed data structures; running Pass 2 first would force every method signature in Pass 2 to retype against still-String fields, which inverts the natural type-flow direction.
- Pass 1 *does not block* M6 Block 4 (verb-by-verb walks) — Block 4 is Chat Claude + Joe work, runs in parallel.
- Pass 1 *does not block* M6 (new) Phase 0 retrospective work — separate workstream.

---

## Cross-references

- `DECISIONS.md` D-072 — XGID Adoption v1 (architectural commitment, the type vocabulary Pass 1 retypes into).
- `DECISIONS.md` D-073 — Field-name-vs-type discipline (the composition rule Pass 1 instantiates at scale).
- `docs/xgen_appendix_j_en.md` — Canonical XGID document (§J.5 wire-format invariance, §J.11 adoption discipline, §J.8 scope boundaries).
- `docs/xgen_ch3_specification.md` §3.0 — Terse normative section.
- `tasks/XGID_ADOPTION_IMPL.md` — XGID Adoption v1 implementation runbook (COMPLETED v1.1). Pass 1's runbook follows the same shape but for the broader cross-crate retype.
- `tasks/XGID_DOC_SWEEP.md` — Phase 2 doc-tree sweep classification table (COMPLETED v1.2). Canonical input for Pass 1's doc scope.
- `docs/ROADMAP.md` — Near future section's Pass 1 paragraph + Visual structure tree's Pass 1 row (both updated to "core data structures" framing in same commit as this runbook skeleton).
- `JOURNAL.md` J-095 — XGID Adoption v1 milestone close entry. The "Carry-overs" section names the canonical-form module move + convenience constructors that Pass 1 picks up.

---

*End of XGID Retrofit Pass 1 Implementation Runbook skeleton.*  
