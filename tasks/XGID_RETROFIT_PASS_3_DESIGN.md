# XGID Retrofit Pass 3 — Design
> **Status**: ACTIVE  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-27 (J-NNN — Pass 3 design phase opened. §1 framing + §2 surface enumeration shipped at design-phase kickoff; §3 governing principle + §4 architectural decisions + §5 layered-B3 expected answer + §6 historical/future-pointer entries + §7 discipline-notes deferred to subsequent walk-and-lock sessions per Joe-lock at session open. Pass 3 scope: xgen-node + Appendix D — federation_session, fanout, app handlers, reconnect scheduler, the six per-space HashMap keys at NodeRuntime deferred from Pass 2 per design doc §4.1 Q2.8.c, and Appendix D doc retypes. Seven surfaces enumerated in dependency order at §2 (NodeRuntime per-space HashMap keys → dispatch_event peer parameter → federation_session.rs handler slots → fanout.rs handler slots → app.rs handler slots → reconnect scheduler identifiers → Appendix D doc retypes). Per Rule 0 + D-065 + D-067 + D-069 + D-071 + D-074 + D-077 + D-078.)  
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

Pass 2's design doc was ~31 KB. Pass 3 carries slightly more surface count (seven vs Pass 2's five) and the six-HashMap-keys retype carries genuine reasoning load — the very call-site-density heuristic that deferred them from Pass 2 (candidate D-NNN-γ per J-126 Sub-section 8) needs explicit walk-and-lock at Pass 3. Pass 3 design doc target weight: **~30-40 KB**, slightly heavier than Pass 2 but still substantially lighter than trilogy precedent (~80-100 KB), respecting Pass-internal-consistency per the Pass 2 §7.7 framing.

Three precedent-positioning notes:

1. **No re-walk reservation at design open.** Pass 2 §1.2 also opened without §11 re-walk reservation; that pattern carried through cleanly with zero re-walks fired. Pass 3 opens the same way. If a re-walk fires post-design-close, this document gains §11 amendment in place per topo-sort J-099 precedent.

2. **Single governing principle expected to inherit from Pass 2 unchanged.** Pass 2 §3 locked: *identifier slots retype to typed XGIDs; descriptive-string slots stay String; internal variables bind as typed references; &str projection happens at the call-site boundary via Borrow<str>; no Deref<Target = str> shortcuts.* Pass 3 surfaces consume the same Borrow<str> projection mechanism inherited from Pass 1. The governing principle at §3 is expected to read identical; design phase verifies-not-assumes by walking each surface explicitly. If a federation-transport-layer or wire-format surface forces a wrinkle, §3 records the divergence honestly.

3. **Pass 3 spans multiple xgen-node modules + a doc.** Pass 2's surface was concentrated in `xgen-core/src/{message/exchange,node/runtime,node/pending,federation/registry,identity/registry}.rs`. Pass 3 surfaces span `xgen-node/src/{federation_session,fanout,app}.rs` plus the `NodeRuntime` HashMap-key sub-surface (which sits at `xgen-core/src/node/runtime.rs` per the Pass 2 deferral) plus a reconnect scheduler surface plus `docs/xgen_appendix_d_en.md`. The multi-file spread informs the runbook authoring step's contingent-split posture (D-NNN-δ promotion-watch per J-126 Sub-section 8).

### §1.3 What this document is NOT

- **Not a re-audit of Pass 2's xgen-core retypes.** Pass 2's COMPLETED locks stand authoritative.
- **Not the Pass 4 xgen-client design.** ops:: verb signatures, AI control flows, session state retypes defer to Pass 4 per honest-broadening (named explicitly at the per-surface walks in §2).
- **Not the Pass 5 test-fixture / trace-field / Debug-Display sweep.** Trace event fields, Display impls, Debug impls in handlers defer to Pass 5 per honest-broadening (named explicitly at the per-surface walks in §2).
- **Not a runbook.** The implementation sequencing, commit shape, Joe-lock checkpoints, and verification rigour live in `tasks/XGID_RETROFIT_PASS_3_IMPL.md` authored at runbook phase per topo-sort + persistence-amendment + Pass 1 + Pass 2 precedent.

---

## §2 Surface enumeration (dependency order)

Seven surfaces walked in dependency order. Each surface is named here at the framing level; the per-surface Q-tables (current type shape → identifier slots requiring retype → defers-to-Pass-4/5 enumeration → wire-format implications if any) fill in at subsequent design-phase walk sessions.

The dependency order rationale: foundational surfaces (those producing types other surfaces consume) walk first; surfaces consuming those types walk later. Confirming dependency-order at design phase keeps the runbook authoring step's commit-sequence shape downstream-clean — earlier surfaces' lib changes don't accumulate test-fixture churn against later surfaces' unretyped call sites.

### §2.1 Surface #1 — `NodeRuntime` six per-space HashMap keys (xgen-core/src/node/runtime.rs)

**What this is.** Six `HashMap<String, _>` field types on the `NodeRuntime` struct keyed by space_id, deferred from Pass 2 per Pass 2 design doc §4.1 Q2.8.c on call-site-density grounds. Pass 2 retyped only the small-cardinality `peer_urls` HashMap (small enough that per-site Borrow<str> projection was structurally cheap); the six per-space maps were deferred to the Pass that touches their primary call-site crate (xgen-node), which is Pass 3.

**Foundational position.** These HashMap keys are foundational to most xgen-node handler call sites — the handlers look up per-space state by space_id, which means every space-id lookup site is a call site against these six maps. Retyping the field types first means downstream handler retypes consume already-typed keys; retyping handlers first against still-String keys would require sweep churn at the HashMap retype landing.

**Crate boundary note.** Surface #1 lives in `xgen-core/src/node/runtime.rs` even though every other Pass 3 surface lives in `xgen-node/src/`. The deferral was based on call-site-density (most call sites are in xgen-node) but the field-type definitions are in xgen-core. Pass 3 retypes both the field-type definitions (xgen-core) and the call sites (xgen-node) atomically per the principle that field-type and call-site retypes belong in the same commit.

**Per-surface Q-table (filled at subsequent design walk session).**

### §2.2 Surface #2 — `dispatch_event` `peer_node_id: Option<&str>` parameter (xgen-core/src/node/runtime.rs)

**What this is.** The federation-channel entry point parameter on `NodeRuntime::dispatch_event` carrying the wire-authenticated peer's node_id when the event arrived via federation (None when locally-submitted). Pass 2 retyped `dispatch_event`'s internal logic but left the parameter signature at `Option<&str>` with an inline code-comment marker indicating Pass 3 widens this to `Option<NodeXgid>` or `Option<&NodeXgid>`.

**Foundational position.** Every call site invoking `dispatch_event` from xgen-node passes a `peer_node_id` value — federation_session.rs constructs this from handshake state; app.rs's local-submit path passes None; fanout.rs's federation-push path passes the destination peer's node_id. Retyping the parameter forces every call site to projection-clean its peer_node_id type at the boundary. Doing this before the handler-side retypes (Surfaces #3-#5) means handler retypes consume an already-typed parameter signature.

**Borrowed-or-owned question for the Q-table.** Pass 2's Borrow<str> projection mechanism answers most boundary questions at call-site time, but `Option<&NodeXgid>` vs `Option<NodeXgid>` carries semantic weight: borrowed implies the caller owns the lifetime; owned implies a clone at the boundary. Pass 2's `peer_urls` retype landed as owned `NodeXgid` insert; the dispatch_event parameter question is structurally similar but the lifetime semantics differ. Q-table walks this at §2.2's per-surface session.

**Per-surface Q-table (filled at subsequent design walk session).**

### §2.3 Surface #3 — `federation_session.rs` handler identifier slots (xgen-node/src/federation_session.rs)

**What this is.** The federation transport handler module hosting wire-format handshake logic, per-peer session state, and the `apply_federation_push` entry point that Pass 2's J-111 trace-event retrofit (G2 retrofit per the Phase 9 Commit 3b-3-pre work) instrumented with `local_node_id: &str`. Identifier slots in this module: peer node_id (from handshake), local node_id (from runtime), space_id slots passed through to runtime calls, event_id slots on the wire path.

**Consumes Surfaces #1 + #2.** federation_session.rs calls into `NodeRuntime::dispatch_event` and into the per-space HashMap lookups; both are foundational dependencies retyped at #1 and #2.

**Wire-format check at Q-table walk.** federation_session.rs hosts wire-format serialization. The Pass 2 design principle (`identifier slots retype to typed XGIDs`) holds at the in-memory layer; the wire-format layer is independent (wire is canonicalised bytes, not Rust types). Q-table walks confirms-or-flags whether any wire-format identifier appears as a Rust String at the boundary — if so, the boundary's typed-Rust vs canonicalised-wire mapping is a design point worth recording.

**Per-surface Q-table (filled at subsequent design walk session).**

### §2.4 Surface #4 — `fanout.rs` handler identifier slots (xgen-node/src/fanout.rs)

**What this is.** The fanout dispatcher module hosting per-space history computation (`compute_federation_delta_for_space`), topological sort (`topological_sort_events`), and the federation-push delivery path. Identifier slots: destination peer node_id, source local node_id, space_id, event_id slots on the delta-computation and push paths.

**Sibling to #3.** fanout.rs's interaction with the type graph is structurally similar to federation_session.rs's; differences are functional (delta computation + topo sort vs handshake + per-peer session). Walking #3 before #4 may resolve most #4 questions; walking #4 may surface delta-computation-specific wrinkles (e.g., the topo-sort's Pass-1-neutral `&str` sort at `topological_sort_events:193` was deliberately Pass-3-deferred per J-097 design lock for the topo-sort milestone — confirm whether Pass 3 retypes this slot or whether it stays Pass-3-deferred at this Pass 3 milestone too).

**Per-surface Q-table (filled at subsequent design walk session).**

### §2.5 Surface #5 — `app.rs` handler identifier slots (xgen-node/src/app.rs)

**What this is.** The xgen-node application-level entry point hosting top-level event handlers (`process_inbound`, identity handlers, the pipe server admin verbs at Block 4 of M6 (new) if M6 (new) ships before or during Pass 3), the reconnect scheduler invocation surface, and the bootstrap logic. Identifier slots: local node_id, peer node_ids in many forms (from handshake, from registry, from pipe-server admin verbs), space_ids at handler boundaries, event_ids at handler boundaries.

**Consumes #1 + #2 + #3 + #4.** app.rs sits at the top of the xgen-node module dependency graph; every other Pass 3 surface feeds into it.

**M6 (new) coordination flag.** If M6 (new) Block 4 verb-by-verb walks ship pipe-server admin verbs before Pass 3 implementation, those verbs land with String-typed identifier slots that Pass 3 retypes. If Pass 3 ships first, M6 (new) Block 4 verbs land already-Pass-3-typed. The coordination flag stays at the runbook authoring phase per honest-broadening discipline — the design doc just names the coordination surface; the runbook decides sequencing.

**Per-surface Q-table (filled at subsequent design walk session).**

### §2.6 Surface #6 — Reconnect scheduler identifiers (xgen-node/src/app.rs reconnect scheduler region; possibly extracted to separate module)

**What this is.** The reconnect scheduler logic that re-establishes federation sessions after transient disconnects. Identifier slots: peer node_ids in the schedule table, space_ids if scheduling is per-space, attempt-count + backoff-state structures keyed by peer node_id.

**Orthogonal-ish to #3-#5.** Reconnect scheduler logic interacts with federation_session.rs (it spawns new sessions) and with the FederationRegistry (it consults peer_urls retyped at Pass 2's Surface #2); it does not interact with fanout.rs or with the per-space HashMap surface directly. Walking it last in the xgen-node sequence keeps the design walk focused per surface.

**Scope-boundary check at Q-table walk.** The reconnect scheduler may have its own per-peer state (e.g. last-attempt timestamp, backoff multiplier) that is not identifier-shaped and stays String/other primitive at Pass 3. Q-table confirms which slots are identifier slots in scope vs which are scheduler-state slots out of scope.

**Per-surface Q-table (filled at subsequent design walk session).**

### §2.7 Surface #7 — Appendix D doc retypes (docs/xgen_appendix_d_en.md)

**What this is.** The Node-side storage and privacy appendix at `docs/xgen_appendix_d_en.md`. This is the Pass 3 doc-retype surface analogous to Pass 1's Appendix C + Appendix I (data structure schemas) and what Pass 4 will do for Appendix F (client-side) per the Pass 1 doc-sweep classification.

**Hit count TBD at design walk.** The XGID hit count in Appendix D is unknown at design-phase open. The Phase 2 doc-tree sweep at J-095 classified Appendix D as in-scope for Pass 3 but did not exhaustively enumerate the hit count. Design walk includes a hit count + section breakdown at §2.7's per-surface session. If the hit count is small (≤ ~20), Appendix D folds into the Pass 3 doc-pass commit cleanly. If large (~100+ similar to Appendix I's 122), the runbook may dedicate a separate commit per Pass 1's pattern.

**Independent of #1-#6.** Doc retypes do not consume types from the code surfaces; the Appendix D content describes Node-side storage shapes, persistence boundaries, privacy guarantees — the retype updates the textual XGID-vs-String descriptions to match the post-Pass-3 code surface. Walking #7 last in the design sequence keeps the dependency-order narrative clean.

**Per-surface Q-table (filled at subsequent design walk session).**

### §2.8 Out-of-scope enumeration (honest broadening)

Surfaces NOT in Pass 3 scope, named explicitly per honest-broadening discipline:

- **Pass 4 (xgen-client + AI control docs).** ops:: verb signatures, AiBehavior trait, session state, batch dispatcher, CLI dispatcher, AI service, Tauri commands, Appendix F client-side sections, `docs/xgen_aicontrol_implementation.md` reply schemas, Ch6 §6.15 client-side spec.
- **Pass 5 (test fixtures + trace fields + Debug/Display + workspace test-count restoration).** Trace event field types in xgen-node modules, log line formatters at Appendix G, Debug/Display impls, test fixture builders, integration test helpers, `cargo build --workspace` restoration.
- **M6 (new) Node admin write path (Block 4 verb-by-verb walks).** The pipe-server admin verbs at `docs/xgen_node_admin_ops_design.md` §6 (~35 verbs across 7 categories). If M6 (new) ships before Pass 3 implementation, those verbs land String-typed and Pass 3 retypes them at #5; if Pass 3 ships first, M6 (new) verbs land Pass-3-typed. Coordination is a runbook-phase question, not a Pass-3-design question.
- **Sibling event constructors beyond the Pass 1 narrow scope.** `state.federation_add`, `membership.*`, `message.*` event constructors stay at their post-Pass-1 + post-Pass-2 type discipline; Pass 3 does not re-audit constructor shape.
- **Wire-format canonicalisation.** Wire format is bytes, not Rust types; the in-memory Rust type retype does not change wire-format canonical encoding. If a surface walks a wire-format slot at Q-table phase, the design records the typed-Rust vs canonical-wire boundary mapping without changing the canonical-wire side.

---

## §3 Governing principle — DEFERRED to design walk close

Locked at design walk close per the sequencing decided at session open. Expected (verified-not-assumed at walk): inherit Pass 2's principle unchanged — *identifier slots retype to typed XGIDs; descriptive-string slots stay String; internal variables bind as typed references; &str projection happens at the call-site boundary via Borrow<str>; no Deref<Target = str> shortcuts.* If a federation-transport-layer or wire-format surface forces a wrinkle, §3 records the divergence honestly.

---

## §4 Architectural decisions — DEFERRED to design walk close

Locked at design walk close. Anticipated topics surfaced at §2's surface walks:

- **§4.1** — Six per-space HashMap key retype shape (D-NNN-γ promotion-watch from J-126).
- **§4.2** — `dispatch_event` `Option<&NodeXgid>` vs `Option<NodeXgid>` boundary.
- **§4.3** — Topological sort `&str` slot at fanout.rs:193 retype scope (retype now vs defer to Pass 5 per J-097 lock).
- **§4.4** — Appendix D doc retype commit shape (folded into doc-pass vs separate commit per hit count at §2.7's walk).
- **§4.5** — M6 (new) coordination (runbook-phase question only; design records the surface).

Additional decisions surface at walk per Q-table outputs.

---

## §5 Layered-B3 audit answer — DEFERRED to design walk close

Pass 2 §5.5 expected null; J-126 confirmed null (zero layered surfaces). Pass 1 J-122 also null. Pass 3 expectation at design open: **likely null** — Pass 3 scope is identifier-slot shape at the xgen-node binary surface, not algorithm validation or invariant encoding. The B3-shape surfaces (layered encodings of the same invariant) historically emerged in algorithm-bearing scopes (topo-sort J-101 found two, persistence-amendment J-108 found one); Pass 1 + Pass 2 scopes found zero. Pass 3 walks the question explicitly at design close and records the expected answer in advance, sibling-shape to Pass 2 §5.5.

---

## §6 Historical / future-pointer entries — DEFERRED to design walk close

Sibling-shape to Pass 2 §6.7 (Shape α — pointer-style). Pass 3 may gain its own §6.x entries at implementation milestone events per the precedent.

---

## §7 Discipline-notes — DEFERRED to design walk close (included only if precedent-departure self-defense or load-bearing discipline note arises)

Pass 2 design doc §7 was substantial (§7.7 Pass-internal-consistency framing was load-bearing). Pass 3 §7 inclusion depends on whether the surface walks produce discipline-notes worth recording at design layer vs runbook layer.

---

## §8 Cross-references

- **Pass 1 design**: implicit in Pass 1 runbook authoring (no separate design doc per the Pass 1 era's lighter framing).
- **Pass 1 implementation runbook**: `tasks/XGID_RETROFIT_PASS_1_IMPL.md` (COMPLETED v2.1 at J-122).
- **Pass 2 design**: `tasks/XGID_RETROFIT_PASS_2_DESIGN.md` (COMPLETED v1.1 at J-126) — Pass 3 inherits Borrow<str> projection mechanism + governing principle + §6.7 Shape-α pointer-style.
- **Pass 2 implementation runbook**: `tasks/XGID_RETROFIT_PASS_2_IMPL.md` (COMPLETED v1.1 at J-126) — Pass 3 inherits the contingent-split posture framing + three Joe-lock checkpoints pattern.
- **Topological-sort design**: `tasks/FEDERATION_TOPOSORT_DESIGN.md` (COMPLETED v1.1 at J-101) — §4.3 references topo-sort's J-097 lock on the `&str` sort slot at fanout.rs:193.
- **Phase 2 doc-tree sweep classification**: `tasks/XGID_DOC_SWEEP.md` (COMPLETED at J-095) — Appendix D classified in-scope for Pass 3.
- **D-076 v1.1**: byte-identical determinism + causal-DAG-respecting order at the wire-format layer (no impact on Pass 3 in-memory retypes; orthogonal concern).
- **D-077**: backward-coherence discipline at silent-discard / fallible-discard sites (Pass 3 should ask the question at any silent-discard surface in xgen-node handlers if surfaced).
- **D-078**: production-grounded test enumeration (Pass 3 runbook phase will apply at any per-surface unit-test enumeration).

---

*End of XGID Retrofit Pass 3 Design Document v1.0 — §1 framing + §2 surface enumeration complete; §3-§7 deferred to subsequent design walk sessions.*
