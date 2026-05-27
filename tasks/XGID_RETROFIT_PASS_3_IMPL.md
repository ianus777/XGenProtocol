# XGID Retrofit Pass 3 — Implementation Runbook
> **Status**: ACTIVE  
> Version: 1.1  
> Date: May 2026  
> **Last updated**: 2026-05-27 (J-129 — Track 1 canonical-record amendment shipped. Re-aligned §4.1 + §4.7 + §4.10 surface ordering to design doc §2 verbatim (Surfaces #1↔#2 + #5↔#6 swapped at v1.0; corrected); corrected Surface #3 location for `handle_federation_incoming` from `federation_session.rs` → `app.rs` (`handle_federation_incoming` is defined at `xgen-node/src/app.rs:976`, not at `federation_session.rs`; `federation_session.rs` has zero `tokio::spawn` and zero `handle_federation_incoming`); clarified §4.5 + §4.7 Surface #5 (reconnect) wording around "three spawned functions"; new §9 amendment-provenance section. **D-078 second prospective-catch at runbook-authoring layer** (J-115 + J-116 were prospective catches at runbook-implementation-by-Clair layer; this J-129 is prospective catch at runbook-authoring-by-Chat-Claude layer — distinct surface). Discipline data point recorded at §7.11 (new): when authoring a runbook from a session-bridge summary rather than fresh from the design doc, the surface enumeration MUST be cross-checked against design doc §2 verbatim BEFORE the runbook §4 ships. Previous v1.0 J-128 update content stands authoritative in spirit — amended in place at v1.1 for cells reference design doc §2 verbatim.) Previous 2026-05-27 (J-128 — Runbook authored at design-close-plus-one session per Pass 2 + trilogy precedent; sibling-in-shape to `tasks/XGID_RETROFIT_PASS_2_IMPL.md` COMPLETED v1.1 with three structural extensions for Pass 3's seven-surface scope: §4.7 per-surface tests heavier (+10 target vs Pass 2's +2); §7 nine sub-sections (Pass 2 had eight); §7.10 Pass 5 consolidation flag recorded as future-walk candidate for runbook §7 deduplication across the five-Pass arc.)  
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

Commit 1 ships the minimal doc-pass that reflects Pass 3 implementation kickoff at the canonical project surface (ROADMAP + CLAUDE PLAY + JOURNAL chain). Design doc + this runbook stay untouched at Commit 1 because both are already at terminal Status (design doc COMPLETED v1.2; runbook ACTIVE v1.0). Three-file atomic per D-074.

### §3.2 Files in this commit

1. `docs/ROADMAP.md` — version bump (v1.38 → v1.39 assuming J-128 lands v1.38; Clair verifies current version at session open and bumps accordingly); visual structure tree Pass 3 Implementation row added (NEW; rendering "🟢 Commit 1 doc-pass ✅"); Present section flipped from "Pass 3 design ✅ at J-127; runbook authoring next-active" → "Pass 3 implementation Commit 1 doc-pass ✅; Clair pickup at runbook §4 Commit 2 next"; Past section gains Commit 1 entry; header chain entry.
2. `CLAUDE.md` — header chain entry; PLAY block flip from "XGID Retrofit Pass 3 implementation ACTIVE — Clair pickup at runbook §3 Commit 1" → "XGID Retrofit Pass 3 implementation ACTIVE — Clair pickup at runbook §4 Commit 2 (Commit 1 doc-pass ✅)".
3. `JOURNAL.md` — header chain entry only (NO body J-NNN entry, sibling-shape to J-123 + J-124 + J-125 doc-only milestone-event precedent; pattern durable at three project instances per D-077/D-078).

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

**Surface #5 — app.rs handler identifier slots + handle_federation_incoming + persistence-format boundary** (2 tests target):
- `app_handlers_persistence_format_round_trip_string_at_boundary` — round-trip test: write JSON HashMap with String keys → read back via `replay_spaces_from_dir` → verify String keys project cleanly to typed XGIDs at consumption layer. Covers both persistence-format (§4.6) and wire-format (`IdentityReplicateMessage::Replicate` destructure) sub-instances.
- `handle_federation_incoming_spawned_task_owns_node_xgid_capture` — verify forced-owned `NodeXgid` parameter compiles + behaves correctly across `tokio::spawn` boundary at `xgen-node/src/app.rs:976`; uses `Arc<NodeXgid>` if shared reference needed inside spawn body.

**Surface #6 — reconnect.rs three spawned functions** (2 tests target):
- `reconnect_spawned_functions_each_own_typed_capture` — verify all three spawned functions (`spawn_reconnect_scheduler` + `scheduler_tick` + `attempt_reconnect`) accept forced-owned typed parameters; covers all three instances atomically.
- `reconnect_spawned_functions_arc_shared_reference_pattern_when_needed` — verify `Arc<TypedXgid>` shared-reference pattern works for spawned-task captures that need read-only access to the same typed value across multiple spawned tasks.

**Surface #7 — Appendix D doc-tree sweep** (0 tests target):
- Doc-only edit; no test required. Mechanical edit of four markdown table classification rows.

**Total per-surface test target: 10 tests** (+10 vs J-126 baseline of 491; new total 501 if all land). Distribution per Surface: #1 (3) + #2 (1) + #3 (1) + #4 (1) + #5 (2) + #6 (2) + #7 (0) = 10. Joe may add or remove tests at checkpoint #2 approval.

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

