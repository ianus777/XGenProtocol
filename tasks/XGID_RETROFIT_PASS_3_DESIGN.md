# XGID Retrofit Pass 3 — Design
> **Status**: COMPLETED  
> Version: 1.4  
> Date: May 2026  
> **Last updated**: 2026-05-28 (J-134 — In-place rewrite-correction of the J-133 §2.3 Q3.6 v1.3 rewrite, which carried a wrong claim about `SpaceState.federation_nodes` current production type. Clair's J-134 atom-prep D-078 grep against the struct definition at xgen-core/src/space/state.rs:132 found the field IS already `Vec<NodeXgid>` (retyped at Pass 1 Commit 4 `774fe9d`, J-122 milestone-close arc, confirmed via `git blame` + the inline comment at state.rs:423 "Pass 1 retypes federation_nodes to Vec<NodeXgid>"). The J-133 Q3.6 v1.3 rewrite — which I (Clair) authored — claimed the retype "lands when `SpaceState.federation_nodes: Vec<String>` retypes to `Vec<NodeXgid>` — flagged as Surface #1 Q1.1 extension"; that claim was authored from inference against the xgen-node-side federation_session.rs:248 local-variable annotation (`let federation_nodes: Vec<String> = ...` against an actual `Vec<NodeXgid>` source, a Pass-1-broken xgen-node compile error per Path A intermediate state) without greping the SpaceState struct definition. **Q3.6 v1.4 strikes the "Vec<String> → Vec<NodeXgid> retype lands at Q1.1 extension" sentence entirely**; replaces with honest framing: destination-peer concern satisfied by inheritance from already-typed `SpaceState.federation_nodes: Vec<NodeXgid>` (Pass 1 close); xgen-node call-site annotation fix at federation_session.rs:248 (drop the `: Vec<String>` annotation so type-inference accepts the typed source) is Surface #3 inheritance-row work, NOT a Surface #1 field-type extension. **Third instance of candidate D-NNN-κ** "design-doc Q-table-vs-production-code parameter-attribution discipline" surfaced at this fresh catch-event (J-134 atom prep, distinct from J-133's catch-event which closed Drifts #1 + #2 in the v1.0/v1.1/v1.2 original Q3.6 + Q5.14). Three instances across two distinct catch-events meets D-069 promotion threshold per sibling-shape to D-077 + D-078; **D-NNN-κ PROMOTED to D-079** in this atom — "Design-doc Q-table grounded by symbol-definition grep". Sub-shape worth recording in JOURNAL J-134 body (not promoted as separate candidate; κ applied to itself): "fix-author re-instantiates the discipline-failure being fixed" — the J-133 atom whose entire purpose was closing κ-drifts itself introduced a κ-drift because the amendment-author trusted a call-site annotation over a struct-def grep. §6.2 J-133-provenance prose carrying the same wrong "Q1.1 extension target" framing stays as historical record per anti-tempfile-deletion-of-decision-records + J-129/J-130 historical-record-preservation precedent; new §6.3 amendment-provenance documents the correction. §2.1 Q1.1 row intact at v1.4 (no Q1.11 row added — there is no field-type retype to enumerate; the field is already typed at Pass 1). "Honest longer work over fast shortcuts" Pass 3 count increments to TWO at J-134 — recurrence shape (a wrong canonical Q-row shipped to origin/main at J-133, corrected at J-134), distinct from prospective-catch shape (J-115/J-116/checkpoint-#2 stopped BEFORE shipping). Three-file atomic per D-074 (thirty-first instance) + Lock #3 per-commit cadence + D-079 promotion atom: design doc v1.3 → v1.4 + DECISIONS.md (new D-079 prepended) + JOURNAL.md (J-134 body §-entry). CLAUDE PLAY does NOT flip; ROADMAP NOT touched. Per Rule 0 + D-065 + D-067 + D-069 + D-071 + D-074 + D-077 + D-078 + D-079 (this atom's promotion).) Previous 2026-05-28 (J-133 — Track 1 canonical-record amendment at Joe-lock checkpoint #2 of Pass 3 Commit 2 prep. Two parameter-attribution drifts surfaced by Clair's D-078 verification of design doc §2 against production code: (1) §2.3 Q3.6 claimed `apply_federation_push` parameter `peer_node_id: &str` (destination peer) — production has no such parameter (5 params total: event, origin, runtime, federation_peer_senders, local_node_id at xgen-node/src/federation_session.rs:202-208); destination peer flows internally via `peer_id` loop variable bound from `Vec<String>` (`federation_nodes` snapshot) at federation_session.rs:261. (2) §2.5 Q5.14 mis-attributed parameter to type — claimed "`OutboundMsg.peer_node_id: String` (line 1165) — internal handler message struct field" — production: `OutboundMsg` is an enum at xgen-node/src/fanout.rs:31 with variants Event/HistoryBatch/SyncComplete and NO peer_node_id field; line 1165 in app.rs is a `peer_node_id: String` parameter on `pub(crate) async fn run_federation_session_post_handshake` (line 1152), which §2.5 sub-region enumeration omits from "Top-level orchestrators" alongside process_inbound + run_node. Both drifts share structural shape — parameter-attribution drift — opening candidate D-NNN-κ "design-doc Q-table-vs-production-code parameter-attribution discipline" flagged-not-promoted per D-069 (two instances within single catch-event; three-instance threshold wants distinct catch-events to establish durability; promotion-watch opens at Pass 4 design walk or earlier if a sibling fires at any design-doc-vs-production surface). Joe locked Path α (Track 1 canonical-record amendment in this session) over Path β (Clair latitude at surface-extraction) + Path γ (fold into Commit 2). Amendment scope: §2.3 Q3.6 rewritten to production-true (internal binding + `SpaceState.federation_nodes` field retype flagged as Surface #1 Q1.1 extension target); §2.5 Q5.14 rewritten to production-true (`run_federation_session_post_handshake` parameter set retyped per §4.2 v1.2 async-spawned-captures sub-rule + §4.3 format-boundary rule for wire-derived peer_tips); §2.5 "Three sub-regions identified at grep" prose extended to add `run_federation_session_post_handshake` to "Top-level orchestrators"; new §6.2 v1.2 → v1.3 amendment-provenance sub-section. CLAUDE PLAY does NOT flip; ROADMAP NOT touched; JOURNAL gets a body §-entry per J-133 (substantive prospective-catch at new surface layer with new candidate opened). "Honest longer work" Pass 3 count does NOT increment — sibling-shape to J-115/J-116 prospective catches (checkpoint mechanism working as designed). D-NNN-ζ stays at one instance; this is a distinct surface layer one rung deeper. Two-file atomic per D-074 (thirtieth instance) + Lock #3 per-commit cadence; not a milestone-close. Per Rule 0 + D-065 + D-067 + D-069 + D-071 + D-074 + D-077 + D-078.) Previous 2026-05-27 (J-127 — **Pass 3 design phase CLOSED.** Surfaces #5 (app.rs handlers) + #6 (reconnect.rs) + #7 (Appendix D doc retypes) Q-tables walked and locked, completing the full seven-surface walk. §4.3 consolidated to "format-boundary preservation (wire OR persistence)" framing — Surface #5's filesystem-path + on-disk JSON HashMap slots instantiated the same pattern as §4.3 wire-format boundary; one principle, two layers, one drift surface per amend-in-place sibling-shape to D-076 v1 → v1.1 framing. §4.2 sibling-shape rule table extended with async-spawned-task-captures sub-rule (Surface #5 + #6 instantiation). §5.5 layered-B3 confirmed null at full seven-surface scope — third Pass-arc instance after Pass 1 J-122 + Pass 2 J-126; pattern's durability at three instances now matches D-077/D-078 promotion-threshold framing. §6.1 historical-pointer entry filled in Shape α (pointer-style sibling to Pass 2 §6.7). §7 discipline-notes section filled with five sub-sections (§7.1 format-boundary preservation unified pattern + §7.2 async-spawned-task-captures sub-rule + §7.3 forced-owned return shape rule + §7.4 xgen-node-internal type confusion at v1.0 framing data point + §7.5 doc-tree sweep classification-vs-content-shape gap). Three candidate D-NNNs flagged-not-promoted per D-069 — D-NNN-γ "small-cardinality vs large-cardinality identifier-keyed maps per-Pass call-site density" (2 instances) + D-NNN-δ "format-boundary preservation wire+persistence" (2 instances) + D-NNN-ε "async-spawned task captures force owned" (3 instances at same xgen-node module-family surface; promotion-watch opens at Pass 4 surfacing a structurally different fourth at xgen-client async surfaces — Tauri commands, AI service spawns, batch dispatcher workers). DECISIONS.md NOT amended at this design close per honest framing — D-NNN-ε's three instances are sibling-shape variants of one Tokio idiom at the same module-family surface, not a cross-cutting project principle. Single governing principle (§3) confirmed inherited from Pass 2 unchanged across full seven-surface walk — no wrinkles. Per Rule 0 + D-065 + D-067 + D-069 + D-071 + D-074 + D-077 + D-078.) Previous 2026-05-27 (J-NNN — Surfaces #1 (NodeRuntime six per-space HashMap keys) + #2 (dispatch_event peer_node_id parameter) + #3 (federation_session.rs handler identifier slots) + #4 (fanout.rs handler identifier slots) Q-tables walked and locked. §3 governing principle confirmed inherited from Pass 2 unchanged after four-surface walk produced no wrinkles. §4 architectural decisions added: §4.1 six per-space HashMap keys retype shape; §4.2 dispatch_event Option<&NodeXgid> borrowed boundary; §4.3 wire-format String boundary preservation (deliberate); §4.4 event_space_id return shape forced-owned per construction-required branch; §4.5 ClientSenders + FederationPeerSenders retype scope (Pass 3, not Pass 4); §4.6 topo-sort &str slot at fanout.rs:193 stays unchanged (no separate identifier slot; existing Borrow<str> via Option<EventXgid>). §5.1 (deferred surfaces) honest-broadening updated with structural findings from #3/#4 walks. §3-§5 deferred sections from v1.0 now filled for Surfaces #1-#4; Surfaces #5-#7 still pending walk. Per Rule 0 + D-065 + D-067 + D-069 + D-071 + D-074 + D-077 + D-078.) Previous 2026-05-27 (J-NNN — Pass 3 design phase opened. §1 framing + §2 surface enumeration shipped at design-phase kickoff; §3 governing principle + §4 architectural decisions + §5 layered-B3 expected answer + §6 historical/future-pointer entries + §7 discipline-notes deferred to subsequent walk-and-lock sessions per Joe-lock at session open. Pass 3 scope: xgen-node + Appendix D — federation_session, fanout, app handlers, reconnect scheduler, the six per-space HashMap keys at NodeRuntime deferred from Pass 2 per design doc §4.1 Q2.8.c, and Appendix D doc retypes. Seven surfaces enumerated in dependency order at §2 (NodeRuntime per-space HashMap keys → dispatch_event peer parameter → federation_session.rs handler slots → fanout.rs handler slots → app.rs handler slots → reconnect scheduler identifiers → Appendix D doc retypes). Per Rule 0 + D-065 + D-067 + D-069 + D-071 + D-074 + D-077 + D-078.)  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 Framing

### §1.1 What this document is

The Pass 3 design closure for the XGID Retrofit milestone series. Pass 1 closed at J-122 (2026-05-26) with the core data structures in `xgen-common` + `xgen-core` retyped to typed XGID flavours. Pass 2 closed at J-126 (2026-05-27) with the xgen-core algorithm-bearing functions retyped (`validate_event`, `dispatch_event`, `PendingBuffer` arrival hooks, registry APIs, `accept_message`). Pass 3 retypes the **xgen-node binary surface** that hosts those algorithms in production — the federation transport handlers, the fanout dispatcher, the application-level event router, the reconnect scheduler, plus the six per-space `HashMap<String, _>` keys at `NodeRuntime` that Pass 2 deferred per design doc §4.1 Q2.8.c — and the **Appendix D doc retypes** at `docs/xgen_appendix_d_en.md`.

This is **Phase A** of Pass 3's audit → design → runbook → implementation → close arc. Like Pass 2, the audit phase is absorbed into prior reconnaissance: Pass 1's pre-walk flagged Pass 2's xgen-core surfaces with inline code-comment markers; Pass 2's implementation similarly flagged Pass 3's xgen-node surfaces during the dispatch_event retype work (the `peer_node_id: Option<&str>` parameter on `dispatch_event` already carries a Pass 3 marker; the six per-space HashMap keys at `NodeRuntime` were explicitly deferred to Pass 3 per Pass 2 design doc §4.1 Q2.8.c). Per D-065 honest framing, this design doc opens directly at design with the question set pre-surfaced.

### §1.2 Precedent-positioning relative to Pass 2

Pass 2's design doc was ~31 KB. Pass 3 v1.2 ships at ~58 KB after full seven-surface walk — slightly heavier than Pass 2 but lighter than trilogy precedent (~80-100 KB) per Pass-internal-consistency framing inherited from Pass 2 §7.7. The extra weight comes from (a) the seven-vs-five surface count, (b) §4 architectural decisions count (six locks vs Pass 2's two), and (c) the §7 discipline-notes section's five sub-sections recording the format-boundary-preservation pattern + forced-owned return rule + async-spawned-captures sub-rule + two design-walk data points.

Three precedent-positioning notes:

1. **No re-walk reservation at design open held cleanly.** Pass 2 §1.2 also opened without §11 re-walk reservation; that pattern carried through cleanly with zero re-walks fired. Pass 3 closed the same way at v1.2 — zero §11 re-walks fired across the full seven-surface walk. If a re-walk fires post-design-close (during runbook authoring or implementation), this document gains §11 amendment in place per topo-sort J-099 precedent.

2. **Single governing principle inherited from Pass 2 unchanged across seven-surface walk.** Pass 2 §3 locked: *identifier slots retype to typed XGIDs; descriptive-string slots stay String; internal variables bind as typed references; &str projection happens at the call-site boundary via Borrow<str>; no Deref<Target = str> shortcuts.* Pass 3 surfaces consume the same Borrow<str> projection mechanism inherited from Pass 1. **At v1.2 close: confirmed inherited unchanged across all seven surfaces.** The format-boundary that emerged at §4.3 (consolidated to cover wire OR persistence at v1.2) did NOT modify the principle itself — the principle's "&str projection via Borrow<str>" clause anticipates format-boundary slots stay String by construction (format IS bytes-and-strings, not typed Rust newtypes); §4.3 makes this explicit as an architectural decision, not a principle amendment.

3. **Pass 3 spans multiple xgen-node modules + a doc.** Pass 2's surface was concentrated in `xgen-core/src/{message/exchange,node/runtime,node/pending,federation/registry,identity/registry}.rs`. Pass 3 surfaces span `xgen-node/src/{federation_session,fanout,app,reconnect}.rs` plus the `NodeRuntime` HashMap-key sub-surface (which sits at `xgen-core/src/node/runtime.rs` per the Pass 2 deferral) plus `docs/xgen_appendix_d_en.md`. The multi-file spread informs the runbook authoring step's contingent-split posture (D-NNN-δ promotion-watch per J-126 Sub-section 8).

### §1.3 What this document is NOT

- **Not a re-audit of Pass 2's xgen-core retypes.** Pass 2's COMPLETED locks stand authoritative.
- **Not the Pass 4 xgen-client design.** ops:: verb signatures, AI control flows, session state retypes defer to Pass 4 per honest-broadening (named explicitly at the per-surface walks in §2).
- **Not the Pass 5 test-fixture / trace-field / Debug-Display sweep.** Trace event fields, Display impls, Debug impls in handlers defer to Pass 5 per honest-broadening (named explicitly at the per-surface walks in §2).
- **Not a runbook.** The implementation sequencing, commit shape, Joe-lock checkpoints, and verification rigour live in `tasks/XGID_RETROFIT_PASS_3_IMPL.md` authored at runbook phase per topo-sort + persistence-amendment + Pass 1 + Pass 2 precedent.

---

## §2 Surface enumeration (dependency order)

Seven surfaces walked in dependency order. Surfaces #1-#4 walked at v1.1; Surfaces #5-#7 walked at v1.2. All seven Q-tables locked.

### §2.1 Surface #1 — `NodeRuntime` six per-space HashMap keys (xgen-core/src/node/runtime.rs)

**What this is.** Six `HashMap<String, _>` field types on the `NodeRuntime` struct keyed by space_id, deferred from Pass 2 per Pass 2 design doc §4.1 Q2.8.c on call-site-density grounds. Pass 2 retyped only the small-cardinality `peer_urls` HashMap (small enough that per-site Borrow<str> projection was structurally cheap); the six per-space maps were deferred to the Pass that touches their primary call-site crate (xgen-node), which is Pass 3.

**Foundational position.** These HashMap keys are foundational to most xgen-node handler call sites — the handlers look up per-space state by space_id, which means every space-id lookup site is a call site against these six maps. Retyping the field types first means downstream handler retypes consume already-typed keys; retyping handlers first against still-String keys would require sweep churn at the HashMap retype landing.

**Crate boundary note.** Surface #1 lives in `xgen-core/src/node/runtime.rs` even though every other Pass 3 surface lives in `xgen-node/src/`. The deferral was based on call-site-density (most call sites are in xgen-node) but the field-type definitions are in xgen-core. Pass 3 retypes both the field-type definitions (xgen-core) and the call sites (xgen-node) atomically per the principle that field-type and call-site retypes belong in the same commit.

**Q-table (locked at v1.1).**

| Q | Question | Answer | Reasoning |
|---|---|---|---|
| Q1.1 | What retypes? | Six fields on `NodeRuntime`: `spaces`, `stores`, `graphs`, `pending`, `dm_proposals`, `space_local_metadata` retype `HashMap<String, _>` → `HashMap<SpaceXgid, _>`. Seventh map `peer_urls` already typed at Pass 2. | Pass 2 §4.1 Q2.8.c deferral. |
| Q1.2 | Why now, not Pass 2? | Call-site density: most lookup/insert sites live in xgen-node (process_inbound, app handlers, replay_spaces_from_dir, federation_session). Retyping at Pass 2 would have generated sweep churn against still-String xgen-node call sites; deferring to Pass 3 lets field-type + call-sites retype atomically. | D-NNN-γ candidate sub-principle from Pass 2 §4.1. |
| Q1.3 | Internal xgen-core projection sites? | `ingest_event`, `accept_message`, `drain_pending_messages`, `dispatch_event`, `drain_pending_uniform`, `drain_pending_by_identity`, `drain_pending_by_federation_relationship`, `all_events`, `dag_tips`. Currently project `space_id.as_str().to_string()` at HashMap entry points. | Pass 2's Borrow<str> handles lookup projection; internal variables bind as typed at Pass 3. |
| Q1.4 | Boundary at internal `&str` parameters on helpers? | Retype to `&SpaceXgid`. Internal helpers (`drain_pending_messages`, `drain_pending_uniform`, `drain_pending_by_identity`, `drain_pending_by_federation_relationship`, `all_events`, `dag_tips`) take `space_id: &str` today. All call sites are within xgen-core + xgen-node (no public xgen-client API). | Inheriting Pass 2's principle: internal variables bind as typed references. |
| Q1.5 | Public-API retype: `all_events(&self, space_id: &str)` + `dag_tips(&self, space_id: &str)`? | **Retype both to `&SpaceXgid`.** Joe-lock confirmed at design walk (Q-A): preserves Pass-internal-consistency with Pass 2's principle. Borrow<str> means call sites holding `&str` continue to work via projection where needed; sites holding `SpaceXgid` work natively. | Pass 1's Borrow<str> additive API is load-bearing here. |
| Q1.6 | Test fixture sweep impact? | **Substantial** — `phase_7_5_tests` mod and `persistence_amendment_commit_2a_tests` mod in `runtime.rs` use `&space_id` extensively against typed APIs; also xgen-node test modules. Estimated test-fixture error count at lib-clean: moderate-to-high. | Calls for contingent-split posture pre-locked at runbook authoring (D-NNN-δ promotion-watch). |
| Q1.7 | `space_local_metadata` already-typed quirk? | `SpaceLocalMetadata.space_id` is `SpaceXgid` post-Pass-1 (Pass 1 Commit 3). HashMap key retype to `SpaceXgid` lands cleanly; inner field already typed. | No structural surprise. |
| Q1.8 | What defers to Pass 4? | Nothing here — these maps don't surface to xgen-client. | Out-of-scope. |
| Q1.9 | What defers to Pass 5? | Nothing here — no Debug/Display formatting on these fields, no trace fields. Trace events that reference `space_id` get retyped at the handler retype (Surface #3/#4/#5), not here. | Out-of-scope. |
| Q1.10 | Wire-format implications? | None — these are in-memory runtime maps, never serialised. | Out-of-scope. |

### §2.2 Surface #2 — `dispatch_event` `peer_node_id: Option<&str>` parameter (xgen-core/src/node/runtime.rs)

**What this is.** The federation-channel entry point parameter on `NodeRuntime::dispatch_event` carrying the wire-authenticated peer's node_id when the event arrived via federation (None when locally-submitted). Pass 2 retyped `dispatch_event`'s internal logic but left the parameter signature at `Option<&str>` with an inline code-comment marker indicating Pass 3 widens this to `Option<&NodeXgid>`.

**Foundational position.** Every call site invoking `dispatch_event` from xgen-node passes a `peer_node_id` value — federation_session.rs constructs this from handshake state; app.rs's local-submit path passes None; fanout.rs's federation-push path passes the destination peer's node_id. Retyping the parameter forces every call site to projection-clean its peer_node_id type at the boundary. Doing this before the handler-side retypes (Surfaces #3-#5) means handler retypes consume an already-typed parameter signature.

**Q-table (locked at v1.1).**

| Q | Question | Answer | Reasoning |
|---|---|---|---|
| Q2.1 | Parameter shape: `Option<&NodeXgid>` vs `Option<NodeXgid>`? | **`Option<&NodeXgid>` (borrowed).** Joe-lock confirmed at design walk (Q-B). | Parameter never stored — federation_session.rs passes a `&NodeXgid` it owns; locally-submitted calls pass `None`. Owned would force unnecessary clones at every call site. |
| Q2.2 | All call sites? | Federation channel: `xgen-node/src/app.rs::process_inbound` (federation route), `federation_session.rs::apply_federation_push` chain. Local: `app.rs::process_inbound` (local route), test sites, `drain_pending_*` sites pass `None`. | Exhaustive sweep at runbook authoring. |
| Q2.3 | Internal binding inside `dispatch_event`? | Currently `peer_node_id.is_some()` checks + `let skip_f3 = matches!(...)` use — none depend on string content. String-content uses: F-3 relationship check (`.federation_nodes.iter().any(\|n\| n.as_str() == peer)`) and HeldPending buffer key construction (`NodeXgid::from_xgid(Xgid::new(peer.to_string()))`). | Both retype cleanly: F-3 check becomes `.iter().any(\|n\| n == peer_node_id)` (NodeXgid PartialEq via inner Xgid); buffer-key construction becomes `peer_node_id.clone()` (no `Xgid::new` wrap needed). |
| Q2.4 | D-075 vantage derivation block (the `fed_add_drain_pair`)? | Currently builds `(String, String)` peer/space_id pairs. With typed `peer_node_id: Option<&NodeXgid>` available, this block builds typed pairs. The drain helpers `drain_pending_by_federation_relationship` take `&str` peer/space args today — those signatures retype too (Q1.4 territory: internal helpers). | Atomic with helper signature retypes; no cross-Pass split. |
| Q2.5 | drain_pending_* helpers' `peer_node_id: &str` / `resolved_space_id: &str` params? | Retype to `&NodeXgid` / `&SpaceXgid`. Xgen-core-internal helpers called from `dispatch_event` + from `xgen-node/src/app.rs::handle_identity_replicate_msg` (drain_pending_by_identity only). | Internal helpers + the only xgen-node caller is Pass 3 scope anyway. |
| Q2.6 | Wire-format / serialisation? | None — `peer_node_id` is purely runtime; never serialised. | Out-of-scope. |
| Q2.7 | Trace event field `peer_node_id = %peer`? | Stays as-is at Pass 3 (the `%peer` formatter calls Display on NodeXgid which projects to inner string). Trace-field type-level audit defers to Pass 5. | Per honest-broadening. |
| Q2.8 | What defers to Pass 4? | Nothing — dispatch_event is xgen-node-facing only. | Out-of-scope. |
| Q2.9 | What defers to Pass 5? | Trace field formatter audit. | Out-of-scope. |
| Q2.10 | Pass 2 inline marker locations? | `xgen-core/src/node/runtime.rs` — comments at `dispatch_event` signature + at the `NodeXgid::from_xgid(Xgid::new(peer.to_string()))` buffer-key construction site + at the D-075 vantage block. | All collapse to typed-borrow / typed-clone at Pass 3. |

### §2.3 Surface #3 — `federation_session.rs` handler identifier slots (xgen-node/src/federation_session.rs)

**What this is.** The federation transport handler module hosting wire-format handshake logic, per-peer session state, and the `apply_federation_push` entry point that Pass 2's J-111 trace-event retrofit instrumented with `local_node_id: &str`. Identifier slots in this module: peer node_id (from handshake), local node_id (from runtime), space_id slots passed through to runtime calls, event_id slots on the wire path.

**Consumes Surfaces #1 + #2.** federation_session.rs calls into `NodeRuntime::dispatch_event` and into per-space HashMap lookups; both are foundational dependencies retyped at #1 and #2.

**Wire-format vs in-memory split (load-bearing finding from walk).** federation_session.rs hosts wire-format serialisation. The wire-format layer (TransportMessage parsing/emission) is independent of the in-memory Rust type layer (wire is canonical bytes per Appendix J, not typed Rust newtypes). Pass 3 retypes the **in-memory Rust slots** (function parameters, internal variables, struct fields); wire-format String slots (`peer_tips: &BTreeMap<String, String>`, `since: &str`, `continue_from: Option<String>`) stay String at function boundaries by deliberate design — see §4.3 architectural lock (consolidated at v1.2 to cover wire OR persistence).

**Q-table (locked at v1.1).**

| Q | Question | Answer | Reasoning |
|---|---|---|---|
| Q3.1 | `stream_federation_delta` parameter `shared_spaces: &[String]`? | **Retype to `&[SpaceXgid]`.** Source is `runtime.spaces.keys()` which post-Surface-#1 returns `&SpaceXgid`. | In-memory Rust slot, not wire-format. |
| Q3.2 | `stream_federation_delta` parameter `peer_tips: &BTreeMap<String, String>`? | **Stays as `&BTreeMap<String, String>`.** Wire-derived from TransportMessage::Hello/Capabilities. | §4.3 format-boundary preservation. Conversion to typed XGIDs happens at the per-iteration lookup, not in the BTreeMap itself. |
| Q3.3 | `stream_federation_delta` parameter `peer_node_id: &str`? | **Retype to `&NodeXgid`.** Source is the post-handshake session state (in-memory). | In-memory Rust slot. |
| Q3.4 | `stream_federation_delta` parameter `session_id: &str` + `negotiated_version: &str` + `negotiated_serialisation: &str`? | **Stay as `&str`.** Descriptive-string slots per Pass 2 §3 principle. None of these are typed XGIDs. | Pass 2 principle: descriptive-string slots stay String. |
| Q3.5 | `apply_federation_push` parameter `local_node_id: &str` (J-111 retrofit)? | **Retype to `&NodeXgid`.** Source is `home_node_id` from runtime (in-memory, will be NodeXgid post-Surface-#1). | In-memory Rust slot. |
| Q3.6 | `apply_federation_push` destination-peer slot — what retypes? | **No retype work at Surface #1 or Surface #3 field-definition layer; xgen-node call-site annotation fix only, at Surface #3 inheritance-row.** Rewritten at v1.4 per J-134 D-078 grep correction: `SpaceState.federation_nodes` is already `Vec<NodeXgid>` at xgen-core/src/space/state.rs:132 (retyped at Pass 1 Commit 4 `774fe9d`, J-122 close arc; confirmed via `git blame` + inline comment at state.rs:423 "Pass 1 retypes federation_nodes to Vec<NodeXgid>"). The destination-peer typed-flow is satisfied by inheritance from the already-typed struct field — no Pass 3 field-type retype lands here. The xgen-node-side compile error at federation_session.rs:248 — `let federation_nodes: Vec<String> = { ... s.federation_nodes.clone() ... }` annotation incompatible with the `Vec<NodeXgid>` source — is one of the Pass-1-broken-by-Path-A xgen-node intermediate-state compile errors. Pass 3's actual work at this slot is to **drop the `: Vec<String>` annotation at federation_session.rs:248 (or change to `: Vec<NodeXgid>`) so type-inference accepts the typed source from the typed struct field**. This is Surface #3 inheritance-row work (federation_session.rs handler identifier slots inheriting from Surface #1's already-done retype), NOT a Surface #1 field-type extension. The `peer_id` loop variable at federation_session.rs:261 also inherits typing naturally post-annotation-fix. Production at apply_federation_push has 5 parameters at federation_session.rs:202-208 (`event, origin, runtime, federation_peer_senders, local_node_id`) — no `peer_node_id` parameter (the v1.0/v1.1/v1.2 original Q3.6 mis-attribution closed at v1.3; this v1.4 closes a separate mis-attribution about the field's current state). | **v1.4 production-true correction (third)**: v1.0/v1.1/v1.2 claimed a non-existent parameter on apply_federation_push (closed at v1.3 via J-133). v1.3 claimed a field-type retype that production had already done at Pass 1 (closed at v1.4 via J-134). Third D-NNN-κ instance at a fresh catch-event; D-NNN-κ promoted to **D-079** at this atom. Verified anchors: state.rs:132 (struct definition) + state.rs:423 (inline comment) + federation_session.rs:202-208 (apply_federation_push signature) + federation_session.rs:248-261 (xgen-node call-site annotation + loop). |
| Q3.7 | Trace events `peer_node_id = %peer` / `local_node_id = %local_node_id` / `space_id = %space_id` / `event_id = %event_id_for_log`? | **Stay as-is.** The `%` formatter calls Display on NodeXgid/SpaceXgid which projects to inner string for log output. | Trace-field type-level audit defers to Pass 5. |
| Q3.8 | What defers to Pass 4? | Nothing — federation_session.rs is xgen-node-only. | Out-of-scope. |
| Q3.9 | What defers to Pass 5? | Trace field formatter audit; Debug/Display impls if any. | Out-of-scope. |
| Q3.10 | Pass-3-deferred markers from prior milestones? | Pass 2's `local_node_id` retrofit at J-111 explicitly carried "type stays String per Q1; XGID Retrofit Pass 1 will sweep all four trace fields together" — that sweep happens at Pass 5 trace-field audit, not here. Pass 3 retypes the function-parameter slot only. | Pass-of-record honesty per honest-broadening. |

### §2.4 Surface #4 — `fanout.rs` handler identifier slots (xgen-node/src/fanout.rs)

**What this is.** The fanout dispatcher module hosting per-space history computation (`compute_federation_delta_for_space`), topological sort (`topological_sort_events`), and the local fan-out delivery path (`apply_fanout`). Identifier slots: destination peer node_id, source local node_id, space_id, event_id slots on the delta-computation and push paths; client identity_id keys on `ClientSenders`; peer node_id keys on `FederationPeerSenders`.

**Sibling to #3.** fanout.rs's interaction with the type graph is structurally similar to federation_session.rs's; differences are functional (delta computation + topo sort vs handshake + per-peer session). Walking #3 before #4 resolved most #4 questions; #4's walk surfaced delta-computation-specific structural decisions (event_space_id return shape — §4.4).

**Q-table (locked at v1.1).**

| Q | Question | Answer | Reasoning |
|---|---|---|---|
| Q4.1 | `event_space_id(event: &Event) -> Option<???>` return shape? | **`Option<SpaceXgid>` (owned).** Joe-lock confirmed at design walk (Q-C). | State-create branch constructs a SpaceXgid from event_id (flavour change EventXgid → SpaceXgid); must be owned. Non-state-create branch clones event.space_id (already SpaceXgid). Borrowed would require single-flavour return which breaks Pass 1's flavour discipline. **General rule recorded inline at §4.4: parameters favour borrowed, returns favour owned when any branch must construct.** |
| Q4.2 | `ClientSenders` HashMap key retype? | **Retype to `IdentityXgid`.** ClientSenders is xgen-node-internal (channels never escape to xgen-client; they're consumed by the WebSocket drain loop). Joe-lock confirmed at design walk (Q-E). | Per §4.5 architectural lock: xgen-node-internal types retype at Pass 3, not deferred to Pass 4. |
| Q4.3 | `FederationPeerSenders` HashMap key retype? | **Retype to `NodeXgid`.** Sibling to Q4.2. | Same Pass 3 scope justification. |
| Q4.4 | `apply_fanout` parameter `author_id: &str`? | **Retype to `&IdentityXgid`.** Function reads value for member-recipient filtering. | Pass 2 principle: internal variables bind as typed references. |
| Q4.5 | `FanoutRequest.new_joiner: Option<String>`? | **Retype to `Option<IdentityXgid>`.** Struct field — owned. The xgen-node-internal struct's consumers all operate on identity_id values. | Pass 2 principle: struct fields owned. Pass 3 scope (xgen-node-internal struct). |
| Q4.6 | `compute_federation_delta_for_space(space_id: &str, peer_tip: Option<&str>)`? | **Retype `space_id` → `&SpaceXgid`; `peer_tip` → `Option<&EventXgid>`.** Both are in-memory Rust slots. peer_tip is a wire-derived event_id but passed as `Option<&EventXgid>` to the in-memory function. The wire→typed conversion happens at the boundary in `stream_federation_delta`. | §4.3 format-boundary preservation applies at one layer up; this internal helper consumes already-typed values. |
| Q4.7 | `collect_sync_history(requester_id: &str, since: &str, limit: usize) -> (Vec<Event>, Option<String>)`? | **`requester_id` retype to `&IdentityXgid`** (in-memory). **`since` stays `&str`, `continue_from: Option<String>` stays String.** Both `since` and `continue_from` are wire-format pagination cursors (TransportMessage::SyncRequest::since / SyncComplete::continue_from). | §4.3 format-boundary preservation. WebSocket drain arm calls this function with wire-format Strings; conversion to typed XGIDs would require restructuring the wire-deserialization layer — out of Pass 3 scope. |
| Q4.8 | `topological_sort_events`'s `&str` sort slot at fanout.rs:193? | **No retype needed.** The sort compares `a.event_id.cmp(&b.event_id)` where `event_id: Option<EventXgid>`. The comparison works through existing Borrow<str> projection (Option's Ord uses inner Ord; EventXgid's Ord uses inner Xgid's Ord which delegates to String's Ord). | §4.6 — the J-097 topo-sort lock referenced a slot that no longer exists as a separate identifier slot post-Pass-1. The slot inherently retyped to typed-Option at Pass 1 Commit 3. |
| Q4.9 | `apply_fanout` internal `recipients: Vec<String>` (collected from space.members.keys())? | **Retype to `Vec<IdentityXgid>`.** Post-Pass-1 `space.members` is `HashMap<IdentityXgid, _>`; `.keys().cloned().collect()` produces `Vec<IdentityXgid>` natively. | Pass 1 retype already done; this site just reads the natural type. |
| Q4.10 | Trace events `event_id = %event_id_for_log` / `client_id = %rid`? | Stay as-is. | Pass 5 trace-field audit. |
| Q4.11 | What defers to Pass 4? | Nothing — fanout.rs is xgen-node-internal; channels never cross to xgen-client. | Out-of-scope. |
| Q4.12 | What defers to Pass 5? | Trace fields; Debug/Display impls. | Out-of-scope. |

### §2.5 Surface #5 — `app.rs` handler identifier slots (xgen-node/src/app.rs)

**What this is.** The xgen-node application-level entry point hosting top-level event handlers (`process_inbound`, identity handlers, the pipe server admin verbs at Block 4 of M6 (new) if M6 (new) ships before or during Pass 3), the reconnect scheduler invocation surface, the bootstrap logic, and persistence helpers. Identifier slots: local node_id, peer node_ids in many forms (from handshake, from registry, from pipe-server admin verbs), space_ids at handler boundaries, event_ids at handler boundaries, plus persistence-layer space_id slots at the filesystem boundary.

**Top of xgen-node module graph.** Consumes Surfaces #1 (per-space HashMaps via runtime accessors) + #2 (dispatch_event peer_node_id) + #3 (federation_session entry points) + #4 (fanout.rs ClientSenders + FederationPeerSenders + apply_fanout author_id).

**Three sub-regions identified at grep** (extended at v1.3 — `run_federation_session_post_handshake` added to "Top-level orchestrators" per J-133 D-078 verification surfacing the omission):
- **Wire-format handlers**: `handle_federation_incoming`, `handle_identity_msg`, `handle_identity_replicate_msg`, `push_identity_to_peers` — receive wire-derived messages, dispatch into typed runtime
- **Top-level orchestrators**: `process_inbound`, `run_node`, `run_federation_session_post_handshake` (`pub(crate) async fn` at app.rs:1152; bilateral federation session driver — both Initiator + Receiver post-handshake; takes 13 parameters including 7 identifier-shaped slots covered at Q5.14 v1.3) — sit between WebSocket/federation transport and runtime
- **Admin-state builders + persistence helpers**: `build_node_state`, `persist_event`, `space_file_name`, `load_space_local_metadata`, `save_space_local_metadata`, `replay_spaces_from_dir`

**Persistence-format boundary instantiation at this surface (LOAD-BEARING finding).** `space_file_name` + `persist_event` + `load_space_local_metadata` + `save_space_local_metadata` form the persistence-format boundary. Same shape as §4.3 wire-format: the filesystem-path String AND the on-disk JSON HashMap key String are persistence-format slots; functions that touch them at the boundary stay String; conversion to typed XGID happens at callers' typed-call boundary. **This finding drove the §4.3 v1.1 → v1.2 consolidation from "wire-format boundary" to "format-boundary preservation (wire OR persistence)".**

**M6 (new) coordination flag.** If M6 (new) Block 4 verb-by-verb walks ship pipe-server admin verbs before Pass 3 implementation, those verbs land with String-typed identifier slots that Pass 3 retypes. If Pass 3 ships first, M6 (new) Block 4 verbs land already-Pass-3-typed. The coordination flag stays at the runbook authoring phase per honest-broadening discipline — the design doc just names the coordination surface; the runbook decides sequencing.

**Q-table (locked at v1.2).**

| Q | Question | Answer | Reasoning |
|---|---|---|---|
| Q5.1 | `process_inbound(..., identity_id: &str, home_node_id: &str, ...)` parameters? | Retype to `&IdentityXgid` and `&NodeXgid`. In-memory slots — caller (`run_node` connection loop) constructs both as typed values from authenticated session state. | Pass 2 principle: internal-variables-bind-as-typed-references. |
| Q5.2 | `handle_federation_incoming(..., home_node_id: String, ...)` owned-String parameter? | Retype to `NodeXgid` (owned). The function consumes the value across awaits + passes it deep into spawned tasks; owned avoids lifetime gymnastics across spawn boundaries. | §4.2 sibling-shape rule table: async-spawned task captures force owned (v1.2 sub-rule). |
| Q5.3 | `handle_identity_msg(..., authenticated_id: &str, home_node_id: &str, ...)` parameters? | Retype to `&IdentityXgid`, `&NodeXgid`. Sibling-shape to Q5.1. | Pass 2 principle. |
| Q5.4 | `handle_identity_replicate_msg` — `IdentityReplicateMessage::Replicate { identity_id, ... }` destructured wire-format field? | **Stays String at destructure** (per §4.3 format-boundary preservation). The function projects to `&IdentityXgid` at the typed-call boundary feeding `IdentityRegistry::get/contains` (Pass 2 retyped these methods to take `&IdentityXgid` via Borrow<str>). | §4.3 format-boundary preservation (wire). |
| Q5.5 | `push_identity_to_peers(record: IdentityRecord, ...)` — internal `peer_urls.iter().map(...)` site? | `peer_urls` is already `HashMap<NodeXgid, String>` (Pass 2 §4.1 Q2.6). The `.iter()` yields typed keys natively; no retype here. | Inherited from Pass 2 retype. |
| Q5.6 | `build_node_state(..., node_id: &str, ...)` parameter? | Retype to `&NodeXgid`. Caller constructs from `home_node_id` (in-memory). | Pass 2 principle. |
| Q5.7 | `build_node_state` internal `rt.stores.get(&space.space_id)` site? | Post-Surface-#1 retype, `space.space_id: SpaceXgid` and `rt.stores: HashMap<SpaceXgid, _>`. The `.get(&space.space_id)` works natively (typed lookup). | Inherited from Surface #1. |
| Q5.8 | `space_file_name(space_id: &str) -> String` — filesystem-path generator? | **Stays `&str`** per §4.3 format-boundary preservation (persistence layer). Function computes a String filesystem path from String input; both ends of the function operate at the persistence-format layer. Callers project `SpaceXgid` → `&str` at the call boundary via Borrow<str>. | §4.3 persistence-format boundary. |
| Q5.9 | `persist_event(spaces_dir: &Path, space_id: &str, event: &Event)` parameter? | **Stays `&str`** per §4.3 persistence-format boundary. Function touches the filesystem; on-disk JSON store treats space_id as path-component String. The function never operates on space_id as a typed identifier; only writes/reads to disk. | §4.3 persistence-format boundary. |
| Q5.10 | `load_space_local_metadata` return type `HashMap<String, SpaceLocalMetadata>`? | **Stays `HashMap<String, _>`** at the function signature (persistence-format). Callers project to `HashMap<SpaceXgid, _>` at the typed-call boundary — sibling-shape to wire-format `BTreeMap<String, String>` peer_tips at §4.3. | §4.3 persistence-format boundary. |
| Q5.11 | `save_space_local_metadata` parameter type? | Sibling to Q5.10 — stays `&HashMap<String, _>`. | §4.3 persistence-format boundary. |
| Q5.12 | Disk-format ↔ in-memory boundary projection sites? | Two sites: (a) `run_node` bootstrap — `load_space_local_metadata` returns `HashMap<String, _>`, projection happens at the insert site into `NodeRuntime.space_local_metadata: HashMap<SpaceXgid, _>` (Surface #1 retype); (b) `run_node` 5s state-writer — projection at the save call site from `HashMap<SpaceXgid, _>` → `HashMap<String, _>` via `.iter().map(\|(k,v)\| (k.as_str().to_string(), v.clone())).collect()`. | Documented in implementation runbook for transparency. |
| Q5.13 | `replay_spaces_from_dir(runtime: &mut NodeRuntime, spaces_dir: &Path) -> usize` — interior identifier slots? | Reads disk-format JSON event-store files (persistence-format Strings); calls `runtime.ingest_event(event)` post-Pass-2. The event's typed fields are reconstructed at JSON deserialisation via serde. No new identifier-slot parameters; function-internal projection happens at JSON parse boundary. | §4.3 persistence-format boundary; no new Pass 3 work here. |
| Q5.14 | `run_federation_session_post_handshake` parameter set — what retypes? | **Per-parameter retype matrix.** Rewritten at v1.3 per J-133 D-078 verification: v1.0/v1.1/v1.2 attributed `peer_node_id: String` at line 1165 to `OutboundMsg` as a struct field — production: `OutboundMsg` is an enum at xgen-node/src/fanout.rs:31 with variants `Event(Event)` / `HistoryBatch { events: Vec<Event> }` / `SyncComplete { since, new_tip, continue_from }` and has NO `peer_node_id` field; line 1165 is a parameter on `pub(crate) async fn run_federation_session_post_handshake` (line 1152, the bilateral federation session driver consumed by both Initiator + Receiver roles post-handshake). Production parameter set (13 params; 7 identifier-shaped): `home_node_id: String` → **owned `NodeXgid`** (§4.2 v1.2 row 3 async-spawned-captures forced-owned; spawned-task body consumes the value across await + spawn boundaries); `peer_node_id: String` → **owned `NodeXgid`** (same rationale); `session_id: String` → **stays `String`** (descriptive, not identifier-flavoured); `neg_version: String` → **stays `String`** (descriptive); `serial: String` → **stays `String`** (descriptive — negotiated serialisation name); `peer_shared_spaces: Vec<String>` → **`Vec<SpaceXgid>`** (in-memory typed vec; sibling-shape to Q6.3 `shared_spaces: Vec<SpaceXgid>` at reconnect.rs); `peer_tips: BTreeMap<String, String>` → **stays `BTreeMap<String, String>`** (§4.3 wire-format boundary; wire-derived from TransportMessage Hello/Capabilities; sibling-shape to Q3.2). Non-identifier parameters out of Pass-3 scope: `conn`, `our_role`, `runtime`, `client_senders`, `federation_peer_senders`, `federation_registry`, `federation_registry_path`, `node_keypair`, `spaces_dir`, `identities_path`, `local_mode`, `peer_url` (URL descriptive per §5.4). | **v1.3 production-true correction**: v1.0/v1.1/v1.2 OutboundMsg attribution was structurally wrong (enum-not-struct + no peer_node_id field). The intended retype target was always the function parameter at line 1165, not the OutboundMsg type. v1.3 captures the full parameter matrix per §4.2 v1.2 + §4.3 + §5.4 rule application; closes the §2.5 sub-region enumeration omission of this function alongside (extended at v1.3 prose above). |
| Q5.15 | `ConnectedClientInfo.identity_id: String` (line 173) — admin-state struct field? | Retype to `IdentityXgid`. xgen-node-internal admin-state struct; feeds `build_node_state` + admin display. **Defer-question for Pass 4**: if M6 (new) Block 4 admin verb work surfaces this struct via a pipe-server export, the format-boundary applies at that export site, not here. | Pass 2 principle: struct fields owned typed. |
| Q5.16 | Trace events `peer_node_id = %peer`, `identity_id = ...` across handlers? | Stay as-is. Display impls project to inner string. | Pass 5 trace-field audit. |
| Q5.17 | What defers to Pass 4? | Nothing new from this surface. | Out-of-scope. |
| Q5.18 | What defers to Pass 5? | Trace fields; Debug/Display impls (none load-bearing surfaced). | Out-of-scope. |
| Q5.19 | Pass 3 inline markers? | The dispatch_event peer_node_id Pass 3 marker already at xgen-core/src/node/runtime.rs (Surface #2). app.rs has no separate Pass 3 markers from Pass 2; the inheritance is via Surface #1 (HashMap keys) + #2 (peer_node_id) + Pass 2 retype reach. | No new markers. |

**Surface #5 close**: Twelve identifier slots in-memory retype; four slots at persistence-format boundary stay String per §4.3 extension; one struct field (`ConnectedClientInfo.identity_id`) flagged for M6 (new) coordination at admin-export-time. No new architectural decisions beyond extending §4.3 to cover persistence-format. No layered-B3 surface. Governing principle inherits unchanged.

### §2.6 Surface #6 — Reconnect scheduler identifiers (xgen-node/src/reconnect.rs)

**What this is.** The reconnect scheduler logic at its own module `xgen-node/src/reconnect.rs` (confirmed at v1.1 walk via `Filesystem:list_directory`; not an in-app region as v1.0 §2.6 speculated). Re-establishes federation sessions after transient disconnects. Identifier slots: peer node_ids in the schedule table, space_ids if scheduling is per-space, attempt-count + backoff-state structures keyed by peer node_id.

**Orthogonal-ish to #3-#5.** Reconnect scheduler logic interacts with federation_session.rs (it spawns new sessions) and with the FederationRegistry (it consults peer_urls retyped at Pass 2's Surface #2); it does not interact with fanout.rs or with the per-space HashMap surface directly. Walking it last in the xgen-node sequence keeps the design walk focused per surface.

**Scope-boundary check at Q-table walk.** The reconnect scheduler may have its own per-peer state (last-attempt timestamp, backoff multiplier) that is not identifier-shaped and stays String/other primitive at Pass 3. Q-table confirms which slots are identifier slots in scope vs which are scheduler-state slots out of scope.

**Cleanest of the seven.** Surface #6 is pure inheritance from Surfaces #1+#2+#3+#4. No new architectural decisions. No format-boundary slots (reconnect.rs is purely in-memory orchestration; wire-format work happens inside the federation_session.rs it spawns, not at the scheduler layer).

**Q-table (locked at v1.2).**

| Q | Question | Answer | Reasoning |
|---|---|---|---|
| Q6.1 | `spawn_reconnect_scheduler(..., home_node_id: String, ...)` parameter? | Retype to `NodeXgid` (owned). Spawned task captures across runtime lifetime. | §4.2 sibling-shape rule: async-spawned task captures force owned (v1.2 sub-rule). |
| Q6.2 | `scheduler_tick(..., home_node_id: String, self_url: String, attempt_cursor: AttemptCursor, ...)` parameters? | `home_node_id` retype to `NodeXgid`; `self_url` stays `String` (URL is descriptive, not identifier-flavoured); `attempt_cursor` see Q6.4. | Pass 2 principle for identifier; §5.4 deliberate descriptive-string for URL. |
| Q6.3 | `attempt_reconnect(..., home_node_id, peer_node_id, peer_url, shared_spaces, ...)` parameters? | `home_node_id: NodeXgid` (owned, spawned-captures); `peer_node_id: NodeXgid` (owned, spawned-captures); `peer_url: String` (descriptive); `shared_spaces: Vec<SpaceXgid>` (in-memory typed vec). | Mixed: identifier slots retype owned; URL stays String; Vec of identifiers retypes per Pass 2 principle. |
| Q6.4 | `AttemptCursor = Arc<Mutex<HashMap<String, u32>>>` type alias? | Retype to `Arc<Mutex<HashMap<NodeXgid, u32>>>`. HashMap key is peer node_id; xgen-node-internal type alias never crosses out. | Pass 2 principle: HashMap keys always owned typed. |
| Q6.5 | `cursor.remove(&peer_node_id)` site at line 325? | Works natively post-Q6.4 retype (typed key removal). | Inheritance. |
| Q6.6 | `session.peer_node_id != peer_node_id` comparison at line 311? | Works natively post-Surface-#3 retype. `Session.peer_node_id: NodeXgid` (Surface #3 — federation_session.rs struct field); `peer_node_id` parameter typed via Q6.3. PartialEq via inner Xgid. | Inheritance from Surface #3. |
| Q6.7 | `rel.shared_spaces.clone()` at line 140 site? | Post-Pass-2 + Surface #1, `rel.shared_spaces: Vec<SpaceXgid>` (FederationRelationship struct field retype). Q6.3's `Vec<SpaceXgid>` parameter consumes natively. | Inheritance. |
| Q6.8 | Trace events `peer_node_id = %peer_node_id`, `peer_url = %peer_url`, `actual_peer_node_id = %session.peer_node_id`? | Stay as-is. Display impls project to inner string. | Pass 5 trace-field audit. |
| Q6.9 | Scheduler-state slots NOT in scope? | `peer.next_reconnect_attempt` (Utc DateTime), backoff multiplier (u32), `SCHEDULER_TICK_SECONDS` + `BACKOFF_LADDER_MINUTES` constants — none identifier-flavoured. | Out-of-scope. |
| Q6.10 | What defers to Pass 4? | Nothing — reconnect.rs is xgen-node-internal. | Out-of-scope. |
| Q6.11 | What defers to Pass 5? | Trace fields; URL handling at `connect_url(&peer_url)` (URL parsing layer is its own concern). | Out-of-scope. |

**Surface #6 close**: Six identifier slots retype owned (three async-spawned function parameters + HashMap key in AttemptCursor + two internal uses) all via Pass 2 principle or §4.2 spawned-captures sub-rule. URL slot stays String per §5.4 deliberate descriptive-string. No new architectural decisions. No layered-B3 surface. Cleanest inheritance pattern across the seven surfaces.

### §2.7 Surface #7 — Appendix D doc retypes (docs/xgen_appendix_d_en.md)

**What this is.** The Node-side storage and privacy appendix at `docs/xgen_appendix_d_en.md` (~21 KB, 319 lines). This is the Pass 3 doc-retype surface analogous to Pass 1's Appendix C + Appendix I (data structure schemas) and what Pass 4 will do for Appendix F (client-side) per the Pass 1 doc-sweep classification at J-095.

**Hit count surfaced at design walk: 4 total.** All four identifier-shaped hits live in markdown schema tables (lines 65, 95, 111) describing **stored field semantics**, not Rust type signatures. Appendix D is a **prose document** about Node-side storage architecture written for institutional evaluators, NOT a schema specification. Specifically: line 65 `identity_id` in Identity Record table; line 95 `event_id` in Event Record table; line 96 `sender` in Event Record table; line 111 `peer_node_id` in Federation Record table.

**This shape difference matters for Pass 3 scope sizing.** Pass 1's Appendix C + Appendix I retypes were schema documents with concrete type signatures; touching them required updating Rust code examples + field-type declarations. Appendix D is prose + descriptive tables saying *"this is what a Node stores"* with field names + descriptions. The Pass 3 in-memory type retypes do not affect on-disk persistence shape per §4.3 format-boundary preservation; the field names in tables remain accurate without Rust-type-signature updates.

**Independent of #1-#6.** Doc retypes do not consume types from the code surfaces; the Appendix D content describes Node-side storage shapes, persistence boundaries, privacy guarantees — the retype updates the textual XGID-vs-String descriptions to match the post-Pass-3 code surface. Walking #7 last in the design sequence keeps the dependency-order narrative clean.

**Q-table (locked at v1.2).**

| Q | Question | Answer | Reasoning |
|---|---|---|---|
| Q7.1 | What is Appendix D structurally? | A prose document (~21 KB, 319 lines) describing Node-side storage architecture for institutional evaluators. Not a schema spec; not a code reference. The 4 identifier-shaped hits are all in markdown tables describing record-shape semantics, not Rust type signatures. | Hit count surfaced at design walk; original Phase 2 doc-tree sweep at J-095 classified Appendix D "in-scope" but did not anticipate the prose-vs-schema shape difference. |
| Q7.2 | Required retypes for Pass 3? | **Minimal — possibly zero substantive code-tier retypes.** The field names in tables (`identity_id`, `event_id`, `peer_node_id`) describe **stored field semantics**, which haven't changed at the protocol layer; the on-disk storage IS String per §4.3 format-boundary preservation. The Pass 3 type retypes are in-memory Rust slots; on-disk persistence stays String. | §4.3 format-boundary preservation applies at Appendix D scope. |
| Q7.3 | What MIGHT need updating? | (a) **Possibly nothing.** Pass 1's Appendix C + Appendix I retypes were schema documents (concrete type signatures); Appendix D is prose. (b) If anywhere — narrative passages that explain in-memory Node behaviour vs persistence; cross-references to xgen-common type names if any. Quick verification scan needed at runbook authoring. | Honest framing per D-065 — design phase doesn't lock content changes that don't exist yet. |
| Q7.4 | Header chain entry update? | **Yes.** Header `Last updated` chain entry per standard document-header discipline at any Pass 3 touch. | Mandatory by header structure. |
| Q7.5 | Should Surface #7 be folded into Pass 3 implementation runbook? | **Yes, as a minimal touch commit OR rolled into doc-pass Commit 1.** If runbook authoring confirms zero substantive content edits needed, Appendix D gets only a header chain entry + version bump — single-file edit folded into the doc-pass commit. If substantive edits surface, Appendix D gets its own commit per Pass 1 precedent. | Surface-driven sizing per D-069 + honest framing. |
| Q7.6 | What about future Appendix D revision? | **Per D-071 own-arc.** Appendix D substantive revision (e.g., updating Phase 2 storage security section, adding M6 (new) admin-write-path documentation, expanding the right-to-erasure problem section) is its own future audit-design-impl arc, not Pass 3 scope. | Out-of-scope. |
| Q7.7 | What defers to Pass 4? | Nothing — Appendix D is Node-side; Pass 4 owns Appendix F (client-side). | Out-of-scope. |
| Q7.8 | What defers to Pass 5? | Nothing — Appendix D doesn't contain trace-event field documentation. | Out-of-scope. |

**Surface #7 close**: Minimal-to-no substantive content retypes; header-chain entry mandatory at any touch; sized for fold-into-Commit-1 doc-pass. Honest framing per D-065: J-095's Phase 2 doc-tree sweep classified Appendix D "in-scope for Pass 3" correctly in intent — but the actual content shape (prose-not-schema) means Pass 3 work is minimal at most. **Discipline-note candidate at §7.5**: doc-tree sweep classification should surface hit-count + content-shape per doc; candidate D-NNN watch sibling-shape to D-078 production-grounded test enumeration but at doc-tree classification layer.

### §2.8 Out-of-scope enumeration (honest broadening)

Surfaces NOT in Pass 3 scope, named explicitly per honest-broadening discipline:

- **Pass 4 (xgen-client + AI control docs).** ops:: verb signatures, AiBehavior trait, session state, batch dispatcher, CLI dispatcher, AI service, Tauri commands, Appendix F client-side sections, `docs/xgen_aicontrol_implementation.md` reply schemas, Ch6 §6.15 client-side spec. **Update at v1.1**: ClientSenders + FederationPeerSenders confirmed Pass 3 scope (xgen-node-internal); not Pass 4 as the v1.0 framing might have suggested.
- **Pass 5 (test fixtures + trace fields + Debug/Display + workspace test-count restoration).** Trace event field types in xgen-node modules, log line formatters at Appendix G, Debug/Display impls, test fixture builders, integration test helpers, `cargo build --workspace` restoration. **Update at v1.1**: every trace-event field formatter encountered at Surfaces #3 + #4 walk explicitly flagged Pass 5. **Update at v1.2**: Surface #5 + #6 trace-event fields (peer_node_id, identity_id at handlers; peer_node_id, peer_url, actual_peer_node_id at reconnect scheduler) all confirmed Pass 5 scope.
- **M6 (new) Node admin write path (Block 4 verb-by-verb walks).** The pipe-server admin verbs at `docs/xgen_node_admin_ops_design.md` §6 (~35 verbs across 7 categories). If M6 (new) ships before Pass 3 implementation, those verbs land String-typed and Pass 3 retypes them at #5; if Pass 3 ships first, M6 (new) verbs land Pass-3-typed. Coordination is a runbook-phase question, not a Pass-3-design question.
- **Sibling event constructors beyond the Pass 1 narrow scope.** `state.federation_add`, `membership.*`, `message.*` event constructors stay at their post-Pass-1 + post-Pass-2 type discipline; Pass 3 does not re-audit constructor shape.
- **Wire-format canonicalisation + persistence-format on-disk structure.** Wire format is bytes, not Rust types; the in-memory Rust type retype does not change wire-format canonical encoding. Persistence format (on-disk JSON event stores + filesystem path generation) similarly preserves String at the format-boundary per §4.3 consolidated framing. **Update at v1.2**: §4.3 consolidation formalises this — format-boundary slots (wire OR persistence) stay String at function boundaries; conversion happens at the format/in-memory boundary.
- **TransportMessage parser layer + on-disk JSON parser layer.** The deserialisation layers that produce format-derived `&str` / `String` slots (e.g., `peer_tips: &BTreeMap<String, String>` in `stream_federation_delta`, `since: &str` in `collect_sync_history`, `HashMap<String, SpaceLocalMetadata>` at `load_space_local_metadata`). Touching either layer would push the typed-XGID boundary higher in the stack; Pass 3 deliberately preserves the current format boundaries per §4.3.

---

## §3 Governing principle (locked at v1.2 — confirmed inherited from Pass 2 unchanged across seven-surface walk)

Inherited from Pass 2 §3 unchanged after the full seven-surface walk:

> **Identifier slots retype to typed XGIDs; descriptive-string slots stay String; internal variables bind as typed references; &str projection happens at the call-site boundary via Borrow<str> (Pass 1's additive API at Commit 4 implementation-kickoff lock); no Deref<Target = str> shortcuts.**

The seven-surface walk confirmed the principle applies uniformly to xgen-node's full module surface — federation-transport (#3) + fanout (#4) + top-level handlers + persistence helpers (#5) + reconnect scheduler (#6) + doc retypes (#7) + the foundational xgen-core surfaces deferred from Pass 2 (#1 + #2). The format-boundary that emerged at Surfaces #3 + #4 + #5 did NOT modify the principle itself — the principle's "&str projection via Borrow<str>" clause already anticipates that format-boundary slots stay String by construction (format IS bytes-and-strings, not typed Rust newtypes); §4.3 makes this explicit as an architectural decision at v1.2 consolidated framing, not a principle amendment.

**Zero principle wrinkles surfaced across the seven-surface walk.** Three Pass-arc instances of inheritance-unchanged (Pass 1 implicit at runbook; Pass 2 explicit at design close; Pass 3 explicit at design close) make the governing principle's stability durable at Pass-arc layer.

---

## §4 Architectural decisions (locked at v1.2)

### §4.1 Six per-space HashMap keys retype shape

**Lock**: retype field types AND xgen-core-internal helper signatures AND public-API parameters atomically in Pass 3. All six fields on `NodeRuntime` (`spaces`, `stores`, `graphs`, `pending`, `dm_proposals`, `space_local_metadata`) plus the internal/public helper functions (`drain_pending_*`, `all_events`, `dag_tips`) cross the type discipline together. Sibling-shape to Pass 1's atomic data-structure-plus-test-fixture commit pattern.

**Reasoning**: retyping field types without the helper signatures would force every xgen-node call site to project `&str` → `&SpaceXgid` at every call. Retyping helper signatures without field types would require the helpers to internally project SpaceXgid → String at every HashMap access. The atomic retype eliminates both projection costs and keeps Pass 2's principle (internal variables bind as typed references) intact.

**Promotion candidate D-NNN-γ from J-126** ("small-cardinality vs large-cardinality identifier-keyed maps retype in different Passes per call-site density"): Pass 3 instantiates this principle with the six per-space HashMaps — large-cardinality maps deferred from Pass 2, retyped in the Pass that touches their primary call-site crate. **Two instances now** (Pass 2 §4.1 Q2.8.c flagging + Pass 3 §4.1 instantiation). Three-instance threshold opens at a future Pass-arc milestone if a sibling instance fires. Stays flagged-not-promoted at this design close.

### §4.2 `dispatch_event` `Option<&NodeXgid>` borrowed boundary + sibling-shape rule table (extended at v1.2)

**Lock**: `peer_node_id: Option<&NodeXgid>` (borrowed). Caller owns the NodeXgid (federation session state, peer registry); function reads only.

**Reasoning**: parameters favour borrowed when caller owns the value and function only reads. Q2.3's walk confirmed every internal use of `peer_node_id` inside `dispatch_event` is either an `is_some()` check or a value comparison/clone; none stores the parameter. Owned would force unnecessary clones at every call site (hot path: federation push, dispatch).

**Sibling-shape rule (extended at v1.2 with async-spawned-task-captures sub-rule)**:

| Position | Default | Forced exception |
|---|---|---|
| Parameter, caller-owned value, function reads | Borrowed | If function stores (HashMap insert, struct field assign), must own |
| Parameter, function constructs/computes | Owned (no input to borrow from) | — |
| **Parameter, async-spawned task captures (v1.2)** | **Owned** | **Forced — `'static` bound on `tokio::spawn` closures requires owned values to cross the spawn boundary. Instantiated at Surface #5 (`handle_federation_incoming` `home_node_id: String`) and Surface #6 (`spawn_reconnect_scheduler` + `scheduler_tick` + `attempt_reconnect` all-owned identifier params).** |
| Return, value exists in input at same flavour | Could borrow (lifetime permitting); owned for ergonomics | — |
| Return, value must be constructed (flavour change, type wrap) | Owned | Forced — see §4.4 |
| HashMap key, struct field | Always owned | — |

### §4.3 Format-boundary preservation (wire OR persistence) — consolidated at v1.2

**Lock**: function signatures that consume format-derived identifiers keep `String` / `&str` / `Option<String>` / `HashMap<String, _>` slots as-is at the format boundary. Conversion to typed XGIDs happens at the format/in-memory boundary (one projection per direction). **This applies uniformly to wire-format boundaries (TransportMessage deserialisation) AND persistence-format boundaries (filesystem path generation, on-disk JSON deserialisation).**

**Affected slots at v1.2 walk**:

**Wire-format (Surfaces #3 + #4)**:
- `stream_federation_delta`'s `peer_tips: &BTreeMap<String, String>` (deserialised from TransportMessage::Hello/Capabilities)
- `collect_sync_history`'s `since: &str` (deserialised from TransportMessage::SyncRequest)
- `collect_sync_history`'s return `Option<String>` for `continue_from` (serialised to TransportMessage::SyncComplete::continue_from)
- (general) any `TransportMessage::*` variant field carrying identifier-shaped strings

**Persistence-format (Surface #5, NEW at v1.2)**:
- `space_file_name(space_id: &str) -> String` — filesystem path generation
- `persist_event(spaces_dir: &Path, space_id: &str, ...)` — on-disk JSON event store writer
- `load_space_local_metadata(...) -> HashMap<String, SpaceLocalMetadata>` — on-disk JSON HashMap deserialisation
- `save_space_local_metadata(metadata: &HashMap<String, SpaceLocalMetadata>, ...)` — on-disk JSON HashMap serialisation
- `replay_spaces_from_dir` interior — disk-format JSON event-store file iteration
- `IdentityReplicateMessage::Replicate { identity_id, ... }` destructured wire-message field at Surface #5 Q5.4

**Reasoning**:
1. **Format IS bytes-and-strings.** Per Appendix J (canonical wire encoding) AND the on-disk JSON event store format, both formats are byte-serialized with String-typed identifier slots. Typed Rust newtypes are a memory-layer concern; serialising them re-projects to inner String regardless. Putting typed XGIDs in format-boundary function signatures would be cosmetic, not semantic — every serialise call would project back to String anyway.
2. **Single deserialisation boundary per format**: the TransportMessage parser AND the JSON event store parser (both out of Pass 3 scope per §2.8) are the single points where format → typed-Rust conversion makes sense. Pushing the boundary into format-aware function signatures would require the parsers to retype too, expanding Pass 3 scope to the transport-deserialisation + persistence-deserialisation layers.
3. **Pass-of-record honesty**: future work that consolidates format-aware typing (a hypothetical "Pass 6: format-aware type unification" covering both wire AND persistence layers) is a different audit-design-impl arc per D-071. **Flagged-not-promoted as candidate D-NNN-δ at this design close** (two instances at v1.2 walk: wire-format + persistence-format; three-instance threshold opens at Pass 4 if a client-side serialisation-format slot instantiates).
4. **Amend-in-place not split**: v1.1 §4.3 covered wire-format only; v1.2 consolidates wire AND persistence under one decision-surface rather than splitting into §4.3-wire + §4.7-persistence. One decision = one drift surface; sibling-shape to D-076 v1 → v1.1 amend-in-place reasoning where the second load-bearing property absorbed into the existing decision rather than splitting into D-076 + D-077. If a future Pass surfaces a third format-boundary layer (e.g., gRPC IPC at Tauri commands, AI-control-protocol over HTTP), the principle extends to cover it under the same §4.3 framing.

**Boundary projection sites (extended at v1.2)**:

**Wire-format**:
- `stream_federation_delta` per-Space loop: `peer_tips.get(space_id)` returns `Option<&String>`; projection happens at the typed-call boundary feeding `compute_federation_delta_for_space(&SpaceXgid, Option<&EventXgid>)`.
- `collect_sync_history`: WebSocket drain arm passes wire-format `&str` for `since`; internal projection to typed event_id reference happens at the function body's iteration over candidates.

**Persistence-format**:
- `run_node` bootstrap: `load_space_local_metadata` returns `HashMap<String, SpaceLocalMetadata>`; projection at the insert site into `NodeRuntime.space_local_metadata: HashMap<SpaceXgid, _>` via `.into_iter().map(\|(k,v)\| (SpaceXgid::from_xgid(Xgid::new(k)), v)).collect()`.
- `run_node` 5s state-writer task: projection at the save call site from `HashMap<SpaceXgid, _>` → `HashMap<String, _>` via `.iter().map(\|(k,v)\| (k.as_str().to_string(), v.clone())).collect()`.
- `persist_event` callers: `space_id.as_str()` projection at the call boundary (Borrow<str> handles this implicitly when callers pass `&SpaceXgid`).
- `IdentityReplicateMessage::Replicate` destructure at Q5.4: `identity_id` stays String at the destructure; projection to `&IdentityXgid` happens at the typed-call boundary feeding `IdentityRegistry::get(...)`.

### §4.4 `event_space_id` return shape forced-owned

**Lock**: `event_space_id(event: &Event) -> Option<SpaceXgid>` (owned).

**Reasoning** (recorded explicitly because it instantiates the general rule):
1. **Mixed-shape branch problem**: the state-create branch (`event.space_id.is_empty()`) returns `event.event_id.clone()` — but `event.event_id` is `Option<EventXgid>`, a different newtype flavour from `SpaceXgid`. The branch must **construct** a new `SpaceXgid` from the event_id's inner Xgid. Construction is not borrowing.
2. **Mixed-shape avoidance options rejected**:
   - `Cow<SpaceXgid>` would model the borrow/own distinction but adds complexity at every call site for marginal ergonomic gain.
   - Returning `Option<&Xgid>` (the base type, dropping flavour) breaks Pass 1's flavour discipline.
   - Returning `Option<EventXgid>` from the state-create branch + `Option<&SpaceXgid>` from the normal branch can't compile (return shapes must agree).
3. **Forced**: owned `SpaceXgid` is the only shape that satisfies the flavour-discipline + branch-coherence constraints.

**General rule recorded inline** (for future Pass walks):

> A return type can be borrowed only if **every branch** can borrow from input at the same flavour. If any branch must construct a new value or change flavour, the return type is forced to owned.

### §4.5 `ClientSenders` + `FederationPeerSenders` retype scope

**Lock**: both retype at Pass 3, not deferred to Pass 4.

**Reasoning**: ClientSenders is a `type` alias declared in `xgen-node/src/fanout.rs`; the senders are constructed inside xgen-node (WebSocket connection handler at `app.rs::handle_connection`) and consumed inside xgen-node (drain loop at `app.rs`). **The senders never cross to xgen-client** — xgen-client connects via WebSocket and receives `TransportMessage` instances, not `mpsc::Sender` references. The "xgen-client-facing" framing of v1.0 was incorrect; the v1.1 walk corrected it. FederationPeerSenders is structurally analogous (xgen-node-internal, never crosses).

### §4.6 Topological-sort `&str` slot at fanout.rs:193

**Lock**: no separate retype needed at Pass 3.

**Reasoning**: the topo-sort J-097 design lock referenced a slot at fanout.rs:193 ("v1 ships with &str sort; Pass 3 retypes to EventXgid when xgen-node-side dispatch widens"). The actual code at v1.1 walk: `events.sort_by(\|a, b\| a.event_id.cmp(&b.event_id))` where `event_id: Option<EventXgid>` post-Pass-1. Pass 1 Commit 3 retyped `event.event_id` from `Option<String>` to `Option<EventXgid>`; `Option<EventXgid>` implements `Ord` through inner Xgid's String Ord. The `.cmp()` call works natively. No separate identifier slot exists to retype; the J-097 lock's anticipated retype happened automatically at Pass 1 Commit 3.

**Honest framing per D-065**: the J-097 lock anticipated a retype site that turned out to be already covered. Recording explicitly so the next sibling milestone author doesn't waste a walk session searching for a non-existent slot.

---

## §5 Honest broadening + deferred surfaces (locked at v1.2)

### §5.1 What defers to Pass 4

- **No new surface from any of the seven Pass 3 surface walks.** ClientSenders + FederationPeerSenders confirmed Pass 3 scope (§4.5); event_space_id confirmed Pass 3 scope (xgen-node-internal); per-space HashMap keys confirmed Pass 3 scope; app.rs admin-state struct fields confirmed Pass 3 scope (with M6 (new) coordination flag at Q5.15).
- **Standing Pass 4 scope** (from v1.0 §2.8): ops:: verb signatures, AiBehavior trait, AiPacingTracker, session state, AI service, Tauri commands, Appendix F + AI control doc + Ch6 §6.15 client-side spec.

### §5.2 What defers to Pass 5

- **Trace event field types** at federation_session.rs (`peer_node_id`, `local_node_id`, `space_id`, `event_id`, `client_id`), at app.rs handlers (`peer_node_id`, `identity_id`), at reconnect.rs (`peer_node_id`, `peer_url`, `actual_peer_node_id`, `expected_peer_node_id`, `session_id`) — all confirmed Pass 5 scope. The function-parameter slots retype at Pass 3; the trace-field formatters (`%peer`, `%local_node_id`, etc.) work through Display impls that already project to inner string; the type-level audit of trace fields is its own Pass 5 walk.
- **Debug/Display impls** on NodeRuntime, FanoutRequest, OutboundMsg — none surfaced as load-bearing at any of the seven Pass 3 surface walks; flagged for Pass 5 trace audit.

### §5.3 What defers to D-071 own-arc per audit-precedes-dependent-design

- **TransportMessage parser layer retype** (wire-format String → typed XGID at deserialisation). Could become "Pass 6: format-aware type unification" if dependent work surfaces it; flagged-not-promoted as candidate D-NNN-δ at this design close per §4.3 consolidated framing.
- **On-disk JSON parser layer retype** (persistence-format String → typed XGID at deserialisation). Same future-arc framing as wire-format under §4.3 consolidated.

### §5.4 What stays at v1 forever (deliberate descriptive-string slots)

- `session_id`, `negotiated_version`, `negotiated_serialisation` at `stream_federation_delta` — wire-format descriptive strings, not identifier-flavoured.
- `display_name` and similar at IdentityRecord — descriptive, not identifier.
- Wire-format pagination cursors (`since`, `continue_from`) — descriptive of position in a sequence, not identifier flavours per se (they happen to be event_id Strings but the wire-format role is positional).
- `peer_url`, `self_url` at reconnect.rs — URLs are descriptive infrastructure addresses, not identifier flavours.
- `endpoint`, `mode`, `started_at` at `build_node_state` parameters — admin-state descriptive strings.

### §5.5 Layered-B3 expected answer — confirmed null at full seven-surface scope

**Pass 3 confirmation: null** (sibling to Pass 1 J-122 finding + Pass 2 J-126 finding). Pass 3 scope is identifier-slot shape at the xgen-node binary surface, not algorithm validation or invariant encoding. The B3-shape surfaces (layered encodings of the same invariant) historically emerged in algorithm-bearing scopes (topo-sort J-101 found two, persistence-amendment J-108 found one); Pass 1 + Pass 2 + Pass 3 scopes found zero.

**Three project-Pass-arc instances of expected-null finding (Pass 1 + Pass 2 + Pass 3) now durable** sibling-shape to D-077/D-078 three-instance promotion-threshold framing. Pattern is now established at the Pass-arc layer: identifier-slot retype scopes do not surface layered-B3 because the projection mechanism (`Borrow<str>`) handles type-projection at boundaries uniformly without forcing secondary encodings of the same invariant. The candidate "Pass-arc scopes inherit B3-null expectation" sub-principle stays flagged-not-promoted per D-069 (the inheritance is implicit in the principle inheritance from Pass 2 §3, not a separate decision; if a future Pass surfaces a layered-B3 finding, the sub-principle's surface justifies explicit promotion at that point).

**No layered surface emerged at any of the seven surfaces walked.** The `Borrow<str>` projection inherited from Pass 1 handles type-projection at boundaries uniformly; no secondary encoding of the same invariant surfaced at Surfaces #1 + #2 + #3 + #4 + #5 + #6 + #7.

---

## §6 Historical / future-pointer entries

### §6.1 Pass 3 design close — Shape α (pointer-style, sibling to Pass 2 §6.7)

**Design close at J-127 (2026-05-27).** Full seven-surface walk closed across two design sessions: Surfaces #1-#4 at v1.1 (2026-05-27 morning); Surfaces #5-#7 + §4.3 consolidation + §6 + §7 fills at v1.2 (2026-05-27 afternoon). Zero re-walks fired across the walk. Single governing principle (§3) confirmed inherited from Pass 2 unchanged. Six architectural decisions (§4.1-§4.6) locked. Three candidate D-NNNs flagged-not-promoted per D-069 (D-NNN-γ + D-NNN-δ + D-NNN-ε). Layered-B3 confirmed null at full scope (third Pass-arc instance, pattern durable).

**Implementation kickoff target**: runbook authoring at `tasks/XGID_RETROFIT_PASS_3_IMPL.md` in a fresh session per trilogy + Pass-1 + Pass-2 precedent. **Implementation J-NNN milestone-close placeholder** to be frozen at runbook close per J-108 codification: §6.1 timestamp will be frozen retroactively at milestone-close commit per Pass 2 §6.7 pattern.

### §6.2 v1.2 → v1.3 amendment provenance (Path-α Track 1 at Joe-lock checkpoint #2)

Design doc amended at J-133 (2026-05-28) by Clair (Chat Claude in implementation role) with Joe as a within-milestone Track 1 canonical-record amendment at Pass 3 Commit 2 prep. Triggered at Joe-lock checkpoint #2 D-078 verification of design doc §2 against production code: Clair surfaced two parameter-attribution drifts + one structural omission in the seven-surface enumeration.

**Drift #1 — §2.3 Q3.6 non-existent parameter on `apply_federation_push`.** Production has 5 parameters at xgen-node/src/federation_session.rs:202-208 (`event, origin, runtime, federation_peer_senders, local_node_id`). Design doc claimed a 6th parameter `peer_node_id: &str` (destination peer); no such parameter exists. The typed-destination-peer concern flows internally via `peer_id` loop variable bound from `Vec<String>` (`federation_nodes` snapshot from `runtime.spaces.get(&space_id).federation_nodes`) at federation_session.rs:261. Q3.6 rewritten at v1.3 to production-true: no parameter retype; internal-binding retype via Surface #1 Q1.1 extension target — `SpaceState.federation_nodes: Vec<String>` → `Vec<NodeXgid>` (sibling field retype landing in same Commit 2 atomic; Q1.1 enumeration extended at runbook §4 scope post-J-133).

**Drift #2 — §2.5 Q5.14 mis-attribution OutboundMsg vs run_federation_session_post_handshake.** Design doc claimed `OutboundMsg.peer_node_id: String` (line 1165) as a struct field; production: `OutboundMsg` is an enum at xgen-node/src/fanout.rs:31 with variants Event/HistoryBatch/SyncComplete and NO peer_node_id field. The line 1165 in app.rs is a `peer_node_id: String` parameter on `pub(crate) async fn run_federation_session_post_handshake` at line 1152 — the bilateral federation session driver. The design doc author likely saw the parameter at line 1165 while writing Q5.14 but mis-attributed it to OutboundMsg (which is referenced nearby). Q5.14 rewritten at v1.3 to per-parameter retype matrix for the function's 7 identifier-shaped slots.

**Drift #3 — §2.5 sub-region enumeration omission.** "Three sub-regions identified at grep" prose named only `process_inbound` + `run_node` under "Top-level orchestrators". `run_federation_session_post_handshake` sits in the same boundary (between WebSocket/federation transport and runtime per its doc comment) but was omitted. Sibling-shape to Drift #2 root cause — likely the same authoring miss. v1.3 extends the enumeration to add `run_federation_session_post_handshake` to "Top-level orchestrators".

**Candidate D-NNN-κ "design-doc Q-table-vs-production-code parameter-attribution discipline" opens at this checkpoint.** Two instances within Pass 3's own design doc (Drift #1 + Drift #2, both parameter-attribution shape). Flagged-not-promoted per D-069 at two instances — NOT promoted yet despite two, because both instances are within a single design doc at a single catch-event; D-069's three-instance threshold wants instances across distinct catch-events to establish durability. Third-instance promotion-watch opens at Pass 4 design walk (or earlier if a sibling fires at any design-doc-vs-production surface).

**D-NNN-ζ stays at one instance** — this checkpoint does NOT increment ζ (distinct surface layer: ζ catches runbook-vs-design-doc; κ catches design-doc-Q-table-vs-production-code; one rung deeper). ζ's promotion-watch stays at Pass 4 + Pass 5 runbook authoring as set at J-129.

**Root-cause family note** (recorded, not a candidate): ζ + κ + η ("claimed-atomic-file-count vs git-actually-shipped" at git-staging layer) + D-078 (production-grounded test enumeration) all share the shape "prose claims something the implementing layer silently doesn't honor," at four distinct implementing layers (runbook / design-doc Q-table / git-staging / test enumeration). If a fifth surfaces, consider a parent meta-discipline rather than continuing to spawn per-layer candidates — **consolidation question flagged for Pass 5**, sibling-shape to the §7 discipline-notes consolidation flag at runbook §7.10.

**Path α locked over Path β** (Clair latitude at surface-extraction; rejected on knowingly-incorrect-canonical-source = D-077 + D-078 anti-pattern grounds) **+ Path γ** (fold into Commit 2; rejected on atomic-discipline grounds; same reasoning as J-129 Path γ rejection).

**Two-file atomic** per D-074 (thirtieth instance) + Lock #3 per-commit cadence:
1. This design doc v1.2 → v1.3 (header chain + §2.3 Q3.6 rewrite + §2.5 sub-region prose extension + §2.5 Q5.14 rewrite + this §6.2 section).
2. `JOURNAL.md` J-133 body §-entry (substantive prospective-catch at new surface layer with new candidate opened; sibling-shape to J-129 body entry, contrast J-131 chain-only-then-no-op).

CLAUDE.md NOT amended (entry-point stays Commit 2; sibling-shape to J-132 single-file precedent + J-121 hygiene atom no-PLAY-touch). ROADMAP.md NOT amended (within-milestone canonical-record clear; sibling-shape to J-117 + J-130 no-ROADMAP-touch framing). DECISIONS.md NOT amended (D-NNN-κ flagged-not-promoted at two instances per D-069).

**"Honest longer work over fast shortcuts" count does NOT increment at J-133.** Per Joe-lock: this is a fresh catch at a layer never before audited in Pass 3, surfaced prospectively by the checkpoint mechanism working as designed — that's the discipline succeeding, not a mistake recurring. Sibling-shape to J-115/J-116 prospective catches, which did not increment the count. Recorded as "prospective catch, count unchanged at TWO inherited from J-129 + J-130."

**J-NNN numbering note**: this atom is allocated J-133 despite Joe's session-time instruction "Call it J-132" because J-132 was already committed at `3381ff1` for the runbook §3.2 amendment in the prior turn. Default-cautious allocation per Rule 6 — J-NNN numbers are monotonically increasing and don't get reused. Recorded as a discipline data point: the session-time instruction was authored from compressed context; the implementing layer (git history) reflects J-132 already used. Sibling-shape to the very root-cause family this amendment is closing.

### §6.3 v1.3 → v1.4 amendment provenance (Path-α in-place rewrite-correction of J-133's own Q3.6 v1.3)

Design doc amended at J-134 (2026-05-28) by Clair (Chat Claude in implementation role) with Joe as a within-milestone Track 1 canonical-record amendment at J-134 atom prep. Triggered when Clair's pre-authoring D-078 grep (per Joe's pre-load instruction "If production surprises you (field absent, already `Vec<NodeXgid>`, or a different shape), STOP and surface — do not author a Q-row that production doesn't honor") against `SpaceState.federation_nodes` at xgen-core/src/space/state.rs:132 found the field already typed as `Vec<NodeXgid>` — Pass 1 Commit 4 (`774fe9d`, 2026-05-26, J-122 close arc, 36+ hrs earlier). Inline comment at state.rs:423 reads "Pass 1 retypes federation_nodes to Vec<NodeXgid>; the peer derivation".

**Finding A (production state)**: `SpaceState.federation_nodes: Vec<NodeXgid>` already at xgen-core/src/space/state.rs:132. No Pass 3 field-type retype lands here. The Q1.11 row Joe locked at J-134 atom-shape ("SpaceState.federation_nodes: Vec<String> → Vec<NodeXgid>, stored-field-type-change surface") would itself be a design-doc Q-row that production doesn't honor — the exact failure mode the STOP-and-surface guards against. Q1.11 NOT added.

**Finding B (J-133's own Q3.6 v1.3 carries the same wrong assumption)**: the Q3.6 v1.3 rewrite shipped at J-133 (`7494346`) contains the sentence "The retype lands when `SpaceState.federation_nodes: Vec<String>` retypes to `Vec<NodeXgid>` — flagged as Surface #1 Q1.1 extension". That sentence was authored by me (Clair, acting in implementation role at J-133 amendment time) from inference against the xgen-node-side federation_session.rs:248 local-variable annotation (`let federation_nodes: Vec<String> = { ... s.federation_nodes.clone() ... }`) without greping the struct definition. The :248 annotation is a Pass-1-broken-by-Path-A xgen-node compile error (`: Vec<String>` against an actual `Vec<NodeXgid>` source) — Pass 3's actual work at that slot is to drop the annotation so type-inference accepts the typed source. Pass 3 work lives at Surface #3 inheritance-row, NOT Surface #1 field-type extension.

**Path α locked over Path B** ("amend Q3.6 from v1.3 framing closer to original Q3.6 as a 'no parameter exists' finding") **+ Path C** ("`git revert` of the Q3.6-rewrite portion of J-133 — heavy + linear-J-numbering complication; no correctness gain over A"). Path α — in-place rewrite-correction of Q3.6 v1.3 → v1.4 sibling-shape to J-132's runbook §3.2 amend-in-place precedent — locked.

**Third instance of D-NNN-κ at a fresh catch-event.** D-NNN-κ instances:
1. J-133 Drift #1 — Q3.6 v1.0/v1.1/v1.2 original (non-existent parameter on apply_federation_push). Catch-event: J-133 session-open D-078 verification.
2. J-133 Drift #2 — Q5.14 v1.0/v1.1/v1.2 original (OutboundMsg mis-attribution). Same catch-event as instance 1.
3. **J-134 Finding B — Q3.6 v1.3 rewrite (the J-133 amendment itself).** Distinct catch-event: J-134 atom prep D-078 grep against the struct definition.

Three instances across **two distinct catch-events** meets D-069 promotion threshold per sibling-shape to D-077 + D-078, both promoted at three instances across distinct catch-events. **D-NNN-κ PROMOTED to D-079** at this atom — "Design-doc Q-table grounded by symbol-definition grep" — see DECISIONS.md D-079.

**Sub-shape recorded in JOURNAL J-134 body** (not promoted as separate candidate; κ applied to itself): "fix-author re-instantiates the discipline-failure being fixed." The J-133 atom whose entire purpose was closing κ-drifts itself introduced a κ-drift because the amendment-author trusted a call-site annotation (federation_session.rs:248 `: Vec<String>`) over a struct-def grep. The discipline binds even when authoring a κ-fix. Sub-shape is evidence FOR κ's promotion (not a separate axis), recorded as the canonical cautionary instance in D-079's narrative.

**§6.2 stays as historical record.** §6.2 J-133-provenance prose at lines 397-401 carries the same wrong "Q1.1 extension target" framing — sibling-shape to runbook §3.2's pre-J-129 framing preserved at J-132 amendment + the J-129 HANDOFF kept as-is at COMPLETED v1.1 with its J-129 chain entry preserved (anti-tempfile-deletion-of-decision-records discipline). §6.2 is the authentic historical record of what J-133 claimed at amendment time; §6.3 (this section) is the J-134 correction.

**§2.1 Q1.1 row intact at v1.4.** No Q1.11 added — there is no field-type retype to enumerate. The §2.1 table continues to enumerate exactly the six per-space HashMap field-type retypes per the v1.0 walk. If the J-133 rewrite left a dangling "Q1.1 extension" reference anywhere in §2.1 (Joe's pre-load instruction), the §2.1 table was grep-verified at J-134 atom prep — no such reference exists; the dangling references were only in §2.3 Q3.6 v1.3 (now rewritten at v1.4) + §6.2 historical record (preserved as-is). Clean.

**"Honest longer work over fast shortcuts" Pass 3 count increments to TWO** at J-134 (J-129 → ONE; this is → TWO). This IS a recurrence, not a prospective catch: J-133 shipped a wrong canonical Q-row to origin/main (`7494346` pushed); J-134 corrects it. That's the recurrence shape — a mistake reached canonical record and needs an honest fix. Distinct from prospective catches (J-115/J-116/J-129's own D-078 verification at session-open audit) that stopped BEFORE shipping. J-129's increment was for the J-128 runbook §4 surface ordering drifts (also reached canonical record before catch); J-134's increment is sibling-shape at a different surface (design doc Q-table v1.3 instead of runbook §4 v1.0).

**Three-file atomic per D-074 (thirty-first instance) + Lock #3 per-commit cadence + D-079 promotion atom**:
1. This design doc v1.3 → v1.4 (header chain prepend + Q3.6 rewrite + this §6.3 sub-section).
2. `DECISIONS.md` (new D-079 prepended at top per file convention — newest first).
3. `JOURNAL.md` (J-134 body §-entry — substantive prospective-catch-becomes-recurrence + κ promotion + sub-shape).

CLAUDE.md NOT amended (entry-point stays Commit 2; sibling-shape to J-132 single-file + J-121 hygiene atom no-PLAY-touch precedent). ROADMAP.md NOT amended (within-milestone canonical-record clear; sibling-shape to J-117 + J-130 framing).

**Verified anchors paste** (per Joe's D-078 lock at J-134 — "Write the verified line anchors verbatim into the v1.4 Q-row"):
- `xgen-core/src/space/state.rs:132` — `pub federation_nodes: Vec<NodeXgid>,` (struct definition)
- `xgen-core/src/space/state.rs:423` — inline comment "Pass 1 retypes federation_nodes to Vec<NodeXgid>; the peer derivation"
- `git blame xgen-core/src/space/state.rs:132` → commit `774fe9d1` (XGID Retrofit Pass 1 Commit 4 — xgen-core data-structure retypes, 2026-05-26)
- `xgen-node/src/federation_session.rs:202-208` — `apply_federation_push` 5-parameter signature (verified at J-133 D-078; confirmed unchanged at J-134)
- `xgen-node/src/federation_session.rs:248-254` — xgen-node local-variable annotation `let federation_nodes: Vec<String> = { ... s.federation_nodes.clone() ... }` (the Pass-1-broken-by-Path-A compile-error site that Pass 3 Surface #3 fixes)
- `xgen-node/src/federation_session.rs:261` — `for peer_id in &federation_nodes { ... }` loop (inherits typed value post-annotation-fix)

---

## §7 Discipline-notes (consolidated at v1.2)

Five sub-sections capturing the load-bearing discipline observations from the seven-surface walk. Pass-internal-consistency over trilogy-internal-consistency per Pass 2 §7.7 framing applies — these notes are Pass-3-specific, not retroactive imposition on Pass 1 + Pass 2 walks.

### §7.1 Format-boundary preservation as unified architectural pattern

The §4.3 v1.1 → v1.2 consolidation from "wire-format boundary preservation" to "format-boundary preservation (wire OR persistence)" is the load-bearing finding from the Surface #5 walk. The principle is one: byte-serialisation boundaries preserve String at function signatures; conversion to typed XGID happens at the typed-call boundary inside the function.

**Sibling-shape to the no-drift-surface discipline family** (D-067 code-organisation + D-070 transport-layer correlation pair + D-075 event-model vantage-awareness + D-076 wire-format determinism + D-077 silent-discard bidirectional sustainability). §4.3 sits at the I/O-boundary layer of the same family. If a third format-boundary surface emerges at Pass 4 (xgen-client IPC, AI-control-protocol over HTTP, gRPC), candidate D-NNN-δ promotes per D-069 three-instance threshold to a formal DECISIONS.md entry.

**Discipline implication for next sibling milestone runbook author**: at any surface walk involving I/O serialisation, ask "is this slot a format-boundary?" explicitly before locking the retype. If yes, the slot stays String at the function signature; the typed-call boundary is the projection site.

### §7.2 Async-spawned task captures force owned parameters

The §4.2 sibling-shape rule table extension at v1.2 (third row: "Parameter, async-spawned task captures — Owned — Forced") captures the rule instantiated at three sites across Surfaces #5 + #6 (`handle_federation_incoming`, `spawn_reconnect_scheduler`, `attempt_reconnect`).

**Why flagged-not-promoted as candidate D-NNN-ε**: three instances at the same module-family surface (xgen-node async handlers) is weaker durability evidence than three instances across structurally different surfaces. The rule itself is a Rust-language idiom (the `'static` bound on `tokio::spawn` closures requires owned values to cross the spawn boundary) — promoting it to DECISIONS.md would record a language fact, not a project decision per D-065 honest framing. The §4.2 rule-table entry at design-doc layer captures the pattern as Pass-internal guidance.

**Promotion-watch opens at Pass 4** surfacing a structurally different fourth instance at xgen-client async surfaces — Tauri commands, AI service spawns, batch dispatcher workers. If Pass 4 instantiates the rule at xgen-client async surfaces, that's a cross-crate fourth instance and would make the principle durable across the project rather than within one module-family.

### §7.3 Forced-owned return shape rule

§4.4 records a general rule (recorded inline): **A return type can be borrowed only if every branch can borrow from input at the same flavour. If any branch must construct a new value or change flavour, the return type is forced to owned.** Instantiated at `event_space_id(event: &Event) -> Option<SpaceXgid>` (Surface #4 Q4.1) where the state-create branch must construct a new SpaceXgid from event_id (flavour change EventXgid → SpaceXgid).

**Reusable for Pass 4 + Pass 5 walks.** Any future return-shape question with mixed-branch construction-vs-borrowing inherits this rule. The rule sits one rung above the §4.2 sibling-shape rule table (which covers parameter shapes); §4.4 covers return shapes specifically.

**Pass-internal-consistency**: the rule is not a project decision; it's a Rust-type-system pattern observation. Sibling-shape to §7.2 (Tokio idiom, not project decision). Both stay at design-doc layer rather than promoting to DECISIONS.md.

### §7.4 xgen-node-internal type confusion at v1.0 framing — data point for sibling milestone review

§4.5 records that v1.0 framing of ClientSenders + FederationPeerSenders implied they might be Pass 4 scope ("xgen-client-facing"). The v1.1 walk corrected this — both types are xgen-node-internal (mpsc::Sender channels never cross to xgen-client; xgen-client connects via WebSocket and receives TransportMessage). The correction shifted ~2-3 sub-surfaces from Pass 4 deferral to Pass 3 scope.

**Data point for next sibling milestone**: at design-walk Surface enumeration time (v1.0 equivalent), explicit verification of "does this type cross a crate boundary?" should precede the Pass-of-record decision. The v1.0 enumeration framed crate-boundary without verifying — caught at v1.1 walk via type-search-and-correction. Pattern is single-instance here; if a sibling milestone surfaces an analogous correction, candidate D-NNN at "crate-boundary verification before Pass-of-record decision" emerges.

### §7.5 Doc-tree sweep classification-vs-content-shape gap — Appendix D Surface #7

§2.7 records the J-095 Phase 2 doc-tree sweep classified Appendix D "in-scope for Pass 3" correctly in intent — but the actual content shape (prose-not-schema, 4 hits in markdown tables) means Pass 3 substantive work at Appendix D is minimal at most.

**Data point for sibling milestone runbook authors**: when classifying docs in a doc-tree sweep for Pass-of-record assignment, surface the **hit count + content shape per doc** (schema vs prose vs hybrid) at classification time, not at design-walk time. The J-095 sweep classified by topic relevance; the v1.2 design walk discovered the content-shape gap.

**Candidate D-NNN watch** sibling-shape to D-078 production-grounded test enumeration but at doc-tree classification layer: "doc-tree sweep produces hit-count + content-shape per doc, not topic relevance alone". One instance at this surface; three-instance threshold would require sibling findings at Pass 4 + Pass 5 doc-tree work or at a future audit-design-impl arc with a doc-classification phase.

---

## §8 Cross-references

- **Pass 1 design**: implicit in Pass 1 runbook authoring (no separate design doc per the Pass 1 era's lighter framing).
- **Pass 1 implementation runbook**: `tasks/XGID_RETROFIT_PASS_1_IMPL.md` (COMPLETED v2.1 at J-122).
- **Pass 2 design**: `tasks/XGID_RETROFIT_PASS_2_DESIGN.md` (COMPLETED v1.1 at J-126) — Pass 3 inherits Borrow<str> projection mechanism + governing principle + §6.7 Shape-α pointer-style.
- **Pass 2 implementation runbook**: `tasks/XGID_RETROFIT_PASS_2_IMPL.md` (COMPLETED v1.1 at J-126) — Pass 3 inherits the contingent-split posture framing + three Joe-lock checkpoints pattern.
- **Topological-sort design**: `tasks/FEDERATION_TOPOSORT_DESIGN.md` (COMPLETED v1.1 at J-101) — §4.6 references topo-sort's J-097 lock on the `&str` sort slot at fanout.rs:193 and records that the anticipated retype was already covered at Pass 1 Commit 3.
- **Phase 2 doc-tree sweep classification**: `tasks/XGID_DOC_SWEEP.md` (COMPLETED at J-095) — Appendix D classified in-scope for Pass 3; §7.5 records the classification-vs-content-shape gap surfaced at v1.2 design walk.
- **D-067 + D-070 + D-075 + D-076 + D-077 no-drift-surface discipline family**: §4.3 sits at I/O-boundary layer of the same family per §7.1 framing.
- **D-076 v1.1**: byte-identical determinism + causal-DAG-respecting order at the wire-format layer (no impact on Pass 3 in-memory retypes; orthogonal concern). The v1 → v1.1 amend-in-place pattern is the model for §4.3 v1.1 → v1.2 consolidation per §4.3 reasoning point 4.
- **D-077**: backward-coherence discipline at silent-discard / fallible-discard sites (Pass 3 should ask the question at any silent-discard surface in xgen-node handlers if surfaced).
- **D-078**: production-grounded test enumeration (Pass 3 runbook phase will apply at any per-surface unit-test enumeration).

---

*End of XGID Retrofit Pass 3 Design Document v1.2 — Full seven-surface walk closed. §3 governing principle confirmed inherited unchanged. §4 architectural decisions §4.1-§4.6 locked. §5.5 layered-B3 confirmed null at full scope (third Pass-arc instance, pattern durable). §6.1 design-close historical entry filled in Shape α. §7 discipline-notes consolidated with five sub-sections. Three candidate D-NNNs flagged-not-promoted per D-069. Status flipped ACTIVE → COMPLETED at J-127. Next-active for Chat Claude + Joe: implementation runbook authoring at `tasks/XGID_RETROFIT_PASS_3_IMPL.md` in a fresh session.*
