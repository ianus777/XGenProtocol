# XGID Retrofit Pass 1 — Implementation Runbook
> **Status**: ACTIVE  
> Version: 2.0  
> Date: May 2026  
> **Last updated**: 2026-05-21 (all four sub-questions locked; runbook walkthrough sections populated. Version bumped 1.4 → 2.0 to signal the runbook is now complete and ready for Clair pickup at Pass 1 implementation kickoff. Sub-question recap: (1) multi-commit sequence, six commits; (2) canonical-form module move as isolated head commit; (3) explicit `.as_str()` projection with code-comment discipline at non-trivial sites; (4) five new invariance tests pinned by name. Code scope walkthrough lists per-commit file-by-file changes; Doc scope walkthrough lists per-Appendix retype scope; Verification gate pins the cargo test/clippy checklist and the new invariance tests; Milestone close pins the cross-doc same-commit discipline including JOURNAL.md per the D-074 candidate. Pass 1 implementation waits for Federation Event Propagation milestone closure (Phase 9 going DONE).)  
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

**[JOE-LOCKED 2026-05-21: multi-commit sequence, six commits.]**

**Lock reasoning.**

Five reasons multi-commit wins over monolithic at Pass 1's scale:

1. **Precedent from XGID Adoption v1.** v1 shipped as two atomic commits + hygiene sibling + milestone-close (four commits total). That worked well — review-comprehensibility was high, each commit had a clear story, the milestone-close commit isolated cross-doc updates cleanly. Pass 1 is structurally larger than v1 (cross-crate, ~10 source files vs v1's 3, plus two appendices), so the case for multi-commit is stronger than at v1, not weaker.
2. **Bisectability matters more here than at v1.** v1's surface was tightly contained — if a regression surfaced post-v1, the suspect range was narrow. Pass 1 retypes data structures consumed by every algorithm in xgen-core, every handler in xgen-node, every command in xgen-client. If a regression surfaces post-Pass-1 (e.g. in Phase 9 integration tests, in M6 work, in a Pass 2 retype that depends on a Pass 1 type), `git bisect` against a monolithic Pass 1 commit can only point to "Pass 1 broke it" — useless. Against a multi-commit sequence, bisect can point to "the SpaceState retype broke it" or "the canonical-form module move broke it" — actionable.
3. **The coordination flag tolerates multi-commit.** The Phase 2 doc-tree sweep's coordination flag says "code + Appx C + Appx I in ONE commit set." A commit set is a coherent group of commits that ship together — sequential commits in one push satisfy this. The promise is that no downstream consumer ever sees the codebase between code-shipped and docs-shipped; sequential commits in one push deliver that. The flag exists to prevent doc/code skew across milestones, not within a milestone's commit sequence.
4. **The carry-overs naturally factor into their own commits.** The `canonical_event_bytes` module move is structurally orthogonal to the retypes (it changes a module's address, not any type). The convenience constructors depend on the module move but are independent of the data-structure retypes. Forcing all three into one commit conflates three structurally distinct motions.
5. **Honest-broadening discipline gets a per-commit check.** With a multi-commit sequence, each commit has its own scope-discipline review: "did this commit stay in its lane, or did it leak into Pass 2 territory?" A monolithic commit makes that check impossible — by the time the diff is final, boundary leaks are baked in.

**Six-commit sequence.**

1. **Canonical-form module move.** Move `canonical_event_bytes`, `canonical_event_json`, `canonical_object_json`, `canonical_value` from `xgen-core/src/wire/canonical.rs` to new `xgen-common/src/canonical.rs`. `xgen-core/src/wire/canonical.rs` becomes a thin re-export shim (`pub use xgen_common::canonical::*;` plus module-level doc comment explaining the move). `xgen-common/src/lib.rs` re-exports the new module. Existing tests in canonical.rs move with the function bodies. **No retypes in this commit — pure module move.** xgen-common Cargo.toml gains any newly required dependencies (likely already covered by `serde_json` already being present from the v1 invariance tests — verify before commit).

2. **Convenience constructors.** Add `EventXgid::from_event(&Event)`, `SpaceXgid::from_space_create(&Event)`, `RoomXgid::from_room_create(&Event)`, `TrustAssertionXgid::from_assertion(&TrustAssertion)` on the flavour wrappers in `xgen-common/src/xgid/flavours.rs`. Update the deferred-carry-over module-level doc comment to reflect closure. Add unit tests for the new constructors covering the byte-equality promise (constructor output matches `from_canonical_bytes` applied to `canonical_event_bytes(&event)`).

3. **xgen-common data-structure retypes.** `Event` struct fields (`event_id`, `sender`, `room_id`, `space_id`, `prev_events`) in `xgen-common/src/wire.rs`. Content structs in `wire.rs` (`StateAiOperatorDelegateContent`, `StateAiOperatorRevokeContent`, `MembershipMuteContent`, and any other identity-bearing content struct). `SpaceLocalMetadata.space_id` in `xgen-common/src/space_local.rs`. Observability structs in `xgen-common/src/state.rs` (`NodeState.node_id`; `ConnectedClient.identity_id`; `FederatedPeer.node_id`, `session_id`, `shared_spaces`; `HostedSpace.space_id`; `HostedRoom.room_id`). All co-located tests updated. Forward-compat lock: the `serde_roundtrip_with_introducer` test pattern from v1 Commit 2 extends to every retyped field — each struct gains a per-call-site wire-format invariance witness for Appendix J §J.5 invariance 2.

4. **xgen-core data-structure retypes.** `SpaceState`, `RoomState`, `SpaceMember`, `PendingInvite` in `xgen-core/src/space/state.rs`. `FederationRegistry`, `PeerOperationalRecord`, `FederationRelationship` in `xgen-core/src/federation/registry.rs`. `IdentityRegistry` and its stored-record type(s) in `xgen-core/src/identity/registry.rs`. `PendingBuffer` and its entry struct(s) in `xgen-core/src/dag/pending.rs`. **Honest-broadening discipline: method signatures are NOT widened in this commit.** Methods that currently take `&str` keep taking `&str`; consumers project through `Deref` per sub-question 3's outcome. Pass 2 owns method-signature widening.

5. **Appendix C + Appendix I retypes.** Both docs' field-table Type columns updated column-by-column from `String` to flavour-typed XGIDs. Field-name column stays unchanged (invariance 1). Wire-key column stays unchanged (invariance 1 + invariance 2). Req/Opt column stays unchanged. Description column gets minimal edits only where the description currently calls the field "a String" or similar type-naming language. Header bumps on both files.

6. **Milestone close.** Status flip on this runbook (ACTIVE → COMPLETED, v1.x → v2.0 or v1.N+1 depending on session-close state). CLAUDE.md PLAY block refresh (Pass 1 ✅ DONE, Pass 2 becoming next-active XGID retrofit slot). ROADMAP.md updates: Past gains Pass 1 closure entry; Present updated; Near future loses the now-shipped Pass 1 line; Visual structure tree updated in same edit per the v1.4 guardrail; header version bumped. JOURNAL.md new J-NNN entry. Test count delta recorded honestly. Header bumps on all touched files. **The milestone-close commit's changed-files list includes JOURNAL.md** per the D-074 candidate principle surfaced 2026-05-20.

**Why six and not five or seven.**

- Five would fold #1 and #2 together. The module move and the constructors are conceptually paired (one enables the other), and a folded commit might read cleanly. But the move is reverse-compatible (the re-export shim keeps xgen-core's API stable) while the constructors are additive new API. Separating them keeps each commit's "what changed" story tight.
- Seven would split #5 into Appx C + Appx I. No benefit — they're both pure documentation, identical mechanical edit shape, no cross-dependency. Splitting them inflates the commit count without improving reviewability.

**Coordination promise restated.** Commits 1–6 ship as a sequential push in one session. Between commits within the sequence the codebase is in honest intermediate states (e.g. after commit 1 the canonical-form module has moved but no retypes have happened yet — that's a clean state, not a broken one). The coordination flag is satisfied because no commit in the sequence ships partial doc/code skew within a single conceptual change — commit 5 ships both docs together, commits 3 and 4 are code-only and don't depend on docs.

**Status:** [JOE-LOCKED 2026-05-21]

---

### Sub-question 2 — `canonical_event_bytes` module-move ordering

**[JOE-LOCKED 2026-05-21: isolated head commit.]**

The module move ships as **commit 1 of the six-commit sequence** (see sub-question 1).

**Lock reasoning.**

- Structurally orthogonal to the retypes: changes a module's address (xgen-core → xgen-common), not any field's type. Isolating the move from the retypes keeps each commit's "what changed" story tight.
- `git log --follow xgen-common/src/canonical.rs` traces the function bodies back to their original xgen-core location cleanly when the move is its own commit. Folding the move into a multi-file retype commit risks tripping git's rename-detection heuristic.
- Unblocks commit 2 mechanically: the convenience constructors (`EventXgid::from_event`, `SpaceXgid::from_space_create`, etc.) call `canonical_event_bytes` and can only land after the module is visible from xgen-common. Sequential commits make the dependency explicit.
- xgen-core's `wire/canonical.rs` becomes a thin re-export shim in commit 1, preserving all existing xgen-core call sites without code churn. The shim disappears in a future cleanup pass (likely Pass 2 or Pass 5) when downstream call sites migrate to importing directly from `xgen_common::canonical`.

**Pre-commit verification.** Before shipping commit 1, Clair confirms:

- `xgen-common`'s `Cargo.toml` already depends on `serde_json` (used by the v1 invariance tests in `xgen-common/tests/xgid_invariance.rs`). If for any reason it doesn't, the dependency is added in commit 1.
- No xgen-common file currently named `canonical.rs` exists (it doesn't — directory listing at runbook authoring time shows only `build_info.rs`, `event_trace.rs`, `lib.rs`, `precedence.rs`, `space_local.rs`, `state.rs`, `wire.rs`, and `xgid/` directory).
- The four functions move together as a unit (`canonical_event_bytes`, `canonical_event_json`, `canonical_object_json`, `canonical_value`) along with the `EVENT_FIELD_ORDER` const — they form a tight internal API and splitting them would create a worse module boundary.
- The existing test module inside `xgen-core/src/wire/canonical.rs` (`#[cfg(test)] mod tests`) moves with the function bodies. The test module stays where the implementation moves to (now `xgen-common/src/canonical.rs`).

**Status:** [JOE-LOCKED 2026-05-21]

---

### Sub-question 3 — Bridging strategy for downstream `&str`/`&String` consumers

**[JOE-LOCKED 2026-05-21: explicit `.as_str()` projection at cross-type-boundary call sites, with code-comment discipline at non-trivial sites.]**

**Lock reasoning.**

Four reasons explicit projection wins over pure `Deref`-reliance:

1. **Precedent from XGID Adoption v1 Commit 2.** The one production retype at v1 (`SpaceLocalMetadata.introducer_node_id`) used explicit `.as_ref().map(|n| n.as_str())` projection at the test-side read site, and explicit `NodeXgid::from_xgid(Xgid::new(peer.to_string()))` at the production caller boundary. v1 set the precedent for explicit projection at boundaries; Pass 1 follows the same shape at scale.
2. **`Xgid` doesn't currently implement `Deref<Target = str>`.** Q2 of the XGID Adoption design walkthrough locked `Deref<Target = Xgid>` on the *flavour wrappers* (so `&NodeXgid` derefs to `&Xgid`), but it did NOT lock `Deref<Target = str>` on the base `Xgid`. Pure auto-deref from `&IdentityXgid` to `&str` does not compile today. Adding `Deref<Target = str>` on the base `Xgid` is a deliberate API addition that v1 explicitly chose not to make (it would make the newtype "leaky" — a `Xgid` would behave as a `&str` everywhere, partially undoing the type-discipline the newtype exists to enforce). Pass 1 does NOT add this impl.
3. **Explicit projection makes type-boundary crossings visible in the diff.** During commits 3 and 4 review, every `.as_str()` call is a flag for honest-broadening discipline: "is this projection in Pass 1's lane, or is it covering for a signature that should be widened in Pass 1 itself?" Pure `Deref`-reliance would hide those decisions.
4. **The code comments act as TODO markers for downstream Passes.** At non-trivial projection sites (e.g. a function that takes `&str` and is called with `xgid.as_str()` where Pass 2's widening would change the signature to take `&IdentityXgid`), Clair adds a one-line code comment: `// Pass 2 widens this to take &IdentityXgid; the .as_str() projection collapses then.` These comments become free Pass 2 / 3 / 4 task-tracking, visible in the codebase at the exact site where the work needs to happen.

**Mechanical rules.**

- **Reading a retyped field that's used as `&str` downstream.** Project with `.as_str()`:
  - `event.sender.as_str()` (where `event.sender: IdentityXgid`).
  - `event.event_id.as_ref().map(|e| e.as_str())` (where `event.event_id: Option<EventXgid>`).
  - `prev_events.iter().map(|e| e.as_str())` (where `prev_events: Vec<EventXgid>`).
- **Cloning a retyped field that's used as `String` downstream.** Project with `.as_str().to_string()` or `.to_string()` (the `Display` impl on `Xgid` writes the inner string). The `to_string()` form is preferred at idiomatic-looking sites; `as_str().to_string()` is preferred at sites where the boundary is being flagged with a comment.
- **Constructing a retyped field from a `&str` or `String` source.** Use the flavour wrapper's construction shape — either a flavour-specific constructor (`NodeXgid::from_pubkey(...)` for principal flavours from a known key) or the explicit two-step wrap (`NodeXgid::from_xgid(Xgid::new(s))`). The two-step form gets a comment at the site flagging that Pass N widens the upstream signature so the wrap collapses.
- **Map / set lookups by `&str` against a key-typed `&IdentityXgid` map.** Project the lookup key: `registry.get(&IdentityXgid::from_xgid(Xgid::new(s.to_string())))`. This is the most painful projection shape and the comment is mandatory here: `// Pass 2 widens registry.get to take &IdentityXgid directly; the wrap collapses then.`
- **Comparisons between a typed XGID and a `&str` literal.** Project the XGID: `event.sender.as_str() == "xgen://pubkey/ed25519:alice"`. NOT `event.sender == Xgid::new("xgen://...".to_string())` — the latter is a worse pattern that hides the comparison shape.

**Code-comment discipline.** Non-trivial projection sites get a one-line comment naming the future Pass that widens the receiver. Trivial sites (a single `.as_str()` inside a `format!` or a log macro) don't need the comment. Judgment call: if a reader 6 months from now would look at the projection and wonder "why is this projection here?", the comment is warranted. If the projection is obviously a stopgap for a `&str` consumer in code Pass 1 doesn't own, the comment is warranted.

**What this means for diff size.** Commits 3 and 4 have non-trivial diff churn from `.as_str()` projections at call sites. That's the honest cost of the staged retrofit discipline — Pass 1's diff carries explicit markers where Pass 2/3/4 work needs to happen, rather than hiding the transition under `Deref` magic. v1's precedent supports this; staged retrofit's whole point is that the transition is *visible*, not seamless.

**Future-tightening note (not in Pass 1 scope).** A future walkthrough may revisit whether `Xgid` should gain `Deref<Target = str>` after Pass 5 closes. By then the "mixed discipline transitionally" clause from D-072 no longer applies, and the API addition would be additive rather than transitional. That's a future-Joe conversation, not a Pass 1 decision.

**Status:** [JOE-LOCKED 2026-05-21]

---

### Sub-question 4 — Invariance test suite extension

**[JOE-LOCKED 2026-05-21: five new invariance tests pinned by name; file placement Clair-latitude per v1 precedent.]**

**Lock reasoning.**

The v1 baseline of five tests covers *type-level* invariance — that a single XGID (base or flavour-wrapped) serialises as a plain string and roundtrips byte-equal. Pass 1 deepens the promise to *aggregate-level* invariance: whole structs with multiple typed XGID fields preserve byte-equality across their serialised form. Plus forward-compat: existing on-disk and on-wire Event/SpaceLocalMetadata/NodeState JSON must keep parsing after Pass 1's retypes.

The structural finding from the runbook skeleton (several runtime data structures don't have Serialize/Deserialize derives) refines the test framing: invariance witnesses live at the *event-source layer* (Events drive construction of SpaceState, registries, PendingBuffer — covered by Test A + D) and the *observability layer* (`xgen-node_state.json` mirrors runtime state — covered by Test C). Direct serde roundtrips on the non-serde runtime structs are not meaningful; their fields are exercised through the Event canonical form and the observability snapshot.

Combined v1 + Pass 1 gives **10 invariance tests total**, symmetric across the type and aggregate layers.

**Five required test names (pinned by name; file placement Clair-latitude).**

- **`event_struct_full_xgid_roundtrip_through_canonical_form`** — construct an Event with every XGID-typed field populated (`event_id: Some(EventXgid::from_canonical_bytes(...))`, `sender: IdentityXgid::from_pubkey(...)`, `room_id: RoomXgid::from_canonical_bytes(...)`, `space_id: SpaceXgid::from_canonical_bytes(...)`, `prev_events: Vec<EventXgid>` with two non-trivial entries). Compute canonical bytes via the moved `canonical_event_bytes`. Serialise the Event to JSON via serde. Deserialise back. Recompute canonical bytes from the deserialised Event. Assert byte-equal across the round-trip. This is the v1 `event_xgid_roundtrip_through_event_canonical_form` test extended to exercise every XGID field rather than just one.
- **`space_local_metadata_full_xgid_roundtrip`** — construct `SpaceLocalMetadata` with both `space_id` (newly typed `SpaceXgid` at Pass 1) and `introducer_node_id` (already typed `NodeXgid` at v1) populated to non-trivial XGID values. Serialise to JSON. Assert the JSON shape has both fields as plain strings, NOT as objects with type-discriminator wrappers (Appendix J §J.5 invariance 2 witness). Deserialise back. Assert byte-equal. This is the v1 `serde_roundtrip_with_introducer` test pattern extended to cover the new `space_id` retype alongside the existing `introducer_node_id` retype.
- **`node_state_observability_xgid_roundtrip`** — construct a `NodeState` with typed XGID fields populated through its sub-structs: `ConnectedClient.identity_id`, `FederatedPeer.node_id`/`session_id`/`shared_spaces`, `HostedSpace.space_id`, `HostedRoom.room_id`. Serialise the whole `NodeState` to JSON (the shape `xgen-node_state.json` is written in). Assert every XGID-bearing field is a plain JSON string. Deserialise back. Assert byte-equal. This locks the observability-layer invariance — operators reading `xgen-node_state.json` see XGIDs in the same shape across the Pass 1 transition.
- **`event_content_struct_xgid_roundtrip`** — construct three Events with typed-XGID content structs: one with `StateAiOperatorDelegateContent` (carrying `space_id`, `ai_identity_id`, `new_operator_identity_id`), one with `StateAiOperatorRevokeContent` (carrying `space_id`, `ai_identity_id`), one with `MembershipMuteContent` (carrying `target_identity`). Serialise each Event. Deserialise. Assert the content-struct XGID fields roundtrip byte-equal. This covers the content-struct retype surface that the v1 `event_xgid_roundtrip_through_event_canonical_form` test did NOT touch (v1's content was a generic `serde_json::Value`).
- **`legacy_string_json_forward_compat_on_event`** — take a hand-crafted pre-Pass-1 JSON shape of an Event (where `event_id`, `sender`, `room_id`, `space_id` are plain strings without typed-XGID wrapping at the type level), deserialise into the post-Pass-1 typed Event via serde. Confirm the deserialised typed XGIDs have the right inner strings (`event.sender.as_str() == "xgen://pubkey/ed25519:alice"` etc.). This is the forward-compat test that prevents on-disk Event JSON (e.g. test fixtures, journal records) and on-wire Event JSON (federation messages, batch JSONL replies) from breaking across the Pass 1 retype. It mirrors the legacy-shape branch of v1's `serde_roundtrip_with_introducer` test.

**Additional tests Clair MAY add.**

- Roundtrip tests on the other content structs in `xgen-common/src/wire.rs` not covered by Test D, if any carry XGID-bearing fields.
- Negative tests confirming that a malformed XGID string (wrong prefix, wrong length) deserialises into a typed XGID newtype as a parse-error case where `pubkey()` would fail — these document the parse-fallible-at-v1 contract (D-072 + Q2 lock).
- Cross-flavour rejection tests: confirm that a `serde_json::from_str` of a `NodeXgid`-shaped JSON into an `IdentityXgid` parameter slot accepts the string (because both are serde-transparent) but a downstream call to `.pubkey()` produces the expected result. This documents the "flavour information lives in the type system, not on the wire" property from D-072 invariance 2.

**File placement.** Clair picks the cleanest location during implementation. Reasonable choices:

- Tests A, D, E in `xgen-common/tests/xgid_invariance.rs` (extending the existing v1 file).
- Test B in `xgen-common/src/space_local.rs`'s `#[cfg(test)] mod tests` (extending the existing `serde_roundtrip_with_introducer` test).
- Test C in `xgen-common/src/state.rs`'s `#[cfg(test)] mod tests` (creating a new test module if one doesn't exist).

Alternative single-file placement (all five in `xgen-common/tests/xgid_invariance.rs`) is also acceptable if Clair prefers a single invariance-test home.

**Pre-existing flake note (unchanged from runbook skeleton).** Two known intermittent flakes carry forward: precedence env-var race (from J-079, ~10–20% workspace runs) and `reconnect_with_existing_tip_small_delta_delivered` (Phase 3 test surfacing under Phase 4 parallelism, ~10% workspace runs). Neither is a Pass 1 regression signature; retry on either failure.

**Status:** [JOE-LOCKED 2026-05-21]

---

## Code scope walkthrough

Walks the Pass 1 retypes commit by commit, per the six-commit sequence locked in sub-question 1.

### Commit 1 — Canonical-form module move

**Files touched (5):**

- `xgen-common/src/canonical.rs` — **new file**. Receives `canonical_event_bytes`, `canonical_event_json`, `canonical_object_json`, `canonical_value`, and `EVENT_FIELD_ORDER` const, copied verbatim from `xgen-core/src/wire/canonical.rs`. Includes the existing `#[cfg(test)] mod tests` block from canonical.rs (the six existing tests: `field_order_is_canonical`, `event_id_and_signature_excluded`, `content_keys_sorted`, `no_whitespace`, `deterministic`, `array_order_preserved`).
- `xgen-common/src/lib.rs` — add `pub mod canonical;` and re-export `pub use canonical::{canonical_event_bytes, canonical_event_json, canonical_object_json, canonical_value};`.
- `xgen-common/Cargo.toml` — verify `serde_json` is present; add if missing (likely already present from v1 invariance tests).
- `xgen-core/src/wire/canonical.rs` — replaced with a thin re-export shim: `pub use xgen_common::canonical::*;` plus a module-level doc comment explaining the move ("This module re-exports the canonical-form helpers that now live in xgen-common. The move was performed at XGID Retrofit Pass 1 commit 1 to make the helpers visible to xgen-common's flavour-wrapper convenience constructors (`EventXgid::from_event` etc.). This shim is scheduled for removal in a future cleanup pass (Pass 2 or Pass 5) when downstream call sites migrate to importing directly from `xgen_common::canonical`.").
- `xgen-core/src/wire/mod.rs` — no change required; existing `pub mod canonical;` still works because the file still exists (just with a different body).

**What does NOT change in commit 1:**

- No data-structure field types retyped (commits 3 and 4 own that).
- No new flavour-wrapper methods added (commit 2 owns that).
- No documentation files touched (commit 5 owns that).
- xgen-core downstream call sites of `canonical_event_bytes` continue to compile unchanged because the shim re-exports the same names.

**Verification at commit 1:** `cargo build --workspace` clean. `cargo test -p xgen-common` clean (the six moved tests now pass in their new location). `cargo test -p xgen-core` clean (the existing call sites continue to work via the shim). `cargo clippy --workspace --all-targets -- -D warnings` clean.

### Commit 2 — Convenience constructors on flavour wrappers

**Files touched (1–2):**

- `xgen-common/src/xgid/flavours.rs` — add four new constructors:
  - `EventXgid::from_event(event: &Event) -> Self` — calls `from_canonical_bytes(&canonical_event_bytes(&serde_json::to_value(event).expect("Event serialises")))`. The `expect` is honest — Event is `Serialize` by derive, serialisation cannot fail in practice; if it did, that's a programmer error caught immediately.
  - `SpaceXgid::from_space_create(event: &Event) -> Self` — same shape; in debug builds asserts `event.event_type == EventType::StateSpaceCreate || event.event_type == EventType::StateDmSpaceCreate`.
  - `RoomXgid::from_room_create(event: &Event) -> Self` — same shape; debug-assert `event.event_type == EventType::StateRoomCreate`.
  - `TrustAssertionXgid::from_assertion(assertion: &TrustAssertion) -> Self` — same shape over the assertion's canonical bytes. If `TrustAssertion` doesn't have its own canonical-form helper yet, this constructor is deferred to Pass 2 (which owns the auth-module surfaces) with a code comment flagging the deferral. Verify at runbook implementation kickoff whether `TrustAssertion` has a canonical-form helper in xgen-core; if not, Clair flags this and pauses for Joe-decision.
- Module-level doc comment in `flavours.rs` updated: remove the v1 deferred-carry-over note (the `// Carry-over to Retrofit Pass 1: ...` block) and replace with a closure note: `// XGID Retrofit Pass 1 commit 2 closed the deferred convenience constructors. EventXgid::from_event, SpaceXgid::from_space_create, RoomXgid::from_room_create, and TrustAssertionXgid::from_assertion are now implemented; they call canonical_event_bytes (moved to xgen-common at commit 1) to compute the canonical bytes that drive the hash anchor.`
- New unit tests in the module:
  - `event_xgid_from_event_matches_from_canonical_bytes` — construct an Event, compute `EventXgid::from_event(&event)`, separately compute `EventXgid::from_canonical_bytes(&canonical_event_bytes(...))`, assert byte-equal. Locks the constructor's invariant that high-level and low-level paths produce identical XGIDs.
  - Equivalent tests for `SpaceXgid::from_space_create`, `RoomXgid::from_room_create`. (TrustAssertion test deferred if the constructor is deferred per above.)

**What does NOT change in commit 2:**

- No data-structure field types retyped.
- No documentation files touched.
- No call-site changes elsewhere in the codebase — the new constructors are additive API.

**Verification at commit 2:** `cargo test -p xgen-common` clean, including the three new constructor tests. `cargo clippy` clean.

### Commit 3 — xgen-common data-structure retypes

**Files touched (3–4):**

- `xgen-common/src/wire.rs` — `Event` struct field retypes:
  - `event_id: Option<EventXgid>` (was `Option<String>`).
  - `sender: IdentityXgid` (was `String`).
  - `room_id: RoomXgid` (was `String`). **Note on empty-string contract:** the existing field comment says "Empty string for space-level events." Pass 1 must decide whether to preserve that empty-string convention (typed as `RoomXgid::from_xgid(Xgid::new(String::new()))`) or refactor to `Option<RoomXgid>`. The refactor is honest-broadening into wire-shape territory — use the empty-string-wrapped form to preserve wire compatibility, with a code comment flagging future cleanup. Same applies to `space_id` for the `state.space_create` event itself.
  - `space_id: SpaceXgid` (was `String`).
  - `prev_events: Vec<EventXgid>` (was `Vec<String>`).
  - `signature` stays `Option<String>` (signature strings are not XGIDs per D-072 "what XGID is not").
  - `Event::new` constructor signature updated to take typed parameters.
  - Existing tests in `wire.rs` updated to construct typed XGIDs in fixtures.
- `xgen-common/src/wire.rs` content structs:
  - `StateAiOperatorDelegateContent.space_id: SpaceXgid`, `.ai_identity_id: IdentityXgid`, `.new_operator_identity_id: IdentityXgid`.
  - `StateAiOperatorRevokeContent.space_id: SpaceXgid`, `.ai_identity_id: IdentityXgid`.
  - `MembershipMuteContent.target_identity: IdentityXgid`.
- `xgen-common/src/space_local.rs` — `SpaceLocalMetadata.space_id: SpaceXgid` (was `String`). Constructors `new_local` and `new_via_federation` take typed parameters. Existing `serde_roundtrip_with_introducer` test extends to cover `space_id` byte-shape lock.
- `xgen-common/src/state.rs` — observability structs:
  - `NodeState.node_id: NodeXgid`.
  - `ConnectedClient.identity_id: IdentityXgid`.
  - `FederatedPeer.node_id: NodeXgid`, `.session_id: String` (session IDs are NOT XGIDs per D-072 "what XGID is not"), `.shared_spaces: Vec<SpaceXgid>`.
  - `HostedSpace.space_id: SpaceXgid`.
  - `HostedRoom.room_id: RoomXgid`.

**Bridging strategy applied (per sub-question 3 lock).** Every consumer of these fields within `xgen-common` (and there are some — e.g. test fixtures) gets explicit `.as_str()` or typed-construction projection. Code-comment discipline at non-trivial sites.

**Invariance tests added in this commit (per sub-question 4):**

- `event_struct_full_xgid_roundtrip_through_canonical_form` (Test A).
- `space_local_metadata_full_xgid_roundtrip` (Test B — extends existing `serde_roundtrip_with_introducer`).
- `node_state_observability_xgid_roundtrip` (Test C).
- `event_content_struct_xgid_roundtrip` (Test D).
- `legacy_string_json_forward_compat_on_event` (Test E).

**Honest-broadening discipline at commit 3.** No xgen-core, xgen-node, or xgen-client files touched in this commit. Downstream consumers of the retyped xgen-common fields will not compile until commit 4 or later, depending on where the consumer lives. This is acceptable mid-sequence — the codebase is in an honest intermediate state. **The verification at commit 3 is `cargo test -p xgen-common` clean, NOT `cargo test --workspace` clean.** `cargo build --workspace` will fail at this point, and that's expected.

### Commit 4 — xgen-core data-structure retypes

**Files touched (4):**

- `xgen-core/src/space/state.rs` — `SpaceState`, `RoomState`, `SpaceMember`, `PendingInvite`:
  - `SpaceState.space_id: SpaceXgid`, `.owner_id: IdentityXgid`, `.home_node: NodeXgid`, `.members: HashMap<IdentityXgid, SpaceMember>`, `.pending_invites: HashMap<IdentityXgid, PendingInvite>`, `.ai_operator_delegations: HashMap<IdentityXgid, IdentityXgid>`.
  - Room map keyed by `RoomXgid` (verify shape against the current `SpaceState.rooms` field at implementation kickoff; if it's `HashMap<String, RoomState>` retype to `HashMap<RoomXgid, RoomState>`).
  - `SpaceState.federation_nodes: Vec<NodeXgid>` (or `HashSet<NodeXgid>` if current shape is set-like).
  - `SpaceMember.identity_id: IdentityXgid`, `.invited_by: Option<IdentityXgid>`.
  - `RoomState.room_id: RoomXgid`, `.space_id: SpaceXgid`, `.members: HashSet<IdentityXgid>`.
  - `PendingInvite` — if any identity-bearing fields, retype.
- `xgen-core/src/federation/registry.rs` — `FederationRegistry`, `PeerOperationalRecord`, `FederationRelationship`:
  - `FederationRegistry.relationships: HashMap<NodeXgid, FederationRelationship>`.
  - `FederationRegistry.peer_records: HashMap<NodeXgid, PeerOperationalRecord>`.
  - `PeerOperationalRecord.peer_node_id: NodeXgid`.
  - `FederationRelationship` — any identifier-bearing fields retyped.
- `xgen-core/src/identity/registry.rs` — `IdentityRegistry` keyed by `IdentityXgid`; stored-record type's identifier fields retyped.
- `xgen-core/src/dag/pending.rs` — `PendingBuffer`:
  - Primary index keyed `EventXgid`.
  - `PendingBuffer` entry struct fields: `missing_predecessors: Vec<EventXgid>`, `missing_identity: Option<IdentityXgid>`, `missing_federation_relationship` carries typed XGIDs in its tuple/struct shape (verify current shape at implementation kickoff).
  - Secondary index `waiting_for_identity: HashMap<IdentityXgid, HashSet<EventXgid>>`.
  - Secondary index for federation-relationship waits (Phase 7.5 surface) keyed by typed XGIDs.

**Honest-broadening discipline at commit 4.** Method signatures on these structures are NOT widened. Methods that currently take `&str` keep taking `&str` (consumers project through `.as_str()` per sub-question 3); methods that return `&String` keep returning that. Pass 2 owns the method-signature widening.

**Bridging strategy applied (per sub-question 3 lock).** Internal call sites within `xgen-core` get explicit `.as_str()` or typed-construction projection with code-comment discipline at non-trivial sites. Cross-crate consumers (xgen-node, xgen-client, tests) are NOT touched in this commit; their build breakage is expected and is Pass 2 / 3 / 4 / 5 territory.

**Critical structural finding to surface at commit 4 implementation.** The runbook's verification expectation at commit 4 needs clarification: `cargo build --workspace` will likely STILL fail at commit 4 close, because xgen-node and xgen-client consume these xgen-core types. **The honest verification at commit 4 is `cargo build -p xgen-common -p xgen-core` clean and `cargo test -p xgen-common -p xgen-core` clean.** The full workspace doesn't go green until Pass 2 (xgen-core algorithm widening), Pass 3 (xgen-node), and Pass 4 (xgen-client) ship. **This is acceptable per Pass 1's scope discipline — the build breakage outside xgen-common/xgen-core is the honest signal that downstream Passes are waiting.** If Clair finds this unacceptable, surface to Joe for a mid-implementation walk on whether to fold a thin downstream bridging shim into Pass 1's scope (would be a real scope expansion, needs explicit Joe-lock).

**Alternative if full-workspace-green is required at Pass 1 close.** Clair may add temporary `.as_str()` projection shims at the xgen-node and xgen-client call sites that consume retyped xgen-core types — minimal touch, no algorithm changes, just type-boundary projections with code comments flagging Pass N. This would inflate Pass 1's scope beyond "data structures only" but produces a workspace-green Pass 1 close. **This is a structural design question; Clair MUST surface to Joe at commit 4 implementation kickoff before deciding.** Either path is defensible; the honest framing here is that Pass 1 "data structures only" pure scope produces a workspace-broken intermediate state, and the workspace-green alternative requires a defined scope expansion.

### Commit 5 — Appendix C + Appendix I retypes

**Files touched (2):**

- `docs/xgen_appendix_c_en.md` — primitive schema field tables. Per the Phase 2 doc-tree sweep, Appx C contains 15 XGID hits across its schema field definitions. Each XGID-bearing field's **Type column** updates from `String` to the appropriate flavour-typed XGID (`EventXgid`, `IdentityXgid`, etc.). **Field name column stays unchanged** (D-072 invariance 1, D-073 name discipline). **Wire-key column stays unchanged** (D-072 invariance 1 + 2). **Req/Opt column stays unchanged.** **Description column** gets minimal edits only where the description currently calls the field "a String" or similar type-naming language — the descriptive content stays the same.
- `docs/xgen_appendix_i_en.md` — data structures field tables. Per the Phase 2 doc-tree sweep, Appx I contains 122 XGID hits across Parts I through IX. Same column-by-column treatment. Parts that need attention:
  - **Part I — Event Envelope.** I.1 `Event` field table: every XGID-bearing field's Type column updated.
  - **Other Parts (II through IX).** Identify XGID-bearing struct tables and apply the same treatment. Specific sub-sections to verify at implementation kickoff (the runbook does not pre-enumerate every table here — Clair walks Appx I top-to-bottom and applies the rule uniformly).
- Header bumps on both files (`Last updated`, `Version`).

**What does NOT change in commit 5:**

- No code changes — commit 5 is pure documentation.
- No edits to other appendices (D, E, F, G, H, J), per the Phase 2 doc-tree sweep classification.
- No edits to Ch3, Ch4, or other spec chapters (sweep classification: Ch3 normative authority already established at v1, Ch4 v1 pointer already landed).

**Verification at commit 5:** No build/test verification (pure docs). Spot-check: every retyped field in commits 3 and 4 has a corresponding Appx C or Appx I table row showing the new type. Diff is reviewable for column-level discipline (no Wire-key drift, no field-name drift).

### Commit 6 — Milestone close

**Files touched (5–6):**

- `tasks/XGID_RETROFIT_PASS_1_IMPL.md` — Status flipped ACTIVE → COMPLETED. Version bumped 2.0 → 2.1. `Last updated` field gains a milestone-close note recording the implementation outcomes (test count delta, any sub-question revisions surfaced during implementation, any structural findings flagged).
- `CLAUDE.md` — PLAY block refreshed. Pass 1 closure noted in DONE-IN-FLIGHT section. Pass 2 becomes the next-active XGID retrofit slot. Header bumped.
- `docs/ROADMAP.md` — Past gains a Pass 1 closure entry; Present updated; Near future loses the now-shipped Pass 1 line; Visual structure tree updated in same edit per the v1.4 guardrail; header bumped.
- `JOURNAL.md` — new J-NNN entry recording the Pass 1 milestone close. Per the D-074 candidate principle (surfaced 2026-05-20), JOURNAL.md MUST be in the milestone-close commit's changed-files list.
- Test count delta recorded honestly in the JOURNAL entry. Expected delta: +5 from the new invariance tests (Tests A–E) plus any incidental unit tests added during commits 1–4.
- Header bumps on all touched files per the convention.

**Definition of Done for commit 6 (and Pass 1 as a whole):**

- [ ] All six commits landed in sequential order.
- [ ] `cargo test -p xgen-common -p xgen-core` clean at every commit boundary.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` clean at every commit boundary.
- [ ] All v1 invariance tests still pass (regression-lock).
- [ ] All five new Pass 1 invariance tests pass.
- [ ] Appx C and Appx I field tables retyped column-by-column with no Wire-key drift.
- [ ] Pre-existing flakes (precedence env-var race, `reconnect_with_existing_tip_small_delta_delivered`) handled per their established retry shape; not counted as Pass 1 regressions.
- [ ] Workspace-broken intermediate state between commit 4 and Pass 2 explicitly acknowledged per sub-question 1's commit-sequence design, OR Joe-locked alternative shipped per the commit-4 "alternative if full-workspace-green is required" hedge.
- [ ] JOURNAL.md includes the Pass 1 close entry.
- [ ] CLAUDE.md and ROADMAP.md state-change is reflected in the same commit as the runbook Status flip.

**No checklist item names "commit pushed."** The milestone-close commit is itself the push, and "commit pushed" is unflippable inside the commit that performs the push (D-074 candidate, chicken-and-egg principle from XGID Adoption v1 close).

---

## Doc scope walkthrough

Appendix C and Appendix I retypes ship together in commit 5 per sub-question 1's lock. Treatment is uniform across both:

**Per-field rule.**

- **Field name column**: unchanged. D-072 invariance 1 + D-073 name discipline.
- **Wire key column**: unchanged. D-072 invariance 2 (the on-wire JSON type stays `string`).
- **Type column**: updated from `String` / string to the appropriate flavour-typed XGID / string (e.g. `EventXgid` / string, `IdentityXgid` / string, `NodeXgid` / string, `SpaceXgid` / string, `RoomXgid` / string, `TrustAssertionXgid` / string).
- **Req/Opt column**: unchanged.
- **Description column**: minimal edits. Where the description currently says "a String containing" or "the String value of" or similar type-naming language, update to the flavour-typed naming. Where the description names the protocol-object role ("the Identity public key of the sender"), no change — role naming was already correct.

**Appendix C scope.** Primitive schemas. 15 XGID hits per the doc-sweep classification. Walk top-to-bottom; apply the rule uniformly. Specific tables to verify at implementation kickoff include any schema that references protocol-object identifiers (sender, room, space, event, identity, node, trust assertion).

**Appendix I scope.** Data structures. 122 XGID hits per the doc-sweep classification. The heaviest doc surface in Pass 1.

- **Part I — Event Envelope.** Section I.1 `Event` field table: retype every XGID-bearing field (`event_id`, `sender`, `room_id`, `space_id`, `prev_events`). Section I.2 `EventType Registry`: no XGID fields directly, but the section may reference XGID-bearing content schemas (cross-check at implementation kickoff).
- **Other Parts.** Walk Parts II through IX. Any struct table with an XGID-bearing field (typically named `*_id`, `sender`, or similar) gets the column-level treatment. Parts likely to need attention: any Federation message types, Identity registration types, Space/Room state types, MLS message types if they carry Identity XGIDs.
- **Description column edits across Appx I.** Several existing field descriptions reference "a String formatted as `xgen://...`" — these update to reference the typed XGID flavour while preserving the URI-format description.

**Header bumps.** Both `docs/xgen_appendix_c_en.md` and `docs/xgen_appendix_i_en.md` get their `Last updated` and `Version` fields bumped, with a one-line milestone-close note in the `Last updated` field naming Pass 1.

**Commit 5 sizing.** Pure documentation, no build verification. Diff size is large (the column-by-column rewrites across 100+ field-table rows) but the edit shape is mechanical. Review is reading the diff to spot any drift in field-name or wire-key columns — those columns should show zero changes; any change there is a structural finding to surface.

---

## Verification gate

Pass 1's verification gate is checked at the **milestone close (commit 6)**, with intermediate per-commit verification at commits 1, 2, 3, and 4. Commit 5 has no build verification (pure docs).

**Per-commit verification (commits 1–4).**

- **Commit 1:** `cargo build --workspace` clean. `cargo test --workspace` clean. `cargo clippy --workspace --all-targets -- -D warnings` clean. (Canonical-form module move is workspace-transparent thanks to the re-export shim.)
- **Commit 2:** `cargo test -p xgen-common` clean. `cargo clippy --workspace --all-targets -- -D warnings` clean. (Convenience constructors are additive API; full-workspace tests stay green.)
- **Commit 3:** `cargo test -p xgen-common` clean, including the five new invariance tests (Tests A–E). `cargo build --workspace` will FAIL at this commit (xgen-core consumers of the retyped xgen-common types don't compile yet). This is the honest intermediate state per sub-question 1's commit-sequence design.
- **Commit 4:** `cargo test -p xgen-common -p xgen-core` clean. `cargo build --workspace` may still fail (xgen-node and xgen-client downstream). This is the structural finding flagged in the code scope walkthrough — Clair surfaces to Joe at commit 4 implementation kickoff for a decision on whether to ship a minimal bridging-shim alternative.

**Milestone-close verification (commit 6, captures the final Pass 1 state).**

- [ ] `cargo test -p xgen-common -p xgen-core` clean. All v1 invariance tests still pass. All five new Pass 1 invariance tests pass.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` clean across xgen-common and xgen-core.
- [ ] `cargo build --workspace` either (a) clean if the bridging-shim alternative was Joe-locked at commit 4, or (b) cleanly broken at xgen-node/xgen-client with documented Pass 2/3/4 boundary signals if the pure-scope discipline was held.
- [ ] Pre-existing flake handling: two known flakes (precedence env-var race from J-079; `reconnect_with_existing_tip_small_delta_delivered` from Phase 3) are NOT a Pass 1 regression signature. If either fires during verification, retry once in isolation per their established flake fix-shape.
- [ ] Test count delta recorded honestly: baseline captured at Pass 1 implementation kickoff (≥ 571 from XGID Adoption v1 close, plus whatever Phase 9 has added in the parallel-track window), delta is at minimum +5 from the new invariance tests plus any incidental tests added during commits 1–4.
- [ ] No `String` → `Xgid::new(String::new())` empty-string projections survive without code comments flagging the wire-shape rationale (the `room_id` empty-string case for space-level events; the `space_id` empty-string case for `state.space_create`).
- [ ] Honest-broadening discipline held: no method signatures widened in commits 3 or 4 except as flagged for Joe-decision at commit 4 implementation kickoff.

**Workspace-parallelism flakes (carried forward from earlier milestones, unchanged at Pass 1).** Per J-082 / J-086 / J-095, two known flakes exist under `cargo test --workspace`: the precedence env-var race introduced at D-068 commit `3e2f311` (~10–20% workspace runs); and `reconnect_with_existing_tip_small_delta_delivered` (Phase 3 test, surfaces under post-Phase-4 parallelism, ~10% workspace runs, 0% isolated). Both are not Pass 1's problem and are not signatures of a Pass 1 regression. Retry on either failure; if a Pass-1-specific test exhibits flake under workspace parallelism, surface as a structural finding (would be new behaviour, not an inherited carry-forward).

---

## Milestone close

Commit 6 ships the milestone close. The commit's changed-files list includes:

1. **`tasks/XGID_RETROFIT_PASS_1_IMPL.md`** — Status flipped ACTIVE → COMPLETED. Version bumped 2.0 → 2.1. `Last updated` field captures the implementation outcomes (test count delta, any sub-question revisions, any structural findings).
2. **`CLAUDE.md`** — PLAY block flipped to reflect Pass 1 closure. Pass 2 becomes the next-active XGID retrofit slot (or M6 Block 4 if Pass 2 isn't immediately picked up). Header bumped.
3. **`docs/ROADMAP.md`** — Past gains a Pass 1 closure entry under the XGID Retrofit Pass series cluster; Present updated to reflect current state; Near future loses the now-shipped Pass 1 line and the Pass 2 line gains its readiness note; Visual structure tree updated in same edit per the v1.4 guardrail (Pass 1 row flips 🟡 → ✅); header bumped.
4. **`JOURNAL.md`** — new J-NNN entry. Per the D-074 candidate principle, JOURNAL.md MUST be in the milestone-close commit's changed-files list. The entry records the six-commit sequence, the test count delta, any sub-question revisions surfaced during implementation, any structural findings flagged, the workspace-build state at close (full green vs broken-with-documented-boundaries per the commit-4 Joe-decision), and the honest-broadening discipline outcomes.
5. **(Conditional) `docs/xgen_appendix_c_en.md` and `docs/xgen_appendix_i_en.md` header re-bumps** if their `Last updated` fields need a final milestone-close note tying them to the Pass 1 close commit (these were already bumped in commit 5; the second bump in commit 6 is optional and depends on the convention's tolerance).

**Discipline notes for the milestone-close commit.**

- The commit message follows the convention from XGID Adoption v1 milestone close: subject line names the milestone (`XGID Retrofit Pass 1 — milestone close`); body paragraphs describe what shipped, test count delta, sub-question outcomes, and any structural findings.
- The commit message body explicitly references the Pass 1 runbook (this file) so a future reader scanning `git log` finds the authoritative scope record.
- If the workspace-broken-intermediate-state alternative was held (sub-question per commit 4), the milestone-close commit's body explicitly names this as a Pass 1 expected outcome and points at Pass 2 as the next step that closes the workspace build.
- If the bridging-shim alternative was Joe-locked at commit 4, the milestone-close commit's body describes the shim and flags its removal as Pass 2 scope.

**No checklist item names "commit pushed."** Per the D-074 candidate principle, the milestone-close commit is itself the push, and `Status: COMPLETED` plus the JOURNAL.md entry are the real signal that the milestone closed.

**Post-close handoff.** After commit 6 lands:

- Pass 2 runbook authoring becomes the next-active XGID retrofit slot. Pass 2's runbook author (Chat Claude) inherits the test-count baseline from Pass 1's close + the honest-broadening boundary signals Pass 1 documented.
- If the workspace-broken intermediate state was held at Pass 1 close, Pass 2's first verification is `cargo build --workspace` after its commits land — Pass 2 going green is the workspace-going-green signal.
- M6 Block 4 (verb-by-verb walks) and any other parallel Chat Claude work continues unblocked.

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
