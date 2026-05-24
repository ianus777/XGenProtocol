# XGen Protocol — Claude Code Briefing
> For: Claude Code (claude.ai/code)  
> Date: May 2026  
> **Status:** ACTIVE  
> **Last updated:** 2026-05-24 (J-109 — Phase 9 survey findings v1.1 → v1.2 amendment shipped: §2.6 Scenario 6 substantive rewrite + new §2.6.1 Phase 7.5 §6 contract walkthrough. Six-file atomic commit per D-074 (twelfth instance). **Cause: canonical-document staleness surfaced at Clair's Pre-Commit-3b-2-equivalent verification read** — survey findings v1.1 §2.6 claimed `DispatchOutcome::Rejected` with `reason = federation_relationship_missing` but code at `xgen-core/src/node/runtime.rs:514-555` emits `DispatchOutcome::HeldPending` with new `disposition = "held_pending"` field on the existing `f3_reject` G2 trace event (Phase 7.5 §6 held-not-bypassed posture, P7.5-B Joe-locked 2026-05-19, shipped J-094 2026-05-20). Survey was authored before Phase 7.5 shipped; v1.1 froze the pre-Phase-7.5 contract. **Reading B locked over Reading A + Reading C** — amend canonical source FIRST in Chat Claude Track 1 atom, then Clair picks up against amended contract (sibling-shape to J-107 Track 1 canonical-record amendments before Clair Track 2 pickup). **D-077 bidirectional sustainability discipline applied at amendment authoring**: pre-amendment grep across federation canonical docs surfaced six file candidates; per-surface lock decisions — three amended (findings v1.1→v1.2 substantive rewrite of §2.6 + new §2.6.1 contract walkthrough; Phase 9 task §3 Commit 4 Scenario 6 drift correction + §3.0a 599→592 typo fix per Clair's catch + baseline correction ~525→~597; survey doc header supersession pointer only with body preserved unchanged per two-document framing locked 2026-05-19), three preserved unchanged with reasoning (completion runbook records Phase 7's locks correctly for its scope; design doc §6.4.1 already incorporates Phase 7.5 P7.5-A/B/C/D paragraphs correctly; C3 compound paragraphs acknowledged-stale in dropped-scenario context). Discipline data point: D-077 application is NOT "grep and edit everywhere" but "grep, then audit each hit, then lock per-surface decisions with reasoning" — NOT-amended decisions are D-077 data points equal to amended-decisions. **Canonical-document-staleness-surfaces-at-dependent-milestone-implementation-time pattern: second project instance** (first was J-099 audit-doc + design-doc §11 in-place amendments at re-walk Step 2; two instances not yet durable pattern; three would be). **D-074 application count: twelfth instance** (J-095 through J-108 are eleven prior; this J-109 twelfth). **"Honest longer work over fast shortcuts" — count inherited at eighth from J-104, NOT incremented at this canonical-record amendment** (within-milestone amendment event, not new milestone-surface event; sibling-shape to J-099 + J-107 Track 1 framing — close-event-not-recurrence-event). **No state transitions in this atom**: Phase 9 milestone stays PLAY (Clair's next-active still Commit 3b-2-equivalent, now against amended contract). Federation Event Propagation milestone stays PLAY. M6 (new) + XGID Retrofit Pass 1 stay PENDING. **Six files in this atomic commit**: (1) JOURNAL.md J-109 entry; (2) tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md v1.1→v1.2 + §2.6 substantive rewrite + new §2.6.1 + header chain; (3) tasks/FEDERATION_PROPAGATION_PHASE_9.md §3 Commit 4 drift correction + §3.0a typo fix + baseline correction + header chain (no version bump); (4) tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY.md header chain supersession pointer only; (5) this CLAUDE.md header `Last updated` chain entry (no PLAY block flip); (6) docs/ROADMAP.md v1.23 → v1.24 small Past entry + header chain. **PLAY block content stays unchanged at J-108's flip** — Clair's next-active is Phase 9 Commit 3b-2-equivalent against the now-amended contract; entry-point file `tasks/FEDERATION_PROPAGATION_PHASE_9.md` is amended; survey findings v1.2 §2.6 + §2.6.1 is the contract source-of-truth. Per Rule 0 + D-065 + D-067 + D-069 + D-071 + D-074 + D-077 discipline. Previous J-108 update content stands authoritative — see header chain below.) Previous J-108 update: 2026-05-24 (J-108 — Persistence-amendment sub-amendment milestone **CLOSED**. Ten-file atomic commit per D-074 (eleventh instance — honest count after staging; runbook §6.2 estimate "approximately twelve files" counted two slots that resolved to NOT-in-this-commit per D-077 backward-coherence audit). Phase 9 Commit 3b-1 collapses per Q4(a) lock; sentinel-tree four files at `xgen-node/src/tests/` (shipped atomic at Commit 3 `a677244`) crown as activating integration-level regression lock at this Commit 4 milestone close. Five-commit Clair-facing sequence: Commit 1 `0ca29e6` doc-pass + Commit 2 `f4f0e4e` Q1 ingest-path + Commit 2a `c88fd73` Q2+Q3 dispatch+persist + Commit 3 `a677244` sentinel-tree refinement + verify (8/8 GREEN) + re-walk Track 1 `b9a30da` at J-107 + this Commit 4 milestone close. Q1 (a).iii.α + (a).ii defensive sort-on-replay; Q2 (a) return-vector; Q3 all-three drain helpers; Q4 (a) sentinel-tree in-scope (Commit 3b-1 collapses). **PLAY flips** from "persistence-amendment milestone RUNBOOK ✅ at J-106; Clair pickup at Commit 1 next" (re-walk Track 1 framing) to "Phase 9 Commit 3b-2-equivalent RESUMES — Scenarios 2 + compounds C2/C5/C7/C9/C10 (C3 dropped per Phase 9 §3.0 v1.1); ~4-6 atomic commits in their own sequence" per Q4(a) Commit-3b-1-collapse lock from J-105. **D-077 promoted at J-107 Track 1** (bidirectional sustainability discipline at silent-discard / conditional-mutation / fallible-discard sites — meta-layer above D-067 + D-070 + D-075 + D-076 v1.1 protocol-layer no-drift-surface family). **Layered-B3 second project-wide instance closes atomically** (drain-hook layer Q2+Q3 + runtime.rs:181 silent-discard layer Q1; sibling-shape to topo-sort Commit 2a J-101 first instance; two instances is not yet durable pattern — three would be). **Candidate D-NNN "ingest path invariant encoding under bidirectional sustainability discipline"** stays flagged-not-promoted at design doc §8 with scope expanded at J-107. **Grep guardrail scope discipline codified at JOURNAL J-108 sub-section 8 as fifth project instance of surfacing-gap-becomes-codified-discipline pattern**: rule's scope is freeze-site sources (canonical code/spec/test docs hosting J-NNN placeholders), NOT narrative prose in milestone-event documents (CLAUDE.md + JOURNAL.md entries) which use J-NNN as historical pointer at authoring time. Verification command updated form `grep -rn 'J-NNN' . --include='*.rs' --include='docs/*.md' --include='tasks/*.md'` MUST return ZERO post-Commit-4; unconstrained grep returns non-zero by design (~30 narrative-prose hits expected in CLAUDE.md + JOURNAL.md J-107 prose). Flag for runbook §6.7 + §6.8 amendment at next sibling milestone. Sibling-shape lineage: D-070 + D-075 + D-076 v1 → v1.1 + Rule 0 + D-077 + this guardrail discipline form the surfacing-gap-becomes-codified-discipline pattern (five project instances make pattern durable). **Ten files in this atomic commit per D-074 (eleventh instance)**: JOURNAL.md (J-108 entry, eight sub-sections); CLAUDE.md (this header chain + PLAY flip); docs/ROADMAP.md v1.22 → v1.23; tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_IMPL.md ACTIVE → COMPLETED v1.2 + body J-NNN → J-108 freezes (~46 sites); tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_DESIGN.md header chain + body J-NNN → J-108 freezes (8 sites); tasks/FEDERATION_PROPAGATION_PHASE_9.md header Last updated chain; tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md new M16 row + caught-by-Phase-9 count update 12 of 15 → 13 of 16; docs/xgen_federation_propagation_design.md §15 row J-NNN → J-108 freeze + Q1 a.iii.α body correction; xgen-core/src/node/runtime.rs three J-NNN code-comment freezes (lines 74, 216, 832); xgen-node/src/app.rs one J-NNN code-comment freeze (line 2768). HANDOFFs not in Commit 4 (both COMPLETED on remote at J-107). Sentinel-tree files already shipped at Commit 3. Workspace test count at Commit 3 baseline: 592 (xgen-core 431 / xgen-node 68 / xgen-common 24 / xgen-client 47 / integration buckets 7,6,5,2,1,1); +7 across milestone (+2 Commit 2 + +5 Commit 2a + +0 Commit 3 unit tests because §3 work is harness-refinement). No test changes at Commit 4 per runbook §6.7 DoD. Pre-existing flakes did NOT fire during 8/8 verification runs. **"Honest longer work over fast shortcuts" — count inherited at eighth from J-104**, NOT incremented at this milestone close (close-event-not-recurrence-event sibling-shape to topo-sort J-101's seventh-recurrence framing). **D-074 application count: eleventh instance** (J-095 first; J-096 second; J-097 third; J-098-across-two-commits fourth; J-099 fifth; J-100 sixth; J-101 seventh; J-105 eighth; J-106 ninth; J-107 tenth; this J-108 eleventh). **Phase 9 milestone stays PLAY** (RESUMES at Commit 3b-2-equivalent). **Federation Event Propagation milestone stays PLAY** (waiting on Phase 9). **M6 (new) + XGID Retrofit Pass 1 stay PENDING**; dependency chain extended by one more node, depth unchanged in shape. **Next-active for Clair**: Phase 9 Commit 3b-2-equivalent per `tasks/FEDERATION_PROPAGATION_PHASE_9.md` (Status ACTIVE v1.1, RESUMED at J-108). **Next-active for Chat Claude**: standby until Clair's Commit 3b-2-equivalent arc closes; parallel-eligible items M6 (new) Block 4 verb-by-verb walks + future-walk of candidate D-NNN if Joe locks it. Per Rule 0 + D-065 + D-067 + D-069 + D-071 + D-074 + D-077 + grep guardrail scope discipline (this commit's codification) discipline.) Previous J-107 update: 2026-05-24 (J-107 — Persistence-amendment sub-amendment milestone **re-walk Track 1 SHIPPED**: canonical-record amendments + D-077 promotion + HANDOFF Status flip + Track-1-session-close bridge-handoff Status flip in eight-file atomic commit per D-074 (tenth instance). Y-lock cross-milestone Phase 7 B3 amendment dependency surfaced at Clair's Commit 2 forced revert of Q1 from (a).iii.β to (a).iii.α — B3 federation-bootstrap path (J-088, locked 2026-05-20) implicitly relied on `let _ = graph.add_event(...)` silent-discard as a feature; Result-propagation would have broken B3 at SpaceState mutation layer. Five resolution options walked with Joe; then Option X (apply bidirectional sustainability broadly) vs Option Y (revert + name discipline + document broader audit as future-walk); **Joe locked Y on error-loop-risk grounds**. **D-077 promoted** as new principle: bidirectional sustainability discipline at silent-discard / fallible-discard sites — sustainability question MUST be asked in both directions (forward-drift AND backward-coherence) before locking any fix. D-077 sits at meta-layer above no-drift-surface discipline family (D-067 + D-070 + D-075 + D-076 v1.1 at protocol layers; D-077 + Rule 0 at meta layer). Origin pattern sibling-shape to D-076 v1 → v1.1 + Rule 0 + D-075: principle-stated → implementation surfaced gap → amendment makes the missing dimension explicit (four project instances make the pattern durable). **Track-1-while-Clair-active first project instance**, allowed because Track 1 here is record-of-already-locked-decision (Y-lock made in conversation with Clair at session close before Track 1 was authored), not decision-input — directional asymmetry from topo-sort J-099/J-100 Track-1-as-decision-input precedent recorded inline in J-107 sub-section 6. **Eight-file atomic commit per D-074 (tenth instance)** — was seven-file in original J-107 enumeration but expanded to eight when the mid-session bridge-handoff `tasks/HANDOFF_TRACK_1_SESSION_CLOSE_2026-05-24.md` (authored at Chat Claude's session close to bridge Track 1's mid-flight state into the next session) folded into the atomic per anti-tempfile-deletion-of-decision-records discipline (D-065 + sibling-shape to `tasks/HANDOFF_TOPOSORT_RUNBOOK_REVISION.md` retention at J-100): (1) DECISIONS.md — new D-077 entry; (2) JOURNAL.md — J-107 entry (sub-section 9 cluster-framing amendment absorbs Clair's three within-Commit-3 backward-coherence audit gaps as D-077 worked example #2 — abort-fold + identity-registry-persist + space-event-store-persist closed atomically inside Clair's Commit 3 per the locked prospective-sweep ruling); (3) `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_DESIGN.md` v1.1 → v1.2 (§3 amendment subsection + §8 expanded scope + header chain); (4) `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_IMPL.md` v1.0 → v1.1 (§4 amendments .1-.10 + new §7.8 discipline-notes + §4.9 package-scoped-verification correction text + header chain); (5) this CLAUDE.md header `Last updated` re-walk entry chained in front of J-106 — **PLAY block content stays at the persistence-amendment milestone framing per HANDOFF §3.4** (Commit 4 milestone close handles the PLAY flip after Clair's full five-commit sequence closes); (6) `docs/ROADMAP.md` v1.21 → v1.22 (visual structure tree persistence-amendment cluster Commit 2 + Commit 2a + Track-1 + Commit 3 rows annotated; cross-cutting principles section gains D-077 row + candidate D-NNN expanded scope; Past section gains Track-1-shipped + Commit 2 + Commit 2a + Commit 3 entries; header chain); (7) `tasks/HANDOFF_PERSISTENCE_AMENDMENT_REWALK.md` Status ACTIVE → COMPLETED v1.1; (8) `tasks/HANDOFF_TRACK_1_SESSION_CLOSE_2026-05-24.md` Status ACTIVE → COMPLETED v1.1 (bridge-handoff retained as historical record of session-close mechanism per anti-tempfile-deletion discipline; first project instance of Chat-Claude-mid-Track-1-session-close pattern; sibling-shape precedent for future recurrences). **Candidate D-NNN "ingest path invariant encoding" expanded scope** at this re-walk to cover five `ingest_event` silents + three drain helpers + M6 reject paths + B3 apply_event dependency; promotion trigger Joe-lock OR dependent-work concrete drift per D-071 surface-driven application. **Discipline-notes data point of session**: re-walk surfaced three Clair-Commit-2 findings the runbook author (Chat Claude at J-106) missed in their respective ways — sentinel-tree gap (`spawn_in_process_node_with_state` + `InProcessNode::shutdown_keep_data` not in `phase9_harness.rs`); test #4 structural infeasibility (no interleaved mutation point); B3 cross-milestone dependency (forward-only sustainability frame at J-105 missed the backward-coherence question entirely). Three findings individually addressable; together they instantiate D-077's pattern — backward-coherence audit at runbook-authoring time would have surfaced findings #1 + #3 before code shipped (finding #2 is structural-infeasibility surface, separate class). Cumulative within-milestone audit-gap count after Clair's Commit 3 prospective sweep: **five** (sentinel-tree compile-blocker + test #4 infeasibility + B3 dependency + Commit 3's three gaps abort-fold + identity-registry-persist + space-event-store-persist; federation-registry-persist audited and confirmed safe via downstream production path `attempt_reconnect` → `run_federation_session_post_handshake` → `reg.save()` at app.rs:1295). **§4.9 package-scoped-verification correction recorded**: workspace test count delta is +2 (tests 3 + 5), not +5 — tests 1+2 dropped on Result-shape revert, test 4 dropped on structural-infeasibility trace; package-scoped verification at Commit 2 per locked Option C path; workspace 8-green-run rigour deferred to Commit 3 per runbook §5.3 — **Clair shipped 8/8 GREEN at Commit 3 `a677244`**. **"Honest longer work over fast shortcuts" — stays inherited at eighth per within-milestone Shape-2 amendment framing**, NOT incremented at this re-walk surface (sibling-shape to topo-sort re-walk Step 2 J-099 + Step 3 J-100 staying at fifth + sixth recurrence within Federation milestone scope, not opening seventh + eighth). **D-074 application count: tenth instance** (J-095 first; J-096 second; J-097 third; J-098-across-two-commits fourth; J-099 fifth; J-100 sixth; J-101 seventh; J-105 eighth; J-106 ninth; this J-107 tenth). **Verbatim code-comment block at `xgen-core/src/node/runtime.rs:181`** (shipped at Clair's `f4f0e4e` Commit 2 under (a).iii.α) carries the milestone-level J-NNN placeholder; freezes at Commit 4 milestone close, NOT at this re-walk Track 1 commit. Four milestone-close freeze sites enumerated by file:line: `xgen-core/src/node/runtime.rs:181` + `docs/xgen_federation_propagation_design.md` §15 row + `tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md` catalogue M16 row + sentinel-tree doc-comments at `xgen-node/src/tests/phase9_*.rs` — all must freeze in Commit 4 with `grep -rn 'J-NNN' .` from project root returning ZERO matches after staging per runbook §6.7 + §6.8 anti-drift guardrail. **Next-active for Clair after this Track 1 ships**: Commit 4 milestone close per runbook §6 — Track 2 already at Commit 3 ✅ `a677244` on remote with 8/8 GREEN verification; Commit 4 freezes the four milestone-close J-NNN sites, flips runbook v1.1 → v1.2 COMPLETED, flips design doc v1.2 → v1.3 (header chain only, already COMPLETED), bumps ROADMAP v1.22 → v1.23, flips CLAUDE.md PLAY block to "Phase 9 Commit 3b RESUMES" per Q4(a) Commit-3b-1-collapse lock from J-105. **Next-active for Chat Claude after this Track 1 ships**: standby until Clair's Commit 4 closes. Per Rule 0 + D-065 + D-067 + D-069 + D-071 + D-074 + D-077 (this commit's promotion) discipline. Previous J-106 update content stands authoritative — see body J-106 entry below.) Previous J-106 update: 2026-05-23 (J-106 — Persistence-amendment sub-amendment milestone **implementation runbook SHIPPED** at `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_IMPL.md` Status: ACTIVE v1.0, ~95 KB, eight sections (§1 framing + §2 sequence overview + §3 Commit 1 doc-pass + §4 Commit 2 Q1 ingest-path + §4a Commit 2a Q2+Q3 dispatch+persist + §5 Commit 3 sentinel-tree refinement + verify + §6 Commit 4 close + §7 discipline notes (§7.1–§7.7) + §8 cross-references). Sibling-in-shape to `tasks/FEDERATION_TOPOSORT_IMPL.md` (COMPLETED v1.2, ~93 KB) — same eight-section shape; same five-commit Clair-facing sequence shape (1 doc / 2 Q1 / 2a Q2+Q3 / 3 sentinel+verify / 4 close); same §7 discipline-notes inclusion with precedent-departure self-defense at §7.1; landed cleanly inside ~80-100 KB target band. **Four-file atomic commit per D-074** (ninth instance — sibling-shape to topo-sort J-098 runbook-landing housekeeping atom shape, but landed atomic at first attempt rather than as fix-up housekeeping atom per the J-098 + J-099 + J-100 prose-then-batch family of slip-and-correct precedents now being avoided by writing each companion file to disk + verifying via `Filesystem:get_file_info` before the next): (1) `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_IMPL.md` NEW ACTIVE v1.0 (the runbook itself); (2) JOURNAL J-106 entry; (3) CLAUDE.md (this PLAY block flip + header bump); (4) docs/ROADMAP.md v1.20 → v1.21. **Six runbook-structural Joe-locks** baked into runbook prose at §1.1 and §2.3: (1) Five-commit shape (1 doc / 2 Q1 ingest / 2a Q2+Q3 / 3 sentinel+verify / 4 close); (2) Five Joe-lock checkpoints — #1 post-Commit-1 doc-pass drift / #2 pre-Commit-2 unit-test list / #3 post-Commit-2 / pre-Commit-2a primitive shape / #4 pre-Commit-2a verbatim code-comment block (with rungs-list bullet) / #5 post-Commit-2a / pre-Commit-3 sentinel-tree refinement scope; (3) Verification rigour 5 isolated + 3 workspace = 8 green runs minimum at Commit 3; (4) Sentinel-tree refinement folded into Commit 3 with refinement-vs-rework distinction at §5.2; (5) §15 row in Commit 1 with J-NNN placeholder freezing at Commit 4; (6) §7 discipline notes section with six sub-sections (§7.1–§7.7); §7.8 skipped per runbook-authoring §7 lock. **Two pre-draft code-trace findings shaped §4 + §4a narrow scope**: (a) Q1 covers ONLY `graph.add_event` Result-handling at runtime.rs line ~210, NOT the other four silent-discard sites in `ingest_event` (event_id-missing-return; store.insert silent; two state.apply_event silents) — those four sites belong to design doc §8 candidate D-NNN "ingest path invariant encoding" future-walk question, NOT this milestone's scope; §4.1 includes explicit narrow-scope sub-section sibling-shape to topo-sort runbook §4's `build_room_create_event`-only narrow scope; (b) Recursive drain pattern is Shape β2 (each drain helper returns `Vec<Event>`; `dispatch_event` aggregates via concatenation; `process_inbound` persists initial event + iterates `additional_persisted` for drained events) — chosen over Shape β1 (accumulator threaded through recursion) on five grounds named at §4a.4. **Sibling-in-shape fourth recurrence count at this commit**: Phase 7.5 first (J-093/J-104); bidirectional federation_nodes second (J-096); topological-sort third (J-097/J-098/J-099/J-100/J-101); this milestone fourth (J-104/J-105/J-106/upcoming milestone-close). Four recurrences make audit → design → runbook → implementation → milestone-close shape durable. **D-074 application count at this commit: ninth instance** (J-095 first; J-096 second; J-097 third; J-098-across-two-commits fourth; J-099 fifth; J-100 sixth; J-101 seventh; J-105 eighth; J-106 ninth — landed atomic at first attempt per honest framing improvement over J-098 slip-and-correct shape). **"Honest longer work over fast shortcuts" — count inherited at eighth from J-104, NOT incremented** at runbook-authoring close (recurrences counted at milestone-events; this is within-milestone runbook-landing event). **Next-active for Clair**: pickup at Commit 1 of five-commit sequence per runbook §3. Sentinel working tree (four files at `xgen-node/src/tests/`) remains uncommitted as verification artifact for milestone close per Q4(a) — do NOT `git restore`. Per Rule 0 + D-065 + D-067 + D-069 + D-071 + D-074. Previous J-105 v1.20 content stands authoritative — see body J-105 entry below for the four design-phase Joe-locks.) Previous J-105 update: 2026-05-23 (J-105 — Persistence-amendment sub-amendment milestone design phase SHIPPED. Five-file atomic commit per D-074 (eighth instance). Four Joe-locks recorded: **Q1 → (a).ii + (a).iii.β + candidate D-NNN flag** (sort-on-replay in `replay_spaces_from_dir` + `ingest_event` returns `Result<(), GraphError>` + candidate D-NNN "ingest path invariant encoding" flagged for future walk); **Q2 → (a) return-vector** (`DispatchOutcome::Accepted { new_joiner, additional_persisted: Vec<Event> }`); **Q3 → all three drain helpers** (`drain_pending_uniform`, `drain_pending_by_identity`, `drain_pending_by_federation_relationship`); **Q4 → (a) in-scope** (sentinel-tree ships atomic at milestone close; Commit 3b-1 collapses into milestone close). No re-walk fired per Lock #2 discipline. Sustainability question forced mid-walk reframing of Q1 from (a).iii.α (log-level) to (a).iii.β (type-level) after "is this future-proof?" challenge surfaced three drift surfaces (a).iii.α does NOT catch; second-order question "is (a).iii.β the future-proof solution?" forced honesty about rungs above (ValidatedEvent wrapper, sealed traits, formal verification); resolution: lock (a).iii.β as immediate answer + flag candidate D-NNN for future walk per D-069. Layered-B3 recurrence count: second project-wide instance (first topo-sort Commit 2a at J-101). Five files: (1) `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_DESIGN.md` NEW ACTIVE v1.0; (2) `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT.md` Status flipped ACTIVE → COMPLETED v1.1; (3) JOURNAL.md (J-105 entry); (4) CLAUDE.md (this PLAY block flip + header bump); (5) `docs/ROADMAP.md` v1.19 → v1.20. **Next-active for Chat Claude + Joe**: implementation runbook authoring at `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_IMPL.md` per topo-sort precedent J-098. Clair stays stood down through milestone close; sentinel working tree (four files) remains uncommitted as verification artifact per Q4(a). Per D-065 + D-067 + D-069 + D-071 + D-074. Previous J-101 update content stands authoritative — see J-101 summary below.) Previous J-101 update: 2026-05-23 (J-101 — Topological-sort wire-order determinism milestone **CLOSED**. Five-commit Clair-facing sequence shipped under amended D-076 v1.1: Commit 1 doc-pass (parents of `0543a86`) + Commit 2 determinism layer (`0543a86`) + Commit 2a causality layer (`4a6fd74`) + Commit 3 Phase 9 Scenario 1 second `#[ignore]` lift (`b370dc7`) + Commit 4 milestone close (this commit, eight files atomic per D-074). **Commit 2a expansion under Option E.** Scoped to one file at runbook authoring (xgen-core/src/space/state.rs build_room_create_event); expanded to six files atomic during implementation when the initial Path B edit surfaced two layered B3-shape encodings of the DAG-root invariant — `is_dag_root_type` at xgen-core/src/dag/graph.rs:29 (validator companion #1) and `validate_dag_structure` at xgen-core/src/message/exchange.rs:550 (validator companion #2, inline `matches!` block re-encoding the same DAG-root set). Per D-067 no-drift-surface discipline, Option E unification: `is_dag_root_type` made `pub(crate)`, inline encoding replaced with delegated call. Plus Posture β dag-test fixture updates across 17 sites in 4 files (graph.rs 7 + mod.rs 7 + pending.rs 2 + store.rs 1) switching `EventType::StateRoomCreate` → `EventType::StateSpaceCreate` for codebase-wide invariant coherence. **Two layered B3-shape surfaces closed atomically within Commit 2a** — structurally novel pattern. Bidirectional Commit 2.5 (J-096) closed one B3 surface; Commit 2a closed two layered surfaces (graph.rs + exchange.rs), both surfaced during implementation testing of the same primary fix. The "primary fix traversed two sibling-layer encodings of the same invariant before codebase coherence" pattern is the discipline-notes data point for future audit work to look for layered B3 surfaces, not just single ones. **Verification rigour at Commit 3**: 5 isolated runs (cargo clean between each) + 3 workspace runs = 8 green runs minimum per runbook §5.3; all 8 passed; pre-existing flakes (precedence env-var race; reconnect_with_existing_tip_small_delta_delivered) did not fire. **D-074 application count via grep at authoring time**: 21 mentions in JOURNAL.md; simple-counting milestone-close applications J-095–J-101 = seventh instance at J-101. **"Honest longer work over fast shortcuts" — seventh recurrence within Federation Event Propagation milestone scope** (Phase 7.5 + bidirectional + topological-sort design close J-097 + runbook landing J-098 + re-walk Step 2 J-099 + re-walk Step 3 J-100 + topological-sort implementation J-101). **D-071 fifth project-wide instance closes** at this commit. **Prose-then-batch atomicity-slip family — fourth recurrence acknowledged**: design doc §15 line 1140 retroactive J-096 freeze (bidirectional milestone close authoring intended the freeze but the placeholder did not land in J-096's Commit 4; gap surfaced at Commit 4 authoring of THIS milestone; sibling-shape to J-094 + J-098 + J-099-eighth-file). **Latent test-comment-vs-actual-behaviour finding** (sidebar, lower-severity, non-blocking): xgen-client/src/app.rs:1551 + :4539 DM-second-room negative tests finally exercise the `DmSecondRoomNotAllowed` rule they advertised in their `pass!` messages — pre-Commit-2a they were actually witnessing `validate_dag_structure`'s root-event rejection first; post-Commit-2a they witness the rule they claim to lock. "Tests-finally-lock-what-they-claimed" pattern. **Topological-sort milestone flips PLAY → DONE; Phase 9 Commit 3b RESUMES**; Phase 9 milestone stays PLAY (waiting on Commit 3b's own completion); Federation Event Propagation milestone stays PLAY; M6 (new) + XGID Retrofit Pass 1 stay PENDING. **Eight files at this commit per D-074**: JOURNAL.md (this J-101 entry) + CLAUDE.md (PLAY block flipped from "Clair pickup at Commit 2a → Commit 3 → Commit 4 ←── HERE" to "Phase 9 Commit 3b ←── HERE") + docs/ROADMAP.md v1.16 → v1.17 + tasks/FEDERATION_TOPOSORT_IMPL.md Status ACTIVE → COMPLETED v1.2 + tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md gains M15 catalogue row + tasks/FEDERATION_PROPAGATION_PHASE_9.md header Last updated paragraph (Commit 3b RESUMED) + docs/xgen_federation_propagation_design.md §15 two J-NNN freezes (line 1140 → J-096 retroactive; line 1141 → J-101) + xgen-node/src/tests/phase9_two_node_smoke.rs J-NNN → J-101 freeze in doc-comment. Per D-065 + D-067 + D-069 + D-071 + D-074 + D-075 + D-076 v1.1 discipline. Previous J-100 update content preserved authoritative — see J-100 summary below.) Previous J-100 update: 2026-05-22 (J-100 — Topological-sort design-phase re-walk **Step 3 SHIPPED**: implementation runbook revised v1.0 → v1.1 via five-file atomic commit per D-074. Substantive change: new §4a Commit 2a section inserted in `tasks/FEDERATION_TOPOSORT_IMPL.md` between §4 (existing Commit 2 — preserved as determinism-layer historical record) and §5 (Commit 3 — `#[ignore]` lift), plus eleven local amendments making the two-property contract under amended D-076 v1.1 visible end-to-end. Five-file atomic commit per D-074: revised runbook + Step-3 HANDOFF closed + JOURNAL J-100 + this CLAUDE.md PLAY block flip + ROADMAP.md v1.15 → v1.16. **"Commit 2a" naming Joe-locked at Step 3 open** on three grounds (honest about sequence, table-friendly, precedent-fit: letter-suffix-insertion-at-different-layer rather than the bidirectional precedent's decimal-sibling-half-step-at-same-layer shape). **Step-2-bis fix-up atom retrospective** included in J-100: `tasks/HANDOFF_TOPOSORT_RUNBOOK_REVISION.md` was named in J-099's eight-file enumeration but did not land in the e0c5d36 commit; surfaced at post-J-099 session-open verification; authored as single-file fix-up atom rather than rolled into Step 3 per D-065 honest-framing discipline; gets single-sentence acknowledgement inside J-100, no separate J-NNN entry (sibling-shape to J-098's housekeeping atom precedent). **Path B locked at J-098 session close** stays the substantive Clair-facing fix: modify `build_room_create_event` at `xgen-core/src/space/state.rs:797` to set `prev_events: vec![space_id.to_string()]`. Narrow scope (constructor only; sibling event constructors deferred to D-071 audit arc if dependent work surfaces need). **Five-commit Clair-facing sequence:** Commit 1 ✅ + Commit 2 ✅ (determinism layer) + Commit 2a 🟡 (causality layer, NEW) + Commit 3 🟡 (Scenario 1 second `#[ignore]` lift) + Commit 4 🟡 (milestone close per D-074). **Four Joe-lock checkpoints** (was three): #1 post-Commit-1, #2 pre-Commit-2, #3 post-Commit-2 / pre-Commit-3 (extended to cover Commit 2a stability), **#4 NEW pre-Commit-2a verbatim code-comment block content** with four locked structural elements. **Clair's stand-down ends with this commit.** She resumes at Commit 2a per the revised runbook; her existing Commit 3 working tree stays as sentinel through this Step 3 close. CLAUDE.md PLAY block flipped from "Step 3 runbook revision authoring ←── HERE (Chat Claude + Joe next session)" to "Clair pickup at Commit 2a → Commit 3 → Commit 4 ←── HERE". ROADMAP.md v1.15 → v1.16. **D-074 application count:** sixth instance (J-095 locked; J-096 + J-097 + J-098-across-two-commits + J-099 + J-100). **"Honest longer work over fast shortcuts" — sixth recurrence within Federation Event Propagation milestone scope** (Phase 7.5; bidirectional; topological-sort design close J-097; runbook landing J-098; re-walk Step 2 J-099; re-walk Step 3 J-100). Per D-065 + D-069 + D-071 + D-074 + D-076 v1.1 discipline. Previous J-099 content stands authoritative — see DONE-IN-FLIGHT block.) Previous 2026-05-22 update: J-099 — Topological-sort design-phase re-walk **Step 2 SHIPPED** in an eight-file atomic commit per D-074: audit doc §11 amendment + header v1.0 → v1.1 recording the framing gap surfaced at Clair's Commit 3 verification (Q3 asked "is determinism normative?" but did not ask "what semantic property must the canonical order satisfy?"); design doc §11 amendment + header v1.0 → v1.1 recording Q4 (causal-DAG-respecting order as load-bearing property) + Q1 supplement (Path B at event-construction layer); DECISIONS.md **D-076 amended in place** (new "Amendment (2026-05-22)" subsection between "Decision" and "Originating incident" — v1 prose stays authoritative; amendment names the second load-bearing property: byte-identical-across-senders determinism + causal-DAG-respecting order, two complementary properties of one principle, no D-077 split); JOURNAL J-099 entry with the discipline retrospective; **CLAUDE.md Rule 0 added** as fourth member of the MANDATORY Behaviour rules — mandatory session-open reading sequence (CLAUDE.md PLAY block → latest JOURNAL entry → ACTIVE HANDOFF notes → then whatever document Joe pointed at), holds regardless of how the user opens the session, narrow pointer "read X" treated as "expand to context, then read X"; sibling-shape to how D-076 v1.1 made the second load-bearing property explicit (v1 contract written → gap surfaced at implementation → amendment makes the missing property explicit; same pattern at the meta-level for session-open discipline); CLAUDE.md PLAY block flipped from "Implementation ←── HERE (Clair pickup)" to "Step 3 runbook revision authoring ←── HERE (Chat Claude + Joe next session)"; ROADMAP.md v1.14 → v1.15; Step-2 HANDOFF Status flipped ACTIVE → COMPLETED v1.1; new Step-3 HANDOFF authored at `tasks/HANDOFF_TOPOSORT_RUNBOOK_REVISION.md` Status: ACTIVE v1.0. **Path B locked at J-098 session close** as the fix shape: modify `build_room_create_event` at `xgen-core/src/space/state.rs:797` to set `prev_events: vec![space_id.to_string()]` so the event-DAG honestly reflects the protocol-level parent-child relationship the function's own doc-comment already claims. Narrow scope: `build_room_create_event` only; sibling event constructors (`state.federation_add`, `membership.*`, `message.*`) NOT audited this milestone, may surface later as own audit-precedes-dependent-design arc per D-071. **Commit 2's Shape A v1 sort fix stays useful** at the determinism layer beneath the causality layer (not reverted; safety net for events that legitimately tie at the DAG layer). The two fixes layer cleanly: causality first (Path B), determinism second (Commit 2 sort). Rejected alternatives at re-walk: Path A (EventType-priority sort at topo primitive — wrong layer); Path C (broader sibling-constructor audit — own future phase per D-071 if dependent work surfaces need). Procedural shape is Shape 2 (targeted patch, not full audit→design→impl re-walk) per Joe-lock at J-098 session close: Step 1 (J-098-session-close Joe-lock conversation) ✅ closed; Step 2 (this commit's eight-file canonical-record amendments) ✅ closed; Step 3 (runbook v1.0 → v1.1 revision in own session-arc per `tasks/HANDOFF_TOPOSORT_RUNBOOK_REVISION.md`) 🟡 next-active; Clair stays stood down until Step 3 closes; her Commit 3 working tree (`xgen-node/src/tests/phase9_two_node_smoke.rs`, uncommitted) remains as sentinel. **Rule 0 origin story.** The post-J-098 session opened with a narrow pointer (the user pasted `HANDOFF_TOPOSORT_DESIGN_REWALK.md` filename only, no surrounding context); Chat Claude's defensible narrow-reading interpretation was "read this in isolation," which bypassed the bridges (PLAY block + JOURNAL + HANDOFF) that the project's structural defences exist to provide. The runbook v1.0 was partially superseded at session-open time; reading it as ground truth produced an offer to do work that was two commits stale and missed the Path B Joe-lock entirely. Rule 0 makes the session-open reading sequence permanent project discipline rather than tacit expectation. Sibling to D-070 (originated from J-081's M6 §9 framing gap), D-075 (from bidirectional vantage-awareness gap), D-076 v1 (from this milestone's framing gap); each principle originated from a discipline failure that surfaced. Rule 0's surface is meta-level (session-open discipline) rather than protocol-level. **"Honest longer work over fast shortcuts" — fifth recurrence within Federation Event Propagation milestone scope.** Phase 7.5 first; bidirectional second; topological-sort design close (J-097) third; topological-sort implementation runbook landing (J-098) fourth; topological-sort design-phase re-walk Step 2 (this entry) fifth. Each delays milestone closure by one session-arc; each closes a real gap before it ships. Federation Event Propagation milestone closure dependency chain extended by one more node (this re-walk); Phase 9 Scenario 1 stays `#[ignore]`-annotated pending the amended fix; Phase 9 Commit 3b stays paused inside milestone scope; Federation Event Propagation milestone stays PLAY; M6 (new) + Pass 1 stay PENDING. **D-074 application count.** J-095 (XGID Adoption v1 milestone close) first; J-096 (bidirectional milestone close) second; J-097 (topological-sort design-phase close) third; J-098 across two commits (runbook-landing + companion-updates housekeeping) fourth; J-099 (this commit's Step-2 atomic commit) fifth instance. Per D-065 + D-069 + D-071 + D-074 + D-076 discipline. Previous J-098 runbook-shipped + companion-updates content stands authoritative — see DONE-IN-FLIGHT block.) Previous 2026-05-22 update: J-098 — Topological-sort wire-order non-determinism **implementation runbook SHIPPED** at `tasks/FEDERATION_TOPOSORT_IMPL.md` Status: ACTIVE v1.0, eight sections, sibling-in-shape to `tasks/FEDERATION_BIDIRECTIONAL_NODES_IMPL.md` (COMPLETED v1.1) with one structural addition — §7 discipline notes; precedent departure self-defended at §7.1 on three grounds (trilogy-internal consistency outranks one-step-earlier-precedent consistency when they conflict; bidirectional precedent's absence of §7 was absence-of-need at second sibling-recurrence; D-076 family-completion as fourth member of no-drift-surface discipline family needs runbook-visible pointer). Four-commit Clair-facing sequence for the locked design phase (J-097): Commit 1 doc-pass (audit + design Status flips, canonical design doc §6.4.3 sibling subsection + §15 row); Commit 2 primitive fix at `xgen-node/src/fanout.rs:193` + sibling Site 1 fix at `:321` + three-to-five unit tests including the wire-order-determinism witness as load-bearing fourth; Commit 3 Phase 9 Scenario 1 second `#[ignore]` lift with 5 isolated + 3 workspace = 8 green runs minimum verification; Commit 4 milestone close per D-074. Three Joe-lock checkpoints for Clair (post-Commit-1 drift; pre-Commit-2 test-list proposal; post-Commit-2 / pre-Commit-3 primitive shape locked). Three Joe-locks from design phase (Q3.ii canonical wire ordering required; Q2 middle + Q2.γ primitive-fix + Node-to-Client forward-binding; Q1 Shape A v1 + sibling Site 1 fix) carried forward as already-decided per inline-lock pattern third recurrence. D-076 family-completion runbook-visible at §7.5 as fourth member of no-drift-surface discipline family across four protocol layers (D-067 + D-070 + D-075 + D-076). **J-098 shipped across two commits per honest provenance**: original runbook-landing commit shipped tasks/FEDERATION_TOPOSORT_IMPL.md only because Chat Claude drafted the three companion-file edits (CLAUDE.md + ROADMAP.md + JOURNAL.md) as chat prose without writing them to disk turn-by-turn, missing D-074 same-commit-atomicity at the original commit; companion-updates housekeeping atom (this commit) closes the gap with this entry framing the slip explicitly per D-065 honest-behaviour-over-polite-behaviour. Discipline note: when drafting multi-file commits, write each file edit to disk via `Filesystem:edit_file` before moving to the next file's draft, not as prose-then-batch — prose-then-batch defers tool calls past the point where confirmation requests trigger, breaking the implicit assumption that drafted content has landed. Topological-sort phase moves from ✅ Design + 🟢 Implementation runbook authoring ←── HERE to ✅ Design + ✅ Implementation runbook + 🟡 Implementation ←── HERE for Clair pickup. Phase 9 Scenario 1 stays `#[ignore]`-annotated pending Commit 3 of Clair's arc; Phase 9 Commit 3b stays paused pending Commit 4 of Clair's arc; Federation Event Propagation milestone stays PLAY pending Phase 9 closure; M6 (new) + Pass 1 stay PENDING pending Federation milestone closure. ROADMAP.md v1.13 → v1.14 in same commit per same-commit discipline. Per D-069 + D-071 + D-074 + D-076 discipline. Previous J-097 design-phase-close content stands authoritative — see DONE-IN-FLIGHT block.) Previous 2026-05-22 update: Topological-sort wire-order non-determinism design phase SHIPPED. Three Joe-locks recorded across the design walkthrough: Q3 at Q3.ii (canonical wire ordering required — wire-order determinism is a sender-side normative property for Node-to-Node federation; two senders with identical Space history MUST produce byte-identical federation deltas modulo signature-bearing fields); Q2 at Q2 middle + Q2.γ (fix the primitive's contract once; explicit forward-binding to Node-to-Client sender output where analogous — `collect_sync_history` + `apply_fanout` history-push flagged for future scheduling); Q1 at Shape A v1 + sibling Site 1 fix (event_id lexicographic sort at `topological_sort_events:193` ready-siblings tie-break + sibling sort at `compute_federation_delta_for_space:321` HashMap-feed). Pass-1-neutral via v1 `&str` sort + code-comment block flagging Pass 3 retype to `EventXgid`. D-076 promoted to DECISIONS.md as the protocol-design principle the locks instantiate, sibling-distinct from D-067 (code-organisation) + D-075 (event-model); pairs with D-070 (transport-layer correlation pair) as the four-decision no-drift-surface discipline family. Six files in this atomic close commit per D-074 same-commit discipline: JOURNAL.md (J-097), `tasks/FEDERATION_TOPOSORT_DESIGN.md` (NEW, ACTIVE v1.0), DECISIONS.md (D-076 promoted), `tasks/FEDERATION_TOPOSORT_AUDIT.md` (Status flipped ACTIVE → COMPLETED v1.0), CLAUDE.md (this PLAY block flip + header bump), `docs/ROADMAP.md` (v1.12 → v1.13, visual tree Design row ✅ + new runbook-authoring row 🟢 + Past gains design-shipped paragraph). Implementation runbook authoring is the next-active step for Chat Claude + Joe in a fresh session per the split-session discipline (bidirectional precedent: design task file + runbook kept as separate artefacts; different audiences benefit from different headspace — design exposition for Chat Claude + Joe re-entry, runbook commit-shape for Clair). Topological-sort phase moves from 🟢 Design ←── HERE to ✅ Design + 🟢 Implementation runbook authoring ←── HERE. Phase 9 Scenario 1 stays `#[ignore]`-annotated pending fix landing; Phase 9 Commit 3b stays paused inside milestone scope. Previous J-096 milestone-close content stands authoritative — see DONE-IN-FLIGHT block.) Previous 2026-05-22 update: Topological-sort wire-order non-determinism audit doc SHIPPED at `tasks/FEDERATION_TOPOSORT_AUDIT.md` Status: ACTIVE v1.0. Ten sections per the sibling-in-shape template at `tasks/FEDERATION_BIDIRECTIONAL_NODES_AUDIT.md`. Frames Q1 (tie-break choice), Q2 (D-067 audit scope — narrow/middle/wide readings), Q3 (wire-format normative question — LOAD-BEARING) and four candidate fix shapes (A `event_id` sort at topo primitive / B timestamp sort / C canonical-event-bytes sort / D EventStore container change with D.1 BTreeMap + D.2 IndexMap sub-options) with §7.5 summary table. Phase 9 Scenario 1 stays `#[ignore]`-annotated pending fix. Phase 9 Commit 3b stays paused. Topological-sort phase moves from 🟢 Audit ←── HERE to 🟢 Design ←── HERE in the four-step arc; Chat Claude + Joe author `tasks/FEDERATION_TOPOSORT_DESIGN.md` next, may promote a new D-NNN to DECISIONS.md at design close. Previous J-096 milestone-close content stands authoritative — see DONE-IN-FLIGHT block. Previous 2026-05-21 update: Bidirectional `federation_nodes` implementation milestone CLOSED in four-commit sequence per JOURNAL J-096 — Commit 1 doc-pass (`e975162`), Commit 2 origin-aware applier + plumbing + six unit tests (`a730eda`), Commit 2.5 sibling vantage-aware drain hook (`cbceb41`, in-flight gap closure), Commit 3 Phase 9 Scenario 1 resurrection (`f051039`), Commit 4 milestone close (this commit). Test count 571 → 577 across the milestone (+6 unit tests in Commit 2; Scenario 1 originally lifted in Commit 3 then re-stood-down in this commit on a separate topological-sort finding, net wash on ignored count). Bidirectional fix is verified-correct — six unit tests in `xgen-core/src/space/state.rs::mod tests` (notably `apply_federation_add_two_vantages_mirror`) remain green and stand as the live D-075 regression lock. SEPARATE pre-existing finding surfaced during Commit 4 verification, NOT implicated by the bidirectional fix: wire-order non-determinism in `topological_sort_events` at `xgen-node/src/fanout.rs:193` fed by non-deterministic `HashMap.values()` iteration in `compute_federation_delta_for_space` at the same file ~:321. Both `state.space_create` and `state.room_create` are DAG roots with empty `prev_events`; they tie at the top of the topo sort and their wire order tracks HashMap iteration. Evidence: 105 `dispatch_event` calls on B in both pass and fail runs (divergence is upstream wire order); in a failing run `room_create` reaches B's dispatcher before `space_create` and is rejected with "space not found", cascading through the bootstrap chain. Per Joe-direction this opens as its own phase per D-071, sibling-shape to the bidirectional `federation_nodes` audit → design → impl arc just closed. Phase 9 Scenario 1 re-stood-down with updated stand-down comment naming the topological-sort finding and forward-referencing placeholder `tasks/FEDERATION_TOPOSORT_AUDIT.md` (to be authored by Chat Claude as the new active phase). Phase 9 Commit 3b stays paused inside milestone scope. ROADMAP.md v1.10 → v1.11 in same commit per same-commit discipline; `tasks/FEDERATION_BIDIRECTIONAL_NODES_IMPL.md` Status flipped ACTIVE → COMPLETED v1.1. Per D-069 canonical-document + D-071 audit-precedes-dependent-design + D-074 milestone-close-includes-JOURNAL + D-075 + "honest longer work over fast shortcut" discipline. M6 (new) blocking chain extended by one more node, unchanged in shape. Pass 1 of XGID Retrofit stays at Status: ACTIVE v2.0 unchanged.)  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  

---

## 🟢 PLAY — Phase 9 Commit 3b-2-equivalent RESUMES — Scenarios 2 + compounds C2/C5/C7/C9/C10 (C3 dropped per Phase 9 §3.0 v1.1); persistence-amendment sub-amendment milestone CLOSED at J-108 per Q4(a) lock; Commit 3b-1 numbering effectively skipped (collapsed into sub-amendment milestone close)

**Persistence-amendment sub-amendment milestone CLOSED 2026-05-24 at J-108** in a twelve-file atomic commit per D-074 (eleventh instance). Sentinel-tree four files at `xgen-node/src/tests/` shipped atomic at Commit 3 `a677244` (`phase9_harness.rs` gained `SavedNodeState` struct + `InProcessNode::shutdown_keep_data` method + `spawn_in_process_node_with_state` free fn; `phase9_three_node_anti_transitivity.rs` NEW; `phase9_drop_and_recover.rs` NEW J-104-authored; `mod.rs` flip two new sentinel mod declarations) and crown as activating integration-level regression lock at this milestone close per Q4(a). Scenario 3 transition FAIL → PASS verifies the persistence fix end-to-end at integration level.

**Five-commit Clair-facing sequence + re-walk Track 1** shipped: Commit 1 `0ca29e6` doc-pass + Commit 2 `f4f0e4e` Q1 ingest-path + Commit 2a `c88fd73` Q2+Q3 dispatch+persist + Commit 3 `a677244` sentinel-tree refinement + verify (5 isolated + 3 workspace = 8/8 GREEN runs per runbook §5.3) + re-walk Track 1 `b9a30da` at J-107 (D-077 promoted) + this Commit 4 milestone close.

**Five Joe-locks closed across milestone Q1→Q4 walk + re-walk Y-lock**:
- **Q1 (a).iii.α + (a).ii defensive sort-on-replay** — reverted from (a).iii.β at J-107 Track 1 per Y-lock on cross-milestone Phase 7 B3 amendment dependency. `NodeRuntime::ingest_event` keeps binary-void signature + `tracing::error!` at the `graph.add_event` silent site + verbatim code-comment block locked at the call site (lines 74 + 216 + 832 of `xgen-core/src/node/runtime.rs` reference J-108). `replay_spaces_from_dir` at `xgen-node/src/app.rs:2628` sorts events topologically via xgen-core's re-exported `topological_sort` before each `ingest_event` call.
- **Q2 (a) return-vector** — `DispatchOutcome::Accepted` gains `additional_persisted: Vec<Event>` field; `dispatch_event` aggregates drained-event vectors at three drain call sites via concatenation (Shape β2 — initial event stays with `process_inbound`'s existing persist site).
- **Q3 all-three drain helpers** — `drain_pending_uniform` + `drain_pending_by_identity` + `drain_pending_by_federation_relationship` all change signature to return `Vec<Event>` containing drained events; same-family-same-atomic-close.
- **Q4 (a) sentinel-tree in-scope** — four files ship atomic at Commit 3 + crown at milestone close. Phase 9 Commit 3b-1 collapses into this sub-amendment milestone close.

**D-077 promoted at J-107 Track 1**: bidirectional sustainability discipline at silent-discard / conditional-mutation / fallible-discard sites — sustainability question MUST be asked in both directions (forward-drift AND backward-coherence) before locking any fix. Meta-layer above D-067 + D-070 + D-075 + D-076 v1.1 protocol-layer no-drift-surface family.

**Layered-B3 second project-wide instance closes atomically**: drain-hook layer (Q2+Q3) + `xgen-core/src/node/runtime.rs:181` silent-discard layer (Q1) — primary fix surfaces secondary silent-error encoding closed inside same milestone scope. Sibling-shape to topo-sort Commit 2a J-101 first instance. Two instances is not yet durable pattern — three would be.

**Candidate D-NNN "ingest path invariant encoding under bidirectional sustainability discipline"** stays flagged-not-promoted at design doc §8 with scope expanded at J-107 (covering five `ingest_event` silents + three drain helpers + M6 reject paths + Phase 7 B3 apply_event dependency). Promotion trigger per D-071 surface-driven application: dependent work surfacing concrete drift OR Joe-lock on philosophical grounds.

**Grep guardrail scope discipline codified at JOURNAL J-108 sub-section 8 as fifth project instance** of the surfacing-gap-becomes-codified-discipline pattern (sibling-shape lineage: D-070 + D-075 + D-076 v1 → v1.1 + Rule 0 + D-077 + this guardrail discipline). The runbook §6.7 + §6.8 grep guardrail's scope is freeze-site sources (canonical code/spec/test docs hosting J-NNN placeholders), NOT narrative prose in milestone-event documents (CLAUDE.md + JOURNAL.md entries) which use J-NNN as historical pointer at authoring time. Verification command updated form: `grep -rn 'J-NNN' . --include='*.rs' --include='docs/*.md' --include='tasks/*.md'` MUST return ZERO post-Commit-4; unconstrained `grep -rn 'J-NNN' .` returns non-zero by design. Flag for runbook §6.7 + §6.8 amendment at next sibling milestone.

**Ten files in this atomic commit per D-074 (eleventh instance)**:

1. `JOURNAL.md` — J-108 entry (eight sub-sections, ~2500 words) chained ahead of J-107
2. `CLAUDE.md` — this PLAY block flip + header chain
3. `docs/ROADMAP.md` v1.22 → v1.23 — visual tree + Past + Present + frontier lines + header chain
4. `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_IMPL.md` Status ACTIVE → COMPLETED v1.2 + body J-NNN → J-108 freezes (~46 sites) + header chain
5. `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_DESIGN.md` header chain (already COMPLETED v1.2 from Track 1) + body J-NNN → J-108 freezes (8 sites)
6. `tasks/FEDERATION_PROPAGATION_PHASE_9.md` header `Last updated` chain — Commit 3b-1 collapsed; RESUMES at Commit 3b-2-equivalent
7. `tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md` new M16 catalogue row + "Caught by Phase 9 set" count update 12 of 15 → 13 of 16
8. `docs/xgen_federation_propagation_design.md` §15 row J-NNN → J-108 freeze + Q1 a.iii.α body correction (was a.iii.β at Commit 1 doc-pass)
9. `xgen-core/src/node/runtime.rs` three J-NNN code-comment freezes → J-108 (lines 74, 216, 832)
10. `xgen-node/src/app.rs` one J-NNN code-comment freeze → J-108 (line 2768)
11. (HANDOFFs not in Commit 4 — both already COMPLETED on remote at J-107)
12. (Sentinel-tree files already shipped at Commit 3 with no J-NNN markers in doc-comments — authored with the milestone reference, not a J-NNN placeholder)

**Workspace test count at Commit 3 baseline**: 592 (xgen-core 431 / xgen-node 68 / xgen-common 24 / xgen-client 47 / integration buckets 7,6,5,2,1,1). Delta across milestone: +7 (+2 Commit 2 + +5 Commit 2a + +0 Commit 3 unit tests because §3 work is harness-refinement; the sentinel tests at `phase9_three_node_anti_transitivity.rs` + `phase9_drop_and_recover.rs` are themselves the regression locks). No test changes at this Commit 4 per runbook §6.7 DoD. Pre-existing flakes (precedence env-var race; `reconnect_with_existing_tip_small_delta_delivered`) did NOT fire during 8/8 verification runs at Commit 3.

**"Honest longer work over fast shortcuts" — count inherited at eighth from J-104, NOT incremented at this milestone close** (close-event-not-recurrence-event sibling-shape to topo-sort J-101's seventh-recurrence framing).

**D-074 application count: eleventh instance** (J-095 first; J-096 second; J-097 third; J-098-across-two-commits fourth; J-099 fifth; J-100 sixth; J-101 seventh; J-105 eighth; J-106 ninth; J-107 tenth; this J-108 eleventh).

**Phase 9 Commit 3b-2-equivalent next-active scope** per `tasks/FEDERATION_PROPAGATION_PHASE_9.md` (Status ACTIVE v1.1, RESUMED at J-108): Scenarios 2 (three-Node anti-transitivity) + compound scenarios C2/C5/C7/C9/C10 per existing Q4 Lock from J-091 + Phase 9 §3.0 revised five-commit shape with C3 dropped to `tasks/FEDERATION_STRESS_FOLLOWON.md`. Scenario 2 may also be considered closed at this milestone if the sentinel-tree file `phase9_three_node_anti_transitivity.rs` activates Scenario 2 alongside Scenario 3 — Clair confirms at Commit 3b-2-equivalent open. Expected ~4-6 atomic commits in their own sequence. After Phase 9 Commit 3b-2-equivalent ships, Phase 9 milestone flips PLAY → DONE, Federation Event Propagation milestone flips PLAY → DONE, and M6 (new) + XGID Retrofit Pass 1 unblock simultaneously.

**What stays paused/pending.** Phase 9 milestone stays PLAY (RESUMES at Commit 3b-2-equivalent). Federation Event Propagation milestone stays PLAY (waiting on Phase 9). M6 (new) + XGID Retrofit Pass 1 stay PENDING (chain extended by one more node — this sub-amendment milestone — dependency depth unchanged in shape).

**Track 1 (Chat Claude + Joe): no active work** until Clair's Commit 3b-2-equivalent arc closes. Possible parallel work: M6 (new) Block 4 verb-by-verb walks; future-walk of candidate D-NNN "ingest path invariant encoding" if Joe locks it as worth pursuing.

**Track 2 (Clair): pickup at Commit 3b-2-equivalent** per `tasks/FEDERATION_PROPAGATION_PHASE_9.md` (Status ACTIVE v1.1, RESUMED at J-108). Stand-down ends with this milestone close.

**Entry point for Clair: `tasks/FEDERATION_PROPAGATION_PHASE_9.md`** (read CLAUDE.md PLAY block + J-108 entry + J-107 entry first per Rule 0, then Phase 9 task file §3.0 revised five-commit shape, then Commit 3b-2-equivalent scoping).

**Entry point for Chat Claude's next session (no active work): standby** until Clair's Commit 3b-2-equivalent arc closes; parallel-eligible items above.

---

## ✅ DONE-IN-FLIGHT — Persistence-amendment sub-amendment milestone RUNBOOK ✅ at J-106 (then re-walk Track 1 at J-107 amended in-place); Clair shipped Commits 1+2+2a+3; persistence-amendment milestone CLOSED at J-108 per Q4(a) Commit-3b-1 collapse

**Persistence-amendment implementation runbook SHIPPED 2026-05-23 at J-106** at `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_IMPL.md` Status: ACTIVE v1.0 (~95 KB, eight sections, inside ~80-100 KB target band). Sibling-in-shape to `tasks/FEDERATION_TOPOSORT_IMPL.md` (COMPLETED v1.2). Four-file atomic commit per D-074 ninth instance: runbook NEW ACTIVE v1.0 + JOURNAL J-106 + CLAUDE.md (this PLAY block flip + header bump) + ROADMAP.md v1.20 → v1.21.

**Six runbook-structural Joe-locks** baked into runbook prose (full statements at runbook §1.1 + §2.3):

1. **Five-commit shape**: 1 doc-pass / 2 Q1 ingest-path / 2a Q2+Q3 dispatch+persist / 3 sentinel+verify / 4 close.
2. **Five Joe-lock checkpoints**: #1 post-Commit-1 doc-pass drift / #2 pre-Commit-2 unit-test list proposal / #3 post-Commit-2 / pre-Commit-2a primitive shape / #4 pre-Commit-2a verbatim code-comment block content (with rungs-list bullet for ValidatedEvent wrapper + sealed traits + visitor pattern + formal verification rungs above (a).iii.β) / #5 post-Commit-2a / pre-Commit-3 sentinel-tree refinement scope.
3. **Verification rigour** 5 isolated + 3 workspace = 8 green runs minimum at Commit 3, sibling-shape to topo-sort J-101 verification rigour.
4. **Sentinel-tree refinement folded into Commit 3** with refinement-vs-rework distinction explicit at §5.2 (refinement folds; rework escalates to Checkpoint #5).
5. **§15 row of canonical design doc** lands in Commit 1 with J-NNN placeholder; freezes at Commit 4 to milestone-close J-number.
6. **§7 discipline notes section** with six sub-sections (§7.1–§7.7); §7.8 skipped per runbook-authoring §7 lock.

**Two pre-draft code-trace findings shaped §4 + §4a narrow scope**:

- **Finding A — Q1 narrow scope**: code-trace surfaced five silent-discard sites in `ingest_event` total; Q1 covers ONLY one (`graph.add_event` at line ~210). The four other sites (`event_id`-missing-return; `store.insert` silent; two `apply_event` silents) belong to design doc §8 candidate D-NNN "ingest path invariant encoding" future-walk question, NOT this milestone's scope. Sibling-shape to topo-sort runbook §4's narrow scope for `build_room_create_event` only (sibling event constructors NOT audited that milestone).
- **Finding B — Recursive drain Shape β2**: each drain helper returns `Vec<Event>`; `dispatch_event` aggregates via concatenation; `process_inbound` persists initial event + iterates `additional_persisted` for drained events. Initial event NOT in returned vector — stays with `process_inbound`'s existing persist site. Chosen over Shape β1 (accumulator threaded through recursion) on five grounds at runbook §4a.4: self-documenting signatures + easier code-review + bounded recursion depth makes Vec allocation cost negligible + sibling-shape to existing `drain_pending_messages` pattern + avoids "outer caller forgets accumulator" footgun.

**Seven additional pre-draft code-trace findings surfaced inline in runbook §4 + §4a**: (1) `ingest_event` has five silent-discard sites total; (2) only one production caller in xgen-core (`dispatch_event` at ~559); (3) xgen-node has its own caller via `replay_spaces_from_dir` at app.rs:2628 — Commit 2 is therefore multi-crate atomic; (4) three drain helpers already silently discard at lines ~643, ~680, ~720; (5) `process_inbound` at xgen-node/src/app.rs:~1500 needs `additional_persisted` handling + sibling block in `handle_identity_replicate_msg`; (6) `GraphError` visibility check needed (likely needs `pub` widening); (7) `topological_sort` re-export from xgen-core::node::runtime needed as `pub` per D-067 + D-076 no-drift-surface family.

**Verbatim code-comment block locked at Joe-lock checkpoint #4** at runbook §4.3 (four structural elements + rungs-list bullet): (1) Reference to candidate D-NNN flag at JOURNAL J-105 + design doc §8; (2) Silent-discard-pattern-served-two-masters framing; (3) Rung-above-(a).iii.β three-line bullet (ValidatedEvent wrapper / sealed traits + visitor pattern / formal verification); (4) Narrow-scope note — "do not broaden without Joe-lock at future audit phase."

**Layered-B3 recurrence count: still second project-wide instance** (no increment at this runbook-landing event; pattern increments at code-shipping events, not runbook-authoring events). First instance: topo-sort Commit 2a at J-101. Second instance: this milestone (drain-hook layer + runtime.rs:181 graph.add_event silent-discard). Two instances is not yet durable pattern; three would be.

**One MCP tool-call discipline data point**: the three-Filesystem-write sequence (`write_file` for §1+§2+§3 → `edit_file` for §4+§4a+§5 → `edit_file` for §6+§7+§8) verified each file on disk via `Filesystem:get_file_info` between writes per J-098 + J-099 prose-then-batch discipline. Second write's first attempt returned `Input validation error: path expected string, received undefined` because Chat Claude omitted `path` parameter; single-response recovery (retry with `path` populated landed cleanly). Distinct in shape from J-098's prose-then-batch slip family — single-tool-call schema error recovered within same response without fix-up atom commit needed. Recorded per D-065 honest framing; no persistent canonical-record impact.

**"Honest longer work over fast shortcuts" — count inherited at eighth from J-104, NOT incremented at this runbook-authoring close** (within-milestone event, not new milestone-surface event).

**D-074 application count at this commit: ninth instance** (J-095 first; J-096 second; J-097 third; J-098-across-two-commits fourth; J-099 fifth; J-100 sixth; J-101 seventh; J-105 eighth; J-106 ninth). Landed atomic at first attempt per honest framing improvement over J-098's slip-and-correct shape.

**Next-active for Clair: pickup at Commit 1** of the five-commit sequence per runbook §3. Stand-down ends with this commit.

**What stays paused/pending.** Phase 9 milestone stays PLAY (waiting on sub-amendment milestone close → Commit 3b-1 collapse, then Phase 9 resumes at Commit 3b-2-equivalent with compounds C2/C3/C5/C7/C9/C10 remaining). Federation Event Propagation milestone stays PLAY (waiting on Phase 9). M6 (new) + XGID Retrofit Pass 1 stay PENDING (chain extended by one more node — this sub-amendment milestone — dependency depth unchanged in shape).

**Track 1 (Chat Claude + Joe): no active work** until Clair's five-commit sequence closes. Possible parallel work: M6 (new) Block 4 verb-by-verb walks; future-walk of candidate D-NNN "ingest path invariant encoding" if Joe locks it as worth pursuing.

**Track 2 (Clair): pickup at Commit 1** of `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_IMPL.md` per runbook §3. Sentinel working tree (four files at `xgen-node/src/tests/`: phase9_harness.rs, phase9_three_node_anti_transitivity.rs, phase9_drop_and_recover.rs, mod.rs) remains uncommitted as verification artifact for milestone close per Q4(a). Do NOT `git restore`.

**Entry point for Clair: `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_IMPL.md`** (read CLAUDE.md PLAY block + J-106 entry first per Rule 0, then runbook §1–§3, then Commit 1 doc-pass).

**Entry point for Chat Claude's next session (no active work): standby** until Clair's five-commit sequence closes; parallel-eligible items above.

---

## ✅ DONE-IN-FLIGHT — Persistence-amendment sub-amendment milestone DESIGN phase ✅ at J-105; implementation runbook authoring next-active for Chat Claude + Joe; Phase 9 Commit 3b-1 PAUSED collapses into sub-amendment milestone close per Q4(a) lock

**Persistence-amendment design phase SHIPPED 2026-05-23 at J-105.** Four Joe-locks recorded across Q1→Q4 walkthrough (sibling-shape to topo-sort design phase J-097). No re-walk fired per Lock #2 discipline. Five-file atomic commit per D-074 eighth instance: design doc NEW ACTIVE v1.0 + audit doc Status ACTIVE → COMPLETED v1.1 + JOURNAL J-105 + CLAUDE.md (this PLAY block flip + header bump) + ROADMAP.md v1.19 → v1.20.

**Four Joe-locks summary** (full reasoning at `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_DESIGN.md` §3–§6):

1. **Q1 → (a).ii + (a).iii.β + candidate D-NNN flag.** Sort-on-replay in `replay_spaces_from_dir` at `xgen-node/src/app.rs:2628` (defensive layer); `ingest_event` signature change at `xgen-core/src/node/runtime.rs:156` to `pub fn ingest_event(&mut self, event: Event) -> Result<(), GraphError>` (compiler-forced caller handling at all sites). Candidate D-NNN "ingest path invariant encoding" flagged per D-069 audit-vs-design boundary discipline — NOT promoted to DECISIONS.md at this design close; goes through its own audit → design → impl arc in future session-arc if Joe locks the candidate as worth pursuing. Sibling-shape to D-076's v1 → v1.1 progression.
2. **Q2 → (a) return-vector.** `dispatch_event` returns `DispatchOutcome::Accepted { new_joiner, additional_persisted: Vec<Event> }`. Drain helpers return drained Accepted events to dispatch_event; aggregation into vector; `process_inbound` iterates and persists each. Layer separation preserved (xgen-core stays I/O-free). Pairs cleanly with Q1's Result chain.
3. **Q3 → all three drain helpers.** `drain_pending_uniform` (line ~670, Phase 4 / F-4a), `drain_pending_by_identity` (line ~745, Phase 6 / F-10), `drain_pending_by_federation_relationship` (line ~795, Phase 7.5 / F-3) all get the Q2(a) return-vector treatment. Same gap pattern at all three sites; same-family-same-atomic-close sibling-shape to topo-sort Commit 2a layered-B3 atomic close at J-101.
4. **Q4 → (a) in-scope.** Four sentinel-tree files (`phase9_harness.rs`, `phase9_three_node_anti_transitivity.rs`, `phase9_drop_and_recover.rs`, `mod.rs`) ship atomic at milestone close as activating regression lock for the persistence fix (Scenario 3 transition FAIL → PASS verifies the fix at integration level). **Commit 3b-1 collapses into the sub-amendment milestone close** — Phase 9 resumes at Commit 3b-2-equivalent (compounds C2/C3/C5/C7/C9/C10) after milestone close.

**The sustainability question reframed Q1 mid-walk.** Initial recommendation (a).iii.α (log-level `tracing::error!`); user's "is this future-proof?" challenge surfaced three drift surfaces log-level vigilance does NOT catch (future caller bypasses validate_event; disk format change; future async-predecessor protocol revision); revised to (a).iii.β (type-level Result-returning). Second-order question "is (a).iii.β the future-proof solution?" forced honesty that nothing is future-proof in absolute terms; rungs above (a).iii.β named explicitly (ValidatedEvent wrapper, sealed traits + visitor pattern, formal verification). Resolution: lock (a).iii.β as the *immediate* answer (smallest correct-today fix raising the floor meaningfully) AND flag candidate D-NNN for future walk preserving optionality on the right rung. Sibling-shape lesson recorded in JOURNAL J-105 + design doc §11.8.

**Layered-B3 recurrence count: second project-wide instance.** First instance topo-sort Commit 2a at J-101 (two layered surfaces at graph.rs `is_dag_root_type` + exchange.rs `validate_dag_structure` closed atomically per D-067 Option E); second instance this milestone (primary fix at drain-hook layer surfaces secondary silent-error encoding at `runtime.rs:181` `graph.add_event` UnknownPrevEvent silent-discard; both closed atomically inside Q1+Q2+Q3 locks). Two instances is not yet a durable pattern; three would be. Future audits should look for the shape but not pre-assume its presence.

**"Honest longer work over fast shortcuts" — eighth recurrence count inherited from J-104, not incremented at design close.** Recurrences counted at milestone-events, not design-events. This design close is *inside* the eighth recurrence's milestone scope.

**Next-active for Chat Claude + Joe: implementation runbook authoring** at `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_IMPL.md` in a future session-arc per topo-sort precedent J-098. Runbook handoff requirements specified in design doc §9: commit sequence (candidate 4 commits; runbook-authoring locks exact shape); Joe-lock checkpoints for Clair; verification rigour (5 isolated + 3 workspace = 8 green runs minimum candidate); sentinel-tree integration shape; milestone-close file count (~12 candidate per D-074); J-NNN placeholder freeze sites; B3-shape audit answer (candidate: no gap at milestone close).

**What stays paused/pending.** Phase 9 milestone stays PLAY (waiting on sub-amendment milestone close → Commit 3b-1 collapse, then Phase 9 resumes at Commit 3b-2-equivalent with compounds C2/C3/C5/C7/C9/C10 remaining). Federation Event Propagation milestone stays PLAY (waiting on Phase 9). M6 (new) + XGID Retrofit Pass 1 stay PENDING (chain extended by one more node — this sub-amendment milestone — dependency depth unchanged in shape).

**Track 1 (Chat Claude + Joe): implementation runbook authoring next-active.** Entry point: this PLAY block → `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_DESIGN.md` (ACTIVE v1.0 — read §9 implementation runbook handoff requirements as the runbook-authoring spec) → J-105 entry in JOURNAL.md → then author runbook per topo-sort J-098 precedent shape. Possible parallel work: M6 (new) Block 4 verb-by-verb walks; future-walk of candidate D-NNN "ingest path invariant encoding" if Joe locks it as worth pursuing.

**Track 2 (Clair): stood down** through milestone close. Sentinel working tree (four files at `xgen-node/src/tests/`) remains uncommitted as verification artifact for the milestone close per Q4(a) lock. Do NOT `git restore`.

**Entry point for next Chat Claude session: `tasks/PHASE_7_5_PERSISTENCE_AMENDMENT_DESIGN.md`** (read CLAUDE.md PLAY block + J-105 entry first per Rule 0, then design doc §9 runbook handoff requirements, then author runbook).

**Entry point for Clair: standby** until milestone close.

---

## ✅ DONE-IN-FLIGHT — Federation Event Propagation Phase 9 Commit 3b ←── HERE; topological-sort milestone CLOSED at J-101; Scenario 1 is now the activating integration-level regression lock for D-075 + D-076 v1.1 (both layers)

**Topological-sort wire-order determinism milestone SHIPPED 2026-05-23 at J-101.** Five-commit Clair-facing sequence under amended D-076 v1.1 closed atomically: Commit 1 doc-pass (parents of `0543a86`) + Commit 2 determinism layer (`0543a86`) + Commit 2a causality layer (`4a6fd74` — Path B fix at `xgen-core/src/space/state.rs:797` `build_room_create_event` + validator companions unified per D-067 Option E across `xgen-core/src/dag/graph.rs::is_dag_root_type` and `xgen-core/src/message/exchange.rs::validate_dag_structure` + 17-site dag-test fixture updates under Posture β) + Commit 3 Phase 9 Scenario 1 second `#[ignore]` lift (`b370dc7` — doc-comment rewrite to five-event chronology + three-decision regression-lock framing per runbook §5.5; 5 isolated + 3 workspace = 8 green runs minimum verification rigour) + Commit 4 milestone close (this commit, eight files atomic per D-074).

**Two layered B3-shape surfaces closed atomically within Commit 2a** — structurally novel pattern. The primary fix (Path B at construction layer) traversed two sibling-layer encodings of the same DAG-root invariant before codebase coherence: validator companion #1 at `is_dag_root_type` (graph.rs); validator companion #2 at `validate_dag_structure` (exchange.rs). Both unified via Option E delegation per D-067 no-drift-surface discipline. Bidirectional Commit 2.5 (J-096) closed one B3 surface; Commit 2a closed two layered surfaces. Future audit work should look for layered B3 surfaces (Commit 2a's pattern), not just single ones. Pattern name for canonical record: "primary fix traversed two sibling-layer encodings of the same invariant before codebase coherence."

**Next-active for Clair: Phase 9 Commit 3b** per `tasks/FEDERATION_PROPAGATION_PHASE_9.md` (Status ACTIVE v1.0, RESUMED at J-101). Scope: Scenarios 2 + 3 + compound scenarios C2/C3/C5/C7/C9/C10 per the existing Q4 Lock from J-091. Expected ~5-7 atomic commits in their own sequence. After Phase 9 Commit 3b ships, Phase 9 milestone flips PLAY → DONE, Federation Event Propagation milestone flips PLAY → DONE, and M6 (new) + XGID Retrofit Pass 1 unblock simultaneously.

**What stays paused/pending.** Phase 9 milestone stays PLAY (the umbrella is in-flight until Phase 9 Commit 3b ships its own milestone-close commit). Federation Event Propagation milestone stays PLAY (waiting on Phase 9). M6 (new) + XGID Retrofit Pass 1 stay PENDING (chain unchanged in shape — extended by this milestone's session-arcs, dependency depth unchanged).

**Track 1 (Chat Claude + Joe): no active work** until Clair's Phase 9 Commit 3b arc closes. Possible parallel work: M6 (new) Block 4 verb-by-verb walks; the two JOURNAL gap retrospectives. Both unblocked, both shape-known, neither blocking anything else.

**Track 2 (Clair): Phase 9 Commit 3b.** Entry point: this PLAY block → `tasks/FEDERATION_PROPAGATION_PHASE_9.md` (RESUMED at J-101 per header `Last updated`). Scenarios 2 + 3 + compound scenarios C2/C3/C5/C7/C9/C10.

**Entry point for Clair's next session: `tasks/FEDERATION_PROPAGATION_PHASE_9.md`.**

**Entry point for Chat Claude's next session (no active work): standby until Clair's Phase 9 Commit 3b arc closes; parallel-eligible items above.**

---

## ✅ DONE-IN-FLIGHT — Topological-sort wire-order determinism milestone CLOSED at J-101: five-commit Clair-facing sequence shipped under amended D-076 v1.1; two layered B3-shape surfaces closed atomically in Commit 2a; design-doc §15 line-1140 retroactive J-096 freeze surfaces fourth prose-then-batch atomicity-slip recurrence

**Status: SHIPPED — J-101 (2026-05-23).** Eight-file atomic commit per D-074. Topological-sort milestone flips PLAY → DONE; Phase 9 Commit 3b RESUMES (see PLAY block above); Phase 9 milestone stays PLAY; Federation Event Propagation milestone stays PLAY; M6 (new) + XGID Retrofit Pass 1 stay PENDING. See J-101 for full retrospective + B3-shape recurrence count + prose-then-batch atomicity-slip fourth recurrence + Option E unification details.

---

## ✅ DONE-IN-FLIGHT — Clair pickup at Commit 2a → Commit 3 → Commit 4 under revised runbook v1.1 from J-100 — SHIPPED at J-101 across three atomic commits (4a6fd74 Commit 2a + b370dc7 Commit 3 + J-101 milestone close)

**Step 3 of the post-J-098 design-phase re-walk SHIPPED 2026-05-22** in a five-file atomic commit per D-074 per `tasks/HANDOFF_TOPOSORT_RUNBOOK_REVISION.md` (Status flipped ACTIVE → COMPLETED v1.1 at this commit). The implementation runbook (`tasks/FEDERATION_TOPOSORT_IMPL.md`) revised v1.0 → v1.1 with new §4a Commit 2a section + eleven local amendments end-to-end. JOURNAL J-100 entry recording Step 3's close + the Step-2-bis fix-up atom retrospective.

**"Commit 2a" naming Joe-locked at Step 3 open** rather than "Commit-2-amendment" (HANDOFF body) or "Commit 2.5" (bidirectional decimal-sibling precedent). Three grounds: (a) honest about sequence — letter suffix says "between Commit 2 and Commit 3" structurally; (b) table-friendly — fits cleanly in §2.1 sequence table; (c) precedent fit — sequential-insertion-at-different-layer is letter-suffix shape, not the decimal-sibling-half-step shape bidirectional Commit 2.5 used for an in-flight sibling fix at the same layer. The HANDOFF body's "Commit-2-amendment" references are preserved as historical record of Step-2-bis-time naming; the runbook ships with "Commit 2a" throughout.

**Five-commit Clair-facing sequence under amended D-076 v1.1:**
1. **Commit 1 (doc-pass)** ✅ — shipped at `0543a86`'s parent commits.
2. **Commit 2 (primitive + sibling sort fix + unit tests)** ✅ — shipped at `0543a86`. Closes the **determinism layer** of D-076 v1.1.
3. **Commit 2a (Path B fix at event-construction layer)** 🟡 — NEW for Clair. Closes the **causality layer** of D-076 v1.1. Modifies `build_room_create_event` at `xgen-core/src/space/state.rs:797` to set `prev_events: vec![space_id.to_string()]` + verbatim code-comment block (four Joe-locked structural elements: D-076 v1.1 reference; Path B citation by design §11; doc-comment-was-already-correct framing; narrow-scope note) + new unit test `room_create_event_records_space_create_as_predecessor` in `xgen-core/src/space/state.rs::mod tests`. **Scope-honesty paragraph required in commit message** per runbook §4a.5: sibling event constructors (`state.federation_add`, `membership.*`, `message.*`) NOT audited; narrow scope per J-098 Joe-lock; D-071 covers future sibling-constructor audit.
4. **Commit 3 (Phase 9 Scenario 1 second `#[ignore]` lift)** 🟡 — doc-comment rewritten per runbook §5.5 for five-event chronology + three-decision regression-lock framing (D-075 + D-076 v1.1 determinism layer + D-076 v1.1 causality layer); `#[serial_test::serial]` posture decision per §5.4; verification rigour 5 isolated + 3 workspace = 8 green runs minimum.
5. **Commit 4 (milestone close per D-074)** 🟡 — six files atomic per runbook §6.2; M15 catalogue row added per §6.3 with both-layers Detection text; J-NNN placeholders frozen across three sites.

**Four Joe-lock checkpoints** (was three at v1.0): #1 post-Commit-1 doc-pass drift; #2 pre-Commit-2 unit-test list; #3 post-Commit-2 / pre-Commit-3 primitive shape locked (extended to include Commit 2a stability in the check); **#4 NEW — pre-Commit-2a verbatim code-comment block content** with the four locked structural elements.

**Clair's stand-down ends with this commit.** She picks up at Commit 2a per the revised runbook — NOT at Commit 3 yet. Her existing Commit 3 working tree (`xgen-node/src/tests/phase9_two_node_smoke.rs`, uncommitted) remains as sentinel through Step 3 close per Joe-lock; the doc-comment text is currently forward-looking (describes the fix as landed) which was false at Step 3 open but becomes true after Commit 2a lands. She picks up the uncommitted file as part of Commit 3 after Commit 2a ships.

**Phase 9 Scenario 1 stays `#[ignore]`-annotated** until Commit 3 lifts it. Phase 9 Commit 3b stays paused inside milestone scope until Commit 4 ships. Federation Event Propagation milestone stays PLAY. M6 (new) + Pass 1 stay PENDING. The dependency chain is unchanged in shape.

**Track 1 (Chat Claude + Joe): no active work** until Clair's three-commit arc closes. Possible parallel work: M6 (new) Block 4 verb-by-verb walks; the two JOURNAL gap retrospectives. Both unblocked, both shape-known, neither blocking anything else.

**Track 2 (Clair): topological-sort implementation per revised runbook.** Entry point: `CLAUDE.md` PLAY block (this block) → `tasks/FEDERATION_TOPOSORT_IMPL.md` v1.1 §1.1 reading order → §4a (Commit 2a, NEW) → §5 (Commit 3) → §6 (Commit 4).

**Entry point for Clair's next session: `tasks/FEDERATION_TOPOSORT_IMPL.md`** (v1.1).

**Entry point for Chat Claude's next session (no active work): standby until Clair's three-commit arc closes; parallel-eligible items above.**

---

## ✅ DONE-IN-FLIGHT — Topological-sort design-phase re-walk Step 3 SHIPPED: runbook revised v1.0 → v1.1 with new §4a Commit 2a section; J-100; Clair's stand-down ends; she resumes at Commit 2a

**Status: SHIPPED — J-100 (2026-05-22).** Five-file atomic commit per D-074. Revised runbook + Step-3 HANDOFF closed + JOURNAL J-100 + CLAUDE.md PLAY block flipped (this commit) + ROADMAP.md v1.15 → v1.16. Documentation only; no code changes; test count unchanged at 577 + 1 ignored. See "PLAY" block above for active work; see J-100 for full retrospective + Step-2-bis fix-up atom acknowledgement + naming-decision reasoning + eleven-amendment enumeration.

---

## ✅ DONE-IN-FLIGHT — Topological-sort design-phase re-walk Step 3 (runbook v1.0 → v1.1 revision) — SHIPPED at J-100 per the planning block below; Path B locked at event-construction layer; D-076 v1.1 in DECISIONS.md

**Step 2 of the post-J-098 design-phase re-walk shipped 2026-05-22** in an eight-file atomic commit per D-074. Canonical-record amendments: audit doc §11 (v1.0 → v1.1) + design doc §11 (v1.0 → v1.1) + DECISIONS.md D-076 in-place amendment naming causal-DAG-respecting order as the second load-bearing property of the principle. CLAUDE.md Rule 0 added as fourth member of MANDATORY Behaviour rules (mandatory session-open reading sequence). Step-2 HANDOFF (`tasks/HANDOFF_TOPOSORT_DESIGN_REWALK.md`) Status flipped COMPLETED v1.1; Step-3 HANDOFF authored at `tasks/HANDOFF_TOPOSORT_RUNBOOK_REVISION.md` Status: ACTIVE v1.0. JOURNAL J-099 entry recording the framing-gap retrospective + Rule 0's origin story.

**Path B locked as the substantive fix** (Joe-locked at J-098 session close, recorded canonically at Step 2):
- **Path B** — modify `build_room_create_event` at `xgen-core/src/space/state.rs:797` to set `prev_events: vec![space_id.to_string()]` so the event-DAG honestly reflects the protocol-level parent-child relationship the function's own doc-comment already claims ("`space_id` is the event_id of the parent state.space_create"). `state.room_create` becomes a non-root event whose predecessor is `state.space_create`; the topological sort places it after the parent regardless of tie-break logic.
- **Commit 2's Shape A v1 sort fix stays useful** at the determinism layer beneath the causality layer (not reverted; safety net for events that legitimately tie at the DAG layer).
- **Two fixes layer cleanly:** causality first (Path B at DAG-construction layer), determinism second (Commit 2 sort at tie-break layer). Neither is sufficient alone; both are committed surfaces under amended D-076.
- **Path B scope is narrow** by Joe-lock: `build_room_create_event` only; sibling event constructors not audited this milestone, may surface later as own audit-precedes-dependent-design arc per D-071 if dependent work surfaces need.

**D-076 amended in place** — not split into D-076 + D-077. The two properties (byte-identical-across-senders determinism + causal-DAG-respecting order) cannot vary independently; both are needed; splitting would have broken the no-drift-surface discipline family's one-per-layer shape (D-067 + D-070 + D-075 + D-076 across code-organisation + transport + event-model + wire-format).

**Step 3 (this PLAY block's named-active work) revises the runbook** per `tasks/HANDOFF_TOPOSORT_RUNBOOK_REVISION.md`:
1. `tasks/FEDERATION_TOPOSORT_IMPL.md` v1.0 → v1.1 with new Commit-2-amendment section (Path B fix + new unit test `room_create_event_records_space_create_as_predecessor`); existing Commit 2 + Commit 3 + Commit 4 sections updated to reflect amended D-076 + new Commit-2-amendment placement.
2. Step-3 HANDOFF (`tasks/HANDOFF_TOPOSORT_RUNBOOK_REVISION.md`) Status flipped ACTIVE → COMPLETED.
3. JOURNAL J-100 entry recording Step 3's runbook revision close.
4. CLAUDE.md PLAY block flipped to "Clair pickup at Commit-2-amendment→Commit 3→Commit 4 ←── HERE" + header bump.
5. ROADMAP.md v1.15 → v1.16 with Past entry + Present updated for Clair-active state.

Five files. Smaller than Step 2 because the amended-runbook content is in scope; cross-doc surface changes are minimal.

**Clair stays stood down until Step 3 closes.** Her Commit 3 working tree (`xgen-node/src/tests/phase9_two_node_smoke.rs`, uncommitted) remains as sentinel per Joe-lock; the doc-comment text is currently forward-looking (describes the fix as landed) which is false but expected; do NOT `git restore`. Step 3's revised runbook tells Clair what to ship next; she picks up at Commit-2-amendment after Step 3 lands.

**Phase 9 Scenario 1 stays `#[ignore]`-annotated** until Clair's Commit 3 (post-Commit-2-amendment). Phase 9 Commit 3b stays paused inside milestone scope. Federation Event Propagation milestone stays PLAY. M6 (new) + Pass 1 stay PENDING. The dependency chain is extended by one more node (this re-walk), unchanged in shape.

**Track 1 (Chat Claude + Joe): Step 3 next-active.** Read entry point: this PLAY block → J-099 entry → `tasks/HANDOFF_TOPOSORT_RUNBOOK_REVISION.md` → then the canonical-record amendments (audit doc §11, design doc §11, DECISIONS.md D-076 amendment) and the runbook v1.0 to be revised. Possible parallel work: M6 (new) Block 4 verb-by-verb walks; the two JOURNAL gap retrospectives. Both unblocked, both shape-known, neither blocking anything else.

**Track 2 (Clair): stood down.** Resumes at Commit-2-amendment after Step 3 ships.

**Entry point for the next Chat Claude session: `tasks/HANDOFF_TOPOSORT_RUNBOOK_REVISION.md`.** (Read CLAUDE.md PLAY block + J-099 entry first per Rule 0, then this HANDOFF, then proceed to runbook revision per the HANDOFF's §3.)

**Entry point for Clair's next session: standby until Step 3 closes.**

---

## 🟢 DONE-IN-FLIGHT — Topological-sort wire-order non-determinism phase: Audit ✅ + Design ✅ + Implementation runbook ✅ + Implementation Commit 1 + Commit 2 ✅ SHIPPED; Commit 3 sentinel-state; design-phase re-walk in flight per Shape 2 procedure

**Implementation runbook shipped 2026-05-22** at `tasks/FEDERATION_TOPOSORT_IMPL.md` (Status: ACTIVE v1.0, eight sections, sibling-in-shape to `tasks/FEDERATION_BIDIRECTIONAL_NODES_IMPL.md` (COMPLETED v1.1) with one structural addition — §7 discipline notes self-defended at §7.1). Audit + design phases ✅ at prior session-arcs (audit doc + design task file both at Status: ACTIVE v1.0; both flip COMPLETED in Commit 1 of Clair's arc). D-076 in DECISIONS.md as the fourth member of the no-drift-surface discipline family (D-067 + D-070 + D-075 + D-076 — code-organisation + transport + event-model + wire-format).

**Three Joe-locks settled** (carried forward from design phase as already-decided per inline-lock pattern third recurrence):
- **Q3.ii canonical wire ordering required** — wire-order determinism is a sender-side normative property for Node-to-Node federation; two senders with identical Space history MUST produce byte-identical federation deltas modulo signature-bearing fields.
- **Q2 middle + Q2.γ** — fix the primitive's contract once at `topological_sort_events:193` (canonical-regardless-of-input semantics); Q2.γ forward-binding to Node-to-Client siblings (`collect_sync_history`, `apply_fanout` history-push) flagged for future scheduling.
- **Q1 Shape A v1 + sibling Site 1 fix** — event_id lex sort at the primitive + sort `Vec<Event>` at `compute_federation_delta_for_space:321` before passing; v1 `&str` sort with verbatim code-comment block flagging Pass 3 retype to `EventXgid`; Pass-1-neutral.

**Clair's four-commit sequence per runbook §2:**
1. **Commit 1 doc-pass** — canonical design doc §6.4.3 sibling subsection + §15 row (J-NNN+ placeholder); audit + design Status flips ACTIVE → COMPLETED v1.0. No code; test count unchanged.
2. **Commit 2 primitive + sibling fix + unit tests** — `xgen-node/src/fanout.rs:193` event_id lex sort + verbatim code-comment block (structural elements locked per runbook §4.3 — D-076 reference + four-member family naming + Pass 3 retype marker + Appendix J citation + D-076 contract statement quoted); `xgen-node/src/fanout.rs:321` HashMap-feed sort with brief code-comment; three-to-five unit tests including the wire-order-determinism witness `compute_federation_delta_byte_identical_across_two_senders` as load-bearing fourth (structural sibling to bidirectional's `apply_federation_add_two_vantages_mirror`).
3. **Commit 3 Phase 9 Scenario 1 second `#[ignore]` lift** — `xgen-node/src/tests/phase9_two_node_smoke.rs::two_node_federation_push_smoke_100_messages`; doc-comment rewritten to forward-looking text per runbook §5.5 (locked verbatim with J-NNN placeholder); `#[serial_test::serial]` posture decision per runbook §5.4 (default-keep silent; remove requires commit-message justification + doubled-run evidence). Verification rigour: 5 isolated runs with `cargo clean` between + 3 workspace runs = 8 green runs minimum before lift ships. Scenario becomes activating integration-level regression lock for both D-075 and D-076.
4. **Commit 4 milestone close per D-074** — six files atomic: JOURNAL.md (milestone-close J-NNN entry; J-NNN-placeholder freeze across three sites — §15 row + doc-comment + catalogue M15 row); CLAUDE.md PLAY block flip (topological-sort ✅; new PLAY block for Phase 9 Commit 3b; Federation Event Propagation milestone stays 🟢); ROADMAP.md tree + Past + Present + version bump v1.14 → v1.15; this runbook Status ACTIVE → COMPLETED v1.1; `tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md` gains M15 catalogue row per runbook §6.3 exact phrasing; `tasks/FEDERATION_PROPAGATION_PHASE_9.md` header Last updated paragraph honestly states Commit 3b RESUMED.

**Three Joe-lock checkpoints for Clair** (additional pauses encouraged per CLAUDE.md Rule 3):
1. Post-Commit-1, if doc-pass surfaces drift in the canonical design doc.
2. Pre-Commit-2 unit-test list proposal (Clair proposes the final list of three-to-five before writing tests; Joe locks).
3. Post-Commit-2 / pre-Commit-3 primitive shape locked (before lifting `#[ignore]` on Phase 9 Scenario 1).

**What this milestone CANNOT close.** Commit 4 closes the topological-sort milestone only. Phase 9 Commit 3b unblocks → resumes (Scenarios 2 + 3 + compound scenarios C2/C3/C5/C7/C9/C10, ~5-7 atomic commits in their own sequence). Phase 9 milestone stays PLAY. Federation Event Propagation milestone stays PLAY. M6 (new) + XGID Retrofit Pass 1 stay PENDING. Three-state-change framing applied at four sites in the runbook (§2.4 + §6.5 + §6.6 + §6.7 final DoD).

**Phase 9 Scenario 1 stays `#[ignore]`-annotated** until Commit 3 of Clair's arc. Phase 9 Commit 3b stays paused inside milestone scope until Commit 4 of Clair's arc.

**J-098 shipped across two commits per honest provenance.** Original runbook-landing commit shipped `tasks/FEDERATION_TOPOSORT_IMPL.md` only; companion-updates housekeeping atom (CLAUDE.md + ROADMAP.md + JOURNAL.md) lands as a separate commit because Chat Claude drafted the three companion-file edits as chat prose without writing them to disk turn-by-turn, missing D-074 same-commit-atomicity at the original commit. Slip framed explicitly in JOURNAL J-098 per D-065 honest-behaviour-over-polite-behaviour. Discipline lesson recorded: when drafting multi-file commits, write each file edit to disk via `Filesystem:edit_file` before moving to the next file's draft, not as prose-then-batch.

**Track 1 (Chat Claude + Joe): no active work** until Clair's milestone closes. Possible parallel work: M6 (new) Block 4 verb-by-verb walks; the two JOURNAL gap retrospectives. Both unblocked, both shape-known, neither blocking anything else.

**Track 2 (Clair): topological-sort implementation per runbook.** Entry point: `CLAUDE.md` PLAY block → `tasks/FEDERATION_TOPOSORT_IMPL.md` §1 reading order → design task file §3 + §4 + §5 + DECISIONS.md D-076 + audit doc §3 + §3.5 + canonical design doc §6.4 + §6.4.1 + §6.4.2 → back to runbook §3 onward for per-commit work.

**M6 (new) Node admin write path** remains 🟡 PENDING behind Federation Event Propagation milestone closure (unchanged in shape; chain extended by one milestone node — this milestone — per the §2.4 framing).

**Entry point for Clair's next session: `tasks/FEDERATION_TOPOSORT_IMPL.md`.**

**Entry point for Chat Claude's next session (no active work): standby until Clair's milestone closes; parallel-eligible items above.**

---

## ✅ DONE-IN-FLIGHT — Bidirectional `federation_nodes` implementation milestone CLOSED (J-096, 4 commits, 571 → 577 tests, +6; Scenario 1 re-stood-down on separate topological-sort finding)

**Status: SHIPPED — J-096 (2026-05-21).** Four-commit sequence per `tasks/FEDERATION_BIDIRECTIONAL_NODES_IMPL.md` (Status: ACTIVE → COMPLETED v1.1 at this milestone close). Commit 1 doc-pass (`e975162`) — canonical design doc gained §6.4.2 sibling subsection (sibling to Phase 7.5's §6.4.1) + §15 Implementation Complete row; design task file Status ACTIVE → COMPLETED; audit doc Status ACTIVE → COMPLETED. Commit 2 origin-aware applier + plumbing + six unit tests (`a730eda`) — `apply_federation_add` in `xgen-core/src/space/state.rs:351-363` gained `my_node_id: &str` parameter with verbatim code-comment block citing D-075 design §3.3; `SpaceState::apply_event` threaded `my_node_id`; both `NodeRuntime::ingest_event` call sites updated; six unit tests covered the two vantage branches + the mirror property (`apply_federation_add_two_vantages_mirror` — unit-level regression lock for D-075) + the third-party observer case + the DM constraint preservation + the missing-field rejection. Commit 2.5 sibling vantage-aware drain hook (`cbceb41`, in-flight gap closure) — drain-pair derivation at `NodeRuntime::dispatch_event` Step 7 in `xgen-core/src/node/runtime.rs:650-666` mirrors the applier's vantage logic; verbatim code-comment block at the drain-pair derivation site cross-references the sibling `apply_federation_add` site by `file:line`. Commit 3 Phase 9 Scenario 1 resurrection (`f051039`) — `#[ignore]` originally lifted; scenario passed in isolation and as part of full workspace; became activating regression lock at integration level. Commit 4 milestone close (this commit) — JOURNAL J-096 + CLAUDE.md PLAY block flip + ROADMAP.md state move + runbook Status flip + Phase 9 task file paragraph update + Scenario 1 re-stood-down on separate topological-sort finding.

**Test count progression.** 571 baseline + 1 ignored → 577 + 1 ignored after Commit 2 → 578 + 0 ignored at f051039 (Commit 3 lifted ignore) → 577 + 1 ignored after Commit 4 (this milestone-close commit re-stood-down Scenario 1 on the topological-sort finding). Net delta across the milestone: +6 unit tests in Commit 2; Scenario 1 originally lifted in Commit 3 then re-stood-down in Commit 4, net wash on ignored count.

**Bidirectional fix verified-correct.** xgen-node-side diagnostic instrumentation during Commit 4 verification confirmed 105 `dispatch_event` calls on B in both pass and fail runs of Scenario 1. The fix's drain hook is exercised correctly in both. The intermittent failure of Scenario 1 is a separate pre-existing wire-order non-determinism upstream of the dispatcher (see PLAY block above for the topological-sort phase that now picks this up). Six unit tests remain green and stand as the live D-075 regression lock while the integration-level scenario is re-ignored.

**B3-shape gap question asked and answered.** Honest framing per the discipline pattern J-095 established: the drain hook in `dispatch_event` Step 7 was added by Commit 2.5 as a sibling fix; the six unit tests in Commit 2 cover the applier directly, not the drain hook. The hook is a small piece of code (~17 lines in `xgen-core/src/node/runtime.rs:650-666`) whose vantage logic mirrors the applier's vantage logic verbatim; a divergent bug here is structurally unlikely but theoretically possible. When Scenario 1 resurrects under the topological-sort fix, the drain hook gets its integration-level lock back. Recording the question explicitly so a future contributor knows it was considered.

**D-075 in DECISIONS.md.** `state.federation_add` is one party's act; `federation_nodes` is a vantage-aware derived projection. Sibling-distinct from D-070's transport-layer "two events" principle. The disciplines operate at different layers. D-075 commits the protocol's general approach: every relationship-shaped event records one party's act; the resulting data object is a derived projection; vantage-aware appliers are the legitimate pattern when the event has a sender-vs-other-party asymmetry.

**Wire-format-neutral. Pass-1-neutral.** `tasks/XGID_RETROFIT_PASS_1_IMPL.md` stays at Status: ACTIVE v2.0 unchanged. The applier-signature change adds a parameter typed as `&str` at v1, which Pass 3 may want to retype to `&NodeXgid` when the surrounding subsystem retrofit lands.

---

## ✅ DONE-IN-FLIGHT — Bidirectional `federation_nodes` design phase closed (Q1 + Shape A + A.1 locked, D-075 promoted)

**Status: SHIPPED — 2026-05-21.** Design phase walkthrough closed same-session as the audit doc. Three Joe-locks per `tasks/FEDERATION_BIDIRECTIONAL_NODES_DESIGN.md` §3: Q1 Reading (i) (`state.federation_add` is one event recording one party's act; `federation_nodes` is a derived projection with vantage-aware applier logic), Shape A (origin-aware applier; wire format unchanged), sub-option A.1 (re-derive on load; native fit verified). D-075 promoted to DECISIONS.md as the protocol-design principle the locks instantiate. Implementation runbook authored same-day at `tasks/FEDERATION_BIDIRECTIONAL_NODES_IMPL.md` (Status: ACTIVE v1.0). All chat-side work for this phase is complete; Clair's four-commit implementation is the remaining work.

**Verification check at design close.** Performed against `xgen-core/src/node/runtime.rs` to confirm A.1's architectural fit. Result: `SpaceState` is not persisted anywhere; `NodeRuntime::new()` initialises `spaces: HashMap::new()` (line 134); no `load_space` / `restore_space` / `persist_space` functions exist for `SpaceState`; adjacent runtime fields (`replica_registry`, `dm_proposals`) carry explicit "not persisted; rebuilt on restart" comments per Phase 2 simplification. A.1 is not a deviation from the existing model; it IS the existing model. The fix lands and self-heals on next Node start.

**Reading (ii) considered and rejected.** Two-events-per-relationship would have introduced the first event family in the protocol whose semantic completeness requires two signed assertions of the same type, one per party. The protocol could have legitimately gone there (federation is the only relationship between Nodes rather than between Identities, and Nodes are infrastructure peers with no natural asymmetry), but precedent cost (no analog in the current registry; every other relationship-shaped event is one party's act) and operational complexity (intermediate "half-federated" states, replay edge cases, reciprocal-mint timing concerns) tipped the lock toward Reading (i). The rejected alternative is preserved in design task file §3.1 reasoning as a coherent alternative for any future revisit.

**Shape D considered and rejected.** D.1 (add `peer_node_id` field duplicating `sender` into content) is worst-of-both-worlds: pays wire cost AND keeps vantage logic in the applier; duplication invites future bugs. D.2 (symmetric `{a_node, b_node}` field) is cleaner semantically but secretly re-introduces Reading (ii) thinking through schema design; if D.2 became the right schema, the right move would be to re-open the Q1 lock and commit to Reading (ii) honestly. Shape D is also the only shape with Pass-1 impact (one field row in Appendix C + I; one extra row in Pass 1 coverage table). Shape A is wire-format-neutral and Pass-1-neutral.

**Bidirectional fix scope (from design task file §4).** Applier change (`apply_federation_add` gains `my_node_id: &str` parameter; vantage-aware peer derivation); apply-pipeline plumbing (`SpaceState::apply_event` signature change; parameter threaded to the federation_add arm only); call-site updates (two in `NodeRuntime::ingest_event`; others surface during compile); unit tests (both vantage branches + mirror property + DM constraint + missing field); Phase 9 Scenario 1 resurrection (lifts `#[ignore]`; becomes activating regression lock); doc-pass (canonical design doc §6.4.2 + §15 row).

**Phase 9 + milestone implications.** Phase 9 stays PAUSED at Commit 3a boundary; resumes from Commit 3b after this phase ships. Federation Event Propagation milestone closure delayed by this phase (foreseen at audit close, not a surprise). M6 (new) blocking chain extended by one node, unchanged in shape. Pass 1 of XGID Retrofit stays at Status: ACTIVE v2.0 unchanged.

---

## ✅ DONE-IN-FLIGHT — XGID Adoption v1 implementation milestone CLOSED (J-095, 4 commits, 556 → 571 tests, +15)

**Status: SHIPPED — J-095 (2026-05-20).** Two substantive commits + one hygiene sibling atom + one milestone-close commit, four atomic commits total. Commit 1 (`c95584a`): new `xgen-common/src/xgid/` module ships base `Xgid(String)` newtype `#[serde(transparent)]` + six flavour wrappers each `Deref<Target = Xgid>` and serde-transparent (`EventXgid`, `SpaceXgid`, `RoomXgid`, `TrustAssertionXgid`, `NodeXgid`, `IdentityXgid`) + `XgidLike` trait + `XgidDecodeError` + hash-anchored `from_canonical_bytes(&[u8])` constructors (matching `xgen-core::crypto::hashing::hash_uri` byte-for-byte) + principal `from_pubkey(&VerifyingKey)` infallible constructors + principal `pubkey() -> Result<VerifyingKey, XgidDecodeError>` parse-fallible-at-v1 decode methods. New deps `ed25519-dalek = "2"` + `sha2 = "0.10"` + `base64 = "0.21"` + `thiserror = "1"` added to `xgen-common/Cargo.toml` (all matching xgen-core's pinned versions). Five required invariance tests by name in `xgen-common/tests/xgid_invariance.rs`: `xgid_serializes_as_plain_string`, `xgid_deserializes_from_plain_string`, `flavour_wrapper_is_serde_transparent`, `event_xgid_roundtrip_through_event_canonical_form`, `node_xgid_roundtrip_through_handshake_message`. Plus 10 in-module flavour tests in `flavours.rs` (legacy-format URI matches, decode rejections, Deref chain, XgidLike trait unification).

**v1 scope-discipline carry-over to Retrofit Pass 1:** hash-anchored convenience constructors (`from_event` etc.) deferred from Commit 1 — implementing them requires `canonical_event_bytes()` which lives in `xgen-core` and is not visible to `xgen-common`. Runbook's "where it is clean to do so" hedge applied. Module-level doc comment in `flavours.rs` flags this. Pass 1 picks up: move canonical-form helpers from `xgen-core/src/wire/canonical.rs` to `xgen-common/src/canonical.rs` (with `xgen-core` re-export preserving call sites), then add the convenience constructors. Both motions coordinate in one Pass 1 commit set.

Commit 2 (`24a255b`): `SpaceLocalMetadata.introducer_node_id` retypes from `Option<String>` to `Option<NodeXgid>`. Three files touched: `xgen-common/src/space_local.rs` (struct + constructor + tests, with strengthened `serde_roundtrip_with_introducer` acting as wire-format invariance witness including forward-compat from pre-XGID JSON shape); `xgen-core/src/node/runtime.rs` (production caller wraps wire-authenticated peer ID into `NodeXgid::from_xgid(Xgid::new(peer.to_string()))` at type-boundary entry, with code comment flagging that Retrofit Pass 3 widens `dispatch_event(peer_node_id: Option<&str>)` to `Option<&NodeXgid>` at which point the wrap collapses); `xgen-node/src/tests/cold_start_bootstrap_integration.rs` (single read-side `.as_deref()` → `.as_ref().map(|n| n.as_str())` projection). Honest-broadening warning held throughout — six adjacent String-typed XGID fields presented as tempting parallel retypes, each deliberately left untouched (belongs to Retrofit Pass 3 or wherever the subsystem retrofit lands).

Hygiene atom (`904441b`, `chore(workspace): clippy cleanups for new toolchain`) shipped same-session as separate sibling commit. Workspace clippy gate flipped red → green under Rust 1.95.0; 26 files touched across all four crates; +191/-89 LOC. One behaviour-adjacent fix (`filter_map(|l| l.ok())` → `map_while(Result::ok)` on `std::io::Lines` in `xgen-node/src/pipe.rs` + `xgen-client/src/batch.rs`, closing a potential spin-forever loop on persistent read errors). Rest mechanical or `#[allow]` with per-case rationale comments (notable: `clippy::manual_clamp` on `clamp_temperature` because NaN handling is load-bearing; `clippy::result_large_err` on `HandshakeError` module because real fix is boxing the variant which belongs to a future error-type-size discipline pass; `clippy::needless_range_loop` file-level in `xgen-client/src/app.rs` because parallel-array indexing is the honest shape). Honest provenance: zero XGID code in this commit; future `git log -- xgen-common` will show hygiene and XGID as distinct commits.

Milestone-close commit (this commit) carries five files: JOURNAL.md (J-095 entry written), CLAUDE.md (this PLAY block + header), `docs/ROADMAP.md` (Past gains the closure entry, Present updated, Near future loses the now-shipped XGID Adoption v1 implementation line), `tasks/XGID_ADOPTION_IMPL.md` (Status flip ACTIVE → COMPLETED v1.1), `docs/xgen_ch4_implementation.md` (one-line v1 follow-on pointer per Phase 2 sweep A5 Joe-lock — Scope-B blockquote shape matching Phase 1 normative pointers in `docs/xgen_aicontrol_implementation.md` and `docs/xgen_appendix_f_en.md`).

**Discipline notes (full detail in J-095):** Rule 4 in action — milestone-close commit's changed-files list includes JOURNAL.md alongside the cross-doc updates (the candidate sibling principle flagged in J-094 being followed pre-emptively). D-069 canonical-document discipline as sustained pattern — same coordinated-deliverable shape XGID Adoption v1 used at design close (`a5f3c8b` eight artefacts atomically) now used at implementation close. B3-shape gap question asked and answered (no gap — XGID is structurally narrower than Phase 7.5, all surfaces exercised by tests written in the same commits).

---

## ✅ DONE-IN-FLIGHT — Federation Event Propagation Phase 9 Commits 1 + 2 (J-092)

**Phase 9 IMPL LOCKED in J-091 (2026-05-19).** Survey findings (tasks/FEDERATION_PROPAGATION_PHASE_9_SURVEY_FINDINGS.md COMPLETED v1.1) Joe-locked across all four §8 open questions. Implementation task file at tasks/FEDERATION_PROPAGATION_PHASE_9.md (Status: ACTIVE, v1.0). Scope: 12 scenarios (6 baseline + 6 compounds: C2 anti-transitivity at queue depth, C3 F-3 during F-1a recovery, C5 validation asymmetry under load, C7 pagination at boundary, C9 F-3 drain-time hazard, C10 identity-replicate hook under contention). Four compounds deferred (C1, C4, C6, C8) to tasks/FEDERATION_STRESS_FOLLOWON.md (PENDING) — blocked on clock-injection seam. Sequence: 7 atomic commits — (1) observability preconditions G1+G2+G3 ✅, (2) flake fixes #[serial_test::serial] ✅, (3) baseline deployment scenarios 1-3 PAUSED (M5), (4) baseline 5+6+B1 binary-level, (5) compound deployment C2+C3, (6) compound NodeRuntime (Scenario 4, C5, C7, C9, C10), (7) milestone close flipping CLAUDE.md + ROADMAP.md + design doc §15 to ✅ DONE. Escalation rule documented in task file §6: if any new Phase 9 integration test exhibits 127.0.0.1:0 bind race or WS frame-ordering inconsistency under workspace parallelism, STOP and walk back to option (ii) on Q3 (investigate underlying race). Failure-mode catalogue: 11 of 14 bugs caught; M6, M8, M13 flagged for Client-Side Consequences Audit post-milestone.

**Commits 1 + 2 shipped J-092:** G1 (xgen-node_state.json::peers populated from FederationRegistry::peer_records via new `build_federated_peers` helper; FederatedPeer schema extended with `lost_connection`/`last_successful_session`/`next_reconnect_attempt` operational fields). G2 (seven stable `event = "..."` trace fields across F-1 push paths in `federation_session.rs`, F-3 / F-4 reject paths in `xgen-core/src/node/runtime.rs`, and the unified rejection wrapper at `xgen-node/src/app.rs::process_inbound`). G3 (two trace events in `fanout.rs::apply_fanout`). Commit 2: `serial_test = "3"` dev-dep in xgen-common + xgen-node; `#[serial_test::serial(xgen_log_env)]` on the four `resolve_log_level_*` tests; unnamed `#[serial_test::serial]` on the three federation_delta_integration tests. 10 consecutive `cargo test --workspace` runs PASS, 519 tests every run. Q3 escalation criterion not triggered.

**Phase 8 closed in J-089 (2026-05-19).** Documentation pass closing the six drift surfaces accumulated across Phases 5-7. No code changes; 519 tests at Phase 8 close (unchanged from Phase 7). Six drift items closed: Ch4 §4.11.2 SQLite-schema-rewrite-to-JSON-shape, CLAUDE.md Tier-1 federation file corrected, runbook §3.5 schema-decision paragraph fixed, Ch4 §4.12.3 Pending Event Buffer paragraph updated for post-F-10 + post-F-1a, spec §3.9.6 + §3.9.8 4006 entry added, design doc §6.4 federation_nodes clarification + B1 skip-rule note. Plus the standard "forward-reference → implementation-complete" updates that the milestone's Pass-3 design phase scheduled: Ch4 §4.11.3, Ch4 §4.12.3, admin-ops §4.2 all updated from "Until that milestone closes" to "Implementation shipped J-082..J-088." New §15 Implementation Complete section in the canonical design doc records the eight-phase shipped state. DoD "four file headers" = Ch3 + Ch4 + admin-ops + design doc; CLAUDE.md + runbook header bumps happen alongside per the header convention but aren't part of the DoD count (recorded in J-089 to pre-empt future audit questions). Two out-of-scope SQLite drift surfaces in Ch4 §4.9 (Identity Registry) and Ch4 §4.12 (Event Store) flagged for a future doc-pass milestone — see "Known doc-drift outside this milestone's scope" below.

**Phase 7 closed in J-088 (2026-05-19).** F-3 federation-relationship verification gate per design doc §6 + runbook §3.7 + §3.7.1. Phase 2's F-4 dispatcher pipeline shape (§7.7 step 2) reserved a structural placeholder; Phase 7 fills it with the real check. The new gate is the inbound symmetric of Phase 4's `apply_federation_push` outbound decision — both consult `SpaceState.federation_nodes`, closing the symmetric pair (events sent to peer X for Space S only if X is federated, AND events received from peer X for Space S only if X is federated). Three locks: Lock A (A1 `SpaceState.federation_nodes` data source; A2 `FederationRegistry.shared_spaces` would have created a two-source-drift surface; A3 consult-both is performative defense-in-depth); Lock B (B1 skip F-3 for `state.federation_add` events with verbatim code-comment block — relationship bootstrap chicken-and-egg; B2 self-establishing tightening NOT done at v1, layers cleanly on B1 if a future threat model justifies); Lock C (C1 `peer_node_id: Option<&str>` parameter on `dispatch_event`; honours the existing `runtime.rs:349` placeholder comment "Step 2 lives here"; C2 caller-pre-computes-bool would have spread lookup; C3 caller-pre-rejects would have broken DispatchOutcome single-gate invariant). Drain-time re-dispatch passes `None` for `peer_node_id` — same shape Phase 4 anticipated for `origin` tracking; future tightening is `BufferedEntry.peer_node_id: Option<String>` mirroring Phase 6's `missing_identity` addition.

**Phase 6 closed in J-087 (2026-05-19).** F-10 HeldPending generalisation per design doc §13 + runbook §3.6 + §3.6.1. The validation-asymmetry concern from audit J-081 §3 closes further: federation first-contact events whose signer Identity is not yet replicated to this Node now buffer pending arrival (30 s uniform timeout per F-10a, recovery via F-1a tip-exchange on next reconnect per F-10 §13.6), rather than reject-then-re-pull. Four locks: Lock A (A2 — per-`PendingBuffer` `waiting_for_identity` secondary index; `NodeRuntime::drain_pending_by_identity` fans out across Spaces; rejected A1 sync-burden / A3 over-engineered middle ground); Lock B (B1 — struct variant `HeldPending { missing_predecessors, missing_identity }` natively expressing "predecessor OR identity OR both"); Lock C (C2 — `pending_identity_replication: usize` counter in `xgen-node_state.json` via `build_node_state`, sibling field to Phase 5's federation registry surface); Lock D (D1 — `TimedOut.missing_identity: Option<String>`; D3 — new error code with namespace verification surfacing **4006** since 4001-4005 were all allocated in `xgen-core::resolution::mod.rs`; predecessor-code-wins sub-rule for the both-missing case, with verbatim code-comment block locked at the timeout-emit site). Step 1 legacy-path verification: `validate_steps_8_13` reachable only via the `pub fn accept_message` test-only path; no mirror change to `ExchangeError::HeldPending(Vec<String>)` needed.

**Phase 5 closed in J-086 (2026-05-19).** F-1c per-peer operational record + reconnect scheduler per design doc §4.6 + runbook §3.5 + §3.5.1. The "zero production callers of `run_initiating` in xgen-node/src/" verdict from audit J-081 §2.2 is closed — the reconnect scheduler is the first production caller. Six locks: Lock A (A3 — `peer_records: HashMap<String, PeerOperationalRecord>` field inside `FederationRegistry`; single JSON file, single save site, type-clean separation through field shape; rejected A1 protocol/operational state mixing on `FederationRelationship` and A2 two-file sibling shape); Lock B1 (60s scheduler tick); Lock B2 (15/30/60/120-min ladder capped, reset on handshake-ACTIVE not TCP-connect); Lock B3 (15-min initial delay after first observed loss); Lock B4 (parallel detached `tokio::spawn` per due peer per tick — verbatim code-comment block at the spawn site explaining head-of-line-blocking avoidance); plus the implementation-session lock (α — in-memory-only backoff cursor rather than persisted, intentional post-restart aggressive probe per operator-restart-correlates-with-operator-fix reasoning). The Phase 5 survey also found that pre-Phase-5 the `FederationRegistry` type existed in xgen-core but was never loaded or saved by xgen-node production code — closed in the same commit as the F-1c work (now loaded at startup, mutated on every handshake-ACTIVE / session-end, saved to `data_dir/xgen-node_federation.json`).

**Phase 4 closed in J-085 (2026-05-19).** F-1 federation event push + F-1b drop-on-peer-down + F-5 origin gating. The "missing mechanism" verdict from audit J-081 §2 is closed — federation push exists as a production mechanism. Three locks: Q1 `EventOrigin` enum, Q2 `FederationPeerSenders` mirroring `ClientSenders`, Q3 reuse of `process_inbound` with two-comment overload documentation. R12-R15 Clair-latitude items.

**Phase 3 closed in J-084 (2026-05-19).** F-1a federation handshake reshape to bilateral tip exchange.

**Phase 2 closed in J-083 (2026-05-18).** F-4 `process_inbound` validation pipeline unification.

**Phase 1 closed in J-082 (2026-05-18).** F-6 (`transport.sync_complete`) + F-7 (response-size pagination).

**Quick orientation:**
- **Canonical design doc:** `docs/xgen_federation_propagation_design.md` (Status: ACTIVE, v1.0). All ten framework decisions (F-1 through F-10) locked. New §15 Implementation Complete records the eight implementation phases shipped J-082..J-089.
- **Runbook:** `tasks/FEDERATION_PROPAGATION_COMPLETION.md`. Nine phases — **Phase 1 ✅, Phase 2 ✅, Phase 3 ✅, Phase 4 ✅, Phase 5 ✅, Phase 6 ✅, Phase 7 ✅, Phase 8 ✅**, Phase 9 PENDING.
- **Tests:** 468 (handoff) → 476 (Phase 1) → 480 (Phase 2) → 488 (Phase 3) → 491 (Phase 4) → 505 (Phase 5) → 516 (Phase 6) → 519 (Phase 7) → **519** (Phase 8 close — documentation only per DoD; test count unchanged).
- **Blocks:** M6 (new) still blocked behind this milestone going DONE; Phase 9 closes the milestone.
- **Next active phase:** Phase 9 — Integration tests for full federation push path per runbook §3.9. Six DoD scenarios at deployment level: two-Node push smoke, three-Node anti-transitivity (F-5), drop-and-recover (F-1b + F-1a recovery), validation-asymmetry regression (post-F-4 + post-F-7), unknown-signer first-contact (F-10), federation-relationship rejection (F-3). After Phase 9 ships, milestone flips PLAY → DONE and M6 (new) unblocks.

**Known doc-drift outside this milestone's scope (candidates for a future doc-pass milestone).** Two additional Ch4 SQLite-drift surfaces surfaced during the Phase 8 survey but are explicitly outside Phase 8's runbook §3.8 scope: Ch4 §4.9 Identity Registry SQLite framing (verify against `xgen-core/src/identity/registry.rs`) and Ch4 §4.12 Event Store one-DB-per-Space framing (audit J-081 noted file-system-based actual). Recording here so the next contributor doing a similar doc-pass finds these without re-discovering them.

**Phase 7 shipped these structural changes:**
- `dispatch_event` signature in [`xgen-core/src/node/runtime.rs`](xgen-core/src/node/runtime.rs) gains `peer_node_id: Option<&str>` parameter per Lock C1. Step 2 placeholder (the structural seam Phase 2 left at runtime.rs:349) is filled with the real F-3 check: federation-channel events (`peer_node_id.is_some()`) consult `self.spaces.get(&space_id)?.federation_nodes.iter().any(|n| n == peer)` per Lock A1; `EventType::StateFederationAdd` skips the check per Lock B1 with the verbatim code-comment block. On miss returns `DispatchOutcome::Rejected("federation_relationship_missing: peer {peer} has no federation relationship for Space {space_id}")`.
- Drain-time recursive `dispatch_event` calls inside `drain_pending_uniform` and `drain_pending_by_identity` pass `None` for `peer_node_id` with the documented Phase-4-shaped approximation (F-3 not re-checked on drain; narrow hazard bounded by 30 s HeldPending window; future tightening is `BufferedEntry.peer_node_id` mirroring Phase 6's `missing_identity`).
- `process_inbound` in [`xgen-node/src/app.rs`](xgen-node/src/app.rs) threads `peer_node_id_for_f3 = match origin { ReceivedViaFederation => Some(identity_id), LocallySubmitted => None }` into the new dispatch_event parameter. Sources the peer ID from the Q3-overloaded identity_id (peer Node URI when origin is federation per Phase 4 §3.4.1 Q3 lock). Existing `DispatchOutcome::Rejected` handler at app.rs:1440 (which uses `tracing::error!`) co-locates the rejection log line for `federation_relationship_missing` rejections alongside other rejection causes per the runbook's "co-locate with existing rejection paths" load-bearing instruction.
- 16 dispatch_event call sites updated across `xgen-node/src/fanout.rs` (7), `xgen-node/src/app.rs` (1), `xgen-node/src/tests/federation_push_integration.rs` (1), `xgen-node/src/tests/heldpending_identity_integration.rs` (6), `xgen-core/src/node/runtime.rs` (2 internal recursive). The heldpending tests' `build_node_with_alice_member` helper gained a federation_peer setup (state.federation_add ingested into the Space's federation_nodes) so subsequent `ReceivedViaFederation` dispatches pass the new F-3 check — Step 4 audit work surfacing at implementation time.
- New module [`xgen-node/src/tests/federation_relationship_integration.rs`](xgen-node/src/tests/federation_relationship_integration.rs) with three NodeRuntime-level integration tests: (1) peer-without-relationship-rejects with `federation_relationship_missing`; (2) peer-with-relationship-accepts as positive regression; (3) state_federation_add_skips_f3_check verifies B1 — asserts the negative ("outcome is NOT `federation_relationship_missing`") since the downstream HeldPending outcome from Phase 6's unknown-signer rule is orthogonal to F-3's skip rule.

**Phase 6 shipped these structural changes:**
- `ValidationOutcome::HeldPending` becomes struct variant `{ missing_predecessors: Vec<String>, missing_identity: Option<String> }` in [`xgen-core/src/message/exchange.rs`](xgen-core/src/message/exchange.rs). `validate_event` step 11 unknown-signer branch now returns this with `missing_identity: Some(sender.clone())` rather than the pre-Phase-6 `Rejected(UnknownSender)` — the F-10 load-bearing change.
- New `PendingBuffer::waiting_for_identity: HashMap<identity_id, HashSet<event_id>>` reverse index in [`xgen-core/src/dag/pending.rs`](xgen-core/src/dag/pending.rs) with new `resolve_identity` arrival hook. Existing `resolve` signature now takes `&IdentityRegistry` too — unified `try_release` short-circuits identity check for entries with `missing_identity: None` (so structural-only layers like `RoomDag` pass an empty registry without surprises). `TimedOut.missing_identity: Option<String>` per Lock D1. New `pending_identity_count()` for the observability counter.
- New [`NodeRuntime::drain_pending_by_identity(identity_id, origin)`](xgen-core/src/node/runtime.rs) in `xgen-core` — cross-Space fan-out per Lock A2. Iterates `self.pending` keys, calls each `PendingBuffer::resolve_identity`, re-dispatches released events via the unified `dispatch_event` path.
- Identity-arrival hook wired in [`handle_identity_replicate_msg`](xgen-node/src/app.rs) adjacent to the existing `handle_incoming_replicate` call site (~line 1555): on `Ok(())`, calls `rt.drain_pending_by_identity(&identity_id, EventOrigin::ReceivedViaFederation)` inside the same runtime-lock critical section.
- `NodeState::pending_identity_replication: usize` field with `#[serde(default)]` in [`xgen-common/src/state.rs`](xgen-common/src/state.rs); computed in `build_node_state` as `sum(buf.pending_identity_count())`. Operators detect "Identity replication is the bottleneck" by polling `xgen-node_state.json`.
- Timeout sweep task in `run_node` extended with predecessor-code-wins branching: 4002 when `missing_predecessors` non-empty, 4006 otherwise. Verbatim code-comment block locked at the branch site per the runbook.
- New error code **4006 `identity_record_timeout`** in domain 4000-4999 (state resolution). Step 6 namespace verification surfaced that 4001-4005 were all already allocated in `xgen-core::resolution::mod.rs::ResolutionError`; 4006 is next-free.
- Four new integration tests in [`xgen-node/src/tests/heldpending_identity_integration.rs`](xgen-node/src/tests/heldpending_identity_integration.rs) covering F-10 §13.7 DoD scenarios (a) identity-arrives-within-timeout, (b) predecessors-first-then-identity, (c) identity-never-arrives-timeout-fires, (d) both-missing-identity-first-then-predecessor. NodeRuntime-level tests — no transport scaffolding because the F-10 surface is entirely in-process.

**Phase 5 shipped these structural changes:**
- `PeerOperationalRecord` struct in [`xgen-core/src/federation/registry.rs`](xgen-core/src/federation/registry.rs) with fields locked by §3.5.1: `peer_node_id`, `lost_connection`, `last_seen`, `last_successful_session`, `next_reconnect_attempt`, `operator_notes`, `priority`. `FederationRegistry::peer_records` field with `#[serde(default)]` for forward-compat. Operational API: `mark_active`, `mark_lost`, `update_next_reconnect`, `peer_record`, `due_for_reconnect`, `peer_records`. Registry JSON shape now `{ relationships: {...}, peer_records: {...} }`; forward-compat-loading of pre-Phase-5 shape pinned by a unit test.
- New module [`xgen-node/src/reconnect.rs`](xgen-node/src/reconnect.rs): `spawn_reconnect_scheduler` (long-running task), `scheduler_tick` (extracted body for test-direct firing without sleeping out 60 s), `attempt_reconnect` (the first production caller of `run_initiating`). Lock B4 verbatim code-comment block at the spawn site.
- New `SessionRole` enum (`Initiator` / `Receiver`) + `pub(crate) async fn run_federation_session_post_handshake<S>` extracted in [`xgen-node/src/app.rs`](xgen-node/src/app.rs) from the receiver-side post-handshake block. Initiator side drains inbound until receiver's SyncComplete first (consuming receiver's catch-up via the same `process_inbound` pipeline as the F-2 loop), then both sides stream their own delta (§3.3.1 Lock 7 R5 production caller), register in `FederationPeerSenders`, call `mark_active` + upsert relationship + save registry, run the F-2 loop, deregister + `mark_lost` + save on exit.
- `FederationRegistry` is now loaded at Node startup from `data_dir/xgen-node_federation.json` and wrapped in `Arc<Mutex<>>`. Threaded through `handle_connection` → `handle_federation_incoming` → `run_federation_session_post_handshake`. First production wiring of the registry in xgen-node — type existed pre-Phase-5 but had zero production callers (silent companion to the `run_initiating` finding from audit §2.2).
- `process_inbound`, `handle_identity_msg`, `handle_identity_replicate_msg` in `app.rs` + `stream_federation_delta` in `federation_session.rs` all made generic over `S: AsyncRead + AsyncWrite + Unpin`. Required so the same code path serves both server-accept (`Connection<TcpStream>`) and outbound-connect (`Connection<MaybeTlsStream<TcpStream>>`).
- Reconnect scheduler spawned at Node startup in `run_node` right after the registry is constructed.
- Two bilateral integration tests in new [`xgen-node/src/tests/reconnect_integration.rs`](xgen-node/src/tests/reconnect_integration.rs) covering A-initiates-recovery and B-initiates-recovery scenarios (DoD's bilateral pair). Both call `scheduler_tick` directly against an in-memory `FederationRegistry` pre-populated with a lost-peer record whose `next_reconnect_attempt` is in the past, then poll for the registry transitioning to `lost_connection: false` and `FederationPeerSenders` containing the peer.

**Phase 4 / Phase 1 cross-reference (retained for navigation):**
- Phase 4 `EventOrigin` flows through `dispatch_event` / `process_inbound` (now generic over S) / `apply_federation_push`. `FederationPeerSenders` keyed by `peer_node_id` lives alongside `ClientSenders`.
- Phase 1 `TransportMessage::SyncRequest::limit: Option<u32>` + `TransportMessage::SyncComplete { since, new_tip, continue_from }`; `collect_sync_history` returns `(Vec<Event>, Option<String>)`; `[sync]` config section on both binaries (`completion_timeout_seconds` 5, `batch_size` 1000).

**Phase 8 doc-pass items recorded for visibility (Phase 8 IS the closing item).** Cumulative tally — 6 items (3 from Phase 5 + 2 from Phase 6 + 1 from Phase 7):

1. `docs/xgen_ch4_implementation.md` §4.11.2 — SQLite `federation_relationships` + `peer_announcements` tables described but never implemented (Phase 5).
2. `CLAUDE.md` Tier-1 file table — lists `xgen-node_federation.db` (SQLite); actual file is `xgen-node_federation.json` (JSON) (Phase 5).
3. Runbook §3.5 "Schema decision" paragraph — frames the F-1c storage choice as "extend `peer_announcements` columns vs sibling table" assuming SQLite; the actual choice (Lock A) was between Rust struct extension strategies for a JSON-backed registry (Phase 5).
4. `docs/xgen_ch4_implementation.md` §4.12.3 — Pending Event Buffer paragraph still describes pre-F-1/F-10 behaviour (predecessor-only buffering, no Identity-arrival hook, no F-1a tip-exchange recovery) (Phase 6).
5. `docs/xgen_ch3_specification.md` §3.9.6 — needs new error-code entry for `4006 identity_record_timeout` alongside the existing `4002 predecessor_timeout` (Phase 6).
6. `docs/xgen_federation_propagation_design.md` §6.4 — "federation registry" wording is ambiguous between `FederationRegistry` (the Phase-5-wired protocol-level registry) and `SpaceState.federation_nodes` (the per-Space federation node list). Phase 4 §3.4.1 Q2 and Phase 7 §3.7.1 Lock A1 both name `SpaceState.federation_nodes` as the single source of truth. Phase 8 doc-pass updates §6.4 to be explicit + adds a sentence on the federation_add skip-rule (Lock B1) so it's not tribal knowledge in a code comment only (Phase 7).

**Known test-flake state (carried over from Phase 4 close — unchanged).** Two intermittent flakes under workspace parallelism, both pre-existing relative to their disclosure milestones, disclosed transparently in JOURNAL J-084 + J-085 + (this entry) J-086 per Rule 2:

1. **Precedence env-var race (introduced at D-068 commit `3e2f311`).** Surfaces in ~10–20% of full `cargo test --workspace` runs. Did not fire during Phase 5's verification run.
2. **`reconnect_with_existing_tip_small_delta_delivered` (Phase 3 test, surfaced under Phase 4's parallelism increase).** Surfaces in ~10% of full workspace runs; 0% in isolated runs. Did not fire during Phase 5's verification run. Phase 5 added 2 integration tests to the `xgen-node-lib` bucket — concurrent WS bind/connect activity rose marginally but stayed within the existing tolerance window.

**Fix shape for both** (when prioritised, not now): `#[serial_test::serial]` annotation OR a workspace-level controlled-parallelism configuration. Until then, a `cargo test --workspace` failure on either test that passes on retry is the known-flake signature; not a real regression.

---

## 🔴 MANDATORY — Behaviour rules (read before doing anything else)

These rules exist because fabricated results have occurred. A summary that says "done" when the work was not actually done causes real damage — wasted sessions, false confidence, incorrect state in CLAUDE.md and JOURNAL.md. Honesty about failure is always better than a fabricated success.

**Rule 0 — Mandatory session-open reading sequence.** On any session open, the FIRST reads are always: (1) CLAUDE.md PLAY block; (2) latest JOURNAL entry; (3) any ACTIVE HANDOFF notes in `tasks/`; (4) THEN whatever document Joe pointed the session at. This holds regardless of how the session is opened: a narrow pointer ("read X" or just a filename pasted) is treated as "expand to context per (1)–(3), THEN read X." Runbook-as-ground-truth is a failure mode; runbooks are item 4 on the reading stack, not item 1. The bridges (PLAY block + JOURNAL + HANDOFF) are the project's structural defences against operational-state drift between sessions; bypassing them produces offers to do work that is stale relative to the actual current state. Rule 0 originated from the post-J-098 session-open failure (recorded in J-099): Chat Claude read a runbook in isolation when the runbook was two commits stale, missing a Joe-lock entirely. Sibling-shape to how D-076 v1.1's amendment made the second load-bearing property of the wire-format principle explicit — same pattern at the meta-level for session-open discipline. Skipping Rule 0 is a Rule 3 stop-and-surface moment in retrospect; the safe pattern is to follow it on every session open without exception.

**Rule 1 — Never fabricate results.** If a command fails, report the failure. Do not describe what the output *should* have been. Do not write a journal entry claiming success until success is actually confirmed.

**Rule 2 — Show actual output, not a description of output.** Every verification step requires quoting real terminal output in the journal entry. Do not paraphrase. Do not summarise. Paste the actual lines. If you cannot produce the actual output, the verification step is not complete.

**Rule 3 — Stop and report when a tool fails.** If a shell command, file operation, or any tool call fails or returns an unexpected result: (1) stop immediately, (2) report exactly what failed and the error, (3) do not attempt to work around it silently, (4) do not write a success summary. Joe will decide how to proceed.

**Rule 4 — Write the journal entry last.** The JOURNAL.md entry is written *after* all work is complete and all verification steps are confirmed with real output. Order: do the work → run verification → confirm outputs → write journal entry quoting actual output → update CLAUDE.md → commit and push.

**Rule 5 — Never invent numbers.** Test counts, file counts, line counts — these must come from actual command output. If you did not run `cargo test`, you do not know the current test count — say so.

**Rule 6 — When in doubt, do less and ask.** If a task instruction is ambiguous, or completing it would require a decision not covered by the instruction file, stop and flag the ambiguity. Do not make the decision silently. Write a clear question to Joe and wait.

**Rule 7 — Definition of Done is a checklist, not a formality.** Every task file ends with a Definition of Done checklist. Each item must be independently verified before being marked complete. Mark items complete only when confirmed with actual output or observation.

| Situation | Correct behaviour |
|---|---|
| Command succeeds | Quote actual output in journal |
| Command fails | Stop, report the exact error, do not continue |
| Tool unavailable | Report it, do not fabricate the result |
| Ambiguous instruction | Ask Joe, do not assume |
| Verification step fails | Stop, report, do not write success summary |
| Unknown test count | Run `cargo test` and quote output — never invent a number |

---

## ✅ DONE — CLI Flag Precedence Audit (D-068): SHIPPED — J-079, 5 atomic commits, 463 tests, five violations closed

**Status: SHIPPED — J-079.** The CLI Precedence Audit (`tasks/CLI_PRECEDENCE_AUDIT.md`, D-068) closed on 2026-05-17 in five atomic commits. The audit surfaced and fixed **five distinct violations**, not just the originally-named `--port` defect: one flag-threading bug (`xgen-node --port` was structurally orphaned from `run_node`) plus four parallel hardcoded subscriber-init blocks (`xgen-client --service`, `--service --ai-mode`, Tauri shell; `xgen-node` Tauri shell) silently bypassing `[logging].level` and falling back to a hardcoded `"debug"` literal. Helpers `xgen_common::precedence::resolve_setting<T>` (generic flag>env>config>default) and `resolve_log_level` (XGEN_LOG-aware specialisation) shipped in commit 1. The two previously-compliant subscriber-init paths (Node `run_node`, Client short-lived CLI) were also refactored onto the canonical helper in commit 3 for consistency and regression-locking. After J-079, **every log-level resolution in the codebase routes through one function** — the drift surface that produced these violations is architecturally eliminated, same shape as M5/D-067 eliminated drift in `xgen-client-lib::ops`. Test count 435 → **463** (+10 unit precedence + 5 URL-rewrite + 6 Node integration + 7 Client integration). Doc sync: Appendix F §F.0.6 updated; DECISIONS.md D-068 gained a closing note; both `main.rs` files' doc comments aligned with §F.0.6.

**Commits:** `3e2f311` helper + tests → `f77fe25` `--port` plumbing → `32028ad` four-site convergence → `1b62fed` integration tests → `19714ad` doc sync.

**Carry-overs:**
- ~~`xgen-client --quiet` doesn't gate the per-subcommand `Connecting to <node>...` line~~ **CLOSED in J-080 (2026-05-18, commit `1d991a4`).** All 10 network-doing shims gain `quiet: bool`; gated per Appendix F §F.0.1.
- ~~Short-lived Client CLI log file lands in `<exe_dir>/logs/` instead of `<data_dir>/logs/`~~ **CLOSED in J-080 (commit `c217844`).** `init_logging` takes `data_dir`; symmetric with `--service` / `--ai-mode --service` / Tauri shell. Per D-035.
- ~~`xgen-node/src/desktop.rs::maybe_write_default_config` writes a non-schema `port = N` field~~ **CLOSED in J-080 (commit `73fbbad`).** `default_config_toml()` now serialises a full `NodeConfig` rooted at `data_dir`; roundtrip-tested.
- Plus the M4 carry-over (`cmd_create_space` optimistic-ack UX): **DEFERRED to M6/M7 design phase** per J-080 §4. Investigation revealed this is not a Client-side UX bug but a missing protocol primitive (no positive accept signal exists today). Context recorded in `tasks/NODE_ADMIN_PASS2_PROPOSALS.md` §"Pass-3 input: missing protocol accept signal" for M6 Pass 3 discussion.

---

## 🔴 DEPRECATED — M6 (original) Multiparty baseline pass with present `--batch`: descoped 2026-05-17

**Status: DEPRECATED.** The original M6 milestone (run the full Multiparty suite S1–S5 twice through present `--batch` to fill the "A" baseline column) is descoped on 2026-05-17. Replaced by **M9 Multiparty Redesign** (see roadmap below).

**Why descoped.** Three reasons surfaced when M6 was about to start after J-079:

1. **Shovel-readiness gap.** The two task files (`tasks/MULTIPARTY_S1_tauri_rerun.md`, `tasks/MULTIPARTY_S2_to_S5_present_pass.md`) were written before J-079 and assumed the binary as it stood at M5. The CLI Audit changed five sites in the logging and flag-resolution paths; the task files do not reflect that. Running them as-is would measure a binary whose behaviour has shifted from what the runbook anticipated.
2. **Metric-protocol applicability needs reconfirmation.** The metric set in `tasks/BATCH_FLAG_review.md` §"Baseline metrics protocol" was Joe-locked in conversation on 2026-05-16 (pre-J-079). Whether the same metrics still apply against the post-audit binary is a question that needs Joe's input, not a question Clair should silently answer at runtime.
3. **The bigger problem M6 was meant to solve has shifted.** M6 existed to create A/B evidence for the `--batch` → `--aicontrol` improvement. With the realisation that `--aicontrol` (and `--batch`) must be **read-write on both Node and Client** — not just Client — the surface that needs validating is bigger than originally scoped. A measurement pass on the Client-only present `--batch` would not produce comparable numbers against an improved surface that spans both binaries.

**What the descope means in practice.** The M6 slot is **reused** for the Node admin write path (see M6 (new) PENDING block below). The multiparty work is rescheduled as **M9 Multiparty Redesign** at the end of the M-series trunk — redesigned to measure both binaries' read-write surfaces (`--batch` and `--aicontrol`) against each other, not the original Client-only `--batch` A/B framing.

**Cross-references.** D-066 (original roadmap) gains a closing note pointing at this descope. D-069 (new this session) records the discipline lesson: delegated technical designs must be Joe-locked AND must flag their own open items before the implementing milestone is declared ACTIVE.

**Affected task files** (flipped to DEPRECATED in this same session):
- `tasks/MULTIPARTY_S1_tauri_rerun.md` → Status: DEPRECATED, pointing at M9
- `tasks/MULTIPARTY_S2_to_S5_present_pass.md` → Status: DEPRECATED, pointing at M9

The metric set in `tasks/BATCH_FLAG_review.md` §"Baseline metrics protocol" is retained as a starting point for M9's design phase; M9 may revise it once the both-binaries scope is locked.

---

## ✅ DONE — Propagation Reliability Audit: CLOSED (J-081, canonical doc shipped, federation gap surfaced)

**Status: SHIPPED — J-081.** Audit closed 2026-05-18. Canonical document at `docs/xgen_propagation_reliability.md`. All five stage sections written under per-section Joe-approval gate. Verdicts: §1 PARTIALLY VERIFIED (Stage 5 local fan-out — mechanism correct, two LOW documentation/observability gaps); **§2 GAP IDENTIFIED HIGH (Stage 6 Node-to-Node federation propagation — architecturally absent)**; §3 GAP IDENTIFIED HIGH (Stage 7 — consequence of §2); §4 PARTIALLY VERIFIED (Stage 8 sync catch-up — works for current workloads, spec-vs-impl + scale gaps); **§5 GAP IDENTIFIED HIGH (`TransportMessage::Error` — wire shape lacks `event_id`, no event-acceptance reject paths emit it).**

**Primary finding.** Node-to-Node federation event propagation does not exist as a production mechanism. Three independent traces converged: (1) `run_initiating` has zero production callers in `xgen-node/src/` — only tests; (2) no pull mechanism — `space.join_request` is only received in production, never sent; (3) stress-test "Federation Completeness" measures local-clients delivery only, not cross-Node propagation — J-059's 6/6 PASS is consistent with and expected from a system with no ongoing federation push. The design doc `docs/xgen_node_admin_ops_design.md` §4.2 sentence describing federation push describes a mechanism that does not exist in the codebase.

**Secondary finding.** `TransportMessage::Error` wire shape ([`xgen-core/src/wire/types.rs:75-82`](xgen-core/src/wire/types.rs:75)) has NO `event_id` field. Single production emit site is identity-replicate failure ([`xgen-node/src/app.rs:1085`](xgen-node/src/app.rs:1085)), not event acceptance. None of the event-acceptance reject paths in `process_inbound` emit `Error` — they all just log via `tracing::error!` + `trace_local(RejectEvent)`. The J-080 framing that "Error is the rejection signal for event acceptance" was confidently wrong across multiple sessions, refuted by direct trace.

**Pattern observation.** The audit found drift surfaces in 4 of 5 sections (§2 design doc federation push, §3 `process_inbound` validation asymmetry, §4 Ch4 §implementation sync flow + unimplemented `sync_response`/`sync_complete`, §5 design doc Error shape + emit paths). Recorded as fact in §6.2 of audit doc. Implication ("subsystem audits precede dependent milestones" as a new project principle) is a project-management conversation Chat Claude + Joe will have post-audit.

**Joe-locked direct during close-out — M6 (new) Phase 2 scope adjustment.** Rather than open a Pass 4 design session for the rejection signal, Joe locked the design call: `event_id: Option<String>` at the `TransportMessage` envelope level (base of the transport-message hierarchy); `EventAccepted` is the only new variant; `Error` covers rejection by populating envelope `event_id`. No new `EventRejected` variant. Practical effect: original 6 Phase 2 deliverables stand + envelope field + wire `Error` into 5 reject paths + client-side correlation. **✅ Documentation pass on the design doc closed 2026-05-18** (§3.1 corrected to locked envelope-level shape, §3.2-§3.4 envelope reference, §9 marked SUPERSEDED with pointer to canonical DECISIONS.md D-070; original Pass-3 §9 body preserved as historical record).

**Carry-overs into downstream milestones:**
- All HIGH-severity findings (§2, §3 peer-side ingestion, §5 Error wire shape, §5 reject-path emission) close in two coordinated downstream items: (a) Federation Event Propagation milestone (see PENDING block below), (b) M6 (new) Phase 2 with the Joe-locked envelope scope.
- `process_inbound` validation asymmetry (Paths B/C skip signature verification) is LOW today but HIGH on federation landing; **precondition** of the Federation Completion milestone, not parallel work.
- No follow-on task files filed (per D-069 discipline — downstream milestones go through their own Joe-locked design phase first).
- 468 tests unchanged — no code changes in this audit.

---

## ✅ DONE — Federation Event Propagation design phase: SHIPPED (canonical doc v1.0 ACTIVE, runbook handed off to Clair)

**Status: SHIPPED — Pass 3 close 2026-05-18.** Design phase closed in same-day session that followed Pass 2. All ten framework decisions locked across F-1 (hybrid push direction) + F-1a (tip exchange) + F-1b (drop-on-peer-down) + F-1c (per-peer record) + F-2 (long-lived continuous session) + F-2 lifecycle + F-2a (one WS per pair bidirectional) + F-3 (event signature + federation relationship verification) + F-4 (unified validation core) + F-4a (30s HeldPending uniform) + F-4b (structural before / semantic after) + F-5 (transitive locked-out v1) + F-6 (sync_complete fold-in) + F-6a/b (wire shape + 5s configurable safety-net) + F-7 (pagination fold-in) + F-7a (1000 default `[sync].batch_size`) + F-8 + F-9 (Ch4 + admin-ops doc corrections at Pass 3) + F-10 (HeldPending extended for unknown signer Identity) + F-10a.

Canonical doc: `docs/xgen_federation_propagation_design.md` (v1.0, Status ACTIVE). Three Pass-2 addenda consolidated into the main doc as §10 (F-7), §11 (F-8), §12 (F-9), §13 (F-10) at Pass 3 and deleted from disk. All `[JOE-LOCK]` markers walked to final form: `[JOE-LOCK: locked 2026-05-18 (Pass 2 conversation, Pass 3 promotion)]`. F-8 corrections applied to `docs/xgen_ch4_implementation.md` §4.11.3 + §4.12.3 (forward-references to canonical design doc; located by content match against unique phrases "per-peer outbound queue" and "Node sends `transport.sync_request` to its peers for the missing predecessors" rather than the audit's stale line numbers). F-9 correction applied to `docs/xgen_node_admin_ops_design.md` §4.2 (Federation propagation Stage-6 sub-bullet now a forward-reference). All in the Pass 3 commit.

**Next: Federation Event Propagation implementation (🟡 PENDING).** Runbook at `tasks/FEDERATION_PROPAGATION_COMPLETION.md` (Status: ACTIVE, v1.0) is the next-active task for Clair. Nine phases: (1) `sync_complete` + pagination wire shape + four-call-site migration, (2) `process_inbound` validation pipeline unification — the precondition, (3) federation handshake reshape to tip exchange, (4) federation event push — the load-bearing phase, (5) per-peer record + reconnect scheduling, (6) HeldPending generalisation for unknown signer Identity, (7) F-3 federation-relationship verification gate, (8) documentation pass, (9) integration tests. **Hard ordering: Phase 2 MUST land before Phase 4 — federation push without validation asymmetry closure lands the audit's HIGH-severity vulnerability vector.** Runbook makes the ordering hard.

**Coordination with M6 (new) Phase 2:** the envelope-level `event_id` on `TransportMessage::Error` work locked at audit close (per M6 design doc §6.5) wires into the rejection paths that this milestone's Phase 2 + Phase 7 produce. M6 (new) is blocked behind this milestone going DONE; M6 Phase 2 ships its wire-layer rejection signal in M6's own milestone.

**Test baseline at runbook handoff: 468.** No code changes in Pass 3.

**Carry-overs at design close:**
- ✅ D-070 promoted to DECISIONS.md (2026-05-18, same-day post-Pass-3): "Two events of equal importance, opposite direction" named protocol principle, with corrected post-audit framing requiring BOTH existence (acceptance + rejection signals) AND envelope-level `event_id` correlation on both directions. M6 design doc §9 draft preserved as historical record; DECISIONS.md D-070 is the canonical authoritative form.
- ✅ D-071 promoted to DECISIONS.md (2026-05-18, same-day post-D-070): "Subsystem audits precede dependent milestones" project-management principle. Sibling to D-065 and D-070 (protocol-design); D-071 names the discipline J-081 retroactively instantiated. Pairs with D-069: audit phase → design phase → implementation phase, each producing a canonical artefact.
- Pass 2 task file (`tasks/FEDERATION_PROPAGATION_DESIGN.md`) and Pass 3 task file (`tasks/FEDERATION_PROPAGATION_PASS_3.md`) both flipped to COMPLETED in the Pass 3 commit.

**Cross-references:** `docs/xgen_federation_propagation_design.md` (canonical, v1.0 ACTIVE). `tasks/FEDERATION_PROPAGATION_COMPLETION.md` (runbook, ACTIVE). `tasks/FEDERATION_PROPAGATION_DESIGN.md` (Pass 2 task, COMPLETED). `tasks/FEDERATION_PROPAGATION_PASS_3.md` (Pass 3 task, COMPLETED). `docs/xgen_propagation_reliability.md` (J-081 audit, ARCHIVED). D-065 (honest behaviour over polite behaviour). D-069 (Joe-locked design phase + canonical-document rule).

---

## 🟡 PENDING — M6 (new) Node admin write path

**Status: PENDING.** Phase 0 (design) closed 2026-05-18: 12 framework decisions locked, canonical design doc shipped at `docs/xgen_node_admin_ops_design.md`. **M6 (new) does not go ACTIVE until BOTH the Propagation Reliability Audit milestone (now ✅ DONE) AND the Federation Event Propagation completion milestone (see PENDING block above) close.** Block 4 (verb-by-verb walks across the seven categories) is also deferred to its own session; the design doc's §6 verb-list sections are stubbed pending Block 4.

**Phase 2 scope adjustment (Joe-locked direct at audit close, no new design pass needed).** The original 6 Phase 2 deliverables in `docs/xgen_node_admin_ops_design.md` §5.2 stand. Added at audit close (J-081 §6.5 of audit doc):

- `event_id: Option<String>` on the `TransportMessage` envelope (base of the transport-message hierarchy), populated when the message pertains to a specific event.
- `EventAccepted` remains the only new variant.
- Event-rejection paths in `process_inbound` ([`xgen-node/src/app.rs:846-851`](xgen-node/src/app.rs:846), [`855-858`](xgen-node/src/app.rs:855), [`885-897`](xgen-node/src/app.rs:885), [`913-921`](xgen-node/src/app.rs:913), [`926-934`](xgen-node/src/app.rs:926)) emit `Error` with `event_id: Some(...)`. No new `EventRejected` variant — `Error` covers rejection by populating envelope `event_id`; `error_code` namespace already encodes semantic meaning.
- Client-side handling correlates envelope `event_id` against in-flight submissions.
- Confirm during implementation: serde derive handles `Option<String>` as omittable for backward-compat with pre-M6 clients (likely yes via `#[serde(skip_serializing_if = "Option::is_none")]`).

Structural realisation latitude for Clair: Rust type design, serde derives, module organisation, internal refactors that preserve wire shape — *cleaner is better*. Wire-format-visible changes beyond the locked envelope `event_id` addition require Joe-lock (threshold: would a future contributor reading the change ask "why was this decided?" — if yes, pause for Joe; if no, ship as normal engineering judgment).

**What this is.** The Node binary today has a partial pipe-server surface: `--batch` shipped in M2 with a **read-only** verb subset (`status`, `connections`, `peers`, `spaces`, `identity list`, `version`, `whoami`). There is no Node-side **write path** for administration. An operator who needs to add a federated peer, register an Auth Module, update Bootstrap configuration, change moderation policy on a hosted Space, or reload config live on a running Node has no automation surface for any of this. `--reload-config` returns honest `NOT_IMPLEMENTED` today.

M6 (new) closes this gap: it ships the Node admin write path as an extension to `--batch` first (humans / scripts / CI), with the same verbs becoming available to `--aicontrol` in M7. The principle is symmetry with the Client: both binaries get full read-write `--batch` AND full read-write `--aicontrol`.

**Why this comes before M7.** `--aicontrol` is the AI-shape protocol over an administrative surface. The surface itself has to exist first. Designing `--aicontrol` Node verbs before the underlying admin subsystems exist would mean designing a JSONL protocol with nothing to call. M6 (new) ships the underlying subsystems; M7 wraps them in the AI-shape protocol.

**Categories of Node admin verbs to design** (sketch — not locked, design-phase deliverable):

- **Federation management** — accept/reject incoming federation requests, initiate federation with a peer, defederate, per-peer allow/deny policy, submit defederation signals to Bootstrap Nodes (§3.15).
- **Auth Module management** — register a new Auth Module, revoke trust, change accepted Tiers.
- **Bootstrap configuration** — register/deregister with Bootstrap Nodes, change `bootstrap_info` metadata, update advertised `auth_tiers_served`.
- **Space and Room operator actions** — force-eject (Node-operator authority, distinct from member-initiated kick), set Node-level moderation policy, trigger Space migration as source Node.
- **Identity registry administration** — revoke a registration with audit trail, update stored Trust Assertion expiry, manage replica relationships.
- **Logging and audit administration** — rotate audit logs, query audit log (read), set log levels per module at runtime (the real `--reload-config` story).
- **Plugin management** — load, configure, unload, query status of moderation plugins (the home of the temperature plugin's runtime surface).

**Design-phase deliverables (must be Joe-locked before M6 is declared ACTIVE, per D-069):**

1. **Verb-set enumeration.** Exact list of verbs per category, with their `args` and `data` schemas. The set probably grows to 30+ verbs.
2. **Privilege model.** Which verbs require what proof of Node-operator authority (the Node keypair? a separate admin keypair? OS-user identity over the pipe?). Today the pipe is unauthenticated on the assumption that pipe-access = Node-operator-on-same-host; whether this holds for write-path verbs is part of the design.
3. **Live-reload semantics.** `--reload-config` becomes a real verb — which config fields are reloadable without restart, which require restart, what the rollback path is on bad config. This is the heart of D-069's "admin makes config updates during the Node's going" use case.
4. **Audit trail integration.** Every write-path verb produces a protocol audit log entry per §3.11.8 (the audit-log facility already specced). Schema additions if needed.
5. **Symmetry with `xgen-client-lib::ops::*`.** The Node equivalent (likely `xgen-node-lib::admin_ops::*` or similar) follows the M5 pattern: one canonical function per verb, three dispatchers (CLI arm, batch arm, future aicontrol arm) all thin shims. No drift surface.

**Not in M6 — explicitly:**
- `--aicontrol` itself. M6 ships the surface that `--aicontrol` will wrap; the wrapping is M7's job.
- Client-side admin verbs. The Client doesn't have an admin role in the same sense; its `ops::*` already covers the Identity-side actions.
- The full canonical `--aicontrol` document. **Already created 2026-05-17** at `docs/xgen_aicontrol_implementation.md`, covering both binaries from day one. M7's design phase resolves its §12 open items and Joe-locks the result; M6 does not edit this document.

**Entry point for the next session:** Federation Event Propagation implementation runbook (`tasks/FEDERATION_PROPAGATION_COMPLETION.md`, Status ACTIVE). M6 (new) sits behind that.

---

## ✅ DONE — M5 `ops::*` refactor: SHIPPED (435 tests, 12 atomic commits, 17/17 smoke PASS, F-003/F-004 architecturally closed)

**Status: SHIPPED — J-078.** Every user-facing `xgen-client` verb (13 total) now routes through a single `xgen-client-lib::ops::<verb>` function. All three dispatchers (`main.rs` CLI arm, `app::run_batch_file` CLI batch driver, `batch::dispatch_line` pipe arm) became thin shims calling the same `ops::*` function; each dispatcher owns its own output format. New `xgen-client/src/session.rs` (`SessionState`, `ClientIdentity`, idempotent `ensure_identity` / `ensure_connected` helpers — extension fields `bindings` / `spaces` present-but-empty for M7-shape stability). New `xgen-client/src/ops.rs` (one `pub async fn <verb>(ctx, args) -> Result<<Verb>Result>` per verb; pure data extraction; the canonical `load_or_default_state` helper). The drift surface that produced F-003/F-004 in J-067 is architecturally eliminated — there is now nowhere a second `get_dag_tips` (or any other implementation duplicate) could be introduced without being noticed. 17/17 smoke PASS against two live Nodes on `:8080`/`:8081` confirms the refactor preserves wire-correct behaviour end-to-end. Test count 429→435 (+6, all from new ops/session unit tests in commits 1-4). D-067 captures the structural outcome.

**Carry-overs:**
- ~~`xgen-node --port <port>` did not override `xgen-node_config.toml::listen` on first invocation during M5 smoke setup; second invocation of the same command succeeded. Flag-vs-config precedence bug in `xgen-node`.~~ **Scheduled as the CLI Precedence Audit (D-068, `tasks/CLI_PRECEDENCE_AUDIT.md`) — see ACTIVE block at the top of this file.**
- Tauri commands for the 13 protocol verbs still don't exist; current Tauri shell is lifecycle-indicator + pipe-server only. When verb-level Tauri commands eventually land they will naturally call `ops::*` — that's M5's prerequisite that's now met.
- `cmd_create_space` optimistic-ack UX bug (J-077, J-078). Future UX pass.

---

## ✅ DONE — M4 AI Client Binary: SHIPPED (429 tests, --ai-mode resident, mention→reply smoke green)

**Status: SHIPPED — J-077.** The AI Client is a *mode of `xgen-client`* (locked §1): `xgen-client --ai-mode --service` runs a long-running headless resident that consumes inbound events through an `AiBehavior` plugin and emits replies under existing pacing + mute constraints. New `xgen-client/src/ai_behavior.rs` (trait + reference `EchoPlugin` with locked deterministic reply format) and `xgen-client/src/ai_service.rs` (runtime loop, `AiPacingTracker` sibling of PacingManager for drop-on-throttle, plugin loader). `__HEALTH__` extended with `mode=ai operator_known=N/M`. Single-Node smoke confirmed: alice mentions bob (AI) → bob replies after `ai_pacing_ms`; back-to-back mention drops the second with literal warn line `dropping reply — pacing cap not yet elapsed (honest behaviour over polite behaviour) ai_pacing_ms=2000`. Spec §6.15 added to Ch6 (10 subsections); D-065 captures M4 architecture AND names the recurring "honest behaviour over polite behaviour" principle with its other instances across the protocol (operator resolution, Node event rejection, mute semantics, the create-space ack bug carry-over).

**Carry-overs (none blocking):**
- ~~`cmd_create_space` doesn't await ack — Client prints "Space created" even on Node-side rejection.~~ **DEFERRED to M6/M7 design phase** in J-080 (2026-05-18). Investigation revealed the underlying problem is not Client-side UX but a missing protocol primitive: no positive accept signal exists today (`xgen-node-lib::fanout` deliberately excludes the originator from fan-out; rationale documented only as a test code comment). Path A: do not speculatively patch fan-out; record the context as a Pass-3 input for M6 design. See `tasks/NODE_ADMIN_PASS2_PROPOSALS.md` §"Pass-3 input: missing protocol accept signal".
- Consolidated Node-side event-accept pipeline. Today's fragmentation (`accept_message` for message.*, dedicated arm for `membership.join`, catch-all `_ =>` for everything else) is fragile. Structural work for a future milestone; not blocking M5 candidates.
- `EventStore` HashMap iteration determinism. Doesn't affect M4 (the AI resident applies events in arrival order, not via sync-request replay).
- `prev_events` integrity for joins from non-members (M3 carry-over, timestamp-sort workaround in `cmd_ai_status` still in place).
- `docs/xgen_appendix_f_en.md` comprehensive example rewrite — Joe's gate of "M2 + M3" reached at M3 close-out; available whenever it surfaces as priority.
- AttachConsole hybrid-app polish (cosmetic Windows console flash).
- Cross-platform pipe server. D-043 still Windows-only.

---

## ✅ DONE — M3 AI Operator Role: SHIPPED (J-075)

411 tests. Operator as distinct role within Spaces (per-(AI, Space)). `SpaceMember.invited_by` + `SpaceState.ai_operator_delegations` + `resolve_operator` three-step fall-upward algorithm (stored delegation → AI's inviter → Space owner, transparently skips members who left). Both `state.space_create` and `state.dm_space_create` from an AI sender rejected with **3041 `ai_role_violation`** (wire name widened from `ai_flag_immutable`; code unchanged). Client CLI: `init --ai [--cap k=v]`, `register` honours `[ai]` config, new `ai delegate`/`ai revoke`/`ai status` subcommand group. Two-Node federation smoke (Rust integration) verifies decision #6's three cross-Node scenarios with strict assertions. Spec §3.6.10.6 rewritten; D-064 captures locked architecture.

---

## ✅ DONE — M2 Node Pipe Server: SHIPPED (J-074)

Six Node-side flags (`--ping`, `--health`, `--stop`, `--reload-config`, plus pipe-side `--batch`) became real implementations. New `xgen-node/src/pipe.rs` ports the Client's pipe-server skeleton to the Node with the four control commands plus a read-only `__BATCH__` subset (status / connections / peers / spaces / identity list / version / whoami). `__HEALTH__` returns the rich `HEALTHY pid=… state=RUNNING conns=… peers=… spaces=… uptime=…s` line. `__RELOAD_CONFIG__` returns honest `NOT_IMPLEMENTED` (real reload is a separate milestone). Pipe server spawns inside `app::run_node` so both `--service` and Tauri get it; `_pipe_shutdown_hold` at the `run_node` async-block scope (J-071 lesson). 391 tests held through M2.

---

## ✅ DONE — M1 Binary Consolidation: SHIPPED (J-073)

Six-commit chain (`e864715` → `c23c06a` → `1da3f1e` → `df877cb` → `4a9243b` → J-073 commit) collapsed four binaries to two: Tauri compiled into both per D-062, library-first dispatch per D-063, all 19 fundamental flags wired, Client `--batch` parallel implementations collapsed, Client `--service` headless resident operational, `cmd_init` instance-aware. Full matrix: 45/49 headless + 4/4 visual cells (N1, N2, C1, C2) confirmed by Joe. Full breakdown: J-068 → J-073.

---

## ✅ DONE — MULTIPARTY_S1 (local fan-out) — first of the five-file Multiparty suite

**Status: COMPLETE — M1 PASS, M2 PASS-with-caveat (J-067, 391 tests pass, 4 bugs found+fixed in-session)**

Detail folded — see prior versions for full F-001 through F-004 history. M1 P1 Smoke PASS; M2 P2 Stress PASS-with-caveat (300 messages dispatched within 96 ms, 294/300 accepted, 6 silently dropped between client WS write and Node receive — cause unclear, follow-up deferred to post-multiparty-redesign).

---

## ✅ DONE — AI Identity, Pacing, and Temperature (D-059, D-060, D-061)

**Status: COMPLETE — 387 tests pass (J-065, 352 xgen-core + 12 xgen-node + 23 xgen-client-lib)**

All three Parts shipped: Part A — AI Identity Extension (D-059, §3.6.10); Part B — Per-Space Pacing Rules (D-060, §3.7.12); Part C — Temperature Property (D-061, §3.7.13). Out of scope deferred: math model that produces temperature values (plugin-owned); Phase 3 Node-side enforcement of pacing / `spontaneous_post`; Svelte UI components; the 13-step manual two-Node verification.

---

## ✅ DONE — Full integration stress test (J-059, 6/6 PASS, 14.6 s, 300 tests)

3-node topology (Node A: 9080, Node B: 9081, Node C: 9082 + Bootstrap). All 6 scenarios pass, 43/43 checks. Two bugs found and fixed during live run: stack overflow in large async fn (32 MB thread dispatch), B↔C federation recv hang (replaced with explicit goodbye). Comm record at `docs/tests/stress_complete_events.json`.

---

## ✅ DONE — Phase 2 integration testing (60/60 PASS, D-054–D-056, J-056–J-058, 300 tests)

All Phase 2 protocol layers (11–19) complete. Integration smoke test `smoke-ph2` passes all 60 steps against two live `xgen-node` processes over real TCP. One transport-layer bug discovered and fixed during the live run (D-056 — `recv()` routing collision between DAG Events and control messages on shared type-prefix strings).

---

## ✅ DONE — Phase 2 Track 1 infrastructure (Sessions 14–18, 173 → 300 tests through this phase)

Tauri scaffold, 11 Client lifecycle states + 7 Node + degraded stacking, named pipe IPC (D-043), `--instance` flag, `--batch` flag, xgen-core crate split (D-022, D-044). Detailed table folded — see prior versions or the per-instruction-file headers under `docs/tests/` for full breakdown.

---

## ✅ PHASE 1 IS COMPLETE — DO NOT RE-IMPLEMENT

All Phase 1 deliverables done: binary wiring, 17-step smoke test against real TCP, documentation gates, stress test. Tag `v0.10.3`. See historical snapshot below for the layer-by-layer record.

---

## ✅ DONE — Phase 1 logging + event tracing

Phase 1 debug logging (`docs/tests/LOGGING_debug_ph1.md` — J-025): datetime-stamped log files, config level switch, subscriber init, operational log calls in both binaries. Audit log (`docs/tests/LOGGING_audit_ph2.md`) deferred — alongside Tier 2+ Auth Module work only.

Global Event tracing interface (`docs/tests/LOGGING_debug_ph2.md` — J-027, J-029): `event_trace` module in `xgen-common/src/` (Fix 17 applied). `Event` and `EventType` moved to `xgen-common/src/wire.rs`. Role gate active. Content field never logged. 173/173 tests; smoke test with debug logging confirmed full Event pairing across client and both Nodes.

---

## ✅ DONE — Documentation fixes (FIXES_ph1.md)

All 17 fixes applied (Fix 14 deferred by project owner). Fix 16 (Node space state replay on restart) and Fix 17 (event_trace relocation) complete in Rust source. Documentation fixes 1–15 applied to Ch3/Ch4.

---

## ⏸ POSTPONED — UI Phase 2 prep (run 1.5)

UI design work for Phase 2 Track 1 is paused at the element-modelling step (J-033, 2026-05-08). Resume condition: confirmed absent-element list in `ui/docs/xgen-ui-design-brainstorm.md` (Points 2 and 3) reconciled with Ch3's authoritative event taxonomy + Run 3 design briefing drafted. Until those gate, no visual merge work begins. Recorded in `JOURNAL.md` J-033 and `DECISIONS.md` D-041.

---

XGen Protocol is an open, federated, identity-verified communication protocol. Think of what Discord would have been if built as open infrastructure. The core thesis: no single entity should own the communication layer.

This is not a product — it is protocol infrastructure. Phase 1 is a minimal working implementation. Phase 2 is the full protocol. Phase 3+ is everything else.

**The spec is authoritative.** When this file and the spec conflict, the spec wins. When the spec is ambiguous, flag it — do not resolve it silently.

---

## Current State — Where We Are

**Federation Event Propagation implementation Phases 1-8 shipped (J-082 + J-083 + J-084 + J-085 + J-086 + J-087 + J-088 + J-089 across 2026-05-18 and 2026-05-19).** Phase 8 (J-089) closes the documentation pass — six accumulated doc-vs-code drift surfaces from Phases 5-7 fixed, plus the standard "forward-reference → implementation-complete" updates. Tests: 468 (handoff) → 476 (Phase 1) → 480 (Phase 2) → 488 (Phase 3) → 491 (Phase 4) → 505 (Phase 5) → 516 (Phase 6) → 519 (Phase 7) → **519** (Phase 8 close — documentation only per DoD). Next active phase: Phase 9 (deployment-level integration tests, six DoD scenarios). After Phase 9 ships, milestone flips PLAY → DONE and M6 (new) unblocks. Roadmap: M5 ✅ → CLI Audit ✅ → J-080 ✅ → M6 Phase 0 Pass 3 ✅ → Propagation Reliability Audit ✅ → Federation design (Pass 2 + Pass 3) ✅ → **Federation implementation Phases 1-8 ✅, Phase 9 (deployment integration tests) next** → M6 (new) → M7 → M8 → M9.

Current project status as of 2026-05-19:

- **Phase 1**: complete (J-029, tag `v0.10.3`, 17-step smoke test passing over real TCP). See historical snapshot below.
- **Phase 2 protocol**: complete (J-058, `smoke-ph2` 60/60 PASS, layers 11–19 all shipped).
- **Phase 2 Track 1 UI**: partially complete; deeper visual-merge work POSTPONED.
- **Post-Phase-2 protocol work shipped:** AI Identity + Pacing + Temperature (J-065), full integration stress test (J-059).
- **M1–M5 shipped**: binary consolidation, Node pipe server, AI operator role, AI Client resident mode, ops refactor.
- **CLI Audit shipped (D-068)**: J-079, 5 atomic commits, 463 tests, five violations closed.
- **J-080 carry-over pass**: 468 tests; 3 of 4 carry-overs closed; item 4 deferred to M6 design.
- **M6 Phase 0 closed 2026-05-18**: 12 framework decisions locked, canonical design doc shipped at `docs/xgen_node_admin_ops_design.md`.
- **Propagation Reliability Audit CLOSED (J-081, 2026-05-18)**: 4 of 5 sections found drift; Stage 6 federation propagation architecturally absent.
- **Federation Event Propagation design phase**: SHIPPED (Pass 2 + Pass 3 closed 2026-05-18). Canonical design doc at `docs/xgen_federation_propagation_design.md` (v1.0, Status ACTIVE) — all 10 F-items locked, three Pass-2 addenda consolidated as §10–§13, F-8 + F-9 corrections shipped.
- **Federation Event Propagation implementation**: 🟢 PLAY. Nine-phase runbook; **Phases 1-8 ✅ SHIPPED** (J-082..J-089). **Phase 8 ✅ (J-089, 2026-05-19)** — Documentation pass closing the six accumulated doc-vs-code drift surfaces from Phases 5-7 plus the standard forward-reference → implementation-complete updates. Ch3 §3.3.6 wire shape rewritten to shipped `{ protocol_version, since, new_tip, continue_from }`; Ch3 §3.9.6 + §3.9.8 add `4006 identity_record_timeout` with predecessor-code-wins sub-rule; Ch4 §4.11.2 rewritten to JSON-backed `FederationRegistry`; Ch4 §4.11.3 + §4.12.3 + admin-ops §4.2 forward-references → implementation-complete; design doc §6.4 leading authority paragraph names `SpaceState.federation_nodes` + B1 implementation note; new design doc §15 Implementation Complete records all eight shipped phases. 519 tests at Phase 8 close (unchanged — documentation only). Phase 9 (deployment-level integration tests covering six DoD scenarios) is next-active.
- **M6 (new)**: Node admin write path PENDING. Phase 0 design closed; ACTIVE flip waits behind Federation Event Propagation milestone closure.
- **Phase 3 areas**: state migration depth, federation depth, MLS operationalisation. D3 (MLS) parallel.

### Historical snapshot — Phase 1 completion (April 2026, tag `v0.10.3`, 173 tests)

This table records how Phase 1 landed and is preserved as a historical reference. Test counts and tags are frozen as of April 2026; current counts and milestones are above.

| Layer | Content | Tests | Tag |
|---|---|---|---|
| 1 | Crypto (Ed25519, SHA-256, base64url, ChaCha20+Argon2id) | 25 | v0.1.1 |
| 2 | Wire format (Event, EventType, framing, validation steps 1–7) | 53 | v0.2.2 |
| 3 | DAG event store (append-only, tips, pending buffer) | 79 | v0.3.2 |
| 4 | WebSocket transport (challenge-response auth, keepalive) | 88 | v0.4.2 |
| 5 | Node identity and announcement | 100 | v0.5.2 |
| 6 | Federation handshake (state machine, registry) | 121 | v0.6.2 |
| 7 | Identity registration (8-step pipeline, registry) | 142 | v0.7.2 |
| 8 | Space and Room protocol (state machine, roles, permissions) | 160 | v0.8.2 |
| 9 | Message exchange (validation steps 8–13, accept_event) | 171 | v0.9.3 |
| 10 | Smoke test — spec 3.7.11, 17-step end-to-end | 173 | v0.10.1 |
| CLI | init, status, connections, spaces, peers, identity list, whoami (D-025–D-028) | 173 | v0.10.2 |
| Binaries | xgen-node WebSocket server + xgen-client network commands + 17-step smoke test over real TCP | 173 | v0.10.3 |

---

## Architecture Rules — Non-Negotiable

**1. Library-first.** All protocol logic lives in `lib.rs`. `main.rs` is a thin CLI shell only — argument parsing, startup, shutdown. No business logic in `main.rs`. This is what makes Phase 2 Tauri integration possible without rewriting.

**2. Spec is authoritative.** `docs/xgen_ch3_specification.md` is the source of truth. `IMPLEMENTATION_GUIDE_ph1.md` is the implementation guide. When they conflict, the spec wins.

**3. Verify after every write.** Read back every file after writing it. Silent write failures have caused reconstruction work in past sessions.

**4. DECISIONS.md before advancing.** Every implementation decision beyond spec prescription must be recorded in `DECISIONS.md` before moving to the next layer. Format: title, date, layer, spec reference, decision narrative.

**5. Tests before advancing.** Run `cargo test` and confirm all tests pass before moving to the next layer. Do not skip.

---

## File Placement Rules (D-025 — Updated)

All runtime files are prefixed with the binary name. **`xgen-node_*` for all Node files, `xgen-client_*` for all client files.**

**Tier 1 — System files: mandatory co-location with binary, not configurable**

| File | Binary | Description |
|---|---|---|
| `xgen-node_config.toml` | xgen-node | Node configuration (TOML) |
| `xgen-node_state.json` | xgen-node | Live status snapshot, written every 5s (D-026) |
| `xgen-node_identities.db` | xgen-node | Identity registry (SQLite) |
| `xgen-node_federation.json` | xgen-node | Federation registry (JSON-backed `FederationRegistry`) |
| `xgen-client_config.toml` | xgen-client | Client configuration (TOML) |
| `xgen-client_state.json` | xgen-client | Identity, known nodes, joined spaces |

**Tier 2 — User-configurable files: default to binary folder, redirectable via config**

| File | Config field | Description |
|---|---|---|
| `xgen-node_keypair.enc` | `keypair_path` | Ed25519 private key — may redirect to HSM or secure share |
| `xgen-client_keypair.enc` | `keypair_path` | Ed25519 private key — may redirect to OS keystore (Phase 2) |
| Log output | `log_path` | May route to system log aggregator |

No file moves silently. Every Tier 2 redirect is explicit in config.

---

## meta_atts Key Namespace Rules (Spec 3.1.3)

`meta_atts` keys use dot-separated namespaces:

- `xgen.*` — **reserved** for protocol use only. Examples: `xgen.client`, `xgen.thread_id`, `xgen.tags`
- Third-party keys MUST use reverse-domain prefix. Examples: `com.example.priority`, `org.myapp.color`
- All lowercase, snake_case segments, dots as separators, no hyphens
- Max key length: 128 characters
- Values are strings. Structured values are JSON-encoded strings, not nested objects.

---

## Error Code Convention

Error codes are plain integers on the wire and in exit codes (e.g. `4002`). For human-readable display in logs, UI, and documentation, codes are shown with an `E` prefix and zero-padded to 6 digits (e.g. `E004002`). The `E` prefix is display-only — never transmitted, never used programmatically. `E004002` and `4002` are the same error.

Domain ranges: 1000–1999 transport, 2000–2999 federation, 3000–3999 identity, 4000–4999 state resolution, 5000–5999 E2E encryption, 6000–6999 migration, 7000–7999 bootstrap, 8000–8999 reputation, 9000–9999 DM promotion. Future domains extend naturally: domain 10 = 10000–10999, etc.

---

## Transport Pluggability (Spec 3.3.1)

WebSocket over TLS is the mandatory production transport. The protocol also explicitly permits Tor hidden services, I2P, and pluggable transport proxies as alternative stream transports — no protocol changes required. Phase 1 uses `ws://` localhost only. Production uses `wss://`. DPI resistance is a Phase 3 area; no Phase 1 impact.

---

## Key Cryptographic Decisions

- **Keypair encryption at rest:** ChaCha20-Poly1305 + Argon2id KDF. Phase 1 local node uses empty passphrase (file still encrypted for integrity).
- **Event ID derivation:** SHA-256 hash of canonical JSON → `xgen://hash/sha256:<hex>`
- **Signature format:** `ed25519:<base64url-pubkey>:<base64url-sig>` — covers canonical form only, not wire bytes
- **Canonical form:** fixed field order, no whitespace, object keys sorted lexicographically, `event_id` and `signature` excluded
- **DAG root types:** `state.space_create`, `state.room_create`, `state.dm_space_create` require empty `prev_events`. All others require at least one.
- **Cycle detection:** reduces to self-reference check only at insertion time (append-only store invariant)
- **prev_events fanin limit:** 10 (Phase 1)
- **Node announcement TTL:** 90 days
- **Session ID derivation:** `hash_uri(sort([node_a_id, node_b_id]) + timestamp)` — sorted so both sides derive same value

---

## Versioning Scheme

`[state].[layer].[session]` — three components, stored in `Cargo.toml`.

- `state`: 0 while building Phases 1 and 2; 1 when Phase 1 and Phase 2 complete and stable
- `layer`: implementation layer number (1–10)
- `session`: work session in which that layer was completed

Tags are monotonically increasing: `v0.1.1` → `v0.2.2` → … → `v0.10.x`

---

## Phase 2 — Status

Phase 2 shipped in two tracks. Both reached their Phase-2 deliverables; deeper work in each track has been scheduled as separate milestones.

### Track 1 — UI infrastructure (Phase-2 deliverables shipped; visual merge POSTPONED)

The Tauri scaffolding, lifecycle state machines, named pipe IPC, `--instance` segregation, `--batch` flag, and `xgen-core` crate split all landed during Sessions 14–18. Both binaries open windows with custom chrome; lifecycle states from Appendix E are wired; Node systray works with state-coloured icons; first-run SETUP is functional; `--service` headless mode works on both binaries.

The **visual merge of design Claude's chat mockups onto Miss Design's semantic skeleton** is POSTPONED at the element-modelling step (J-033). The gating condition has not been met; see the `⏸ POSTPONED — UI Phase 2 prep (run 1.5)` section earlier in this file for the full status.

### Track 2 — Protocol (Phase-2 deliverables shipped; Phase 3 areas open)

All Phase-2 protocol layers (11–19) shipped. `smoke-ph2` runs 60/60 PASS. `stress-complete` runs 6/6 PASS. xgen-core crate split landed at J-045; dual-licence boundary in place.

Post-Phase-2 protocol work shipped: AI Identity + per-Space pacing + temperature property (D-059/D-060/D-061, J-065); M1–M5 series.

**Phase 3 areas — specced but unimplemented:**

| Area | Status | Reference |
|---|---|---|
| State migration depth | Wire shape specced (3.12, Layer 14); deep testing pending | Future milestone (folded into M8) |
| Federation depth | Foundational gap closes in Federation Event Propagation milestone; deeper work (N-Node topologies, defederation flow, reputation merge) folded into M8 | Federation Event Propagation milestone (PENDING) + M8 |
| MLS operationalisation | Wire shape specced (3.10, Appendix I Part X.6); openmls integration pending | Future milestone (D3, parallel workstream alongside M-series) |
| `self` account | Local-only synthetic Identity, accessible from any client | D-021 — deferred |
| Registry file encryption | Identity and federation registries at rest | Deferred |
| Slovak translation pass | Single pass after full document completion | Deferred |
| DPI resistance | Investigation only | D-023 — Phase 3 |

**Roadmap:** M5 ✅ → CLI Audit ✅ → J-080 ✅ → M6 Phase 0 Pass 3 ✅ → Propagation Reliability Audit ✅ → Federation design (Pass 2 + Pass 3) ✅ → **Federation implementation Phases 1-5 ✅, Phase 6 (F-10 HeldPending generalisation) next** → ~~M6 multiparty~~ DEPRECATED → M6 (new) → M7 → M8 → M9. D3 (MLS) parallel.

---

## Repository Layout

```
docs/
  xgen_ch0_content.md             # table of contents
  xgen_ch1_philosophy.md          # philosophy, motivation
  xgen_ch2_architecture.md        # architecture, primitives, deployment model
  xgen_ch3_specification.md       # AUTHORITATIVE SPEC (§3.1–3.16 complete)
  xgen_ch4_implementation.md      # Phase 1 complete; Phase 2 scope defined
  xgen_ch5_protocol.md            # stub
  xgen_ch6_client_design.md       # UI architecture
  xgen_appendix_*.md              # supporting appendices
  xgen_federation_propagation_design.md      # Canonical Federation Event Propagation design (v1.0, ACTIVE — Pass 3 consolidated)
  xgen_propagation_reliability.md            # J-081 audit canonical doc
  xgen_node_admin_ops_design.md              # M6 Phase 0 canonical design doc
  ROADMAP.md                                  # Coarse-grained project navigation map
tasks/
  FEDERATION_PROPAGATION_DESIGN.md           # Pass 2 task file (COMPLETED at Pass 3 close)
  FEDERATION_PROPAGATION_PASS_3.md           # Pass 3 task file (COMPLETED at session close)
  FEDERATION_PROPAGATION_COMPLETION.md       # Implementation runbook for Clair (Status ACTIVE)
  ... (other task files for past milestones)
ui/
  ... (UI skeletons, postponed work)
IMPLEMENTATION_GUIDE_ph1.md       # Phase 1 layer-by-layer guide — COMPLETED
IMPLEMENTATION_GUIDE_ph2.md       # Phase 2 layer-by-layer guide
DECISIONS.md                      # Implementation decision log (D-000 through D-069)
JOURNAL.md                        # Contemporaneous development journal (IP record)
CLAUDE.md                         # This file
LICENSE                           # BSL 1.1
```

Source crates:
```
xgen-common/    # shared types (no runtime, no I/O) — BSL 1.1
xgen-core/      # all protocol logic — GPL-2.0-or-later (created in Phase 2 crate split)
xgen-node/      # thin Node shell — main.rs + lifecycle, depends on xgen-core — BSL 1.1
xgen-client/    # thin client shell — main.rs + commands, depends on xgen-core — BSL 1.1
```

Build target directory is kept outside the project folder to avoid file locking:
```
C:/cargo-targets/XGenProtocol
```

---

## Document Header Convention

### Core pattern

```
# Title
> **Status**: {}  
> Version: {}  
> Date: {MMM YYYY}  
> **Last updated**: YYYY-MM-DD  
> Language: {}  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  
```

### Specification

- Every `> ...` line requires **two trailing spaces before EOL** (mandatory for correct line rendering)
- `{MMM YYYY}` = month-name + year, e.g. `May 2026`
- **This header MUST be updated on every file edit**

Status values:
- `ACTIVE` — current, act on it
- `PENDING` — written, not yet the current task
- `COMPLETED` — done, do not re-execute
- `DEPRECATED` — no longer valid / replaced — replacement named if applicable
- `ARCHIVED` — frozen historical record, do not modify

**When looking for the next task**, scan `tasks/` and `docs/tests/` file headers. The next instruction file to run is the first one with `PENDING` or `ACTIVE` status that is not explicitly deferred.

**Note on folder convention:** New instruction files for Code Claude are written to `tasks/` at the project root (not under `docs/`). The `docs/tests/` folder holds the legacy instruction files written before this convention; it stays in place until a future cleanup migrates everything to `tasks/`. Both folders are scanned for `PENDING`/`ACTIVE` files.

---

## License Header

Every source file MUST carry this exact header:

```rust
// Copyright (c) 2026 Jozef Nižnanský / Alchemy Dump
// SPDX-License-Identifier: BUSL-1.1
// Licensed under the Business Source License 1.1
// Change Date: upon handover to XGen Protocol Foundation, or 4 years from release
// Change License: GPL-2.0-or-later
// See LICENSE in the project root for full terms.
```

Not PolyForm. Not MIT. Not any other license. BSL 1.1 exactly as above.

---

## Build Commands

```sh
cargo build                              # debug build
cargo build --release                    # release build
cargo test                               # run all tests
cargo test smoke                         # run smoke test only
cargo test --package xgen-common         # test one crate
```

Build output goes to `C:/cargo-targets/XGenProtocol` (set via `CARGO_TARGET_DIR` in `build.sh`). Binaries are copied to `bin/` in the project folder by `build.sh`.

---

*Read `DECISIONS.md` (current range D-000 through D-075) before making any decision that isn't explicitly covered by the spec. If you're unsure whether something needs a DECISIONS.md entry, it does.*
