# XGID Retrofit Pass 3 — Implementation Runbook
> **Status**: COMPLETED  
> Version: 1.6  
> Date: May 2026  
> **Last updated**: 2026-05-28 (J-138 — XGID Retrofit Pass 3 milestone CLOSED. Three Clair-facing commits + this milestone-close commit on `main`: Commit 1 `1be0249` (J-131 doc-pass Option C hybrid minimal) + Commit 2 `67fb48d` (J-136 seven-surface retype atomic + xgen-{common,core,node} libs CLEAN + Path 2 split locked) + Commit 2a `0cdf0ad` (J-137 test-fixture sweep + 11 per-surface tests T1-T11 atomic + 8/8 GREEN verification + 589 tests stable) + Commit 3 this commit (milestone close per D-074 thirty-fifth instance + fourteenth milestone-close). Status flipped ACTIVE → COMPLETED + v1.5 → v1.6. All three Joe-lock checkpoints fired as authored at §2.3: #1 post-Commit-1 doc-pass drift check at J-131 + resolution at J-132 Path-(iii) amend-in-place; #2 pre-Commit-2 verbatim surface list approval at J-133+J-134+J-135 triple-canonical-record-amendment arc; #3 post-Commit-2 split-trigger decision at 638 errors >> ~50 threshold → Path 2 split locked. Layered-B3 audit answer per §6.5 + design doc §5.5: zero layered surfaces emerged (third Pass-arc no-finding instance after Pass 1 J-122 + Pass 2 J-126 — three-instance no-finding chain now durable). Pass 3 "Honest longer work over fast shortcuts" final count: TWO recurrences (J-129 runbook v1.0 surface ordering drift + J-134 design doc §2 v1.3 → v1.4 in-place rewrite-correction; both prospective catches at canonical-record-amendment layer at Clair's session-open Rule-0 audits before any production code touched). Three candidate D-NNNs (γ + δ + ε + format-boundary) status recorded at JOURNAL J-138 Sub-section 8 promotion-watch list; none promoted in this atom per D-069. Final test count: 589 (34 xgen-common lib + 8 invariance + 453 xgen-core + 88 xgen-node lib + 6 precedence); +98 net delta vs Pass 2 J-126 baseline of 491. `cargo build --workspace` deliberately broken at xgen-client consumer sites only per Path A inherited from Pass 1 + Pass 2 (Pass 4 + Pass 5 close this window). Both pre-existing documented flakes did NOT fire across 8 GREEN runs at J-137 milestone-bearing boundary nor at this milestone-close re-verification pass. Both clippy gates clean. Grep J-NNN guardrail per J-108 codification returns ZERO post-staging. Pass 3 unblocks Pass 4 + M6 (new) both ready for next-milestone selection at session open. Per Rule 0 + D-065 + D-067 + D-069 + D-071 + D-074 + D-077 + D-078 + D-079.) Previous 2026-05-28 (J-137 — Pass 3 Commit 2a SHIPPED under Path 2 split per Joe-lock checkpoint #3. Test-fixture projection sweep across xgen-core (4 files, ~160 errors closed) + xgen-node (~20 files, ~478 errors closed) — total 638 errors → 0; mechanical projection-only edits per §5.2 verbatim patterns (sdx/ndx/idx/edx/rdx typed-XGID test helpers; `Some(peer_id.as_str())` → `Some(&ndx(&peer_id))`; `rt.dag_tips(&space_id)` → `rt.dag_tips(&sdx(&space_id))`; HashMap accesses via Borrow<str> projection; drain_pending_by_* typed call sites; ConnectedClientInfo Q5.15; ClientSenders + FederationPeerSenders typed keys; FanoutRequest.new_joiner Option<IdentityXgid>; apply_fanout / collect_sync_history / compute_federation_delta / apply_federation_push / handle_connection / run_federation_session_post_handshake / spawn_reconnect_scheduler / scheduler_tick / attempt_reconnect call-site projections). Plus 11 per-surface tests T1-T11 added per runbook §4.7 by name + production-anchor verbatim: T1-T3 Surface #1 (noderuntime_per_space_map_insert_retrieve_with_typed_key + _six_flavours_isolated + _helper_signatures_typed_at_boundary at runtime.rs); T4 Surface #2 (dispatch_event_with_borrowed_node_xgid_projects_to_str_at_callsite at runtime.rs); T5 Surface #3 (federation_session_handler_identifier_slots_retyped_at_boundary at federation_session.rs); T6 Surface #4 (fanout_topological_sort_event_xgid_slot_pass_1_intact at fanout.rs — Pass 1 carry-over sentinel); T7 + T8 + T11 Surface #5 (app_handlers_persistence_format_round_trip_string_at_boundary + handle_federation_incoming_spawned_task_owns_node_xgid_capture + run_federation_session_post_handshake_spawned_task_owns_typed_captures at app.rs); T9 + T10 Surface #6 (reconnect_spawned_functions_each_own_typed_capture + _arc_shared_reference_pattern_when_needed at reconnect.rs). **Verification rigour per §4.9 + §5.3**: 8 GREEN runs (5 isolated with `cargo clean -p xgen-common -p xgen-core -p xgen-node` between each + 3 consecutive workspace runs of `cargo test -p xgen-common -p xgen-core -p xgen-node`) — ALL 8 GREEN. Test count stable at **589 = 34 xgen-common lib + 8 invariance + 453 xgen-core + 88 xgen-node lib + 6 precedence** (xgen-core 449 → 453 via T1-T4; xgen-node lib 81 → 88 via T5-T11). Both clippy gates clean: `cargo clippy -p xgen-common -p xgen-core -p xgen-node --lib --all-features -- -D warnings` + `--tests -D warnings` (six clippy nits in xgen-core T1+T2 tests + agent-sweep fanout.rs + phase9_compound_c7 closed: `.get(&x).is_some()` → `.contains_key(&x)`; redundant closure `|e| event_id_str(e)` → `event_id_str`; useless `vec![]` → `[]`). `cargo build --workspace` deliberately broken at xgen-client consumer sites per Path A inherited from Pass 1 (192 errors all xgen-client, no regression at xgen-common + xgen-core + xgen-node; Pass 5 close restores). Both pre-existing documented flakes (precedence env-var race; `reconnect_with_existing_tip_small_delta_delivered`) did NOT fire across the 8 GREEN runs. **New §9.7 amendment-provenance** recording Commit 2a ship + parallel-subagent-sweep discipline data point. **"Honest longer work over fast shortcuts" Pass 3 count stays at TWO** inherited from J-129 + J-134 (Commit 2a is within-milestone substantive event, sibling-shape to J-101/J-108/J-122/J-126 close-event-not-recurrence-event framing). Per Rule 0 + D-065 + D-067 + D-069 + D-071 + D-074 + D-077 + D-078 + D-079.) Previous 2026-05-28 (J-136 — Pass 3 Commit 2 SHIPPED under Path 2 (Commit 2a split) per Joe-lock checkpoint #3. Seven-surface retype atomic at xgen-core/src/node/runtime.rs + xgen-node/src/{federation_session,fanout,app,reconnect}.rs + docs/xgen_appendix_d_en.md doc-tree sweep (four markdown table rows annotated with typed-XGID-in-memory + String-on-disk + on-wire per §4.3). xgen-common + xgen-core + xgen-node libs all CLEAN at this commit; xgen-core + xgen-node test fixtures **638 errors total** (160 + 478) above the §5.1 ~50 split threshold — Joe-locked Path 2 at checkpoint #3 to ship Commit 2 lib-clean + Commit 2a test-fixture sweep + 11 per-surface tests T1-T11 atomic separately. clippy `--lib --all-features -D warnings` clean on all three packages. `cargo build --workspace` deliberately broken at xgen-client consumer sites per Path A inherited. WIP branch lineage: branch `wip/pass-3-commit-2-in-flight` carried checkpoint #1 728b834 (Surfaces #1+#2+#3+#4 lib-clean + #5 partial) + checkpoint #2 2f647bf (Surfaces #5+#6 closed; xgen-node lib clean); squashed at this Commit 2 ship per D-074 atomic discipline (single-commit per surface-set per the §4.10 Files-in-this-commit framing). New §9.6 amendment-provenance sub-section recording the Path-2-split decision at checkpoint #3 + the two-WIP-checkpoint lineage + the lib-only verification at Commit 2 vs full 8 GREEN at Commit 2a per §5.3. "Honest longer work over fast shortcuts" Pass 3 count does NOT increment — Commit 2 ship is a within-milestone substantive event, not a recurrence shape (prior recurrences: J-129 + J-134 inherited at TWO). Per Rule 0 + D-065 + D-067 + D-069 + D-071 + D-074 + D-077 + D-078 + D-079.) Previous 2026-05-28 (J-135 — Joe-lock-checkpoint-#2 follow-on amendment at the test-enumeration layer. T11 added to §4.7 Surface #5 as `run_federation_session_post_handshake_spawned_task_owns_typed_captures`: pins the async-spawned forced-owned `NodeXgid` (home_node_id + peer_node_id) + `Vec<SpaceXgid>` (peer_shared_spaces) capture-shape at the bilateral federation session driver `pub(crate) async fn run_federation_session_post_handshake` (app.rs:1152) across `tokio::spawn` boundary. Sibling to T8 (`handle_federation_incoming_spawned_task_owns_node_xgid_capture` at app.rs:976) which covers the wire-format-handler async-spawn pattern; T11 covers the top-level-orchestrator async-spawn pattern at the different §2.5 sub-region. Production anchor verified at J-135 author-time per D-078: app.rs:1152-1171 signature read; `home_node_id: String` at line 1161, `peer_node_id: String` at line 1165, `peer_shared_spaces: Vec<String>` at line 1169 all intact in the Q5.14 v1.3 shape locked at J-133 (post-`OutboundMsg` mis-attribution closure). **D-078 application**: test enumeration corrected against a retype target (`run_federation_session_post_handshake`) that entered Pass 3 scope at J-133's Q5.14 v1.3 rewrite, **post the original +10 enumeration authored at J-128 runbook authoring**. The original +10 was authored before the design doc §2.5 sub-region "Top-level orchestrators" was extended (at J-133) to include `run_federation_session_post_handshake`; the test enumeration carried a coverage gap as a downstream consequence. D-078 working as designed: when a Pass-3-scope retype target enters at canonical-record amendment time, the test enumeration is verified against the new target before Commit 2 production code lands — sibling-shape to D-078's promotion threshold itself (verification at amendment-time rather than retroactively after implementation surfaces drift). §4.7 Surface #5 count 2 → 3 tests (T7 + T8 + T11). Total per-surface test target: +10 → +11 (new total 502 if all land). §9 amendment-provenance gains a v1.2 → v1.3 sub-section recording the D-078 application + T11 anchor verification paste + the J-128 → J-133 → J-135 framing. Joe-lock-approved at session-time turn (T11 APPROVED by name conditional on anchor holding at author-time; anchor verified clean). Single-file atom per D-074 (thirty-second instance, grounded across J-127 24th → J-128 25th → J-129 26th → J-130 27th → J-131 28th → J-132 29th → J-133 30th → J-134 31st → this J-135 32nd) + Lock #3 per-commit cadence. Not a milestone-close — milestone-close tally stays at thirteenth from J-126. CLAUDE PLAY does NOT flip (entry-point stays Commit 2). ROADMAP NOT touched (within-milestone). JOURNAL NOT amended (chain-only-then-no-op per Joe-lock; T11 addition is mechanical completion of an already-surfaced D-078 application — the working-as-designed observation lives in §9 v1.3 provenance, not as a fresh body §-entry). DECISIONS.md NOT amended (no new candidate). "Honest longer work over fast shortcuts" Pass 3 count does NOT increment — prospective catch at test-enumeration layer; T11 added BEFORE Commit 2 ships (sibling-shape to J-115/J-116 prospective-catch framing where catch happens before production code lands; count stays at TWO inherited from J-129 + J-134). Per Rule 0 + D-065 + D-067 + D-069 + D-071 + D-074 + D-077 + D-078 + D-079.) Previous 2026-05-28 (J-132 — Path-(iii) amend-in-place at checkpoint #1 resolution of the honest two-file-vs-three-file count discrepancy surfaced at J-131. §3.2 third-file line rewritten from "JOURNAL.md chain entry only" to "JOURNAL.md NOT amended post-strip" — post-J-129 strip-the-chain discipline per JOURNAL J-129 Sub-section 8 makes a chain-only entry a no-op + sibling-shape to J-123/J-124/J-125 chain-only doc-only milestone-event precedent suppresses body entry. §3.1 file-count corrected three → two. New §9.4 amendment-provenance sub-section recording the v1.1 → v1.2 amendment + cross-forward note about §3.3 carry-forward inconsistency parked for future Pass-arc author consideration. Forward-looking doc-hygiene (preventing future Pass 4 + Pass 5 re-derivation of the same resolution); "Honest longer work" Pass 3 count does NOT increment. CLAUDE PLAY does NOT flip (entry-point stays Commit 2). ROADMAP NOT touched. JOURNAL gets no entry (post-strip no-op — the very discipline being reconciled). Single-file atom per D-074 (twenty-ninth instance) + Lock #3 per-commit cadence.) Previous 2026-05-27 (J-129 — Track 1 canonical-record amendment shipped. Re-aligned §4.1 + §4.7 + §4.10 surface ordering to design doc §2 verbatim (Surfaces #1↔#2 + #5↔#6 swapped at v1.0; corrected); corrected Surface #3 location for `handle_federation_incoming` from `federation_session.rs` → `app.rs` (`handle_federation_incoming` is defined at `xgen-node/src/app.rs:976`, not at `federation_session.rs`; `federation_session.rs` has zero `tokio::spawn` and zero `handle_federation_incoming`); clarified §4.5 + §4.7 Surface #5 (reconnect) wording around "three spawned functions"; new §9 amendment-provenance section. **D-078 second prospective-catch at runbook-authoring layer** (J-115 + J-116 were prospective catches at runbook-implementation-by-Clair layer; this J-129 is prospective catch at runbook-authoring-by-Chat-Claude layer — distinct surface). Discipline data point recorded at §7.11 (new): when authoring a runbook from a session-bridge summary rather than fresh from the design doc, the surface enumeration MUST be cross-checked against design doc §2 verbatim BEFORE the runbook §4 ships. Previous v1.0 J-128 update content stands authoritative in spirit — amended in place at v1.1 for cells reference design doc §2 verbatim.) Previous 2026-05-27 (J-128 — Runbook authored at design-close-plus-one session per Pass 2 + trilogy precedent; sibling-in-shape to `tasks/XGID_RETROFIT_PASS_2_IMPL.md` COMPLETED v1.1 with three structural extensions for Pass 3's seven-surface scope: §4.7 per-surface tests heavier (+10 target vs Pass 2's +2); §7 nine sub-sections (Pass 2 had eight); §7.10 Pass 5 consolidation flag recorded as future-walk candidate for runbook §7 deduplication across the five-Pass arc.)  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 Framing

### §1.1 What this runbook is

This runbook is Clair's complete pickup specification for XGID Retrofit Pass 3 implementation. It is the authoritative entry-point file for Clair's session at Pass 3 implementation kickoff.

Pass 3's scope: retype the seven xgen-node + Appendix D surfaces locked at design doc §2 Q-tables (`tasks/XGID_RETROFIT_PASS_3_DESIGN.md` COMPLETED v1.2 at J-127). The seven surfaces in design doc §2 ordering (v1.1 corrected):

1. `NodeRuntime` six per-space HashMap keys retype shape at `xgen-core/src/node/runtime.rs` (design doc §2.1)
2. `dispatch_event` `peer_node_id: Option<&NodeXgid>` borrowed boundary at `xgen-core/src/node/runtime.rs` (design doc §2.2)
3. `federation_session.rs` handler identifier slots at `xgen-node/src/federation_session.rs` (design doc §2.3)
4. `fanout.rs` verification at `xgen-node/src/fanout.rs` — topo-sort `&str` slot already covered at Pass 1 (design doc §2.4)
5. `app.rs` handler identifier slots at `xgen-node/src/app.rs` — includes `handle_federation_incoming` async-spawned task at app.rs:976 + persistence-format boundary (design doc §2.5)
6. Reconnect scheduler three spawned functions at `xgen-node/src/reconnect.rs` (design doc §2.6)
7. Appendix D doc-tree sweep at `docs/xgen_appendix_d_en.md` — four markdown table hits (design doc §2.7)

### §1.2 Precedent-departure self-defense (sibling-shape to Pass 2 design doc §1.2)

This runbook lands at ~50-70 KB target, slightly heavier than Pass 2's ~43 KB at `tasks/XGID_RETROFIT_PASS_2_IMPL.md` COMPLETED v1.1. Three drivers for the size increase:

- **Seven surfaces vs Pass 2's five** — two additional Q-tables to enumerate at §4.7 per-surface tests.
- **Structurally novel patterns at Pass 3** — async-spawned forced-owned (§4.2 row 3 of design doc rule table); persistence-format boundary preservation (§4.3 consolidated at design doc v1.2); HashMap-key retype atomic for six keys (§4.1). Each pattern earns its own §7 discipline-notes sub-section to give Clair the rationale at implementation time, not just the lock.
- **§7 nine sub-sections vs Pass 2's eight** — one additional sub-section absorbs format-boundary preservation as architectural pattern (§7.6).

Pass-internal-consistency framing per design doc §7.7 still applies: when Pass 3's structural novelty conflicts with Pass 2's lighter framing, Pass-internal consistency wins. The trilogy-internal ~80-100 KB target band is respected at mid-band; Pass 3 lands lighter than the trilogy precedent on grounds of design doc's pre-walk surface enumeration doing the architectural work.

### §1.3 What this runbook does NOT do

- Does NOT touch xgen-client at Pass 3. ClientSenders + FederationPeerSenders Pass 3 scope per design doc §4.5 is xgen-node-internal (mpsc::Sender channels never cross to xgen-client); xgen-client retypes happen at Pass 4.
- Does NOT retype the deferred items at design doc §4.5 Pass 4 + Pass 5 scope flags.
- Does NOT modify the seven-surface enumeration locked at design doc §2 Q-tables. If Clair surfaces a structural gap mid-implementation, STOP per Rule 3 + Lock 1 Trigger (a) and surface for Joe-lock before continuing. Any deviation from the verbatim surface list requires Joe-lock checkpoint #2 re-approval.
- Does NOT amend DECISIONS.md at Pass 3 milestone close. Three candidate D-NNNs flagged at design doc §4 + §7 stay flagged-not-promoted per D-069 (γ at 2 instances; δ at 2 instances pending Pass 4 client-side instantiation; ε at 3 same-surface instances pending Pass 4 structurally-different fourth).

---

## §2 Sequence overview

### §2.1 Two-commit base + contingent Commit 2a + milestone close

| Commit | Scope | Files (target count) | Atomic posture |
|--------|-------|----------------------|----------------|
| 1 | Doc-pass minimal (Option C hybrid) | 3 | D-074 atomic |
| 2 | Seven-surface retype + per-surface tests | 8-12 | D-074 atomic |
| 2a | [CONTINGENT] Test-fixture projection sweep | varies | D-074 atomic |
| 3 | Milestone close | 5-6 | D-074 atomic |

Sibling-shape to Pass 2's three-commit shape with Pass 3 extensions:

- **Commit 1 minimal (Option C)** vs Pass 2's substantive Commit 1 — Pass 3 design doc already at v1.2 COMPLETED at J-127; design doc Status flip is absence-of-need at this Pass. ROADMAP + CLAUDE PLAY + JOURNAL chain-only.
- **Commit 2 seven surfaces atomic** vs Pass 2's five — all surfaces atomic per design doc §2 Q-tables to preserve drift surface uniformity per D-067.
- **Commit 2a [CONTINGENT]** pre-locked posture per Pass 2 §7.3 precedent.
- **Commit 3 milestone close** standard D-074 atomic shape per J-108 codification.

### §2.2 Three split triggers (Lock 1 enumeration)

Three triggers documented at this §2.2 mirror Pass 2's pre-locked contingent-split posture. Each trigger fires Joe-lock STOP per Rule 3 + Lock 1.

- **Trigger (a)** — non-existent production contract per design doc §2 Q-tables. If Clair grep at Commit 2 prep finds a named type or method does not exist in production code (sibling-shape to J-113 + J-115 + J-116 canonical-record-staleness pattern at the federation-survey arc), STOP and surface for Joe-lock canonical-record amendment. **D-078 applies** — production-grounded verification at Joe-lock checkpoint #2 BEFORE any code touches.
- **Trigger (b)** — harness extension beyond design doc §4.6 fanout-already-covered scope. If Clair finds an additional fanout-side slot not covered at Pass 1's `Option<EventXgid>` retype, STOP and surface for Joe-lock. Design doc §4.6 anchors fanout as already-Pass-1-complete; any extension to fanout at Pass 3 is structurally novel.
- **Trigger (c)** — family-boundary size split if Commit 2 alone exceeds ~600 lines OR any single surface exceeds ~400 lines. Family-boundary not arbitrary line count; sibling-shape to J-111 retrospective 3b-3-pre + 3b-3 split pattern. Split candidate boundaries: Surface #2 (six HashMap keys) standalone if it dominates the commit; Surfaces #3 + #5 (async-spawned forced-owned family) as one sub-commit if isolated.

### §2.3 Three Joe-lock checkpoints

- **Checkpoint #1 — post-Commit-1 doc-pass drift check.** Three drift-detection points (vs Pass 2's four — Pass 3 design doc already COMPLETED so its Status flip is absent-by-design): ROADMAP version bump + visual tree row update + Past entry; CLAUDE PLAY flip; JOURNAL chain-only entry per J-123/J-124/J-125 doc-only milestone-event precedent. Joe approves before Commit 2 begins.
- **Checkpoint #2 — pre-Commit-2 verbatim surface list approval.** Clair extracts the seven-surface Q-tables from design doc §2 verbatim and surfaces them to Joe by name. Joe approves each surface by name before any production code lands. This is the LOAD-BEARING D-078 application surface; Trigger (a) fires here if any named type or method does not exist in production.
- **Checkpoint #3 — post-Commit-2 split-trigger decision.** Clair runs `cargo test -p xgen-common -p xgen-core -p xgen-node --tests` and reports test-fixture error count. Joe locks single-Commit-2 (absorb sweep) if errors ≤ ~50, or split (Commit 2 lib-clean + Commit 2a sweep atomic) if errors > ~50. Sibling-shape to Pass 2 checkpoint #3 split-trigger which fired at 93 errors.

---

## §3 Commit 1 — Doc-pass minimal (Option C hybrid)

### §3.1 Scope

Commit 1 ships the minimal doc-pass that reflects Pass 3 implementation kickoff at the canonical project surface (ROADMAP + CLAUDE PLAY). Design doc + this runbook stay untouched at Commit 1 because both are already at terminal Status (design doc COMPLETED v1.2; runbook ACTIVE v1.1 at v1.2-amend-time). **Two-file atomic per D-074** (post-J-129 strip-the-chain discipline + sibling-shape to J-123/J-124/J-125 chain-only doc-only milestone-event precedent: JOURNAL.md gets no edit at this atom). The v1.0/v1.1 framing called this a "three-file atomic" — that framing was authored pre-strip when a JOURNAL chain existed to append to; post-strip the chain doesn't exist, and per Joe-lock at J-131 Pre-Commit-1 the body is suppressed too, so the JOURNAL.md "chain entry only" item maps to a no-op. Corrected at v1.2 per Joe-lock at J-131 checkpoint #1 + Path (iii) amend-in-place; see §9.4 v1.2 amendment provenance.

### §3.2 Files in this commit

1. `docs/ROADMAP.md` — version bump (Clair verifies current version at session open and bumps accordingly; J-131 ship was v1.39 → v1.40 against post-J-129 ROADMAP state); visual structure tree Pass 3 Implementation row update; Present section flipped to "Pass 3 implementation Commit 1 doc-pass ✅; Clair pickup at runbook §4 Commit 2 next"; Past section gains Commit 1 entry; header date bump.
2. `CLAUDE.md` — header date bump; PLAY block flip from "XGID Retrofit Pass 3 implementation ACTIVE — Clair pickup at runbook §3 Commit 1" → "XGID Retrofit Pass 3 implementation ACTIVE — Clair pickup at runbook §4 Commit 2 (Commit 1 doc-pass ✅)".
3. `JOURNAL.md` NOT amended post-strip (strip-the-chain discipline per JOURNAL J-129 Sub-section 8 makes a chain-only entry a no-op; Commit 1 doc-pass is a two-file atomic: ROADMAP + CLAUDE PLAY). Sibling-shape to J-123/J-124/J-125 chain-only doc-only milestone-event precedent under post-strip discipline. **Pre-J-129 framing**: v1.0/v1.1 prescribed "JOURNAL.md chain entry only" assuming a chain existed to append to; post-strip the chain doesn't exist, and per Joe-lock at J-131 Pre-Commit-1 the body is suppressed too, so the original third-file item maps to a no-op. Corrected at v1.2 per Joe-lock at J-131 checkpoint #1 + Path (iii) amend-in-place; see §9.4 v1.2 amendment provenance.

### §3.3 Drift-detection points (3 of 4 vs Pass 2)

Joe-lock checkpoint #1 verifies these three points landed atomically:

1. ROADMAP version bump + visual tree row update ✅
2. CLAUDE PLAY flip ✅
3. JOURNAL chain entry ✅

The absent fourth point (design doc Status flip) is absence-of-need, not absence-of-discipline — design doc is already at v1.2 COMPLETED. Honest framing per D-065.

### §3.4 Verification at Commit 1 boundary

`cargo test -p xgen-common -p xgen-core` — should match J-126 baseline of 491 tests (no code changes at Commit 1; verification is sanity-check only that nothing slipped between J-127 design close and this commit).

`grep -rn 'J-NNN' . --include='*.rs' --include='docs/*.md' --include='tasks/*.md'` — should return ZERO matches post-staging per J-108 codification. Design doc §6.1 implementation-J-NNN placeholder gets frozen at Commit 3 milestone close, not at Commit 1.

---

## §4 Commit 2 — Seven-surface retype + per-surface tests (atomic)

### §4.1 Scope

Commit 2 lands all seven surfaces from design doc §2 Q-tables atomically. Per design doc §2.x Q-tables (verbatim ordering preserved at v1.1):

- **Surface #1** — `NodeRuntime` six per-space HashMap keys at `xgen-core/src/node/runtime.rs` (design doc §2.1) — `HashMap<SpaceXgid, _>` shape per design doc §4.1; field types + helper signatures + public-API parameters retype atomically.
- **Surface #2** — `dispatch_event` `peer_node_id: Option<&NodeXgid>` borrowed boundary at `xgen-core/src/node/runtime.rs` (design doc §2.2) per design doc §4.2.
- **Surface #3** — `federation_session.rs` handler identifier slots at `xgen-node/src/federation_session.rs` (design doc §2.3). Wire-format vs in-memory split surfaced §4.3 wire-format boundary at v1.1 of design doc walk.
- **Surface #4** — `fanout.rs` handler identifier slots at `xgen-node/src/fanout.rs` (design doc §2.4). Verification scope only that Pass 1's `Option<EventXgid>` retype at `fanout.rs:193` still applies cleanly per design doc §4.6; no new code at the topo-sort slot.
- **Surface #5** — `app.rs` handler identifier slots at `xgen-node/src/app.rs` (design doc §2.5). Twelve identifier slots in-memory retype + four slots at persistence-format boundary stay String per §4.3 v1.2 extension (filesystem path generation + on-disk JSON HashMap + `replay_spaces_from_dir` + wire-message destructure). Includes `handle_federation_incoming` at `xgen-node/src/app.rs:976` which captures `home_node_id: String` across `tokio::spawn` boundary (forced-owned per design doc §4.2 v1.2 row 3 async-spawned-task-captures sub-rule).
- **Surface #6** — Reconnect scheduler identifiers at `xgen-node/src/reconnect.rs` (design doc §2.6). Three async-spawned function signatures (`spawn_reconnect_scheduler` line 71 + `scheduler_tick` line 112 + `attempt_reconnect` line 227) take typed owned parameters per design doc §4.2 v1.2 row 3; D-NNN-ε promotion-watch.
- **Surface #7** — Appendix D doc-tree sweep at `docs/xgen_appendix_d_en.md` (design doc §2.7) — four markdown table hits per design doc §7.5; mechanical edit of doc-tree classification rows.

**v1.0 → v1.1 amendment provenance**: v1.0 swapped Surfaces #1↔#2 (presented `dispatch_event` first, then HashMap keys) and #5↔#6 (presented reconnect.rs as Surface #5 and app.rs as Surface #6); also placed `handle_federation_incoming` at `federation_session.rs` instead of `app.rs`. Re-aligned to design doc §2 verbatim at v1.1 per J-129 Track 1 canonical-record amendment. See §9 amendment-provenance.

### §4.2 Narrow scope clarifications

**What Surface #1 retype atomic means.** All three layers retype in same commit per drift surface uniformity (D-067):
- Field types on NodeRuntime struct (six `HashMap<SpaceXgid, _>` fields).
- Helper method signatures that read/write these maps.
- Public API parameters that callers pass through.

Mid-implementation single-layer retype would create a drift surface where field type and helper signature disagree on key shape; D-067 forbids this. All three layers atomic or none.

**What Surface #4 verification means.** Pass 1 Commit 3 retyped `topological_sort_events` parameter at `xgen-node/src/fanout.rs:193` from `&str` slot to `Option<EventXgid>`. Pass 3 Surface #4 confirms this Pass 1 work is intact and projects cleanly under the Pass 3 surrounding retypes. If grep surfaces an unanticipated slot — STOP per Trigger (b), surface for Joe-lock.

**What Surface #5 persistence-format boundary means.** Filesystem path generation (`spaces_dir/<space_xgid>/...`) writes String byte-representation to disk; on-disk JSON HashMap serialises key bytes; `replay_spaces_from_dir` reads back as String; wire-message destructure reads String fields from incoming envelopes. All four sub-surfaces stay String by construction at the I/O byte-serialisation boundary per design doc §4.3 v1.2 consolidation. **Format-boundary preservation rule**: if a slot crosses the disk-serialise boundary or the wire-serialise boundary in either direction, it stays String. Typed XGIDs project to/from String via `Borrow<str>` at call-site only; never at the byte-serialise layer.

**What Surface #5 `handle_federation_incoming` async-spawned means.** `handle_federation_incoming` is defined at `xgen-node/src/app.rs:976` (private to app.rs; called from line 738 inside `App::run`). It captures `home_node_id: String` across the `tokio::spawn` boundary; the `'static` bound forces owned values per design doc §4.2 v1.2 row 3. Verbatim location verified against production at J-129 audit.

### §4.3 Pass 1 carry-over verification

Before Commit 2 begins, Clair verifies these Pass 1 carry-overs are intact at the audit-cleanliness check (sibling-shape to J-120 + J-125 six-dimension audit pattern):

- `Borrow<str>` on `Xgid` + all six flavour wrappers — intact.
- Inline `// Pass 3 widens this method to take typed XGIDs` markers across xgen-node call sites — Pass 3-specific markers should be discoverable at grep. If markers absent, that's a discipline data point (Pass 1's pre-walk reconnaissance flagged Pass 2 surfaces but not Pass 3); flag at JOURNAL J-NNN body if absent.
- `cargo build -p xgen-common -p xgen-core` clean per Path A inherited from Pass 1.

### §4.4 Path A reminder — workspace build deliberately broken

`cargo build --workspace` will remain broken at Pass 3 Commit 2 close because xgen-client + xgen-node consumers depend on types that have not yet been retyped at the consumer call sites. Pass 4 closes the xgen-client + xgen-node consumer-side gap; Pass 5 close restores `cargo build --workspace` clean.

Honest framing per D-065: this is deliberate, not regression. Verification at Commit 2 boundary uses package-scoped `cargo test -p xgen-common -p xgen-core -p xgen-node` per Path A.

### §4.5 Async-spawned forced-owned at Surfaces #5 + #6

Surfaces #5 + #6 instantiate the design doc §4.2 v1.2 row 3 sub-rule: async-spawned task captures force owned parameters. The rule is a Tokio language idiom (the `'static` bound on `tokio::spawn` closures requires owned values to cross the spawn boundary), not a XGen-specific call.

Four instances at Pass 3:
1. Surface #5 — `handle_federation_incoming` at `xgen-node/src/app.rs:976` spawns task that captures `home_node_id: String` (forced-owned post-retype: owned `NodeXgid`).
2. Surface #6 (i) — `spawn_reconnect_scheduler` at `xgen-node/src/reconnect.rs:71` (spawned-function parameter, forced-owned).
3. Surface #6 (ii) — `scheduler_tick` at `xgen-node/src/reconnect.rs:112` (spawned-function parameter, forced-owned).
4. Surface #6 (iii) — `attempt_reconnect` at `xgen-node/src/reconnect.rs:227` (spawned-function parameter, forced-owned).

Four instances at the same module-family surface (xgen-node async task spawns) is still weaker durability evidence than instances across structurally different surfaces per D-077 + D-078 surface-diversity framing. **D-NNN-ε flagged-not-promoted** at design doc §7.2 + this runbook §7.5; promotion-watch opens at Pass 4 if a structurally different fourth-family instance surfaces at xgen-client async surfaces (Tauri commands, AI service spawns, batch dispatcher workers).

### §4.6 Format-boundary preservation at Surface #5

Surface #5 instantiates the design doc §4.3 v1.2 consolidated decision: if a slot crosses the I/O byte-serialisation boundary in either direction (wire OR persistence), it stays String. Typed XGIDs project to/from String via `Borrow<str>` at call-site only; never at the byte-serialise layer.

Two sub-instances at Surface #5:
1. Surface #5 (persistence side) — filesystem path generation + on-disk JSON HashMap (`load_space_local_metadata` + `save_space_local_metadata` round-trip at `xgen-node/src/app.rs`).
2. Surface #5 (wire side) — `IdentityReplicateMessage::Replicate { identity_id, ... }` destructured wire-message field at the receive path.

Sibling-shape to D-076 v1 → v1.1 amend-in-place pattern (one decision, two layers, one decision-surface). **D-NNN-δ flagged-not-promoted** at design doc §4.3 + this runbook §7.6; three-instance threshold opens at Pass 4 if a client-side serialisation-format slot instantiates (Tauri IPC, AI control protocol over HTTP, gRPC).

### §4.7 Per-surface tests — heavy enumeration target (+10)

**Joe-lock checkpoint #2 includes per-surface test list approval by name.** Clair extracts the test names below verbatim and surfaces them to Joe BEFORE any code touches. Test naming follows Pass 2 §4.7 precedent (`<surface>_<flavour>_<scenario>`). Surface numbering per design doc §2 verbatim (v1.1 corrected).

**Surface #1 — NodeRuntime six per-space HashMap keys** (3 tests target):
- `noderuntime_per_space_map_insert_retrieve_with_typed_key` — round-trip test: insert with typed `SpaceXgid` key + retrieve via `Borrow<str>` projection + verify hash-consistency at boundary.
- `noderuntime_per_space_map_six_flavours_isolated` — verify all six maps accept their respective key flavours independently without cross-flavour leak.
- `noderuntime_per_space_map_helper_signatures_typed_at_boundary` — verify helper method signatures expose typed keys at public boundary while internal storage stays `Borrow<str>`-compatible.

**Surface #2 — `dispatch_event` `Option<&NodeXgid>`** (1 test target):
- `dispatch_event_with_borrowed_node_xgid_projects_to_str_at_callsite` — verify borrowed boundary projects cleanly under both `Some(&NodeXgid)` and `None`.

**Surface #3 — federation_session.rs handler identifier slots** (1 test target):
- `federation_session_handler_identifier_slots_retyped_at_boundary` — verify in-memory identifier slots retype cleanly while wire-format String boundary stays preserved per design doc §4.3 v1.1 (wire) framing.

**Surface #4 — fanout.rs verification** (1 test target):
- `fanout_topological_sort_event_xgid_slot_pass_1_intact` — sentinel regression test that Pass 1's `Option<EventXgid>` retype at fanout.rs:193 still projects from the typed slot under Pass 3 surrounding retypes; sibling-shape to Pass 2 sentinel-tree precedent at Phase 9 §3b-1.

**Surface #5 — app.rs handler identifier slots + handle_federation_incoming + run_federation_session_post_handshake + persistence-format boundary** (3 tests target at v1.3; +1 from v1.2's 2 per J-135 D-078 application):
- `app_handlers_persistence_format_round_trip_string_at_boundary` — round-trip test: write JSON HashMap with String keys → read back via `replay_spaces_from_dir` → verify String keys project cleanly to typed XGIDs at consumption layer. Covers both persistence-format (§4.6) and wire-format (`IdentityReplicateMessage::Replicate` destructure) sub-instances.
- `handle_federation_incoming_spawned_task_owns_node_xgid_capture` — verify forced-owned `NodeXgid` parameter compiles + behaves correctly across `tokio::spawn` boundary at `xgen-node/src/app.rs:976`; uses `Arc<NodeXgid>` if shared reference needed inside spawn body. Covers the §2.5 sub-region 1 "Wire-format handlers" async-spawn pattern.
- `run_federation_session_post_handshake_spawned_task_owns_typed_captures` (T11, added at v1.3 per J-135 D-078 application) — verify the bilateral federation session driver's three identifier-shaped slots retype correctly across the spawn boundary: `home_node_id: String` → owned `NodeXgid` (app.rs:1161), `peer_node_id: String` → owned `NodeXgid` (app.rs:1165), `peer_shared_spaces: Vec<String>` → `Vec<SpaceXgid>` (app.rs:1169). Function signature anchor `pub(crate) async fn run_federation_session_post_handshake<S>(...)` at app.rs:1152 verified at J-135 author-time per D-078 (signature unchanged since J-133 Q5.14 v1.3 lock). Covers the §2.5 sub-region 2 "Top-level orchestrators" async-spawn pattern (distinct sub-region from T8's wire-format handler coverage). Test ensures the per-parameter retype matrix Q5.14 v1.3 enumerates lands correctly under both Initiator + Receiver `SessionRole` variants; descriptive-string slots (`session_id`, `neg_version`, `serial`) and wire-format-boundary slots (`peer_tips: BTreeMap<String, String>`) verified NOT-retyped per §4.3 + §5.4 rules.

**Surface #6 — reconnect.rs three spawned functions** (2 tests target):
- `reconnect_spawned_functions_each_own_typed_capture` — verify all three spawned functions (`spawn_reconnect_scheduler` + `scheduler_tick` + `attempt_reconnect`) accept forced-owned typed parameters; covers all three instances atomically.
- `reconnect_spawned_functions_arc_shared_reference_pattern_when_needed` — verify `Arc<TypedXgid>` shared-reference pattern works for spawned-task captures that need read-only access to the same typed value across multiple spawned tasks.

**Surface #7 — Appendix D doc-tree sweep** (0 tests target):
- Doc-only edit; no test required. Mechanical edit of four markdown table classification rows.

**Total per-surface test target: 11 tests at v1.3** (+11 vs J-126 baseline of 491; new total 502 if all land). Distribution per Surface: #1 (3) + #2 (1) + #3 (1) + #4 (1) + #5 (3) + #6 (2) + #7 (0) = 11. **v1.2 → v1.3 amendment at J-135**: Surface #5 count 2 → 3 (T11 `run_federation_session_post_handshake_spawned_task_owns_typed_captures` added per D-078 application — see §9 v1.3 amendment-provenance). T11 APPROVED by name at Joe-lock checkpoint #2 follow-on session-time turn, conditional on production anchor holding at author-time; anchor verified clean (app.rs:1152-1171 signature intact since J-133 Q5.14 v1.3 lock).

### §4.8 Layered-B3 audit at Commit 2 verification

Per design doc §5.5: layered-B3 confirmed expected-null at full seven-surface scope. Pass-arc pattern's durability at three instances (Pass 1 J-122 + Pass 2 J-126 + Pass 3 J-127 design close) makes the expected-null finding evidence-grounded.

However, runbook §4.8 still requires Clair to perform the layered-B3 audit at Commit 2 verification — honesty over assumption, sibling-shape to Pass 2 §5.3 + §6.7 DoD framing. If a layered-B3 surface unexpectedly emerges at implementation time, STOP per Rule 3 and surface for Joe-lock; flag at JOURNAL J-NNN body as discipline data point breaking the Pass-arc pattern.

### §4.9 Verification rigour at Commit 2 milestone-bearing boundary

**8 GREEN runs minimum** sibling-shape to Pass 2 §4.9 + topo-sort J-101 + persistence-amendment J-108:

- 5 isolated runs (`cargo clean -p xgen-common -p xgen-core -p xgen-node` between each).
- 3 consecutive workspace runs of `cargo test -p xgen-common -p xgen-core -p xgen-node`.
- `cargo clippy -p xgen-common -p xgen-core -p xgen-node --lib --all-features -- -D warnings` clean.
- `cargo clippy -p xgen-common -p xgen-core -p xgen-node --tests --all-features -- -D warnings` clean.
- `cargo build --workspace` deliberately broken per Path A; verify the breakage is at xgen-client consumer sites + xgen-node un-retyped consumer sites only, NOT at the seven retyped surfaces.

If pre-existing flakes fire (precedence env-var race; `reconnect_with_existing_tip_small_delta_delivered`), document but do not block per J-101 framing.

### §4.10 Files in this commit

Target 8-12 files atomic per D-074. File-to-Surface mapping per design doc §2 verbatim (v1.1 corrected):

1. `xgen-core/src/node/runtime.rs` — Surface #1 (six per-space HashMap keys) + Surface #2 (dispatch_event peer_node_id borrowed boundary) retypes.
2. `xgen-node/src/federation_session.rs` — Surface #3 (handler identifier slots) retypes.
3. `xgen-node/src/fanout.rs` — Surface #4 verification (likely 0 code changes; doc-comment confirmation only).
4. `xgen-node/src/app.rs` — Surface #5 (handler identifier slots + `handle_federation_incoming` async-spawned at line 976 + persistence-format boundary) retypes.
5. `xgen-node/src/reconnect.rs` — Surface #6 (three spawned functions: `spawn_reconnect_scheduler` line 71 + `scheduler_tick` line 112 + `attempt_reconnect` line 227) retypes.
6. `docs/xgen_appendix_d_en.md` — Surface #7 doc-tree sweep (four markdown table rows).
7. Per-surface test modules — possibly in-place at each surface file's `#[cfg(test)] mod tests` block (Surfaces #1, #2, #3, #4) or in dedicated test modules at `xgen-node/src/tests/` (Surfaces #5, #6 — async-spawned + integration-style tests).
8. `tasks/XGID_RETROFIT_PASS_3_IMPL.md` — header chain entry recording Commit 2 landed (Status stays ACTIVE v1.1).
9. `JOURNAL.md` — J-NNN body entry per D-074 + Lock #3 per-commit cadence.
10. `docs/ROADMAP.md` — version bump + visual tree row + Past entry + header chain.
11. `CLAUDE.md` — header chain entry; PLAY block flip from "Clair pickup at runbook §4 Commit 2" → "Clair pickup at runbook §6 Commit 3 (Commit 2 ✅)" or "Clair pickup at runbook §5 Commit 2a [CONTINGENT]" per checkpoint #3 split decision.

Additional files at Commit 2a [CONTINGENT] if checkpoint #3 split fires: per-test-module sweep across xgen-common + xgen-core + xgen-node test fixtures; counts vary.

---

## §5 Commit 2a — Test-fixture projection sweep [CONTINGENT]

### §5.1 Fires at Joe-lock checkpoint #3 if error count > ~50

Sibling-shape to Pass 2 Commit 2a `58b94a5` (which fired at 93 errors) + Pass 1 Commit 4a `4895446` precedent.

Clair runs `cargo test -p xgen-common -p xgen-core -p xgen-node --tests` after Commit 2 lib-clean verification at §4.9 GREEN. Reports test-fixture error count to Joe. Joe locks:

- **Single-Commit-2 (absorb sweep)** if errors ≤ ~50 — absorb test-fixture updates into Commit 2 itself; no Commit 2a.
- **Split (Commit 2a)** if errors > ~50 — separate atomic commit for test-fixture projection sweep per D-074 preservation of atomic discipline + Pass 1 + Pass 2 precedent.

### §5.2 Scope if fires

Mechanical projection-only edit across test fixtures that construct typed XGIDs. Pattern at Pass 2:

```rust
// BEFORE: untyped String construction
let node_id = "node_a_xgid".to_string();

// AFTER: typed construction
let node_id = NodeXgid::from_str("node_a_xgid").unwrap();
// OR: helper function that hides projection
let node_id = ndx("node_a_xgid");
```

Pass 2 introduced `ndx` helper at xgen-core/src/message/exchange.rs (`#[cfg(test)] mod tests` block) to keep test fixture construction concise. Pass 3 may inherit this helper or add sibling helpers per Clair's judgment at Commit 2a implementation time.

### §5.3 Verification at Commit 2a boundary

Re-run 8 GREEN protocol per §4.9 after sweep lands. Verify total test count matches Commit 2's per-surface test additions (+10 target if all land) on top of J-126 baseline of 491 + any per-surface tests added at Commit 2.

### §5.4 Files in this commit if fires

Per-test-module sweep target 5-15 files at xgen-common + xgen-core + xgen-node + integration test modules. Sibling-shape to Pass 2 Commit 2a's nine xgen-core test modules + Pass 1 Commit 4a's broader sweep.

Additional D-074 atomic files:
- `tasks/XGID_RETROFIT_PASS_3_IMPL.md` — header chain entry recording Commit 2a landed.
- `JOURNAL.md` — J-NNN body entry per D-074 + Lock #3 per-commit cadence.
- `docs/ROADMAP.md` — version bump + visual tree row + Past entry + header chain.
- `CLAUDE.md` — header chain entry; PLAY block flip "Clair pickup at runbook §6 Commit 3 (Commit 2 + 2a ✅)".

---

## §6 Commit 3 — Milestone close

### §6.1 Scope

Pass 3 milestone close per D-074 atomic + J-108 codification. Five-to-six file atomic commit. Sibling-shape to Pass 2 Commit 3 milestone-close `0bdb0b8`.

### §6.2 Files in this commit

1. `tasks/XGID_RETROFIT_PASS_3_IMPL.md` — Status ACTIVE → COMPLETED + version bump v1.0 → v1.1 + Last-updated milestone-close note + DoD checklist verified.
2. `tasks/XGID_RETROFIT_PASS_3_DESIGN.md` — header chain entry only + §6.1 J-NNN placeholder freeze (per J-108 codification + Pass 2 §6.7 freeze pattern).
3. `JOURNAL.md` — J-NNN body entry with full milestone-close pattern per HANDOFF/precedent spec (target eight to ten sub-sections sibling-shape to J-122 + J-126).
4. `CLAUDE.md` — header chain entry; PLAY block flip "XGID Retrofit Pass 3 implementation ACTIVE — Clair pickup at runbook §6 Commit 3" → "XGID Retrofit Pass 3 milestone CLOSED at J-NNN; standby for next-milestone selection (Pass 4 + M6 (new) both ready)".
5. `docs/ROADMAP.md` — version bump + visual tree Pass 3 row 🟢 → ✅ with full sub-bullet detail + Past entry + Present updated + Near future Pass 3 line removed + header chain.

Possibly sixth file: any code-side J-NNN code-comment freezes per J-108 codification grep guardrail (`grep -rn 'J-NNN' . --include='*.rs' --include='docs/*.md' --include='tasks/*.md'` returns ZERO post-staging).

### §6.3 Verification at milestone close

- `cargo test -p xgen-common -p xgen-core -p xgen-node` — final test count recorded in JOURNAL J-NNN body entry.
- `cargo clippy -p xgen-common -p xgen-core -p xgen-node --lib --all-features -- -D warnings` clean.
- `cargo clippy -p xgen-common -p xgen-core -p xgen-node --tests --all-features -- -D warnings` clean.
- `cargo build --workspace` deliberately broken per Path A; verify breakage is at xgen-client consumer sites + xgen-node un-retyped consumer sites only.
- `grep -rn 'J-NNN' . --include='*.rs' --include='docs/*.md' --include='tasks/*.md'` returns ZERO matches post-staging per J-108 codification.

### §6.4 What unblocks

- **XGID Retrofit Pass 4** — xgen-client consumer-side retypes. Runbook authoring is the next Chat Claude work-shape on the XGID retrofit track after Pass 3 close.
- **M6 (new) Node admin write path** — stays unblocked-but-not-selected; opens after Joe selects the next-active milestone at session open. Pass 4 + M6 (new) are both ready for selection; sequencing is Joe's call.

### §6.5 Definition of Done

DoD checklist for milestone close — Clair verifies each before staging:

- [ ] All seven surfaces from design doc §2 Q-tables retyped per locked decisions at design doc §4.
- [ ] Per-surface tests landed (target +10 unless Joe locked different count at checkpoint #2).
- [ ] `cargo test -p xgen-common -p xgen-core -p xgen-node` GREEN (8/8 minimum at Commit 2 boundary; re-verified at Commit 3).
- [ ] Both clippy gates clean (`--lib` + `--tests`, `-D warnings`).
- [ ] `cargo build --workspace` deliberately broken at xgen-client consumer sites only (no regression at xgen-common + xgen-core + xgen-node).
- [ ] Layered-B3 audit answer recorded in JOURNAL J-NNN body (expected null per design doc §5.5 + Pass-arc pattern durability; flag at JOURNAL if surface unexpectedly emerges).
- [ ] Design doc §6.1 J-NNN placeholder frozen to milestone-close J-NNN per J-108 codification.
- [ ] `grep -rn 'J-NNN' . --include='*.rs' --include='docs/*.md' --include='tasks/*.md'` returns ZERO matches post-staging.
- [ ] Three candidate D-NNNs status recorded in JOURNAL J-NNN body (γ promotion-watch + δ promotion-watch + ε promotion-watch).
- [ ] "Honest longer work over fast shortcuts" Pass 3 final count recorded in JOURNAL J-NNN body (target zero per Pass 2 milestone-close precedent).
- [ ] D-074 application count incremented (twenty-fifth at runbook ship at J-128 + per-commit increments through milestone close; milestone-close tally fourteenth at this Commit 3).

### §6.6 What this commit does NOT do

- Does NOT amend DECISIONS.md. Three candidate D-NNNs stay flagged-not-promoted per D-069 (γ at 2 instances; δ at 2 instances; ε at 3 same-surface instances).
- Does NOT touch xgen-client. xgen-client consumer-side retypes happen at Pass 4.
- Does NOT close the D-071 future-removal arc for `validate_steps_8_13` + `accept_event` (Pass 2 §4.2 Q5.b deprecation attributes). That removal arc stays pending; surface-driven per D-071.
- Does NOT close the timestamp-bound validation Gap G6 from Phase 9 survey findings §4.6. Stays pending; surface-driven per D-071.

---

## §7 Discipline notes (nine sub-sections)

### §7.1 Precedent-departure self-defense

Pass 3's runbook is heavier than Pass 2 (~50-70 KB target vs Pass 2's ~43 KB). Pass-internal-consistency framing per design doc §7.7 justifies the heavier framing on three grounds:

1. Seven surfaces vs five — per-surface enumerations at §4.7 + §4.10 are longer by structural necessity.
2. Three structurally novel patterns (async-spawned forced-owned + persistence-format boundary + HashMap-key retype atomic) each earn §7 sub-sections for Clair's mid-implementation reference.
3. Two-session design walk at J-127 (morning + afternoon) is a Pass-internal precedent for "two-session split as deliberate scaffolding" per design doc §8(c); the runbook records this as future-walk discipline guidance at §7.7.

Pass-internal-consistency wins over trilogy-internal-consistency when they conflict per design doc §7.7.

### §7.2 Pass-internal-consistency over trilogy-internal-consistency

When the five-Pass XGID Retrofit arc and the audit-design-impl trilogy precedents conflict on shape, Pass-internal consistency wins. Examples at Pass 3:

- Runbook size heavier than Pass 2 but lighter than trilogy (~50-70 KB vs trilogy's ~80-100 KB) — Pass-internal-consistency framing accepts mid-band.
- Design doc §7 nine sub-sections vs Pass 2's eight + trilogy's ten-to-twelve — Pass-internal-consistency framing accepts mid-band.
- Joe-lock checkpoints three vs Pass 2's three + trilogy's five — same count as Pass 2 per Pass-arc inheritance, not trilogy escalation.

### §7.3 Contingent-split honesty

Commit 2a [CONTINGENT] split posture is pre-locked at this runbook §5.1 rather than emerging mid-implementation per Pass 2 precedent. Honest framing per D-065: the split-trigger criterion (~50 errors at checkpoint #3) is the same Pass 2 used; the criterion is empirically grounded at two prior milestone closes (Pass 1 Commit 4a + Pass 2 Commit 2a).

If checkpoint #3 fires single-Commit-2 (errors ≤ ~50), that's also valid — pre-locking the criterion does not mandate the split outcome; it pre-locks the decision protocol.

### §7.4 `Borrow<str>` load-bearing at HashMap-key retype boundary

Pass 1 Commit 4 introduced `Borrow<str>` additive API on `Xgid` + all six flavour wrappers (J-122 Joe-lock). At Pass 3 Surface #2 (six per-space HashMap keys), `Borrow<str>` is load-bearing for the retype:

- `HashMap<SpaceXgid, V>::get(&str)` lookup works without per-query wrapper allocation.
- Hash-consistent with `&str` per std docs (derived `Hash` + `PartialEq` forward to inner `String` / `str`).
- Newtype's flavour discipline preserved (no `Deref<Target = str>`).

Without `Borrow<str>`, Surface #2 retype would require per-lookup-site explicit-wrap-with-comment churn at hundreds of lookup sites — Pass 1's additive API made the Pass 2 + Pass 3 + Pass 4 lookup-site work mechanically clean. Discipline data point for sibling milestone authors: Pass 1's additive API was the load-bearing enabling decision for the entire five-Pass arc.

### §7.5 Async-spawned task captures force owned parameters (Tokio idiom)

Surfaces #3 + #5 (four total instances) instantiate the Tokio `'static` bound on `tokio::spawn` closures. The rule is a Rust language fact, not a XGen-specific call.

Per design doc §7.2 + this runbook §4.5: D-NNN-ε flagged-not-promoted per D-069 honest framing — promoting a Tokio language idiom to DECISIONS.md would record a language fact rather than a project decision. Promotion-watch opens at Pass 4 surfacing a structurally different fourth instance at xgen-client async surfaces (Tauri commands, AI service spawns, batch dispatcher workers).

Clair implementation pattern at Surfaces #3 + #5:
- Captured XGID parameters declared as `NodeXgid` (owned) not `&NodeXgid` (borrowed) at the spawned-function signature.
- If shared read-only access needed across multiple spawned tasks, wrap in `Arc<NodeXgid>` and clone at spawn site.
- Pattern is mechanical; per-surface tests at §4.7 verify each surface compiles + behaves correctly.

### §7.6 Format-boundary preservation unified (wire + persistence)

Surface #6 (two instances — persistence + wire) instantiates the design doc §4.3 v1.2 consolidated decision: format-boundary preservation at I/O byte-serialisation boundary.

Sibling-shape to no-drift-surface discipline family (D-067 + D-070 + D-075 + D-076 v1.1) at the I/O-boundary layer. The principle in plain language: if a slot crosses the disk-serialise boundary or the wire-serialise boundary in either direction, it stays String. Typed XGIDs project to/from String via `Borrow<str>` at call-site only; never at the byte-serialise layer.

Per design doc §4.3 + this runbook §4.6: D-NNN-δ flagged-not-promoted per D-069 — three-instance threshold opens at Pass 4 if a client-side serialisation-format slot instantiates (Tauri IPC, AI control protocol over HTTP, gRPC).

### §7.7 "Honest longer work over fast shortcuts" — Pass 3 count at one as of J-129

Pass 2 closed with zero recurrences at J-126 (first project milestone since the framework was named to ship with zero). Pass 3 count started fresh at zero at J-127 design close + J-128 runbook authoring (both within-milestone, no recurrence surfaced).

**At J-129 the count increments to one.** First Pass 3 recurrence: runbook v1.0 (J-128) shipped with surface-ordering drift against design doc §2 — Surfaces #1↔#2 swapped + #5↔#6 swapped + `handle_federation_incoming` mis-located to `federation_session.rs` (production code at `xgen-node/src/app.rs:976`). Clair's pre-Clair six-dimension audit at session-open caught the drift as a Trigger (a) candidate before any code landed (sibling-shape to J-115 + J-116 prospective catches but at a distinct surface: J-115/J-116 were prospective catches at runbook-implementation layer; J-129 is prospective catch at runbook-authoring layer). Track 1 amendment in this session re-aligned the runbook to design doc §2 verbatim. The recurrence is real per D-065 honest framing; recording it as one, not zero.

Root cause: runbook §4 was authored at J-128 from the session-bridge summary's compressed surface list rather than fresh from design doc §2 verbatim. The bridge summary preserved the surface-set but compressed the ordering and crate-file mapping. The cross-check against design doc §2 verbatim was not performed before runbook ship. See §7.11 for the discipline data point.

Sibling-in-shape factors at Pass 3 that still favour the final count staying low:
- Design phase named layered-B3 expected-null in advance per §5.5.
- Runbook pre-locks contingent-split posture rather than mid-implementation Joe-lock.
- Pass 1's `Borrow<str>` additive API makes projection structurally cheap.
- Pass-internal-consistency framing respected throughout.
- Pre-Clair six-dimension audit fired prospectively at session-open and caught the J-128 drift before any code landed.

If further recurrences surface at Pass 3 implementation, that's an honest data point — flag at JOURNAL J-NNN body without softening per D-065.

### §7.8 Layered-B3 expected null per Pass-arc pattern durability

Per design doc §5.5: layered-B3 confirmed null at full seven-surface scope. Pass-arc pattern's durability at three instances (Pass 1 J-122 + Pass 2 J-126 + Pass 3 J-127 design close) makes the expected-null finding evidence-grounded.

Runbook §4.8 still requires Clair to perform the audit at Commit 2 verification — honesty over assumption. If layered-B3 unexpectedly emerges, flag at JOURNAL J-NNN body as discipline data point breaking the Pass-arc pattern.

The mechanism: identifier-slot retype scopes do not surface layered-B3 because the projection mechanism (`Borrow<str>`) handles type-projection at boundaries uniformly without forcing secondary encodings of the same invariant. This is the structural reason Pass-arc expects null; the empirical confirmation across three Pass-arc instances grounds the expectation.

### §7.9 D-069 audit-vs-design boundary

Three candidate D-NNNs at Pass 3 (γ + δ + ε) stay flagged-not-promoted per D-069 honest framing. The audit-vs-design boundary:

- **D-069 audit phase** identifies candidate principles by surfacing structural patterns at multiple instances.
- **D-069 design phase** locks the candidate's promotion threshold (three instances minimum across structurally different surfaces per D-077 + D-078 surface-diversity framing).
- **D-069 implementation phase** records the candidate's instance count + promotion-watch status at milestone close JOURNAL J-NNN body.

At Pass 3 milestone close:
- γ promotion-watch — 2 instances (Pass 2 + Pass 3); promotion-watch opens at Pass 4 if structurally similar third instance fires.
- δ promotion-watch — 2 instances at same module (Surface #6 persistence + wire); promotion-watch opens at Pass 4 if client-side serialisation-format slot instantiates.
- ε promotion-watch — 3 instances at same xgen-node module-family; promotion-watch opens at Pass 4 if structurally different fourth instance surfaces at xgen-client async surfaces.

### §7.10 [FUTURE-WALK CANDIDATE] Pass 5 consolidation of runbook §7 across the five-Pass arc

By Pass 5 milestone close, the five-Pass arc will have accumulated ~40-45 sub-sections of §7 discipline-notes across the per-Pass runbooks (Pass 1 § implicit + Pass 2's eight + Pass 3's nine + Pass 4's projected eight-to-ten + Pass 5's projected eight-to-ten). High redundancy across Passes — same `Borrow<str>` load-bearing note, same Pass-internal-consistency framing, same "honest longer work" counting, same D-069 audit-vs-design boundary, etc.

**Recommended action at Pass 5 milestone close**: consolidate §7 across all five runbooks into a single `docs/XGID_RETROFIT_DISCIPLINE.md` reference doc. Each per-Pass runbook's §7 becomes a 3-5 line pointer to the consolidated doc + per-Pass deltas only (the structural novelties specific to that Pass).

Benefits:
- Historical record preserved (per-Pass runbook §7s preserved unchanged via Git history).
- Single source of truth for cross-Pass discipline patterns (sibling-shape to DECISIONS.md as cross-cutting source of truth).
- Future Pass-style milestone runbook authoring overhead reduced (~30-40% size reduction at Pass 6+ if XGID retrofit family extends).

Recorded here at §7.10 as future-walk candidate per D-071 surface-driven application; promotion fires at Pass 5 milestone close if Joe locks the consolidation pattern.

### §7.11 [J-129 amendment] Cross-check runbook §4 against design doc §2 verbatim before ship

**Discipline data point recorded at J-129 Track 1 amendment.** When a runbook is authored across a session boundary from a session-bridge summary (rather than fresh from the design doc in the same session that closed the design), the surface enumeration at runbook §4 MUST be cross-checked against design doc §2 verbatim BEFORE the runbook ships. Session-bridge summaries can preserve the surface-set while compressing ordering, crate-file mapping, or naming. Each of those compressions is a drift surface.

Failure mode at J-128: runbook §4.1 authored from the bridge summary's compressed surface list. Three drifts surfaced:
- Surfaces #1↔#2 ordering swapped (bridge presented `dispatch_event` before HashMap keys; design doc §2.1 → §2.2 has HashMap keys first).
- Surfaces #5↔#6 ordering swapped (bridge presented reconnect.rs as Surface #5; design doc §2.5 has app.rs as #5 + §2.6 has reconnect as #6).
- Crate-file mapping wrong: `handle_federation_incoming` placed at `federation_session.rs`; production code at `xgen-node/src/app.rs:976`.

Clair's pre-Clair six-dimension audit at session-open caught all three drifts as Trigger (a) candidates before any code landed. Sibling-shape to D-078's prospective-catch framing but at a distinct surface layer: D-078 at promotion (J-114) was "production-grounded test enumeration at runbook checkpoints"; J-129 surfaces the same shape one layer up at "design-doc-grounded surface enumeration at runbook authoring."

**Discipline rule** (recorded for sibling milestone runbook authors): when runbook authoring crosses a session boundary from the design close, the first task at runbook §4 ship MUST be `read design doc §2 verbatim + re-state surface enumeration against it`. If the runbook authoring happens in the same session as the design close, the design doc §2 is already in context and the cross-check is implicit. The cross-session case is the failure mode.

Candidate D-NNN "design-doc-grounded surface enumeration at runbook authoring" flagged-not-promoted per D-069 (one instance at this J-129; three-instance threshold not met; may promote at Pass 4 milestone close if a sibling instance fires at Pass 4 or Pass 5 runbook authoring).

---

## §8 Cross-references

### §8.1 Design doc anchors

- `tasks/XGID_RETROFIT_PASS_3_DESIGN.md` COMPLETED v1.2 at J-127:
  - §2 Q-tables — seven-surface enumeration (LOAD-BEARING for §4.1 + §4.7 + checkpoint #2)
  - §3 Single governing principle (inherited from Pass 2 unchanged)
  - §4.1 Six per-space HashMap keys retype shape
  - §4.2 dispatch_event + sibling-shape rule table (v1.2 with async-spawned row 3)
  - §4.3 Format-boundary preservation (wire OR persistence) — v1.2 consolidated
  - §4.4 Forced-owned return shape rule
  - §4.5 ClientSenders + FederationPeerSenders Pass 3 scope
  - §4.6 Topo-sort &str slot at fanout.rs:193 already covered at Pass 1
  - §5.5 Layered-B3 confirmed null at full seven-surface scope
  - §6.1 Historical-pointer (Shape α, pointer-style)
  - §7 Discipline-notes five sub-sections

### §8.2 Pass-arc predecessor runbooks

- `tasks/XGID_RETROFIT_PASS_1_IMPL.md` COMPLETED v2.1 at J-122 (six-commit base; Pass 1 closed with one recurrence at J-121 hygiene atom).
- `tasks/XGID_RETROFIT_PASS_2_IMPL.md` COMPLETED v1.1 at J-126 (three-commit base; Pass 2 closed with zero recurrences — first project milestone since the framework was named).

### §8.3 Sibling-shape trilogy precedent

- `tasks/FEDERATION_TOPOSORT_IMPL.md` COMPLETED v1.2 at J-101 (trilogy precedent at ~93 KB).
- `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_IMPL.md` COMPLETED v1.2 at J-108 (trilogy precedent at ~95 KB).
- `tasks/FEDERATION_PROPAGATION_PHASE_9_COMMIT_3B_4_IMPL.md` COMPLETED v1.2 at J-119 (trilogy precedent at ~57.5 KB — light end of the band).

### §8.4 Cross-cutting principles applied at Pass 3

- **Rule 0** (CLAUDE.md) — mandatory session-open reading sequence; Clair reads CLAUDE PLAY block + JOURNAL latest entry + ACTIVE HANDOFF notes before runbook §4.
- **D-065** honest-behaviour-over-polite-behaviour at all framing decisions.
- **D-067** no-drift-surface code-organisation at Surface #2 atomic three-layer retype.
- **D-069** audit-vs-design boundary for three candidate D-NNNs flagged-not-promoted.
- **D-071** audit-precedes-dependent-design for future-removal arcs (D-NNN promotion-watch + D-071 deferred arcs).
- **D-074** atomic-commit discipline at all commits in this runbook (twenty-fifth instance at J-128; per-commit increments through milestone close; milestone-close tally fourteenth at Commit 3).
- **D-076 v1.1** one-principle-two-properties amend-in-place pattern (sibling-shape to §4.3 v1.2 consolidation).
- **D-077** backward-coherence cross-milestone amendment dependency.
- **D-078** production-grounded test enumeration at Joe-lock checkpoint #2.
- **Grep guardrail scope discipline** (J-108 codification) — `grep -rn 'J-NNN' . --include='*.rs' --include='docs/*.md' --include='tasks/*.md'` returns ZERO post-staging at Commit 3.

---

## §9 Footer — Authoring provenance

### §9.1 J-129 v1.0 → v1.1 amendment provenance (Track 1)

Runbook amended at J-129 (2026-05-27) by Chat Claude with Joe as a within-milestone Track 1 canonical-record amendment. Triggered by Clair's pre-Clair six-dimension audit at session-open which surfaced three drifts at runbook §4 against design doc §2 as Trigger (a) candidates per §2.2 (sibling-shape to J-115 + J-116 prospective-catch precedents but at a distinct surface layer; see §7.11).

Amendment scope:
- **§4.1** re-aligned to design doc §2 verbatim surface ordering (Surfaces #1↔#2 + #5↔#6 swapped at v1.0; corrected at v1.1).
- **§4.1 Surface #3** location stays at `federation_session.rs` per design doc §2.3 (Surface #3 IS the federation_session.rs handler identifier slots surface; the v1.0 error was placing `handle_federation_incoming` at Surface #3 when it actually lives at Surface #5 / app.rs).
- **§4.1 Surface #5** corrected to include `handle_federation_incoming` at `xgen-node/src/app.rs:976` per design doc §2.5 + §4.2 rule table line 277 verbatim.
- **§4.5** + **§4.7** Surface #5 text re-targeted to reconnect.rs (per re-alignment); Surface #5 async-spawned coverage clarified as `handle_federation_incoming` (one instance at app.rs) + reconnect three spawned functions (three instances at reconnect.rs) per design doc §4.2 v1.2 row 3.
- **§4.10** file-to-surface mapping re-aligned to design doc §2 verbatim.
- **§7.7** "Honest longer work over fast shortcuts" count incremented from zero to one (first Pass 3 recurrence — see §7.7 body for full root-cause record).
- **§7.11 NEW** discipline data point: cross-check runbook §4 against design doc §2 verbatim before ship; candidate D-NNN flagged-not-promoted per D-069.

Five-file atomic per D-074 (twenty-sixth instance) + Lock #3 per-commit cadence:
1. This runbook v1.0 → v1.1 (header chain + body amendments per scope above).
2. `docs/ROADMAP.md` v1.38 → v1.39 + visual tree row annotation refined + Past entry + header chain.
3. `CLAUDE.md` header chain entry; PLAY block stays substantively unchanged (Clair's pickup still at runbook §3 Commit 1, but against amended v1.1).
4. `JOURNAL.md` J-129 body entry + header chain.
5. `tasks/HANDOFF_TOPOSORT_RUNBOOK_AUTHORING.md` Status ACTIVE → COMPLETED v1.1 (stale flag fix — topo-sort closed at J-101; Status flag never flipped). Sibling-shape to J-107's eight-file expansion when bridge-handoff folded into atomic per anti-tempfile-deletion discipline; this stale-HANDOFF flip folded into atomic because Clair surfaced it during session-open Rule 0 sweep alongside the runbook drift.

**"Honest longer work over fast shortcuts" Pass 3 count at this commit: ONE** (first Pass 3 recurrence; recorded honestly per D-065).

**D-074 application count at this commit: twenty-sixth instance** (Lock #3 per-commit cadence; not a milestone-close so milestone-close tally — thirteenth at J-126 — does NOT increment).

### §9.2 J-128 v1.0 authoring provenance (original)

Runbook authored at J-128 (2026-05-27) by Chat Claude with Joe at design-close-plus-one session per Pass 2 + trilogy precedent. Sibling-in-shape to `tasks/XGID_RETROFIT_PASS_2_IMPL.md` COMPLETED v1.1 with three structural extensions for Pass 3's seven-surface scope:

1. §4.7 per-surface tests heavier (+10 target vs Pass 2's +2).
2. §7 nine sub-sections at v1.0 (Pass 2 had eight; Pass 3 added §7.6 format-boundary preservation unified architectural pattern + §7.10 future-walk consolidation flag; §7.11 added at v1.1 per J-129 amendment above).
3. §5 Commit 2a [CONTINGENT] section explicit (Pass 2 inline-referenced; Pass 3 elevates to own §5 for runbook navigability).

Joe-locks at runbook-authoring session (J-128):
- Option C minimal Commit 1 (no design-doc touch at Commit 1; sibling-shape to Pass 2 J-125 J-NNN doc-only milestone-event but absence-of-design-doc-touch since design doc already COMPLETED).
- Per-surface test target +10 (verbal lock at this authoring; Joe may adjust at checkpoint #2 approval).
- §7 nine sub-sections including §7.10 Pass 5 consolidation flag as future-walk candidate.

D-074 application count at v1.0 runbook ship: twenty-fifth instance.

### §9.3 Next-active (post-J-129)

**Next-active for Clair**: pickup at runbook §3 Commit 1 (doc-pass minimal) against amended v1.1. Read CLAUDE.md PLAY block + JOURNAL J-129 entry first per Rule 0, then this runbook §1-§3 in order, then design doc `tasks/XGID_RETROFIT_PASS_3_DESIGN.md` §2 Q-tables verbatim (Joe-lock checkpoint #2 requires verbatim surface list approval before any production code touches).

**Next-active for Chat Claude**: standby until Clair's Commit 1 closes affirmatively at Joe-lock checkpoint #1; parallel-eligible items include M6 (new) Block 4 verb-by-verb walks if Joe selects parallel-track work.

### §9.4 J-132 v1.1 → v1.2 amendment provenance (Path-(iii) amend-in-place)

Runbook amended at J-132 (2026-05-28) by Clair (Chat Claude in implementation role) with Joe as a within-milestone single-file canonical-record amendment. Triggered at Joe-lock checkpoint #1 of J-131 Commit 1 doc-pass when Clair surfaced the honest two-file-vs-three-file count discrepancy: the v1.0/v1.1 §3.2 prose prescribed three files including a JOURNAL.md chain entry, but post-J-129 strip-the-chain discipline + post-J-130 cleared canonical record + sibling-shape to J-123/J-124/J-125 chain-only doc-only milestone-event precedent collide to make the third-file item a no-op at execution time. Joe locked Path (iii) over Path (i) "mark obsolete but leave wrong prose" + Path (ii) "promote candidate D-NNN-ι 'post-strip-discipline meaning of chain entry only'" — reasoning: this is a stale-canonical-source question, not a discipline-promotion one; D-077 + D-078 were promoted exactly to prevent knowingly-stale-canonical-source anti-patterns; D-NNN-η + D-NNN-ζ stay flagged-not-promoted at current counts and neither absorbs this case.

Amendment scope:
- **§1 header** Version 1.1 → 1.2; `Last updated` chain prepend with J-132 entry; runbook header chain pattern retained per J-129 Sub-section 8 (strip-the-chain discipline applied to CLAUDE.md + JOURNAL.md + ROADMAP.md headers; runbook headers stayed chain-pattern; the question of strip-extension to runbook headers parked as discipline data point per J-130 Sub-section 7).
- **§3.1** Three-file atomic per D-074 → Two-file atomic per D-074; honest framing per D-065 + checkpoint #1 resolution.
- **§3.2 third-file line** rewritten from "JOURNAL.md — header chain entry only (NO body J-NNN entry, sibling-shape to J-123 + J-124 + J-125)" → "JOURNAL.md NOT amended post-strip (strip-the-chain discipline per JOURNAL J-129 Sub-section 8 makes a chain-only entry a no-op; Commit 1 doc-pass is a two-file atomic: ROADMAP + CLAUDE PLAY)". The pre-J-129 framing preserved as historical record + corrective text added pointing to §9.4 for future Pass-arc author reference.
- **§9.4 NEW** this sub-section.

**§3.3 carry-forward inconsistency parked for future Pass-arc author consideration.** §3.3 enumerates the three drift-detection points checkpoint #1 verifies, including "JOURNAL chain entry ✅" at point #3. Post-J-129 strip + Path (iii) lock makes this point a no-op-verified rather than a positive-change-verified. Joe-lock at J-132 explicitly scoped narrow to §3.1 + §3.2 + §9.4; §3.3 was not amended. Future Pass 4 + Pass 5 runbook authors encountering the same collision should consider sibling-amending §3.3 to symmetric framing — but not at J-132. Recorded as discipline data point only.

**Forward-looking doc-hygiene, not recurrence-of-a-mistake.** Per Joe-lock at J-132: "Honest longer work over fast shortcuts" Pass 3 count does NOT increment at v1.2. The v1.2 amendment is preventing future Pass 4 + Pass 5 runbook authors from re-encountering the same collision and re-deriving the same resolution. Sibling-shape to inherit-not-increment framing at J-128 runbook-landing.

**Sequence**: J-132 single-file atom shipped BEFORE Joe-lock checkpoint #2 (verbatim seven-surface extraction from design doc §2). A clean runbook at checkpoint-#2-read-time is worth one extra small push — checkpoint #2 reads runbook §4, but checkpoint-#2-prep reads include §1-§3 per Rule 0, so the v1.2 cleanup lands before Clair re-reads §1-§3 for the surface-extraction work.

**D-074 application count at this commit: twenty-ninth instance** (Lock #3 per-commit cadence; not a milestone-close so milestone-close tally — thirteenth at J-126 — does NOT increment).

**Single-file atom enumeration**:
1. This runbook v1.1 → v1.2 (header chain + §3.1 file-count fix + §3.2 third-file rewrite + this new §9.4).

CLAUDE.md NOT amended (entry-point stays Commit 2; sibling-shape to J-121 hygiene atom no-PLAY-touch). ROADMAP NOT amended (within-milestone doc-fix; sibling-shape to J-117 no-ROADMAP-touch framing). JOURNAL.md NOT amended (post-strip no-op — the very discipline being reconciled). DECISIONS.md NOT amended (no new principles; candidate D-NNN-ι does NOT open per Joe-lock framing this as stale-canonical-source not discipline-promotion).

### §9.5 J-135 v1.2 → v1.3 amendment provenance (D-078 application at test-enumeration layer)

Runbook amended at J-135 (2026-05-28) by Clair (Chat Claude in implementation role) with Joe as a Joe-lock-checkpoint-#2 follow-on at the test-enumeration layer. Triggered when Clair surfaced (during the post-J-134 by-name re-surfacing of all seven surfaces + all ten original tests for checkpoint #2 approval) that the +10 test enumeration authored at J-128 runbook v1.0 did NOT include a pinning test for `run_federation_session_post_handshake` — which became a named Pass 3 retype target at J-133's Q5.14 v1.3 rewrite, **after the original +10 enumeration locked**.

**D-078 application** (test enumeration grounded against production reject-paths / API surfaces being retyped): T8 (`handle_federation_incoming_spawned_task_owns_node_xgid_capture`) pins the async-spawned forced-owned `NodeXgid` capture-shape at the `handle_federation_incoming` wire-format handler (app.rs:976, §2.5 sub-region 1). `run_federation_session_post_handshake` (app.rs:1152, §2.5 sub-region 2 "Top-level orchestrators") is a structurally distinct function: bilateral federation session driver (both Initiator + Receiver post-handshake), longer-lived, 13-parameter signature with 7 identifier-shaped slots — none transitively covered by T8 (T8 names `handle_federation_incoming` explicitly; the function bodies diverge after the post-handshake handoff). Per D-078 strict reading (test enumeration is D-078's application surface; the discipline binds at every retype target with no pinning test): a Pass 3 retype target with no pinning test is the failure mode D-078 codifies.

**Path α locked at session-time turn** over Path β (subsume into T8 — merges two structurally distinct surfaces into one test, sibling-shape to κ pattern recurring at test-name layer) + Path γ (inheritance without verification — D-078's failure mode). Path α adds T11 by name; updates Surface #5 count 2 → 3; updates total +10 → +11.

**T11 production anchor verified at J-135 author-time per D-078**: Read of app.rs:1151-1173 confirmed `pub(crate) async fn run_federation_session_post_handshake<S>` at line 1152 with the three identifier-shaped slots T11 will pin intact in the Q5.14 v1.3 shape:
- `home_node_id: String` at line 1161 → retype target owned `NodeXgid` (async-spawned-captures per §4.2 v1.2)
- `peer_node_id: String` at line 1165 → retype target owned `NodeXgid`
- `peer_shared_spaces: Vec<String>` at line 1169 → retype target `Vec<SpaceXgid>`

Descriptive-string slots verified NOT-retype targets per §5.4: `session_id: String` (line 1166), `neg_version: String` (line 1167), `serial: String` (line 1168). Wire-format-boundary slot verified NOT-retype target per §4.3: `peer_tips: BTreeMap<String, String>` (line 1170).

**J-128 → J-133 → J-135 framing**: the test enumeration coverage gap is a downstream consequence of the natural lifecycle "design doc evolves under amendment → runbook test enumeration captured pre-amendment scope → amendment adds new retype target → test enumeration needs catch-up amendment." The +10 enumeration at J-128 was production-grounded at authoring time; the design doc §2.5 sub-region "Top-level orchestrators" only listed `process_inbound` + `run_node` at J-127 design close. J-133's §2.5 sub-region prose extension added `run_federation_session_post_handshake` (Drift #3 closure); at that point the test enumeration carried a gap as a downstream consequence — not a defect in the J-128 authoring, just elapsed time + canonical-record evolution. D-078 working as designed: when a Pass-3-scope retype target enters at canonical-record amendment time, the test enumeration is verified against the new target before Commit 2 production code lands.

**Sibling-shape to J-114 D-078 promotion atom**: D-078 was promoted at J-114 specifically to codify this discipline at the test-enumeration layer. J-135 is the first prospective application of D-078 within Pass 3 — sibling-shape to D-078's first prospective application at `tasks/FEDERATION_PROPAGATION_PHASE_9_COMMIT_3B_4_IMPL.md` Joe-lock checkpoint #4 (per the D-078 entry's "Application surface" sub-section). The pattern holds: D-078 catches a test-enumeration coverage gap that emerged through canonical-record evolution rather than authoring error.

**J-128 → J-135 evolution chain not a recurrence**. "Honest longer work over fast shortcuts" Pass 3 count does NOT increment at J-135. Sibling-shape to J-115/J-116 prospective catches that did NOT increment the count: T11 added BEFORE Commit 2 production code ships; no fabricated regression-lock reaches canonical record. Distinct from recurrence shape (J-129 + J-134 incremented because a wrong canonical Q-row reached origin/main and needed honest fix). Count stays at TWO inherited from J-129 + J-134.

**§4.7 amendments at this v1.3**:
- Surface #5 header: count "(2 tests target)" → "(3 tests target at v1.3; +1 from v1.2's 2 per J-135 D-078 application)".
- Surface #5 header text extended: "+ `run_federation_session_post_handshake`" added between `handle_federation_incoming` and `+ persistence-format boundary` to reflect the full sub-region 2 coverage.
- Surface #5 list: T11 row added by name with the full Q5.14 v1.3 anchor description + production-anchor lines verbatim + sub-region-distinct-from-T8 framing.
- "Total per-surface test target" line: 10 tests → 11 tests; total 501 → 502; distribution Surface #5 count 2 → 3; v1.2 → v1.3 amendment note inline.

**§7 NOT amended**. The §7.10 Pass 5 consolidation flag still stands; J-135 is a test-enumeration scope-correction, not a new discipline data point worth consolidation-flag attention.

**D-074 application count at this commit: thirty-second instance**. Grounded across JOURNAL grep + commit-message cross-check at J-135 author-time:
- J-127 24th → J-128 25th → J-129 26th → J-130 27th → J-131 28th → J-132 29th → J-133 30th → J-134 31st → this J-135 32nd.

Lock #3 per-commit cadence; not a milestone-close so milestone-close tally — thirteenth at J-126 — does NOT increment.

**Single-file atom enumeration**:
1. This runbook v1.2 → v1.3 (header chain prepend + §4.7 Surface #5 count 2 → 3 + T11 by name + total +10 → +11 + this new §9.5).

CLAUDE.md NOT amended (entry-point stays Commit 2; sibling-shape to J-132 + J-121 no-PLAY-touch precedent). ROADMAP NOT amended (within-milestone canonical-record clear; sibling-shape to J-117 + J-130 + J-132 framing). JOURNAL.md NOT amended (chain-only-then-no-op per Joe-lock at session-time turn; the D-078 working-as-designed observation lives in this §9.5, NOT as a fresh body §-entry — sibling-shape to J-132 framing where mechanical-completion of an already-surfaced catch does not warrant body entry). DECISIONS.md NOT amended (D-078 application surface, no new candidate; this is D-078 working as designed).

**Joe-lock checkpoint #2 status post-J-135**: all seven surfaces APPROVED at prior turn; ten original tests APPROVED at prior turn; T11 APPROVED by name conditional-on-anchor-holding; anchor verified clean at J-135 author-time. Post-J-135 push + Clair's single-line T11-anchor-confirmed message: Joe clears Commit 2. Then Commit 2 production code (seven-surface retype + per-surface tests atomic per runbook §4).

**Session-pacing flag**: Joe flagged at session-time turn that this session has run long + Commit 2 would benefit from a fresh session-open so Rule 0 reading order does its job (CLAUDE.md PLAY + latest JOURNAL + active HANDOFF). Recorded as discipline data point; Clair's call after J-135 ships. Sibling-shape to Pass 2 J-125 session-pacing (Commit 2 production code in fresh session post-J-125 doc-pass push).

### §9.6 J-136 v1.3 → v1.4 amendment provenance (Commit 2 SHIPPED under Path 2 split per checkpoint #3)

Runbook bumped to v1.4 at Commit 2 ship (J-136 — frozen at milestone close per J-108 codification). Header chain entry covers the substantive shipped state; this §9.6 records the WIP-branch lineage + the Joe-lock-checkpoint-#3 Path-2-split decision + the lib-only verification framing.

**WIP-branch lineage**. The Commit 2 atomic work spanned a fresh session-open per the J-135 session-pacing flag (recorded at §9.5 above). Clair (Chat Claude in implementation role) opened a labelled WIP branch `wip/pass-3-commit-2-in-flight` so the in-flight scope could be checkpointed without contaminating origin/main. Two WIP checkpoints landed on the branch before squash:

- **Checkpoint #1 commit `728b834`** — Surfaces #1 (NodeRuntime six per-space HashMap keys) + #2 (`dispatch_event` Option<&NodeXgid>) lib-clean in xgen-core; Surface #3 (`federation_session.rs` — incl. J-134 Finding B annotation drop / D-079 closure) lib-clean in xgen-node; Surface #4 (fanout.rs ClientSenders + FederationPeerSenders + FanoutRequest.new_joiner + event_space_id + apply_fanout + collect_sync_history + compute_federation_delta_for_space + topological_sort_events) lib-clean in xgen-node; Surface #5 partial (build_node_state + persistence-format boundary projections + xgid-imports). 51 errors remained in xgen-node lib post-checkpoint-#1 (Surface #5 substantive + #6 + consumer call-sites pending).

- **Checkpoint #2 commit `2f647bf`** — Surface #5 substantive closed (handle_connection / process_inbound Q5.1 + Q3-overload projection / handle_federation_incoming Q5.2 + spawned-task forced-owned NodeXgid / handle_identity_msg + handle_identity_replicate_msg Q5.3 + Q5.4 / run_federation_session_post_handshake T11 target per Q5.14 v1.3 13-param matrix / ConnectedClientInfo Q5.15 / push_identity_to_peers + replica_registry projections / IdentityMessage::Record wire-format-boundary / persist_event duplicate-guard / short_id admin display); Surface #6 closed (reconnect.rs three spawned functions Q6.1+Q6.2+Q6.3 forced-owned + AttemptCursor Q6.4 + xgen-core wire-format boundary projection). xgen-node lib **CLEAN** at this checkpoint.

**Joe-lock checkpoint #3 numbers at lib-clean boundary** per runbook §2.3 + §5.1 split-trigger:
- xgen-common test fixtures: 0 errors.
- xgen-core test fixtures: 160 errors.
- xgen-node test fixtures: 478 errors.
- **Total: 638 errors** at the §5.1 ~50 threshold.

**638 >> 50 → Path 2 (Commit 2a split) locked at checkpoint #3 by Joe**. Sibling-shape to Pass 2 Commit 2a `58b94a5` (93 errors, J-126 milestone-close arc) + Pass 1 Commit 4a `4895446` (broader sweep). Each commit preserves its own atomic-purpose-discipline per D-074; lib-only verification at Commit 2; full 8 GREEN at Commit 2a per §5.3.

**Surface #7 doc-tree sweep at Commit 2**. Four markdown table classification rows in `docs/xgen_appendix_d_en.md` §2.1 + §2.2 + §2.3 gain typed-XGID-in-memory annotations per design doc §7.5 + Q7.2 honest-minimum framing: `identity_id` (`IdentityXgid`), `home_node` (`NodeXgid`), `event_id` (`EventXgid`), `peer_node_id` (`NodeXgid`). Each annotation preserves the on-disk + on-wire String semantics per design doc §4.3 format-boundary preservation; the typed-XGID label documents the in-memory Rust slot post-Pass-3. Doc-only edit; no test required. Folded into Commit 2 (the lighter touch end of the §4.7 + §4.10 surface-7 scope — Q7.3.a "possibly nothing" + Q7.5 "minimal touch folded into doc-pass commit" guidance honored).

**Verification at Commit 2 boundary (lib-only — full 8 GREEN at Commit 2a per §5.3)**:
- `cargo build -p xgen-common -p xgen-core -p xgen-node` **CLEAN**.
- `cargo clippy -p xgen-common -p xgen-core -p xgen-node --lib --all-features -- -D warnings` **CLEAN**.
- `cargo build --workspace` deliberately broken at xgen-client consumer sites per Path A inherited from Pass 1.
- `cargo test -p xgen-common -p xgen-core -p xgen-node --tests` 638 errors → checkpoint #3 report drives Commit 2a scope.

**Single-commit (squashed) ship per D-074 atomic discipline**. The two WIP checkpoints `728b834` + `2f647bf` squashed into the Commit 2 atomic at ship time — D-074 forbids splitting Surface #1 + #2 + #3 + #4 + #5 + #6 + #7 across commits within Commit 2 (the atomic-purpose is "seven-surface retype landed atomic"). The WIP branch history disappears on main; only the squashed Commit 2 remains.

**Files in this commit per D-074 + §4.10**:
1. `xgen-core/src/node/runtime.rs` — Surface #1 + Surface #2 retypes.
2. `xgen-node/src/federation_session.rs` — Surface #3 retypes (incl. J-134 Finding B / D-079 closure).
3. `xgen-node/src/fanout.rs` — Surface #4 retypes.
4. `xgen-node/src/app.rs` — Surface #5 retypes (12 in-memory identifier slots + handle_federation_incoming async-spawn forced-owned + run_federation_session_post_handshake T11 target + persistence-format boundary projections + ConnectedClientInfo Q5.15).
5. `xgen-node/src/reconnect.rs` — Surface #6 retypes (three spawned functions forced-owned + AttemptCursor).
6. `docs/xgen_appendix_d_en.md` — Surface #7 four markdown table classification rows annotated.
7. This runbook v1.3 → v1.4 (header chain + this new §9.6).
8. `JOURNAL.md` — J-NNN body entry per D-074 + Lock #3 per-commit cadence.
9. `docs/ROADMAP.md` — version bump + visual tree row + Past entry + header chain.
10. `CLAUDE.md` — header chain entry; PLAY block flip "Commit 1 doc-pass ✅; Commit 2 next-active" → "Commit 2 ✅; Clair pickup at Commit 2a [SPLIT] per checkpoint #3".

Ten files atomic. Sibling-in-shape to Pass 2 Commit 2 `22765a0` (eight-file atomic with same lib-only-verification framing).

**D-074 application count at this commit: thirty-third instance**. Lock #3 per-commit cadence; not a milestone-close so milestone-close tally (thirteenth at J-126) does NOT increment.

**"Honest longer work over fast shortcuts" Pass 3 count does NOT increment at Commit 2 ship**. Sibling-shape to the close-event-not-recurrence-event framing at J-101 / J-108 / J-122 / J-126 — Commit 2 is a substantive within-milestone ship, not a recurrence. Count stays at TWO inherited from J-129 + J-134.

**Next-active: Commit 2a** per runbook §5 — test-fixture projection sweep across xgen-core + xgen-node + 11 per-surface tests T1-T11 atomic. Verification at Commit 2a = 8 GREEN runs per §5.3 + §4.9. Then Commit 3 milestone close per §6.

### §9.7 J-137 v1.4 → v1.5 amendment provenance (Commit 2a SHIPPED under Path 2 split per checkpoint #3 ✅)

Runbook bumped to v1.5 at Commit 2a ship (J-137 — frozen at milestone close per J-108 codification). Header chain entry covers the substantive shipped state; this §9.7 records the parallel-subagent-sweep discipline data point + the §5.3 8-GREEN verification artefact set.

**Scope at Commit 2a.** Test-fixture projection sweep + 11 per-surface tests T1-T11 atomic per Joe-lock at checkpoint #3. Total 638 test-fixture errors (160 xgen-core + 478 xgen-node) closed via mechanical projection-only edits per §5.2 verbatim patterns. Plus T1-T11 added per §4.7 by name + production anchors verified.

**Parallel-subagent-sweep discipline data point**. Test-fixture sweep delegated to two parallel subagents (one per crate — xgen-core + xgen-node) under explicit DO-NOT-CROSS-CRATE-BOUNDARY guard-rails. Both completed clean: xgen-core sweep (4 files modified, 0 deviations); xgen-node sweep (~20 files modified, 6 minor deviations — all honest-reported per Rule 1: two `idx` loop-variable shadowing renames (cosmetic), one app.rs `NodeXgid` unused-import note (cleaned up at integration-time), one federation_delta_integration.rs replace_all=true collision caught immediately + recovered, one reconnect_integration.rs mock-receiver wire-format-boundary projection (intentional per §4.3), one smoke.rs `Vec<NodeXgid>::contains(&String)` → `.iter().any()` semantic-equivalent rewrite). **Sibling-shape data point for future Pass-arc Commit-2a-shape runbook authors**: when the test-fixture sweep error count exceeds ~500, parallel-subagent delegation under per-crate guard-rails is a viable shape; the discipline cost is explicit honest-deviation reporting at integration time (Rule 1) + per-crate independence verification (zero cross-crate file modifications via `git diff --stat`). Pre-Commit-2a 8-GREEN verification catches integration-edge regressions (the subagent reports must hold under the 8-GREEN protocol, not just per-subagent test passes).

**T1-T11 enumeration verified at ship-time**:
- T1 (Surface #1): `noderuntime_per_space_map_insert_retrieve_with_typed_key` at xgen-core/src/node/runtime.rs:`mod persistence_amendment_commit_2a_tests`.
- T2 (Surface #1): `noderuntime_per_space_map_six_flavours_isolated` at same mod.
- T3 (Surface #1): `noderuntime_per_space_map_helper_signatures_typed_at_boundary` at same mod.
- T4 (Surface #2): `dispatch_event_with_borrowed_node_xgid_projects_to_str_at_callsite` at same mod.
- T5 (Surface #3): `federation_session_handler_identifier_slots_retyped_at_boundary` at xgen-node/src/federation_session.rs:`mod tests` (new mod added at Commit 2a).
- T6 (Surface #4): `fanout_topological_sort_event_xgid_slot_pass_1_intact` at xgen-node/src/fanout.rs:`mod tests`.
- T7 (Surface #5 Q5.12): `app_handlers_persistence_format_round_trip_string_at_boundary` at xgen-node/src/app.rs:`mod tests`.
- T8 (Surface #5 Q5.2): `handle_federation_incoming_spawned_task_owns_node_xgid_capture` at xgen-node/src/app.rs:`mod tests`.
- T9 (Surface #6): `reconnect_spawned_functions_each_own_typed_capture` at xgen-node/src/reconnect.rs:`mod tests`.
- T10 (Surface #6): `reconnect_spawned_functions_arc_shared_reference_pattern_when_needed` at xgen-node/src/reconnect.rs:`mod tests`.
- T11 (Surface #5 Q5.14 v1.3): `run_federation_session_post_handshake_spawned_task_owns_typed_captures` at xgen-node/src/app.rs:`mod tests`.

**Verification rigour at Commit 2a milestone-bearing boundary** per §4.9 + §5.3:
- 5 isolated runs (cargo clean -p xgen-common -p xgen-core -p xgen-node between each + cargo test -p xgen-common -p xgen-core -p xgen-node) — ALL GREEN.
- 3 consecutive workspace runs (cargo test -p xgen-common -p xgen-core -p xgen-node without intervening clean) — ALL GREEN.
- 8/8 GREEN minimum threshold met.
- `cargo clippy -p xgen-common -p xgen-core -p xgen-node --lib --all-features -- -D warnings` clean.
- `cargo clippy -p xgen-common -p xgen-core -p xgen-node --tests --all-features -- -D warnings` clean (six nits in T1+T2 + agent-sweep fanout.rs + phase9_compound_c7 closed at integration time: `.get(&x).is_some()` → `.contains_key(&x)` + redundant closure `|e| event_id_str(e)` → `event_id_str` + useless `vec![room_id.clone()]` → `[room_id.clone()]`).
- `cargo build --workspace` deliberately broken at xgen-client consumer sites only (192 errors all xgen-client; xgen-common + xgen-core + xgen-node clean; Pass 5 close restores).

**Test count stability at 589 across all 8 runs**: 34 xgen-common lib + 8 invariance + 453 xgen-core + 88 xgen-node lib + 6 precedence = 589. Delta vs J-126 baseline 491: +98 (= 87 xgen-node tests now visible post-lib-clean + 11 per-surface tests T1-T11). Delta vs pre-T1-T11 sweep 578: +11 = T1-T11 target hit per runbook §4.7.

**Both pre-existing documented flakes did NOT fire** across the 8 GREEN runs (precedence env-var race; `reconnect_with_existing_tip_small_delta_delivered`). Honest data point per Rule 2 — flakes stay documented in CLAUDE.md as known.

**Files in this commit per D-074 + §5.4**:
1. `xgen-core/src/node/runtime.rs` — Commit 2 lib retypes were squashed at Commit 2 ship; Commit 2a appends T1-T4 (4 new tests within existing `mod persistence_amendment_commit_2a_tests`) + clippy `.get().is_some()` → `.contains_key()` cleanup.
2. `xgen-core/src/node/tests/phase9_validation_asymmetry.rs` — agent sweep (~109 errors closed).
3. `xgen-core/src/node/tests/phase9_compound_c9_drain_time_hazard.rs` — agent sweep (7 errors).
4. `xgen-core/src/node/tests/phase9_compound_c5_validation_under_load.rs` — agent sweep (2 errors).
5. `xgen-node/src/federation_session.rs` — T5 added (new `mod tests`).
6. `xgen-node/src/fanout.rs` — agent sweep (multiple sites) + T6 added + clippy redundant-closure cleanup.
7. `xgen-node/src/app.rs` — agent sweep (replay_spaces_from_dir test) + T7 + T8 + T11 added.
8. `xgen-node/src/reconnect.rs` — T9 + T10 added.
9. `xgen-node/src/transport/mod.rs` — agent sweep.
10. `xgen-node/src/tests/phase9_harness.rs` — agent sweep (idx/ndx/sdx/edx/rdx helpers + many call-site projections).
11. `xgen-node/src/tests/smoke.rs` — agent sweep.
12. `xgen-node/src/tests/cold_start_bootstrap_integration.rs` — agent sweep (6 scenarios).
13. `xgen-node/src/tests/federation_delta_integration.rs` — agent sweep (3 tests).
14. `xgen-node/src/tests/federation_push_integration.rs` — agent sweep (3 tests).
15. `xgen-node/src/tests/federation_relationship_integration.rs` — agent sweep (3 F-3 tests).
16. `xgen-node/src/tests/heldpending_identity_integration.rs` — agent sweep (4 F-10 scenarios).
17. `xgen-node/src/tests/identity_integration.rs` — agent sweep (registry.contains projection).
18. `xgen-node/src/tests/phase9_compound_c10_identity_lock_contention.rs` — agent sweep.
19. `xgen-node/src/tests/phase9_compound_c2_anti_transitivity_at_load.rs` — agent sweep.
20. `xgen-node/src/tests/phase9_compound_c7_pagination_boundary.rs` — agent sweep + useless-vec clippy nit.
21. `xgen-node/src/tests/phase9_drop_and_recover.rs` — agent sweep.
22. `xgen-node/src/tests/phase9_federation_relationship_rejection.rs` — agent sweep.
23. `xgen-node/src/tests/phase9_three_node_anti_transitivity.rs` — agent sweep.
24. `xgen-node/src/tests/phase9_two_node_smoke.rs` — agent sweep.
25. `xgen-node/src/tests/phase9_unknown_signer_first_contact.rs` — agent sweep.
26. `xgen-node/src/tests/reconnect_integration.rs` — agent sweep (mock-receiver wire-format projection).
27. This runbook v1.4 → v1.5 (header chain prepend + this new §9.7).
28. `JOURNAL.md` — J-NNN body entry per D-074 + Lock #3 per-commit cadence.
29. `docs/ROADMAP.md` v1.41 → v1.42 (visual tree row + Past entry + Present flip).
30. `CLAUDE.md` (header chain entry + PLAY block flip "Commit 2 ✅; Commit 2a [SPLIT] next" → "Commit 2a ✅; Clair pickup at §6 Commit 3 next").

Thirty-file atomic — the heaviest single commit in Pass 3 by file count but appropriate for the test-fixture sweep scope (sibling-shape to Pass 2 Commit 2a `58b94a5` nine-file xgen-core test sweep + Pass 1 Commit 4a `4895446` broader sweep, both of which were similarly heavy at their respective Pass-arc scopes).

**D-074 application count at this commit: thirty-fourth instance**. Lock #3 per-commit cadence; not a milestone-close so milestone-close tally — thirteenth at J-126 — does NOT increment.

**"Honest longer work over fast shortcuts" Pass 3 count does NOT increment at Commit 2a ship**. Sibling-shape to close-event-not-recurrence-event framing at J-101 / J-108 / J-122 / J-126. Count stays at TWO inherited from J-129 + J-134.

**Next-active: Commit 3 milestone close** per runbook §6 — Pass 3 PLAY → DONE; design doc §6.1 J-NNN freeze; runbook ACTIVE → COMPLETED v1.5 → v1.6; ROADMAP visual tree 🟢 → ✅; CLAUDE PLAY flip "standby for next-milestone selection"; grep `J-NNN` guardrail = ZERO post-staging per J-108 codification.

### §9.8 J-138 v1.5 → v1.6 milestone-close amendment provenance (Pass 3 milestone CLOSED)

Runbook bumped to v1.6 at milestone close (J-138). Status flipped ACTIVE → COMPLETED. Header chain entry covers the substantive shipped state; this §9.8 records the milestone-close atomic shape + verification artefact + the DoD checklist verification + the cross-Pass discipline carry-overs.

**Milestone-close atomic shape (five-file per D-074 thirty-fifth instance + fourteenth milestone-close)**:

1. This runbook v1.5 → v1.6 (header chain prepend + Status flip ACTIVE → COMPLETED + this new §9.8).
2. `tasks/XGID_RETROFIT_PASS_3_DESIGN.md` — header chain entry only + §6.1 J-NNN placeholder freeze → J-138 per J-108 codification + Pass 2 §6.7 freeze pattern.
3. `JOURNAL.md` — J-138 body entry (eight sub-sections sibling-shape to J-122 + J-126) + J-136/J-137 self-reference freezes already applied.
4. `CLAUDE.md` — header chain entry + PLAY block flip "Pass 3 implementation ACTIVE — Commit 2a ✅" → "Pass 3 milestone CLOSED at J-138; standby for next-milestone selection (Pass 4 + M6 (new) both ready)".
5. `docs/ROADMAP.md` — version bump + visual tree Pass 3 row 🟢 → ✅ + Past entry + Present updated + Near future Pass 3 line removed + header chain.

**Verification artefact at this milestone-close commit boundary**: single workspace verification pass per checkpoint #4 lock (single re-run vs full 8 GREEN — milestone-bearing boundary 8/8 GREEN already recorded at J-137 Commit 2a ship). 589 tests pass + both clippy gates clean + grep J-NNN guardrail returns ZERO at freeze-site sources post-staging.

**DoD checklist verification per §6.5**:

- [x] All seven surfaces from design doc §2 Q-tables retyped per locked decisions at design doc §4.
- [x] Per-surface tests landed (target +11 = T1-T11; +10 per original §4.7 + T11 per J-135 D-078 application).
- [x] `cargo test -p xgen-common -p xgen-core -p xgen-node` GREEN (8/8 minimum at Commit 2 boundary recorded at J-137; re-verified at Commit 3 single-pass).
- [x] Both clippy gates clean (`--lib` + `--tests`, `-D warnings`).
- [x] `cargo build --workspace` deliberately broken at xgen-client consumer sites only (no regression at xgen-common + xgen-core + xgen-node).
- [x] Layered-B3 audit answer recorded in JOURNAL J-138 body Sub-section 4 (zero layered surfaces emerged — three-instance no-finding chain at Pass-arc layer now durable).
- [x] Design doc §6.1 J-NNN placeholder frozen to J-138 per J-108 codification.
- [x] `grep -rn 'J-NNN' . --include='*.rs' --include='docs/*.md' --include='tasks/*.md'` returns ZERO matches at freeze-site sources post-staging.
- [x] Four candidate D-NNNs status recorded in JOURNAL J-138 body Sub-section 8 (γ + δ + ε + format-boundary promotion-watch; none promoted at this atom per D-069).
- [x] "Honest longer work over fast shortcuts" Pass 3 final count recorded in JOURNAL J-138 body Sub-section 5 (TWO recurrences at J-129 runbook v1.0 surface ordering drift + J-134 design doc §2 v1.3 → v1.4 in-place rewrite-correction; both prospective catches at canonical-record-amendment layer).
- [x] D-074 application count incremented (thirty-fifth instance + fourteenth milestone-close at this Commit 3).

**Cross-Pass discipline carry-overs (load-bearing for Pass 4 + Pass 5)**:

- **Path A**: Three-instance durability across Pass 1 + Pass 2 + Pass 3 now established. Permanent cross-Pass discipline; Pass 4 + Pass 5 inherit without re-lock.
- **Borrow<str> additive API**: Pass 1 Commit 4 introduced; Pass 2 + Pass 3 consumed mechanically; Pass 4 + Pass 5 inherit at all HashMap lookup sites.
- **Layered-B3 expected-null**: Three-instance no-finding chain. Pass-arc-level discipline data point promoted to load-bearing structural fact.
- **Pass-internal-consistency framing over trilogy-internal-consistency**: Pass 2 §7.7 + Pass 3 §7.2 establish the precedent; Pass 4 + Pass 5 design + runbook authors inherit the framing.
- **Pre-locked contingent-split posture**: Pass 2 §7.3 + Pass 3 §5.1 establish the criterion + runbook-authoring shape; Pass 4 + Pass 5 runbook authors inherit the framing as default.

**Next-active**: Pass 4 + M6 (new) both ready for next-milestone selection at session open. Pass 4's runbook authoring is the next Chat Claude work-shape on the XGID retrofit track if Joe picks Pass 4 first; M6 (new) Block 4 verb-by-verb walks (~35 verbs across 7 categories at `docs/xgen_node_admin_ops_design.md` §6) are parallel-eligible. Sequencing is Joe's call.

