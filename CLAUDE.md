# XGen Protocol — Claude Code Briefing
> For: Claude Code (claude.ai/code)  
> Date: May 2026  
> **Status:** ACTIVE  
> **Last updated:** 2026-05-29  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  

---

## 🟢 PLAY — M6 (new) Block 4: terminology fully resolved (D-082 + J-150 audit/sweep); next-active = verb-by-verb walks, starting A6 Logging/audit

**M6 (new) Node admin write path is the selected milestone; Block 4 (verb-by-verb walks, Chat Claude + Joe) is the active track.** Before drafting any verbs, a terminology collision surfaced and was resolved as **D-082** (recorded at J-149, this commit): "operator" was overloaded — the admin-ops design used it 10× in the early "Node owner/admin" sense, colliding with the locked **AI-operator role** (D-059/D-064). D-082's four locks: (1) **"operator" reserved globally for the AI-operator role** (moderator-parallel: operator : AI-identities :: moderator : room + members; fall-upward per D-064; never an owner/admin alias); (2) **Node administrator = distinct infra principal**, register split "administrator" in prose / "admin" in code+CLI+error-codes+config (matches `admin_ops`/`AdminContext`/`AdminError`); v1 OS-user-equals-administrator, session-scoped, no gradation; (3) **owner/super-admin reserved** future sub-tier (M7), not split in v1; (4) **Node administrator has automatic Space-admin authority over Spaces it originates/homes, NOT replicated/federated-in Spaces** (hosts-but-doesn't-own, Ch2) — signing identity for admin-originated Space events deferred to the A4 sub-design.

**Sequence:**
1. ✅ D-082 recorded (J-149).
2. ✅ Step-2 corpus audit (J-150) — found "operator" carries **four senses**; only **Sense D** (the `--batch` admin principal) is the collision. D-082 amended with the four-sense scope map + the inline facet-specifier technique (Joe's addition) for Sense C.
3. ✅ Step-3 Sense-D sweep (J-150) — `xgen_node_admin_ops_design.md` (10 hits → administrator/admin) + `xgen_aicontrol_implementation.md` category mirrors. Senses A (AI-operator role) / B (wire field names `operator_display_name` etc.) / C (infrastructure "Node operator" / data controller) left in place.
4. **▶ Block 4 verb-by-verb walks (NEXT-ACTIVE = A2):** **A6 ✅** · **A5 ✅** · **A3 ✅** · **A1 Federation ✅** (§6.A1, 7 verbs — `signal-defederation` deferred A1-D3; node-level vs Client `federate`; paginated list). Remaining phase order: **A2 Auth** (next — carries the §3.6 revocation-cascade deferral question) → A4 Space/Room (force-eject → `EventAccepted` + signing-identity blocker) → A7 Plugin. One category = one Joe-lock checkpoint; fills §6 of `docs/xgen_node_admin_ops_design.md` (9-field template per verb). JOURNAL/ROADMAP nav consolidate at Block 4 close.

**Gate (implementation only, not Block 4):** M6 (new) implementation is separately gated on the Propagation Reliability Audit (§5.3, Clair-owned). Block 4 is design-only and proceeds now. Track 2 (Clair): stood down until Block 4 closes and implementation opens.

**Entry point for next session: this CLAUDE.md PLAY block + JOURNAL J-150 entry first per Rule 0**, then Block 4 A2 (Auth Module management).

---

## ⚫ (historical, superseded by M6-Block-4-active state above) PLAY — XGID Retrofit arc COMPLETE at J-148 (all five passes closed); next-active = standby for next-milestone selection (M6 (new) Node admin write path ready)

**XGID Retrofit Pass 5 milestone CLOSED at J-148 (2026-05-29, this commit) = the entire five-pass XGID Retrofit arc CLOSED** (Pass 1 J-122 → Pass 2 J-126 → Pass 3 J-138 → Pass 4 J-146 → Pass 5 J-148). Pass 5 was a confirm-clean audit pass: **Audit 1** (trace-field formatter, ~60 `tracing::` sites) found ONE fix — F-1: `app.rs:2288` Debug-formatted `Option<&IdentityXgid>`, projected to `&str`; **Audit 2** (Debug/Display impls on xgen-client public types) was clean. Two commits: **Commit A** (the one-line F-1 fix; verified `cargo build --workspace --all-targets` 0 errors, 637 lib GREEN, clippy clean) + **Commit B** (this milestone-close atomic). **D-081 promoted** — "XGID typing is wire-format and persistence-format invariant" (the principle promised at arc close; numbered **D-081** not D-080 — D-080 was already taken by the Node-storage EventStore decision, collision caught at authoring). **Layered-B3 null — fifth Pass-arc no-finding instance** (J-122 + J-126 + J-138 + J-146 + J-148). **Five-file milestone-close atomic per D-074 forty-fifth instance + sixteenth milestone-close**: DECISIONS.md (D-081) + this CLAUDE PLAY flip + JOURNAL J-148 + ROADMAP + `tasks/XGID_RETROFIT_PASS_5_IMPL.md` ACTIVE → COMPLETED v1.1. §7.10 discipline-doc consolidation SKIPPED per Joe-lock.

**Arc retrospective:** zero serialized bytes changed across all five passes (D-081); Path A opened at Pass 1, closed at Pass 4; "honest longer work" recurrences by pass = 1 / 0 / 2 / 4 / 1. D-073 "field name carries the role, type carries the contract" is now fully realised in code across all four crates; the XGID Adoption Q3 transitional clause retires.

**Next-active: Joe selects the next milestone.** M6 (new) Node admin write path is ready (`docs/xgen_node_admin_ops_design.md` §6; Block 4 verb-by-verb walks are the parallel-eligible Chat-Claude work). Track 2 (Clair): stood down until selection.

**Entry point for next session: this CLAUDE.md PLAY block + JOURNAL J-148 entry first per Rule 0**, then Joe's milestone selection.

---

## ⚫ (historical, superseded by XGID-Retrofit-arc-COMPLETE state above) PLAY — XGID Retrofit Pass 5 scope-amendment Track-1 atomic SHIPPED at J-147; next-active = standby for next-milestone selection (Pass 5 implementation + M6 (new) Node admin write path both ready)

**XGID Retrofit Pass 5 scope-amendment Track-1 atomic SHIPPED at J-147 (2026-05-29, this commit).** Post-Pass-4-close canonical-record amendment. Pass 4 over-delivered (J-145/J-146 — `cargo build --workspace --all-targets` restored 0 errors, Path A closed **at Pass 4**; xgen-client/tests fixture set closed at 0 errors), so Pass 5's deferred set is reduced **4 → 2** (trace-field formatter audit + Debug/Display impl audit on xgen-client public types) and the Path-A-closed-at-Pass-4 reframing is recorded canonically. **Five-file atomic per D-074 forty-fourth instance** (not a milestone-close — milestone-close tally stays fifteenth): design doc v1.6 → v1.7 (§2.9 amendment banner + item 1 SATISFIED-AT-PASS-4) + runbook v1.4 → v1.5 (§1.2 closure note + §11.3 build-broken bullet SUPERSEDED + §11.4 unblock 4 → 2) + JOURNAL J-147 + this PLAY flip + ROADMAP v1.50 → v1.51 (Present live entry refreshed + line-551 🟢→⬛ icon correction + Past entry). DECISIONS.md NOT amended.

**Pass 5's amended scope = 2 items**, both projection-discipline audits independent of compilation: trace-field formatter audit across xgen-client `tracing::` invocations + Debug/Display impl audit on xgen-client public types.

**Next-active: Joe selects the next-active milestone.** Pass 5 implementation (runbook authoring first) + M6 (new) Node admin write path (`docs/xgen_node_admin_ops_design.md` §6) are both ready; sequencing is Joe's call. **Track 2 (Clair): stood down** until selection.

**Entry point for next session: this CLAUDE.md PLAY block + JOURNAL J-147 entry first per Rule 0**, then Joe's milestone selection.

---

## ⚫ (historical, superseded by Pass-5-scope-amendment-SHIPPED state above) PLAY — XGID Retrofit Pass 4 milestone CLOSED at J-146; next-active = Pass 5 scope-amendment Track-1 atomic, then standby for next-milestone selection (Pass 5 + M6 (new) both ready)

**XGID Retrofit Pass 4 milestone CLOSED at J-146 (2026-05-29, this commit).** Two Clair-facing commits on `main`: **Commit 1 `3869d4c`** (J-145 — consolidated fifteen-file xgen-client retype atomic: all seven surfaces + xgen-common §4.1.b additive-API + Surface #8 docs + T1–T15; 8/8 GREEN; checkpoints #2 + #3 closed) + this **Commit 2** milestone close. Commit 1a did NOT fire (checkpoint #3 = 0 integration-test fixture errors → absorbed; unprecedented vs Pass 1/2/3's 296/93/638). **Test count: 637 lib** (61 client + 35 common + 453 core + 88 node) + integration GREEN. **Layered-B3: null** — four-instance Pass-arc no-finding chain (Pass 1 J-122 + Pass 2 J-126 + Pass 3 J-138 + Pass 4 J-146). **`cargo build --workspace --all-targets` RESTORED (0 errors) — Path A CLOSED at Pass 4**, ahead of the §2.9/§11.3 Pass-5 expectation (honest finding per D-065). **"Honest longer work" final count: FOUR** (J-142 count drift + J-143 commit-shape + J-144 classification drift + J-145 workspace-build-restored handling — all prospective catches at the canonical-record/verification layer). **Five-file milestone-close atomic per D-074 forty-third instance + fifteenth milestone-close**: runbook Status → COMPLETED v1.4 + DoD verified + design doc v1.5 → v1.6 (§6.1 J-NNN frozen to J-146) + JOURNAL J-146 + this PLAY flip + ROADMAP. DECISIONS.md NOT amended. Implementation arc: J-141 runbook → J-142 → J-143 → J-144 → J-145 Commit 1 → J-146 close.

**Next-active (Chat Claude + Joe): Pass 5 scope-amendment Track-1 atomic** — amends design doc §2.9 + runbook §11.3 from 4 deferred items → **2** (trace-field formatter audit + Debug/Display impl audit on xgen-client public types; the other two — workspace-build restoration + xgen-client/tests fixture sweep — were satisfied by Pass 4 itself), absorbs the §1.2 runbook future-hygiene, and records the Path-A-closed-at-Pass-4 reframing canonically. After that atomic, Joe selects the next-active milestone (Pass 5 implementation + M6 (new) Node admin write path both ready; sequencing is Joe's call). **Track 2 (Clair): stood down** until selection.

**Entry point for next session: this CLAUDE.md PLAY block + JOURNAL J-146 entry first per Rule 0**, then the Pass 5 scope-amendment atomic (design doc §2.9 + runbook §11.3 + §1.2).

---

## ⚫ (historical, superseded by Pass-4-CLOSED state above) PLAY — Pass 4 Commit 1 SHIPPED at J-145; classification locked at J-144

**XGID Retrofit Pass 4 Commit 1 SHIPPED at J-145 (2026-05-29) — consolidated xgen-client retype atomic.** All seven xgen-client surfaces retyped per design doc §4.1.a + §4.3 + §4.6; xgen-common §4.1.b additive-API (`is_empty` + T3) included; Surface #8 doc fragments shipped atomic with the code; T1–T15 in place. 8/8 GREEN, 637 tests; both clippy gates clean. Checkpoint #2 CLOSED (T2 wire-invariance witness). Checkpoint #3 CLOSED (0 fixture errors → absorb, no Commit 1a). `cargo build --workspace` RESTORED at Pass 4. "Honest longer work" THREE → FOUR. Fifteen-file atomic per D-074 forty-second instance.

**Surfaces #3/#5/#6/#7 classification LOCKED at J-144 (2026-05-29) — checkpoint-#1-equivalent, before the affected retypes.**

**Surfaces #3/#5/#6/#7 classification LOCKED at J-144 (2026-05-29) — checkpoint-#1-equivalent, before the affected retypes.** Resuming the in-flight Commit 1 (Surface #1 ops.rs 49-slot retype + mechanical projections + xgen-common §4.1.b additive-API already landed; `cargo build -p xgen-client --lib` clean), the remaining *declaration* retypes are Surfaces #3/#5/#6/#7 — which had **no Joe-approved slot table** at design close (only §2.x Initial Q-anchors; checkpoint #1 at J-142 verbatim-approved only §4.1.a/§4.3.0/§4.5.0). Per Lock 1 Trigger (a) + D-078, Clair grep-enumerated the actual slots and surfaced them; the grep found **three drift findings** (`SessionState.home_node` is a `ws://` URL not NodeXgid → stays String; `lifecycle::ClientStateEvent` has zero identifier slots; `HealthState.operator_known` is numeric) + **three classification calls**, now locked at design doc **§4.6**: Q2 `subject_id` stays String (D-061 sentinel union); Q3 `get_dag_tips(space_id)` keeps `&str`+`Borrow<str>` (Surface #2 Option α sibling); Q4 `EventContext.ai_identity_id → &IdentityXgid` (§3). Plus: ops.rs `home_node` keeps `NodeXgid` (no Surface #1 rework — it wraps a `ws://` URL via projection; §4.1.a.iii reasoning corrected; Pass-5 future-hygiene flag). **Pass 4 "Honest longer work over fast shortcuts" count: TWO → THREE** — third prospective catch (after J-142 count drift + J-143 commit-shape), all before the affected production code. **Six-file amendment atomic per D-074 forty-first instance:** design doc v1.4 → v1.5 (new §4.6 + §4.1.a.iii correction + §2.x resolution pointers) + runbook v1.2 → v1.3 (§5/§7/§8/§9 Q-items + T6/T10/T12/T15 reframes + §2.3 checkpoint note + §14.5/§14.6) + JOURNAL J-144 + this PLAY flip + ROADMAP Past entry + HANDOFF §3.1 updated. **Net Surfaces #3/#5/#6/#7 retype scope:** #3 = 0 decl (param stays `&str`); #5 = 1 (`ClientIdentity.identity_id`); #6 = 2 (`AiPacingTracker` key + `EventContext.ai_identity_id`); #7 = 7. Next-active: resume Commit 1 against the §4.6 locks (decl retypes → in-tree test fixes → T1–T15 → Surface #8 doc fragments → lib-clean + 8-GREEN per runbook §11.3 → checkpoint #2 + #3). The detail below reflects the J-143 state.

**Commit shape RE-LOCKED at J-143 (2026-05-29) — per-surface 8-9 commits → ONE consolidated xgen-client retype atomic (Pass-3 shape).** At Commit 1 prep, `cargo build -p xgen-client --lib` reported **191 errors** (xgen-client is in the inherited Path A broken state — every surface consumes retyped Pass 1–3 upstream types). The J-141 per-surface commit sequence is **infeasible**: all seven surfaces share one crate (`xgen_client_lib`) that compiles as a unit, and Surfaces #2/#4/#6/#7 errors are independent of ops.rs, so no per-surface commit can leave the lib compiling and T1–T15 can't run until it does. Pass 3 avoided this by shipping all seven xgen-node surfaces in one atomic (Commit 2 `67fb48d`); **Joe re-locked the same shape for Pass 4**: Commit 1 = all seven surfaces + xgen-common §4.1.b additive-API + all Surface #8 doc fragments + T1–T15 (lib-clean + 8-GREEN verified there); Commit 1a = contingent test-fixture sweep; Commit 2 = milestone close (≈3 commits, matching Pass 3's 4). Five-file amendment atomic per D-074 fortieth instance: runbook v1.1 → v1.2 (§2.1 re-lock + §14.4) + design doc v1.3 → v1.4 (§4.4.c SUPERSEDED-note) + JOURNAL J-143 + this PLAY flip + ROADMAP Past entry. The xgen-common §4.1.b additive-API (`is_empty` + T3) is already written and GREEN; it ships inside Commit 1. **Pass 4 "Honest longer work over fast shortcuts" count: ONE → TWO** — second prospective catch (after J-142's count drift), both before any production code. Next-active: Commit 1 consolidated retype atomic. The detail below reflects the J-142 count-correction state (also pre-code).

**Joe-lock checkpoint #1 fired at J-142 (2026-05-29) — §4.1.a slot-count drift caught + corrected BEFORE any production code.** Clair's D-078 production-grounded verification (`grep -cE '^\s+pub [a-z_]+: (String|Option<String>)' xgen-client/src/ops.rs`) found **49** String slots, not the **46** recorded at design close v1.2. Classification *substance* was 100% correct (every field → correct flavour); only the arithmetic drifted (`identity_id` ×4→×3; §4.1.a.i 31→33; §4.1.a.ii 12→11; §4.1.a.iii rows-vs-slots 3→5; total 46→49 = 37 retype + 12 stay). §4.3.0 (16 clap Args) + §4.5.0 (7 async-spawn @ desktop.rs L54/63/90 + ai_service.rs L554/575 + service.rs L183/202) verified CLEAN. Joe locked **"amend design doc to 49, then code"** — Track 1 canonical-record amendment (sibling-shape J-129 / J-133 / J-134): design doc v1.2 → v1.3 (chain stripped per strict `Last updated`) + runbook v1.0 → v1.1 (T1 renamed `..._46_slots_compile` → `..._49_slots_compile`) + JOURNAL J-142 + this PLAY flip + ROADMAP Past entry = **five-file atomic per D-074 thirty-ninth instance**. **Pass 4 "Honest longer work over fast shortcuts" count: 0 → ONE** — prospective catch at the design-doc-grounded verification layer; D-078 working exactly as designed, before a single production line or the misnamed `..._46_slots` test could ship. Checkpoint #1 now closes affirmatively; Commit 1 proceeds against the corrected 49-slot table. **The detail below reflects the J-141 runbook-ship state (runbook now amended to v1.1; design doc to v1.3).**

**XGID Retrofit Pass 4 implementation runbook SHIPPED at J-141 (2026-05-28).** Runbook landed at `tasks/XGID_RETROFIT_PASS_4_IMPL.md` Status: ACTIVE v1.0 (~66 KB / 668 lines, fourteen sections — §1 framing + §2 sequence overview with three Joe-lock checkpoints + §3-§9 per-surface Commits 1-7 (one per xgen-client surface) + §10 Commit 7a [CONTINGENT] + §11 Commit 8 milestone close + §12 discipline notes six sub-sections + §13 cross-references + §14 footer). Sibling-in-shape to `tasks/XGID_RETROFIT_PASS_3_IMPL.md` v1.6 COMPLETED with structural extensions for Pass 4's per-surface-commit Option γ hybrid-split commit-sequence per design doc §4.4 lock at J-140.

**Joe-lock at runbook-authoring J-141: Option B locked-by-recommendation** for §2.1 commit-sequence shape — honest §4.4.4 application (no Commit 1 doc-pass; per-surface code+doc atomic × 7 + Commit 7a CONTINGENT + Commit 8 close = **8-9 commits expected total**) over Option A (Pass-3-shape-mechanical retain zero-content Commit 1 doc-pass). The J-141 runbook-shipping commit IS the kickoff atomic per §4.4.b cross-surface fragments framing (ROADMAP + CLAUDE PLAY + JOURNAL bumps consolidate at milestone close, not as separate Commit 1).

**Per-surface commit sequence at §2.1**: Commit 1 Surface #1 M5 Ops Layer (ops.rs Result-struct retype + Pass 1 additive-API extension at xgen-common + Appendix F fragments + T1-T3 tests) → Commit 2 Surface #2 CLI Dispatcher (app.rs 16 clap Args projection + format paths + Appendix F CLI section + T4-T5) → Commit 3 Surface #3 Batch Pipe Dispatch (batch.rs get_dag_tips + Appendix F batch reply schema + T6-T7) → Commit 4 Surface #4 Tauri Shell (desktop.rs Tauri command return types + lifecycle + T8-T9) → Commit 5 Surface #5 Session State (session.rs + lifecycle.rs + T10-T11) → Commit 6 Surface #6 AI Resident (ai_service.rs + ai_behavior.rs + xgen_aicontrol_implementation.md fragments + T12-T13) → Commit 7 Surface #7 Pacing + Temperature (pacing.rs + temperature.rs + Ch6 §6.15 fragments + T14-T15) → Commit 7a [CONTINGENT] test-fixture sweep → Commit 8 milestone close.

**Per-surface test target +15** (T1-T15 by name at runbook §3.4 + §4.3 + §5.3 + §6.3 + §7.3 + §8.3 + §9.3). 14% more surfaces than Pass 3 (8 vs 7) + per-surface format-boundary witnesses lift count from Pass 3's T1-T11 to Pass 4's T1-T15.

**Three Joe-lock checkpoints at §2.3** (remapped from Pass 3 because Pass 4 has no Commit 1 doc-pass): **#1 pre-Commit-1 verbatim classification-table approval** — Clair extracts design doc §4.1.a 49-slot classification (corrected from "46" at J-142) + §4.3.0 16 clap Args slots + §4.5.0 7 async-spawn sites verbatim; Joe approves before any production code lands; **LOAD-BEARING D-078 application surface for Pass 4 — fired + closed affirmatively at J-142**. **#2 post-Commit-1 first-surface drift check + wire-format invariance witness verification** (T2 `ops_result_struct_serde_transparent_wire_invariance` passes — pre-Pass-4 batch consumer reads byte-identical JSON from post-Pass-4 Result types). **#3 post-Commit-7 split-trigger decision per ~50-error threshold** (sibling-shape Pass 2 + Pass 3 pre-locked contingent-split posture durable cross-Pass discipline).

**Two split triggers at §2.2**: Trigger (a) non-existent production contract per design doc §4.1.a + §4.3.0 + §4.5.0 verbatim tables (D-078 applies; Pass 3 J-129 + J-133 sibling-shape); Trigger (b) family-boundary size split if any Commit 1-7 exceeds ~600 lines diff.

**§12 discipline notes six sub-sections** (lighter than Pass 3's nine because Pass 4 inherits more cross-Pass discipline carry-overs from JOURNAL J-138 Sub-section 2 without re-derivation): precedent-departure self-defense / Option B Joe-locked-by-recommendation / Pass 1 additive-API extension second-instance load-bearing carry-over / format-boundary preservation Option γ split (D-NNN-format-boundary OPEN) / D-NNN-ε CLOSED by honest framing / layered-B3 expected null per four-instance Pass-arc no-finding chain durability.

**Strict `Last updated` discipline applied per Joe instruction at J-141 turn**: runbook + ROADMAP + CLAUDE.md + JOURNAL.md headers carry ONLY `> **Last updated**: 2026-05-28` (no parenthetical chain). Memory entry saved (`feedback_last_updated_strict`) for future sessions.

**Pass 4 "Honest longer work over fast shortcuts" count: TWO** — J-142 (§4.1.a count drift) + J-143 (per-surface-commit infeasibility re-lock), both prospective catches at Commit-1 prep before any production code. J-139 + J-140 + J-141 were within-milestone close-events, not recurrences. Sibling-shape to Pass 3's two recurrences (J-129 + J-134).

**D-074 application count: thirty-eighth instance** + Lock #3 per-commit cadence; not milestone-close so milestone-close tally — fourteenth at J-138 — does NOT increment. **Four-file atomic at this v1.0 runbook ship**: runbook NEW v1.0 ACTIVE + ROADMAP v1.45 → v1.46 (visual tree Pass 4 design row ✅ + new runbook ✅ sub-bullet + Surface #1 🟢 next-active + Past entry) + this CLAUDE.md PLAY block flip + JOURNAL J-141 body entry. DECISIONS.md NOT amended.

**Honest data point recorded per D-065**: ROADMAP Present section drift from J-138/J-139/J-140 noted at J-140 Sub-section 12 as "candidate fix-up at next session" is NOT addressed in this commit per minimal-change discipline — J-141 scope is runbook-authoring atomic, not Present drift-fix. Candidate fix-up remains open for a separate atomic.

**Track 1 (Chat Claude + Joe): standby** until Clair's Commit 1 closes affirmatively at Joe-lock checkpoint #2. Parallel-eligible items unchanged (M6 (new) Block 4 verb-by-verb walks at `docs/xgen_node_admin_ops_design.md` §6 ~35 verbs across 7 categories) if Joe selects parallel-track work.

**Track 2 (Clair): pickup at `tasks/XGID_RETROFIT_PASS_4_IMPL.md` §3 Commit 1** (Surface #1 M5 Ops Layer code+doc atomic). Read CLAUDE.md PLAY block + JOURNAL J-141 entry first per Rule 0, then runbook §1-§3 in order, then design doc `tasks/XGID_RETROFIT_PASS_4_DESIGN.md` v1.2 §4.1.a + §4.3.0 + §4.5.0 classification tables verbatim (Joe-lock checkpoint #1 requires verbatim classification-table approval before any production code touches).

**Entry point for next session: this CLAUDE.md PLAY block + JOURNAL J-141 entry first per Rule 0**, then `tasks/XGID_RETROFIT_PASS_4_IMPL.md` §1-§3, then design doc §4.1.a + §4.3.0 + §4.5.0 classification tables.

---

## ⚫ (historical, superseded by Pass-4-runbook-shipped state above) PLAY — XGID Retrofit Pass 4 design phase CLOSED at J-140; runbook authoring next-active for Chat Claude + Joe in fresh session

**XGID Retrofit Pass 4 design phase CLOSED at J-140 (2026-05-28, this commit).** Design doc `tasks/XGID_RETROFIT_PASS_4_DESIGN.md` Status: ACTIVE → COMPLETED v1.0 → v1.2 single-session full close (two-session split eligible per Pass 3 J-127 Sub-section 8 data point (c) but not exercised at Pass 4 per "let us move ahead" mid-session pivot from Option II pause to Option I continue — recorded honestly per D-065 at §6.2 + §7.5). §3 governing principle locked **inherited unchanged from Pass 2 §3 + Pass 3 §3** — **four-instance Pass-arc inheritance** now established (Pass 1 implicit at runbook; Pass 2 explicit at J-123; Pass 3 explicit at J-127; Pass 4 explicit at this J-140 lock). Sanity-check loop across seven xgen-client surfaces + Surface #8 doc-tree confirmed clean — no Pass-4-specific wrinkle surfaces at §3 layer; three wrinkle-candidates considered honestly per D-065 (Result-struct serde-transparent at Surface #1; async-spawn captures at Surface #4 + #6; Tauri IPC frontend string boundary) all deferred to §4 turf per Pass 3 §4.3 precedent (format-boundary is §4 application of §3 principle, not §3 amendment).

**All five §4 anchors LOCKED at this v1.1 walk:**

- **§4.1 Surface #1 M5 Ops Layer — Result-struct field retype + Pass 1 additive-API extension + serde-transparent wire-neutrality.** Composite of §4.1.0 honest recon corrections (recon claimed 16 pub Result struct types — actual 13 Result + 2 non-Result = 15; recon claimed ~45 String slots — actual 46; pre-guessed borderline candidates `since` / `tip_event_id` / `pubkey_uri` absent from ops.rs — discipline data point for Pass 5 + future Pass-arc recon expectations) + §4.1.a 46-slot classification (31 mechanical identifier retypes per §3 — `identity_id` ×4 + `space_id` ×9 + `event_id` ×7 + `room_id` ×4 + `target_identity` + `owner_identity_id` + `ai_identity_id` ×3 + `new_operator` + `owner_id` + `operator: Option<String>` + `ai_invited_by: Option<String>` + `sender` — to flavour wrappers; 12 mechanical descriptive stays per §3 — `display_name` ×3 + `version` + `name` ×2 + `role` + `registered_at` + `timestamp` + `text` + `ai_member_role: Option<String>`; 3 borderline locks — 2 `NodeXgid` retypes for `home_node` ×3 + `node` + 1 `String` stay for `source` operator-source enum-tag) + §4.1.b Pass 1 additive-API extension Option β locked (inherent `.is_empty()` on flavour wrappers + Option `.as_deref()` per Pass 1 Commit 4 precedent over per-site rewrite) + §4.1.c serde-transparent wire-neutrality confirmed at Surface #1 boundary.

- **§4.2 Format-boundary preservation extended to client-side serialisation surfaces — Option γ split locked; D-NNN-format-boundary promotion-watch STAYS OPEN.** Three Pass 4 candidate instances enumerated: A (Surface #1 stdout JSON) + B (Surface #3 named-pipe JSON) consolidate under Pass 3 wire-shape boundary class (no new count — sibling-shape to Pass 3 §4.3 v1.2 wire-OR-persistence consolidation precedent); C (Surface #4 Tauri IPC bridge) recognised as fresh boundary class at Pass 4 (Rust↔JS process-internal IPC over Tauri serde marshalling — distinct from byte-stream wire + filesystem persistence). Total at this lock: 3 structurally-distinct instances across 2 Pass-arc; D-077 multi-Pass-arc durability NOT yet met (Pass 4 boundary class is fresh-at-Pass-4). Promotion trigger: fourth structurally-distinct instance at Pass 5 OR cross-milestone (M6/M7 admin write path + possible future gRPC / WebRTC / HTTP API surfaces) closes durability gap and promotes to D-080.

- **§4.3 CLI arg parsing boundary — Option α locked (clap parse stays String; project at dispatcher arm).** Walk-time enumeration of clap-derive Args structs surfaced 16 identifier-shaped String slots at Surface #2 parse boundary across 8 Args structs (AiDelegateArgs + AiRevokeArgs + AiStatusArgs + CreateRoomArgs + InviteArgs + JoinArgs + SendArgs + HistoryArgs). Plus 5 descriptive stays + 4 transport/config Cli top-level stays per §3. Option α locked over Option β (`FromStr` on flavour wrappers — out of scope; would require Pass-4-scope agreement on "valid XGID string format at parse-time" substantive protocol-design surface) and Option γ (project at `ops::*` entry — breaks §3-vs-§4 layering by pushing wrap inward). Pass 4 explicitly does NOT add `FromStr` to flavour wrappers; validated `FromStr` is the rung above per D-079 honest-framing precedent, deferred per D-071 audit-design-impl-arc framing.

- **§4.4 Doc-vs-code commit-shape decision — Option γ hybrid split locked (per-surface atomic + consolidated milestone-close).** Doc-tree sweep Surface #8 ~1800 lines total (Appendix F 1193 + xgen_aicontrol_implementation.md 544 + Ch6 §6.15 ~60); per-surface-coupled doc fragments (Appendix F §F.0.6 ops section + xgen_aicontrol_implementation.md AI resident sections + Ch6 §6.15 pacing+temperature subsections) ship atomic with their code surface commit; cross-surface or content-shape doc fragments (high-level architecture intros + ROADMAP + CLAUDE PLAY + JOURNAL J-NNN entries) consolidated in milestone-close commit per D-074. **Runbook-shape pre-frame**: Pass 4 runbook authoring at next session-arc inherits the Option γ default; candidate shape is per-surface code+doc commit × 7 surfaces + Commit 2a test-fixture sweep contingent split (sibling-shape to Pass 3 §4a contingent-split posture pre-locked) + milestone-close commit → total 8-9 commits (heavier than Pass 3's 4 commits but lighter than the trilogy's ~12-commit pattern per Pass-internal-consistency framing).

- **§4.5 Async-spawned task captures sub-rule extension at Surface #4 + #6 — Option γ honest framing closure locked; D-NNN-ε promotion-watch CLOSED.** Walk-time grep enumerated Pass 4 xgen-client production async surfaces: 3 `#[tauri::command]` handlers at desktop.rs (Surface #4) + 4 `tokio::spawn` sites at ai_service.rs:554+575 + service.rs:183+202 (Surface #6) = 7 instances across 2 structurally-different new boundary classes. Combined with Pass 3's xgen-node 4 instances (federation_session + reconnect), total instance count is now 11 across 3+ structurally-different surfaces — D-077 surface-diversity threshold structurally met. **But J-138 Sub-section 8 honest framing pre-answered the substantive question per D-065**: ubiquity strengthens the "Rust idiom" framing (`'static` bound on `tokio::spawn`) rather than promoting to project decision. Pass 3 §4.2 v1.2 third row sibling-shape rule table extended at canonical design-doc layer to record Pass 4 instances; **promotion-watch closed by honest framing** per D-065 + D-079. D-NNN slot preserved for actual XGen-specific decisions (e.g. D-NNN-format-boundary fourth structurally-distinct instance per §4.2). **Cross-Pass discipline carry-over implication**: Pass 3 §4.5 J-127 + Pass 4 §4.5 establish **two-instance pattern of honest-framing-resolution of promotion-watches at Pass-arc design close** — second instance establishes that promotion-watch close-by-honest-framing is a valid discipline action alongside promotion-by-honest-framing (Pass 3 J-134's D-079 promotion atom).

**§5 layered-B3 expected answer LOCKED null at full eight-surface scope** — four-instance Pass-arc no-finding chain established (Pass 1 J-122 + Pass 2 J-126 + Pass 3 J-138 + Pass 4 J-140 all null); per-surface audit at design phase confirms no layered surfaces emerge across the eight-surface scope per Rule 5 + D-065 honest framing; runbook §6.5 verification at Clair's implementation-boundary audit re-runs the audit honestly per Rule 0 + Rule 5 honest-audit-not-honest-assumption discipline.

**§6 historical/future-pointer entries filled in Shape α pointer-style** per Pass 3 §6.1 precedent: §6.1 Pass 4 design phase historical record (J-139 design-open + J-140 design close pointer-style; implementation milestone-close J-NNN placeholder frozen at runbook close per J-108 codification) + §6.2 two-session walk shape Pass-internal precedent inheritance (eligible-but-not-exercised at Pass 4 per honest framing; bimodal Pass-arc precedent state — single-session at Pass 2 + Pass 4; two-session at Pass 3).

**§7 discipline notes consolidated five sub-sections**: §7.1 honest recon corrections per Rule 5 + D-065 (single-digit drift between recon estimates and walk-time actuals — discipline data point for Pass 5 + future Pass-arc recon expectations); §7.2 format-boundary preservation Option γ split as honest-framing-resolution shape (D-NNN-format-boundary promotion-watch held open by surface-diversity threshold not-yet-durably-met); §7.3 doc-vs-code commit-shape decision at design phase per D-069 + D-071 audit-precedes-dependent-design framing (runbook commit-sequence is downstream consequence of doc-tree coupling shape); §7.4 honest-framing-resolution of promotion-watches admits three shapes — promote (D-079 promotion atom precedent) / close-by-honest-framing (Pass 4 §4.5 D-NNN-ε) / hold-open-by-surface-diversity-threshold (Pass 4 §4.2 D-NNN-format-boundary); §7.5 two-session walk shape eligible-but-not-exercised at Pass 4 + revised discipline framing (bimodal Pass-arc precedent; future authors choose per same-session-capacity assessment without violating precedent). §7.6 candidate (Joe-locks-by-recommendation as inline-lock pattern fifth recurrence) skipped per minimal-broadening discipline (Pass 2 §7.2 already codifies).

**Pre-walk reconnaissance delivered via parallel Explore subagent** under "very thorough" search level + no-file-modification guard-rail (sibling-shape to Pass 3 Commit 2a parallel-subagent-sweep discipline data point at runbook §9.7 but at design-phase open rather than test-fixture sweep). Subagent structured report covered: xgen-client String identifier slots per file + Pass 4 forward-looking markers count + xgen-client build baseline per Path A + test infrastructure + doc surfaces sized + subsystem ownership map.

**Reconnaissance headline findings:**

- **192 xgen-client compilation errors per Path A** — three structural categories: (a) Result struct field type-mismatches at ops.rs (typed returns vs String-declared fields); (b) method availability on Xgid newtypes (`.is_empty()`, `.as_deref()` against newtypes); (c) HashMap key-value slot mismatches at pacing.rs + temperature.rs.
- **Zero `// Pass 4 widens` markers in workspace** — **three-instance sparsity chain at Pass-arc level now durable** (Pass 1 → 33 Pass 2 markers per J-125; Pass 2 → 1 Pass 3 marker per J-136 Sub-section 7 data point A; Pass 3 → 0 Pass 4 markers per this recon). N+1 not N+2 design discipline established at three instances per D-077/D-078 promotion-threshold framing. Pass 4 design + runbook authoring cannot rely on pre-walk marker scaffolding from Pass 3.
- **42 String identifier slots across 7 subsystems** — highest density: ops.rs (16 pub Result struct types + ~45 slots HIGHEST IMPACT) + app.rs (42 occurrences across CLI dispatch + integration). Lowest density: desktop.rs (0 direct slots; identifier material flows via Tauri command return types).
- **Doc surfaces sized**: Appendix F 1193 lines + xgen_aicontrol_implementation.md 544 lines + Ch6 §6.15 lines 1326-1388+. Confirms Pass 4 is heaviest doc-work pass per J-095 XGID Adoption v1 Phase 2 doc-tree sweep classification.

**Seven subsystem surfaces enumerated in dependency order** at design doc §2:

1. **Surface #1 M5 Ops Layer** (ops.rs, 1260 LOC) — 16 pub Result struct types + ~45 String slots; serde-transparent dispatcher boundary; HIGHEST IMPACT foundational position.
2. **Surface #2 CLI Dispatcher** (app.rs, 5255 LOC) — Cli arg parse + result format + integration; consumes Surface #1.
3. **Surface #3 Batch Pipe Dispatch** (batch.rs, 814 LOC) — D-043 Windows named-pipe IPC + get_dag_tips canonical impl; consumes Surface #1.
4. **Surface #4 Tauri Shell** (desktop.rs, 241 LOC) — lifecycle state machine + 3 Tauri commands; consumes Surface #1 + #5.
5. **Surface #5 Session State** (session.rs 172 LOC + lifecycle.rs) — per-invocation session cache + lifecycle events; foundational consumed by all four prior surfaces.
6. **Surface #6 AI Resident** (ai_service.rs 661 LOC + ai_behavior.rs 305 LOC) — M4 resident loop + AiBehavior trait + EchoPlugin; consumes Surface #1 + #5.
7. **Surface #7 Pacing + Temperature** (pacing.rs + temperature.rs) — D-060/D-061 per-(space, sender) HashMap + event payloads; sibling-shape to Pass 3 Surface #4 fanout HashMap-key retype.
8. **Surface #8 Doc-tree sweep** — Appendix F + xgen_aicontrol_implementation.md + Ch6 §6.15 per-section typed XGID slot callouts.

**Three precedent-positioning notes at §1.2:**

1. **Pass 4 is the doc-heavy Pass** per J-095 doc-tree sweep classification. Runbook should anticipate heaviest doc-work despite no new classification-table rows.
2. **Zero Pass 4 forward-looking markers** — three-instance sparsity chain durable.
3. **Path A inherited break state at 192 errors all xgen-client** — three-instance Path A discipline durability established at J-138; Pass 4 + Pass 5 close.

**Two D-NNN promotion-watches from J-138 Sub-section 8 advance at Pass 4:**

- **D-NNN-format-boundary** (format-boundary preservation wire OR persistence) — third-instance threshold opens at Pass 4 if client-side serialisation-format slot instantiates (Tauri IPC, AI control protocol over HTTP, gRPC). Surface #1 + Surface #4 + Surface #8 walks identify whether instantiation fires.
- **D-NNN-ε** (async-spawned task captures force owned parameters — Tokio idiom) — promotion-watch opens at Pass 4 if structurally-different fifth instance fires at xgen-client async surfaces (Tauri commands, AI service spawns, batch dispatcher workers). Surface #4 + Surface #6 walks identify whether instantiation fires.

Both flagged-not-promoted at this design open per D-069 honest framing (no instances at this atom; design phase walks identify candidates honestly).

**Pass 4 "Honest longer work over fast shortcuts" count: starts fresh at zero** (J-126 + J-138 close-event-not-recurrence-event framing inherited; design-open is within-milestone substantive event starting milestone scope fresh). Increments per recurrence honestly per D-065 going forward.

**Sibling-in-shape position.** Sibling-in-shape to Pass 2 J-123 design open + Pass 3 J-127 design open: both opened with §1 framing + §2 surface enumeration; both deferred §3-§7 to subsequent walks. Pass 4 lands in same shape with two structural differences: parallel-subagent reconnaissance vs inline grep (discipline data point worth recording); seven heterogeneous-subsystem surfaces vs Pass 3's seven module-family-concentrated surfaces.

**D-074 application count**: J-140 is the **thirty-seventh instance** + Lock #3 per-commit cadence. Not a milestone-close so milestone-close tally stays at fourteenth from J-138. Four-file atomic at this design close: design doc v1.0 → v1.2 + Status ACTIVE → COMPLETED + CLAUDE.md PLAY flip + ROADMAP v1.44 → v1.45 (visual tree row 🟢 → ✅ + Past entry) + JOURNAL J-140 body entry.

**Pass 4 "Honest longer work over fast shortcuts" count stays at zero at this design close** (J-139 design-open + this J-140 design close are within-milestone substantive events, not recurrence shape — sibling-shape to topo-sort J-098/J-099 within-milestone events). Pass 4 milestone scope started fresh at zero per close-event-not-recurrence-event framing inherited from J-126 + J-138; design-phase walks are within-milestone events.

**Track 1 (Chat Claude + Joe): Pass 4 implementation runbook authoring next-active** in fresh session per Pass 2 J-124 + Pass 3 J-128 design-then-runbook precedent. Runbook authoring at `tasks/XGID_RETROFIT_PASS_4_IMPL.md` inherits §4.4 Option γ hybrid-split commit-sequence pre-frame: per-surface code+doc atomic × 7 + Commit 2a test-fixture sweep contingent split (sibling-shape to Pass 3 §4a contingent-split posture pre-locked) + milestone-close commit → 8-9 commits expected total. Three Joe-lock checkpoints anticipated at runbook §2.3 per Pass 3 precedent: #1 post-Commit-1 doc-pass drift check (or first surface commit if Commit 1 collapses per §4.4 Option γ — runbook authoring locks); #2 pre-Commit-2 verbatim surface list (Chat Claude extracts the seven-surface field-classification tables from design doc §4.1.a + §4.3.0 + §4.5.0 verbatim; Joe approves each surface by name before any production code lands); #3 post-Commit-2 split-trigger decision per ~50-error threshold heuristic (Pass-arc precedent durable: Pass 2 fired at 93; Pass 3 fired at 638).

**Track 2 (Clair): stood down** until Pass 4 runbook authoring closes. Parallel-eligible items unchanged: M6 (new) Block 4 verb-by-verb walks (~35 verbs across 7 categories at `docs/xgen_node_admin_ops_design.md` §6).

**Entry point for next session: this CLAUDE.md PLAY block + JOURNAL J-140 entry first per Rule 0**, then `tasks/XGID_RETROFIT_PASS_4_DESIGN.md` v1.2 COMPLETED (start with §3 governing principle + §4 architectural decisions + §5 layered-B3 + §6.1 historical-pointer + §7 discipline notes), then runbook authoring at `tasks/XGID_RETROFIT_PASS_4_IMPL.md`.

---

## ⚫ (historical, superseded by Pass-4-design-OPEN state above) PLAY — XGID Retrofit Pass 3 milestone CLOSED at J-138; standby for next-milestone selection (Pass 4 + M6 (new) both ready)

**XGID Retrofit Pass 3 milestone CLOSED at J-138 (2026-05-28).** Four Clair-facing atomic commits on `main`: Commit 1 `1be0249` doc-pass (J-131 Option C hybrid minimal; honest two-file atomic; honest count discrepancy surfaced + resolved at J-132 Path-(iii) amend-in-place); Commit 2 `67fb48d` seven-surface retype atomic (J-136 ten-file; xgen-{common,core,node} libs CLEAN; Path 2 split locked at Joe-lock checkpoint #3 against 638 test-fixture errors >> ~50 threshold per runbook §5.1); Commit 2a `0cdf0ad` test-fixture sweep (J-137 thirty-file atomic; 638 errors closed via parallel-subagent delegation under per-crate guard-rails + 11 per-surface tests T1-T11 atomic per runbook §4.7; 8/8 GREEN verification at milestone-bearing boundary); this Commit 3 milestone-close commit.

**Test count at close**: 589 (34 xgen-common lib + 8 invariance + 453 xgen-core + 88 xgen-node lib + 6 precedence). +98 net delta vs Pass 2 J-126 baseline of 491. Negative delta vs J-119's 627 stays expected per Path A inherited from Pass 1 + Pass 2 — xgen-client consumes retyped xgen-core + xgen-node types and doesn't build at workspace level until Pass 4 + Pass 5 (~38 tests missing live in xgen-client). Pass 5 close restores ≥ 627 plus all per-Pass invariance + surface tests accumulated.

**What unblocks**: XGID Retrofit Pass 4 (xgen-client consumer-side retypes; runbook authoring is the next Chat Claude work-shape on the XGID retrofit track). M6 (new) Node admin write path stays unblocked-but-not-selected — opens after Joe selects the next-active milestone at session open. Pass 4 + M6 (new) are both ready for selection; sequencing is Joe's call.

**Layered-B3 audit answer per runbook §6.5 + design doc §5.5**: zero layered surfaces emerged. **Third Pass-arc no-finding instance** after Pass 1 J-122 + Pass 2 J-126; **three-instance chain at Pass-arc layer now durable** — pattern matches D-077/D-078 promotion-threshold framing. Pass-arc work whose scope is data-structure-or-function-signature shape (not algorithm validation) naturally avoids the layered-B3 surface; the `Borrow<str>` projection mechanism inherited from Pass 1 Commit 4 handles type-projection at call-site boundaries without secondary encoding surfaces across all retyped functions. Discipline data point promoted: Pass-arc B3-audit-expected-null is now load-bearing structural fact; Pass 4 + Pass 5 inherit the expected-null posture without re-derivation.

**Pass 3 "Honest longer work over fast shortcuts" final count: TWO recurrences** at canonical-record-amendment layer (lighter than Phase 9 3b arc's 10; heavier than Pass 2 J-126's zero):
- **J-129** — Track 1 canonical-record amendment at runbook v1.0 (surface ordering drift against design doc §2 + `handle_federation_incoming` mis-location). Prospective catch at runbook-authoring layer (D-078 second prospective-catch instance).
- **J-134** — Track 1 canonical-record amendment at design doc §2 v1.3 → v1.4 in-place rewrite-correction of J-133's own Q3.6 + D-079 promotion atom. Prospective catch at design-doc-walking-its-own-content layer.

Plus three honest data points inline without recurrence-increment shape per close-event-not-recurrence-event framing: J-130 silent-gitignore-skip drift-fix atom (sub-shape D of prose-then-batch atomicity-slip family; candidate D-NNN-η flagged-not-promoted); J-131 honest two-file-vs-three-file count discrepancy (resolved at J-132 Path-(iii) amend-in-place); J-135 T11 addition at test-enumeration layer (D-078 working-as-designed prospective catch before Commit 2 ships).

**All three Joe-lock checkpoints closed affirmatively** (full sequence detail at JOURNAL J-138 Sub-section 3):
- **#1 post-Commit-1 doc-pass drift check** — fired at J-131 + resolved at J-132 Path-(iii) amend-in-place.
- **#2 pre-Commit-2 verbatim surface list approval** — fired at J-133+J-134+J-135 triple-canonical-record-amendment arc; LOAD-BEARING D-078 application surface; three drift surfaces closed before any Commit 2 code landed.
- **#3 post-Commit-2 split-trigger decision** — fired at J-136; 638 errors >> ~50 threshold → Path 2 split locked. **Pre-locked contingent-split posture validated at execution time for the second sibling milestone in a row** (Pass 2 fired at 93 errors → Pass 3 fired at 638 errors; same authoring discipline + ~7× scaling without re-derivation of decision protocol). D-NNN-δ candidate promotion-watch from J-126 advances at this milestone close.

**Verification at close**: `cargo test -p xgen-common -p xgen-core -p xgen-node`: 589 tests pass (0 failed, 0 ignored). 8/8 GREEN at Commit 2a milestone-bearing boundary (recorded at J-137); single workspace re-verification pass at this milestone-close commit boundary returned 589 GREEN. Both clippy gates clean (`--lib` + `--tests`, `-D warnings`). `cargo build --workspace` deliberately broken per Path A. **J-NNN freeze guardrail** (J-108 codification): `grep -rn 'J-NNN' . --include='*.rs' --include='docs/*.md' --include='tasks/*.md'` returns ZERO matches at freeze-site sources post-staging. Both pre-existing documented flakes (precedence env-var race; `reconnect_with_existing_tip_small_delta_delivered`) did NOT fire across the 8 GREEN runs at J-137 nor at this milestone-close re-verification pass.

**Cross-Pass discipline carry-overs (load-bearing for Pass 4 + Pass 5)** — full enumeration at runbook §9.8 + JOURNAL J-138 Sub-section 2:
- **Path A** — three-instance durability across Pass 1 + Pass 2 + Pass 3 established; permanent cross-Pass discipline; Pass 4 + Pass 5 inherit without re-lock.
- **`Borrow<str>` additive API** — Pass 1 Commit 4 introduced; Pass 2 + Pass 3 consumed mechanically; Pass 4 + Pass 5 inherit at all HashMap lookup sites.
- **Layered-B3 expected-null** — three-instance no-finding chain at Pass-arc layer; load-bearing structural fact.
- **Pass-internal-consistency framing over trilogy-internal-consistency** — Pass 2 §7.7 + Pass 3 §7.2 establish precedent.
- **Pre-locked contingent-split posture** — Pass 2 §7.3 + Pass 3 §5.1 establish criterion + runbook-authoring shape; Pass 4 + Pass 5 inherit framing as default.

**Four candidate D-NNNs promotion-watch (none promoted at this atom per D-069)** — full enumeration at JOURNAL J-138 Sub-section 8:
- **D-NNN-γ** (small-cardinality vs large-cardinality identifier-keyed maps per-Pass call-site density) — second instance at Pass 3 Surface #1 retype.
- **D-NNN-δ** (pre-locked contingent-split posture as honest framing technique) — second validation instance at Pass 3 Commit 2a split firing.
- **D-NNN-ε** (async-spawned task captures force owned parameters — Tokio idiom) — four instances at one xgen-node module-family; promotion-watch opens at Pass 4 if structurally different fifth instance fires at xgen-client async surfaces.
- **D-NNN-format-boundary** (format-boundary preservation wire OR persistence) — two conceptual instances within Pass 3; three-instance threshold opens at Pass 4 if client-side serialisation-format slot instantiates.

**D-074 application count**: J-138 is the **thirty-fifth instance + fourteenth milestone-close** (J-126 was twenty-third + thirteenth). Five-file atomic at this milestone close (runbook + design doc + JOURNAL + CLAUDE.md + ROADMAP.md). Full per-atom count from J-127 → J-138 at JOURNAL J-138 Sub-section 6.

**Track 1 (Clair): stood down** until Joe picks the next-active milestone at session open. **Track 2 (Chat Claude): standby for next-milestone selection.** Pass 4 + M6 (new) are both ready and sized similarly for a Clair work session. Pass 4 requires runbook authoring (Chat Claude work) before Clair implementation; M6 (new) Block 4 verb-by-verb walks (~35 verbs across 7 categories at `docs/xgen_node_admin_ops_design.md` §6) are independent of XGID Retrofit completion and can run in parallel.

**Entry point for next session: this CLAUDE.md PLAY block + JOURNAL J-138 entry first per Rule 0**, then whatever document Joe pointed the session at for milestone selection.

---

## ⚫ (historical, superseded by Pass-3-CLOSED state above) PLAY — XGID Retrofit Pass 3 implementation ACTIVE; Commit 2a ✅ at J-137; Clair pickup at `tasks/XGID_RETROFIT_PASS_3_IMPL.md` §6 Commit 3 milestone close next

**XGID Retrofit Pass 3 Commit 2a SHIPPED at J-137 (2026-05-28)** — test-fixture projection sweep + 11 per-surface tests T1-T11 atomic per D-074 thirty-fourth instance + Lock #3 per-commit cadence. **Thirty-file atomic** — heaviest single commit of Pass 3 by file count; sibling-shape to Pass 2 Commit 2a `58b94a5` (nine-file xgen-core sweep at 93 errors arc) + Pass 1 Commit 4a precedent.

**Test-fixture sweep delivered via parallel-subagent delegation** (one per crate — xgen-core + xgen-node) under DO-NOT-CROSS-CRATE-BOUNDARY guard-rails + mechanical-projection-only-per-§5.2-verbatim-patterns instruction set. Both completed clean: xgen-core (4 files / 160 errors → 0 / 0 deviations); xgen-node (~20 files / 478 errors → 0 / 6 minor deviations all honest-reported per Rule 1).

**11 per-surface tests T1-T11 added per runbook §4.7 by name**: T1-T4 at xgen-core/src/node/runtime.rs `mod persistence_amendment_commit_2a_tests` (Surface #1 three tests + Surface #2 dispatch_event borrowed-boundary); T5 at xgen-node/src/federation_session.rs new `mod tests` (Surface #3 typed slots compile-time contract); T6 at xgen-node/src/fanout.rs `mod tests` (Surface #4 topological_sort Pass 1 sentinel + EventXgid Ord delegation); T7 + T8 + T11 at xgen-node/src/app.rs `mod tests` (Surface #5 persistence-format round-trip Q5.12 + handle_federation_incoming forced-owned Q5.2 + run_federation_session_post_handshake forced-owned Q5.14 v1.3); T9 + T10 at xgen-node/src/reconnect.rs `mod tests` (Surface #6 three spawned functions forced-owned + Arc<NodeXgid> shared reference pattern).

**Verification rigour at Commit 2a milestone-bearing boundary (full 8 GREEN per §5.3 + §4.9)**: 5 isolated runs with `cargo clean -p xgen-common -p xgen-core -p xgen-node` between each + 3 consecutive workspace runs of `cargo test -p xgen-common -p xgen-core -p xgen-node` — **ALL 8 GREEN**. Test count stable at **589 = 34 xgen-common lib + 8 invariance + 453 xgen-core + 88 xgen-node lib + 6 precedence** across all 8 runs. Delta vs pre-T1-T11 sweep 578: +11 = T1-T11 target hit per runbook §4.7.

**Both clippy gates clean**: `--lib -D warnings` + `--tests -D warnings` (six nits closed at integration time: `.get(&x).is_some()` → `.contains_key(&x)` in T1+T2; redundant closure `|e| event_id_str(e)` → `event_id_str` in agent-sweep fanout.rs; useless `vec![room_id.clone()]` → `[room_id.clone()]` in phase9_compound_c7).

**`cargo build --workspace`** deliberately broken at xgen-client consumer sites only per Path A inherited from Pass 1 (192 errors all xgen-client; Pass 5 close restores).

**Both pre-existing documented flakes did NOT fire** across the 8 GREEN runs (precedence env-var race; `reconnect_with_existing_tip_small_delta_delivered`). Honest data point per Rule 2 — flakes stay documented as known.

**Parallel-subagent-sweep discipline data point** recorded at runbook §9.7 for future Pass-arc Commit-2a-shape runbook authors: when test-fixture sweep error count exceeds ~500, parallel-subagent delegation under per-crate guard-rails is a viable shape; discipline cost is explicit honest-deviation reporting at integration time (Rule 1) + per-crate independence verification + pre-Commit-2a 8-GREEN verification catches integration-edge regressions.

**Pass 3 "Honest longer work over fast shortcuts" count stays at TWO** inherited from J-129 + J-134 (Commit 2a is within-milestone substantive event, not recurrence shape).

**Track 1 (Clair): pickup at `tasks/XGID_RETROFIT_PASS_3_IMPL.md` §6 Commit 3 milestone close** — Pass 3 PLAY → DONE; runbook ACTIVE → COMPLETED v1.5 → v1.6 + Last-updated milestone-close note + DoD checklist verified; design doc §6.1 J-NNN freeze per J-108 codification + header chain entry; ROADMAP visual tree 🟢 → ✅ + version bump + Past entry + Present + Near future Pass 3 line removed; this CLAUDE.md PLAY flip "Commit 2a ✅" → "Pass 3 CLOSED at J-138; standby for next-milestone selection (Pass 4 + M6 (new) both ready)"; grep `J-NNN` guardrail = ZERO post-staging.

**Track 2 (Chat Claude): standby** until Clair's Commit 3 closes; parallel-eligible items include M6 (new) Block 4 verb-by-verb walks at `docs/xgen_node_admin_ops_design.md` §6 (~35 verbs across 7 categories).

**Entry point for next session: this CLAUDE.md PLAY block + JOURNAL J-137 entry first per Rule 0**, then runbook §6 Commit 3.

---

## ⚫ (historical, superseded by Commit-2a-shipped state above) PLAY — XGID Retrofit Pass 3 implementation ACTIVE; Commit 2 ✅ at J-136 under Path 2 split per Joe-lock checkpoint #3; Clair pickup at `tasks/XGID_RETROFIT_PASS_3_IMPL.md` §5 Commit 2a next

**XGID Retrofit Pass 3 Commit 2 SHIPPED at J-136 (2026-05-28)** — seven-surface retype atomic per D-074 thirty-third instance + Lock #3 per-commit cadence. **Ten-file atomic**: xgen-core/src/node/runtime.rs (Surface #1 six per-space HashMap keys → SpaceXgid + Surface #2 dispatch_event Option<&NodeXgid>); xgen-node/src/federation_session.rs (Surface #3 + J-134 Finding B annotation drop closing D-079); xgen-node/src/fanout.rs (Surface #4 ClientSenders + FederationPeerSenders + FanoutRequest + event_space_id + apply_fanout + collect_sync_history + compute_federation_delta + topological_sort_events HashSet<EventXgid>); xgen-node/src/app.rs (Surface #5 — 12 in-memory identifier slots + handle_federation_incoming T8 + run_federation_session_post_handshake T11 per Q5.14 v1.3 13-param matrix + ConnectedClientInfo Q5.15 + 4 persistence-format String per §4.3 + Q3-overload projection); xgen-node/src/reconnect.rs (Surface #6 three spawned functions forced-owned + AttemptCursor HashMap<NodeXgid, u32>); docs/xgen_appendix_d_en.md (Surface #7 four markdown table classification rows annotated typed-XGID-in-memory + String on-disk/wire per §4.3); runbook v1.3 → v1.4 + new §9.6 amendment-provenance; JOURNAL J-136 body entry; ROADMAP v1.40 → v1.41 + visual tree row + Past entry; this CLAUDE.md PLAY flip + header bump.

**Verification at Commit 2 boundary (lib-only per §5.3 deferred-GREEN framing)**: `cargo build -p xgen-common -p xgen-core -p xgen-node` **CLEAN**; `cargo clippy -p xgen-common -p xgen-core -p xgen-node --lib --all-features -- -D warnings` **CLEAN**; `cargo build --workspace` deliberately broken at xgen-client consumer sites only per Path A inherited from Pass 1; `cargo test -p xgen-common -p xgen-core -p xgen-node --tests` 638 errors (xgen-core 160 + xgen-node 478). Full 8 GREEN per §4.9 fires at Commit 2a per §5.3.

**Joe-lock checkpoint #3: Path 2 (Commit 2a split) locked** against 638 test-fixture errors >> ~50 split threshold per runbook §5.1. Sibling-shape to Pass 2 Commit 2a `58b94a5` (93 errors) + Pass 1 Commit 4a precedent. Each commit preserves its own atomic-purpose-discipline per D-074.

**WIP-branch lineage**: branch `wip/pass-3-commit-2-in-flight` carried checkpoint #1 `728b834` (Surfaces #1+#2+#3+#4 lib-clean + #5 partial) + checkpoint #2 `2f647bf` (Surfaces #5+#6 closed + xgen-node lib CLEAN); squashed at this Commit 2 ship per D-074 atomic discipline (single-commit per surface-set per §4.10 framing). Branch history disappears on main; only this squashed Commit 2 remains.

**Two discipline data points surfaced for JOURNAL J-NNN body**: (a) Pass-1-pre-walk reconnaissance Pass 3 marker sparsity — only ONE `// Pass 3 widens` marker exists in production at xgen-core/src/node/runtime.rs:588 vs 33 Pass 2 markers per J-125 audit (expected per Pass-arc framing N+1 not N+2 design; data point for Pass 4 + Pass 5 pre-walk discipline); (b) Surface #4 fanout.rs "verification only" framing vs actual lift — runbook §4.1+§4.2 framed as "likely 0 code changes" but baseline showed 9 errors at fanout.rs (Pass 1+2 propagation into Path A inherited break state); substantive Surface #4 work landed to close propagation + Q4.1-Q4.7 retypes.

**Pass 3 "Honest longer work over fast shortcuts" count stays at TWO** inherited from J-129 + J-134 (Commit 2 ship is within-milestone substantive event, not recurrence shape; sibling-shape to close-event-not-recurrence-event at J-101 / J-108 / J-122 / J-126).

**Joe-lock checkpoint #3 closed affirmatively at this atom** (Path 2 locked + lib-clean + clippy clean + 638 error count surfaced honestly).

**Track 1 (Clair): pickup at `tasks/XGID_RETROFIT_PASS_3_IMPL.md` §5 Commit 2a** (test-fixture projection sweep across xgen-core + xgen-node ~638 errors + 11 per-surface tests T1-T11 atomic). Verification at Commit 2a = 8 GREEN runs per §5.3 + §4.9. Then Commit 3 milestone close per §6. Read this CLAUDE.md PLAY block + JOURNAL J-NNN entry first per Rule 0, then runbook §5 in order.

**Track 2 (Chat Claude): standby** until Clair's Commit 2a closes; parallel-eligible items include M6 (new) Block 4 verb-by-verb walks at `docs/xgen_node_admin_ops_design.md` §6 (~35 verbs across 7 categories).

**Entry point for next session: this CLAUDE.md PLAY block + JOURNAL J-NNN entry first per Rule 0**, then runbook §5 Commit 2a.

---

## ⚫ (historical, superseded by Commit-2-shipped state above) PLAY — XGID Retrofit Pass 3 implementation ACTIVE; Commit 1 doc-pass ✅ at J-131; Clair pickup at `tasks/XGID_RETROFIT_PASS_3_IMPL.md` §4 Commit 2 next

**XGID Retrofit Pass 3 Commit 1 doc-pass SHIPPED at J-131 (2026-05-28)** against amended v1.1 runbook (post-J-129 Track 1 canonical-record amendment) + cleared canonical record (post-J-130 silent-gitignore-skip drift-fix atom). Option C hybrid minimal Commit 1 per runbook §3 — no design-doc touch (design doc COMPLETED v1.2 at J-127); no runbook touch (runbook ACTIVE v1.1 at J-129 + cleared at J-130).

**Honest two-file-vs-three-file count discrepancy.** Runbook §3.2 enumerates a three-file atomic: ROADMAP + CLAUDE PLAY flip + JOURNAL chain entry only. Post-J-129 strip-the-chain discipline + post-J-130 cleared canonical record + sibling-shape to J-123/J-124/J-125 chain-only doc-only milestone-event precedent collide here: the runbook's "JOURNAL chain entry only" framing was authored pre-strip when a chain existed to append to; post-strip the chain doesn't exist, and per Joe's Pre-Commit-1 lock there's no body either. Result: JOURNAL.md gets no edit at this atom (the date was already bumped to 2026-05-28 at J-130). The atomic is honestly **two files (ROADMAP + CLAUDE.md)**. Surfaced for Joe-lock checkpoint #1 resolution — sibling-shape to D-NNN-η's claimed-atomic-file-count surface, but at the prose-vs-strip-discipline-collision layer rather than git-staging layer. Pass-3-internal precedent question: post-strip, what does "JOURNAL chain entry only" map to? Worth recording as discipline data point if Joe locks a Pass-3-internal answer at checkpoint #1.

**Two-file atomic per D-074 (twenty-eighth instance) + Lock #3 per-commit cadence (honest count):**

1. `docs/ROADMAP.md` v1.39 → v1.40 + visual tree Pass 3 row gains J-131 Commit 1 ✅ sub-bullet + J-130 drift-fix sub-bullet (acknowledging J-130's prior atomic-shape correction) + line-156 Pass 3 implementation milestone-row flipped from "§3 Commit 1 next-active" → "§4 Commit 2 next-active; Commit 1 doc-pass ✅ at J-131" + Past entry + Present section flipped + header date bump 2026-05-27 → 2026-05-28.
2. `CLAUDE.md` — this PLAY block flip + header date bump 2026-05-27 → 2026-05-28.

**JOURNAL.md NOT amended at this atom** (sibling-shape to J-123/J-124/J-125 chain-only precedent under post-strip discipline; honest count surfaces this as TWO-file rather than three-file).

**DECISIONS.md NOT amended** (no new principles locked at Commit 1).

**Design doc + runbook NOT amended at this atom** per Option C hybrid minimal.

**Verification at Commit 1 boundary**: `cargo test -p xgen-common -p xgen-core` = 491 tests (matches J-126 baseline; sanity-check only — no code touched at Commit 1). `grep -rn 'J-NNN' . --include='*.rs' --include='docs/*.md' --include='tasks/*.md'` returns ZERO matches post-staging per J-108 codification (design doc §6.1 implementation-J-NNN placeholder freezes at Commit 3 milestone close).

**Joe-lock checkpoint #1 fires post-ship** per runbook §3.4: three drift-detection points to confirm (ROADMAP version bump v1.39 → v1.40 + visual tree row ✅; CLAUDE PLAY flip from "Commit 1 against amended v1.1" → "Commit 2 (Commit 1 doc-pass ✅)" ✅; honest two-file-vs-three-file count discrepancy surfaced for resolution).

**Joe-lock checkpoint #2 fires next** per runbook §2.3 once checkpoint #1 closes affirmatively: Clair extracts the seven-surface Q-tables from design doc `tasks/XGID_RETROFIT_PASS_3_DESIGN.md` v1.2 §2 verbatim + surfaces to Joe by name; Joe approves each surface before any production code lands. LOAD-BEARING D-078 application surface. Trigger (a) fires if any named type or method does not exist in production code — but the v1.1 amendment at J-129 already re-aligned surface ordering and file locations against design doc §2 verbatim, so checkpoint #2 should pass cleanly absent fresh drift.

**Pass 3 "Honest longer work over fast shortcuts" count: TWO** (J-129 three runbook §4 drifts + J-130 silent gitignore-skip; both prospectively caught at Clair's session-open audits before any production code touched). Count stays at TWO at this Commit 1 doc-pass close (sibling-shape to topo-sort J-098 + persistence-amendment J-106 + Pass 1 + Pass 2 inherit-not-increment framing — Commit-1-doc-pass events are within-milestone, not new milestone-surface events).

**Sibling-in-shape to Pass 2 Commit 1 doc-pass at J-125** (the only direct precedent for an Option-C-hybrid-minimal Commit 1 against an already-COMPLETED design doc, though Pass 2 had a design-doc §6.7 entry edit making it five-file; Pass 3's design doc was already at v1.2 COMPLETED at J-127 with no §6.7-equivalent slot needed). Pass 3's two-file honest count is *lighter than* Pass 2's five-file due to (a) no design doc touch, (b) no runbook touch (already at v1.1), (c) post-strip discipline absorbing the JOURNAL chain entry.

**Track 1 (Clair): pickup at `tasks/XGID_RETROFIT_PASS_3_IMPL.md` §4 Commit 2** (seven-surface retype + per-surface tests atomic) post-checkpoint #1 + checkpoint #2 approvals. Read CLAUDE.md PLAY block + JOURNAL J-130 entry first per Rule 0, then runbook §4 in order, then design doc `tasks/XGID_RETROFIT_PASS_3_DESIGN.md` v1.2 §2 Q-tables verbatim.

**Track 2 (Chat Claude): standby** until Clair's Commit 2 closes affirmatively at Joe-lock checkpoint #3 (split-trigger decision); parallel-eligible items include M6 (new) Block 4 verb-by-verb walks at `docs/xgen_node_admin_ops_design.md` §6 (~35 verbs across 7 categories).

**Entry point for next session: this CLAUDE.md PLAY block + JOURNAL J-130 entry first per Rule 0**, then runbook §4 Commit 2 against amended v1.1.

---

## ⚫ (historical, superseded by Commit-1-shipped state above) PLAY — XGID Retrofit Pass 3 implementation ACTIVE; runbook ✅ v1.1 at J-129 (Track 1 amendment); Clair pickup at `tasks/XGID_RETROFIT_PASS_3_IMPL.md` §3 Commit 1 against amended v1.1

**XGID Retrofit Pass 3 implementation runbook amended in place at J-129 (2026-05-27)** — Track 1 canonical-record amendment over the v1.0 runbook shipped at J-128. Cause: Clair's pre-Clair six-dimension audit at session-open surfaced three drifts at runbook §4 against design doc §2 (Trigger (a) candidates per runbook §2.2): Surfaces #1↔#2 ordering swapped at v1.0; Surfaces #5↔#6 ordering swapped at v1.0; `handle_federation_incoming` mis-located to `federation_session.rs` (production code lives at `xgen-node/src/app.rs:976`). Joe locked Path α (Track 1 amendment in this session) over Path β (Clair extracts verbatim at checkpoint #2) + Path γ (fold corrections into Commit 1). Runbook now matches design doc §2 verbatim.

**D-078 second prospective-catch at runbook-authoring layer** — J-115 + J-116 were prospective catches at runbook-implementation-by-Clair layer; this J-129 is prospective catch at runbook-authoring-by-Chat-Claude layer (distinct surface; one layer up). Candidate D-NNN-ζ "design-doc-grounded surface enumeration at runbook authoring" flagged-not-promoted per D-069 (one instance at this J-129; three-instance threshold not met; promotion-watch opens at Pass 4 + Pass 5 runbook authoring).

**J-130 drift-fix atom (2026-05-28) closed the silent gitignore-skip** in J-129. Commit `6a7d126`. Four-file atomic — closes the fact that J-129 claimed five-file atomic but git received four because `tasks/HANDOFF_TOPOSORT_RUNBOOK_AUTHORING.md` was explicitly gitignored at `.gitignore:58`. Candidate D-NNN-η flagged-not-promoted per D-069. Pass 3 "Honest longer work" count incremented to TWO. Sub-shape D (gitignored-path silent-skip slip) of prose-then-batch atomicity-slip family — structurally novel within the family.

**Strip-the-chain discipline applied to CLAUDE.md + JOURNAL.md + ROADMAP.md headers at J-129.** The `> **Last updated:**` chain in these three files had grown to 50-125 KB per file (CLAUDE.md L16 alone was 71.8 KB), causing concrete reading + editing failures. CLAUDE.md "Document Header Convention" specifies only `YYYY-MM-DD` for the `Last updated` value; the chain was emergent prose that bled JOURNAL's job into the header line. Discipline data point recorded at JOURNAL J-129 Sub-section 8 for future reference.

**Six-dimension pre-Clair audit pattern instantiated as third Pass-arc instance** (J-120 Pass 1 first; J-125 Pass 2 second; this J-129 Pass 3 third — pattern's durability at three Pass-arc instances now matches D-077/D-078 promotion-threshold framing).

**"Honest longer work over fast shortcuts" — Pass 3 count incremented to ONE at J-129; incremented to TWO at J-130.** Two prospective catches within Pass 3 implementation kickoff — both surfaced at Clair's session-open Rule 0 audits before any production code touched.

**Track 1 (Clair) at J-129 → J-130 → J-131**: pickup at `tasks/XGID_RETROFIT_PASS_3_IMPL.md` §3 Commit 1 against amended v1.1 runbook — ✅ SHIPPED at J-131 (this commit). Next-active is runbook §4 Commit 2.

**Track 2 (Chat Claude) at J-129**: standby until Clair's Commit 1 closes — closed at J-131.

---

## ⚫ (historical, superseded by runbook-shipped state above) PLAY — XGID Retrofit Pass 3 design phase CLOSED at J-127; runbook authoring next-active for Chat Claude + Joe in a fresh session

**XGID Retrofit Pass 3 design phase CLOSED at J-127 (2026-05-27, this commit).** Full seven-surface walk closed across two same-day design sessions: Surfaces #1-#4 at v1.1 (morning) + Surfaces #5-#7 + §4.3 consolidation + §6 + §7 fills at v1.2 (afternoon). Four-file atomic per D-074 (twenty-fourth instance) + Lock #3 per-commit cadence: (1) `tasks/XGID_RETROFIT_PASS_3_DESIGN.md` ACTIVE v1.1 → COMPLETED v1.2 (~75 KB) + (2) `docs/ROADMAP.md` v1.36 → v1.37 + (3) this CLAUDE.md PLAY flip + (4) JOURNAL J-127 body entry. DECISIONS.md not amended.

**Single governing principle (§3) confirmed inherited from Pass 2 unchanged across full seven-surface walk.** Zero wrinkles. The principle reads (verbatim from Pass 2 §3):

> Identifier slots retype to typed XGIDs; descriptive-string slots stay String; internal variables bind as typed references; &str projection happens at the call-site boundary via Borrow<str> (Pass 1's additive API at Commit 4 implementation-kickoff lock); no Deref<Target = str> shortcuts.

Three Pass-arc instances of inheritance-unchanged (Pass 1 implicit at runbook; Pass 2 explicit at J-123; Pass 3 explicit at this J-127) make the governing principle's stability durable at Pass-arc layer.

**Six architectural decisions locked at §4.1-§4.6** (full detail at design doc §4 + JOURNAL J-127 Sub-section 4):

- **§4.1** Six per-space HashMap keys retype shape (atomic: field types + helper signatures + public-API parameters). D-NNN-γ promotion candidate at 2 instances.
- **§4.2** `dispatch_event` `Option<&NodeXgid>` borrowed boundary + sibling-shape rule table extended at v1.2 with **async-spawned-task-captures sub-rule** as third row (instantiated at Surface #5 `handle_federation_incoming` + Surface #6 reconnect.rs three spawned functions).
- **§4.3** Format-boundary preservation (wire OR persistence) — **consolidated at v1.2** from v1.1's wire-only framing. Surface #5 walk surfaced structurally identical persistence-format boundary slots (filesystem path generation + on-disk JSON HashMap + replay_spaces_from_dir + wire-message destructure). Same principle, same reasoning, both at I/O byte-serialisation boundaries; consolidated under one §4.3 rather than splitting into §4.3-wire + §4.7-persistence per sibling-shape to D-076 v1 → v1.1 amend-in-place reasoning. One decision = one drift surface. D-NNN-δ promotion candidate at 2 instances.
- **§4.4** `event_space_id` return shape forced-owned + general rule recorded inline (a return type can be borrowed only if every branch can borrow from input at the same flavour; if any branch must construct a new value or change flavour, the return type is forced to owned).
- **§4.5** ClientSenders + FederationPeerSenders Pass 3 scope (NOT Pass 4) — v1.0 framing corrected at v1.1; xgen-node-internal types (mpsc::Sender channels never cross to xgen-client).
- **§4.6** Topo-sort `&str` slot at fanout.rs:193 already covered at Pass 1 Commit 3; the J-097 anticipated retype was inherent in Pass 1's Option<EventXgid> retype.

**Three candidate D-NNNs flagged-not-promoted per D-069**:

- **D-NNN-γ** "small-cardinality vs large-cardinality identifier-keyed maps per-Pass call-site density" — 2 instances (Pass 2 §4.1 Q2.8.c + Pass 3 §4.1).
- **D-NNN-δ** "format-boundary preservation (wire OR persistence)" — 2 instances (§4.3 v1.1 wire + §4.3 v1.2 persistence-extended). Three-instance threshold opens at Pass 4 if client-side serialisation-format slot (Tauri IPC, AI control protocol over HTTP, gRPC) instantiates.
- **D-NNN-ε** "async-spawned task captures force owned parameters" — 3 instances at same xgen-node module-family surface. Three instances at one module-family is weaker durability evidence than three across structurally different surfaces (per D-077 + D-078 surface-diversity framing). Per D-065 honest framing: the rule is a Rust language idiom (`'static` bound on `tokio::spawn`), not a XGen-specific call. Promotion would record a language fact rather than a project decision. **Promotion-watch opens at Pass 4** surfacing structurally different fourth instance at xgen-client async surfaces.

**Layered-B3 (§5.5) confirmed null at full seven-surface scope.** Third Pass-arc instance after Pass 1 J-122 + Pass 2 J-126; pattern's durability at three instances now matches D-077/D-078 promotion-threshold framing. Pattern is now established at Pass-arc layer: identifier-slot retype scopes do not surface layered-B3 because the projection mechanism (`Borrow<str>`) handles type-projection at boundaries uniformly without forcing secondary encodings of the same invariant.

**§6.1 historical-pointer filled in Shape α** (pointer-style sibling to Pass 2 §6.7). Implementation J-NNN milestone-close placeholder to be frozen at runbook close per J-108 codification.

**§7 discipline-notes consolidated with five sub-sections** (§7.1 format-boundary preservation unified pattern + §7.2 async-spawned task captures sub-rule + §7.3 forced-owned return shape rule + §7.4 xgen-node-internal type confusion v1.0 framing data point + §7.5 doc-tree sweep classification-vs-content-shape gap at Appendix D Surface #7).

**Honest data points worth recording** (full detail at JOURNAL J-127 Sub-section 8):

- **(a)** Design doc size came in heavier (~75 KB) than v1.1 §1.2's ~30-40 KB estimate. Drivers: §4.3 consolidation reasoning + §7 five sub-sections + Surfaces #5/#6/#7 Q-tables fuller than anticipated. Pass-internal-consistency framing accepts the lighter-than-trilogy ~80-100 KB band, Pass 3 lands mid-band rather than at lighter end Pass 2 hit.
- **(b)** §4.3 consolidation in same atom as design close: chose amend-in-place over split-into-sibling per sibling-shape to D-076 v1 → v1.1. Discipline data point for next sibling milestone: when a structural pattern emerges at a downstream surface structurally identical to an upstream lock, prefer amend-in-place over split-into-sibling.
- **(c)** Two-session design walk shape: Pass 2 fit single session at J-123; Pass 3 spanned two same-day sessions (morning #1-#4 + afternoon #5-#7). Pass-internal precedent: future Pass-arc design walks with > 5 surfaces should consider two-session split as deliberate scaffolding rather than honest-longer-work recurrence.
- **(d)** D-NNN-δ consolidation logic worth recording: split would have promoted earlier (wire + persistence counted separately as 2 of 3); consolidation keeps at 2 of 3 with cleaner promotion-eligibility framing under one decision-surface.
- **(e)** Honest framing on Q5.A + Q5.B recommendation reversal pattern: the v1.2 walk's structural finding came from my read of Surface #5 grep results, not from a clean a-priori lock at v1.1 close. Sibling-shape to Pass 1 + Pass 2 walks where some structural calls emerged at walk time from grep-vs-design-anticipation reconciliation. Pattern is normal.

**What this commit does NOT close**:

- **XGID Retrofit Pass 3 milestone**: stays PLAY (design ✅; runbook authoring next-active for Chat Claude + Joe; implementation has not yet started).
- **M6 (new) Node admin write path**: stays unblocked-but-not-selected.
- **XGID Retrofit Pass 4 + Pass 5**: stay PENDING behind Pass 3 close.
- **D-071 future-removal arc for `validate_steps_8_13` + `accept_event`**: stays pending; surface-driven per D-071.
- **Timestamp-bound validation Gap G6**: stays pending; surface-driven per D-071.

**Track 1 (Chat Claude + Joe): runbook authoring at `tasks/XGID_RETROFIT_PASS_3_IMPL.md` in a fresh session** per Pass 2 + trilogy precedent. **Target runbook shape**: three-commit base (1 doc-pass / 2 xgen-node + xgen-core HashMap-keys retype atomic / 3 milestone close) + contingent Commit 2a (test-fixture sweep) firing at Joe-lock checkpoint #3 if test-fixture error count > ~50 — sibling-shape to Pass 2's contingent-split posture pre-locked at design close per D-065 honest framing. **Three Joe-lock checkpoints**: #1 post-Commit-1 drift; #2 pre-Commit-2 verbatim surface list (Chat Claude extracts the seven-surface Q-tables from design doc §2 verbatim and Joe approves by name before code lands); #3 post-Commit-2 split-trigger decision. **Runbook target size**: ~50-70 KB likely (heavier than Pass 2's ~43 KB given Pass 3's seven-surface scope vs Pass 2's five).

**Track 2 (Clair): stood down** until runbook authoring closes.

**Entry point for next session: this CLAUDE.md PLAY block + JOURNAL J-127 entry first per Rule 0**, then design doc `tasks/XGID_RETROFIT_PASS_3_DESIGN.md` v1.2 (start with §2 surface enumeration + §4 architectural decisions + §6.1 + §7 discipline-notes), then runbook authoring at `tasks/XGID_RETROFIT_PASS_3_IMPL.md`.

---

## ⚫ (historical, superseded by Pass 3 design close above) PLAY — XGID Retrofit Pass 2 milestone CLOSED at J-126; standby for next-milestone selection (Pass 3 + M6 (new) both ready)

**XGID Retrofit Pass 2 milestone CLOSED at J-126 (2026-05-27, this commit).** Three-commit Clair-facing sequence on `main`: Commit 1 `5892e9e` doc-pass (J-125) + Commit 2 `22765a0` xgen-core algorithm-bearing retypes (lib-clean per Path A; all five surfaces atomic per design doc §2 locked Q-tables — `validate_event` + `ValidationOutcome` at exchange.rs; `NodeRuntime::dispatch_event` + `DispatchOutcome` at runtime.rs; `PendingBuffer` arrival hooks at pending.rs; `FederationRegistry` + `IdentityRegistry` method APIs at registry.rs; `accept_message`; plus per-surface unit tests at runbook §4.7; deprecation attributes applied to `validate_steps_8_13` + `accept_event` per design doc §4.2 Q5.b) + Commit 2a `58b94a5` test-fixture projection sweep (Joe-lock checkpoint #3 split-trigger fired at 93 errors > ~50 threshold; nine xgen-core test modules updated; sibling-shape to Pass 1 Commit 4a `4895446` precedent) + this Commit 3 milestone-close commit.

**Test count at close.** 491 (xgen-common 34 lib + 8 invariance + xgen-core 449 = +2 vs J-122 baseline of 489 per per-surface tests from Commit 2 at runbook §4.7). Negative delta vs J-119's 627 baseline stays expected per Path A — xgen-node + xgen-client consume retyped types and don't build at workspace level until Pass 3 + Pass 4 (Pass 5 close restores).

**What unblocks.** XGID Retrofit Pass 3 (xgen-node + Appendix D — federation_session, fanout, app handlers, reconnect scheduler; six per-space HashMap key retypes from design doc §4.1 Q2.8.c deferred to this Pass; runbook authoring is the next Chat Claude work-shape on the XGID retrofit track). M6 (new) Node admin write path stays unblocked-but-not-selected — opens after Joe selects the next-active milestone at session open. Pass 3 + M6 (new) are both ready for selection; sequencing is Joe's call.

**Layered-B3 audit answer per runbook §5.3 + design doc §5.5.** Zero. Sibling-shape to Pass 1's J-122 finding — the projection mechanism (`Borrow<str>`) handled type-projection at boundaries without secondary encoding surfaces. The algorithm-bearing functions consume typed fields through `Borrow<str>` exactly the way Pass 1's validators did; no secondary encoding of the same invariant surfaced.

**"Honest longer work over fast shortcuts" — Pass 2 milestone-scope final count: zero recurrences.** **First project milestone to ship with zero recurrences since the framework was named** — Pass 1 closed with one (J-121 hygiene atom), Phase 7.5 with one, bidirectional with one, topo-sort with three, persistence-amendment with one, Federation Phase 9 3b arc with ten. The combination of factors at Pass 2: (a) design phase named layered-B3 as expected-null in advance per §5.5; (b) runbook pre-locked the contingent-split posture rather than mid-implementation Joe-lock; (c) Pass 1's Borrow<str> additive API meant projection was structurally cheap; (d) Shape-α lighter framings throughout respected Pass-internal-consistency per design doc §7.7; (e) pre-Clair audit at J-125 confirmed clean across six dimensions before any code touched.

**All three Joe-lock checkpoints closed affirmatively.**

- **#1 post-Commit-1 doc-pass drift check** — four drift-detection points confirmed at session open after J-125 push: design doc Status flip ✅; ROADMAP version bump ✅; CLAUDE PLAY flip ✅; new design-doc §6.7 entry ✅.
- **#2 pre-Commit-2 verbatim surface-list approval** — five surfaces enumerated from design doc §2 Q-tables, approved by name before any production code landed.
- **#3 post-Commit-2 split-trigger decision** — Clair reported 93 errors from `cargo test -p xgen-common -p xgen-core --tests` after lib retypes verified clean; 93 > ~50 threshold so Joe locked split per Pass 1 Commit 4a precedent.

**Verification at close.** `cargo test -p xgen-common -p xgen-core`: 491 tests pass (34 + 8 + 449, 0 failed, 0 ignored). 8/8 GREEN at Commit 2a milestone-bearing boundary (5 isolated runs with `cargo clean -p xgen-common -p xgen-core` between each + 3 consecutive workspace runs). `cargo clippy -p xgen-common -p xgen-core --lib --all-features -- -D warnings`: clean. `cargo clippy -p xgen-common -p xgen-core --tests --all-features -- -D warnings`: clean (the `#[allow(deprecated)]` on the exchange.rs test mod is the only suppression in the milestone; documented inline with pointer to the D-071 future-removal arc — exercising `accept_event` before its own audit-design-impl removal arc is a known temporary state during Pass 2). `cargo build --workspace`: deliberately broken per Path A. **J-NNN freeze guardrail (J-108 codification)**: `grep -rn 'J-NNN' . --include='*.rs' --include='docs/*.md' --include='tasks/*.md'` returns ZERO matches post-staging.

**Sibling-in-shape data point for next sibling milestone runbook author.** Pass 2's runbook was authored 2026-05-27 (J-124) + executed without any audit-design re-walks + executed without any mid-implementation Joe-locks beyond the planned checkpoints. The pre-locked contingent-split posture (Commit 2a's split-trigger criterion at runbook §2.3 checkpoint #3) fired exactly as authored, validating the "pre-lock the contingent path" framing per design doc §7.3. Runbook quality + audit discipline working together; the lighter-than-trilogy Pass-internal-consistency framing did not compromise quality.

**Candidate D-NNN promotion-watch.** The "small-cardinality vs large-cardinality identifier-keyed maps retype in different Passes per call-site density" sub-principle flagged at design doc §4.1 stays flagged-not-promoted per D-069 (one instance at design close + zero recurrences at implementation; three-instance threshold not met; may promote at Pass 3 milestone close if a sibling instance fires).

**Track 1 (Chat Claude + Joe): standby for next-milestone selection.** No active work until Joe picks next-active. Pass 3 + M6 (new) are both ready and sized similarly for a Clair work session. Pass 3 requires runbook authoring (Chat Claude work) before Clair implementation; M6 (new) Block 4 verb-by-verb walks (~35 verbs at `docs/xgen_node_admin_ops_design.md` §6) are independent of Federation completion and can run in parallel.

**Track 2 (Clair): stood down** until Joe picks next-active milestone.

**Entry point for next session: this CLAUDE.md PLAY block + JOURNAL J-126 entry first per Rule 0**, then whatever document Joe pointed the session at for milestone selection.

---

## 🟢 (historical, superseded by milestone-closed state above) PLAY — XGID Retrofit Pass 2 implementation ACTIVE; Commit 1 doc-pass ✅ at J-125

**XGID Retrofit Pass 2 Commit 1 doc-pass SHIPPED at J-125 (2026-05-27).** Five-file atomic per D-074 (twenty-second instance) + Lock #3 per-commit cadence: design doc Status ACTIVE → COMPLETED v1.0 → v1.1 + new §6.7 entry in **Shape α** (pointer-style, sibling to §6.6 forward-reference style; Joe-locked at session open over heavier §15-equivalent narrative shape); ROADMAP v1.34 → v1.35 + visual tree Pass 2 row updated + Present + Past; this CLAUDE.md PLAY block flipped from "Clair pickup at runbook §3 Commit 1" to "Clair pickup at runbook §4 Commit 2 (Commit 1 doc-pass ✅ at J-125)"; JOURNAL J-125 header chain entry only (sibling-shape to J-123 + J-124 — doc-only milestone-events use chain-only entries; pattern's durability at three project instances now established meeting D-077/D-078 three-instance threshold); runbook header chain.

**Pre-Clair audit at Pass 2 implementation kickoff confirmed CLEAN across six dimensions** per J-124 lock — sibling-shape to J-120 Pass 1 audit precedent (second project-arc instance; pattern durable at the Pass-arc layer). (1) File paths — all five runbook §8.6 source files resolve. (2) Type shapes at named anchors — `ValidationOutcome::HeldPending`, `ExchangeError` six variants, `DispatchOutcome::{Accepted, HeldPending (UNIT per J-116), Rejected(String)}`, `NodeRuntime` struct with `node_id: String` + `peer_urls` + six per-space `HashMap<String, _>` maps, `PendingBuffer` arrival hook `&str` signatures, `IdentityRegistry::get`+`contains` `&str` signatures with inline "Pass 2 widens" markers, `validate_steps_8_13` + `accept_event` + `accept_message` all present. (3) Pass 1 carry-overs intact — `Borrow<str>` on `Xgid` + all six flavour wrappers + **33 inline `// Pass 2 widens this method to take typed XGIDs` markers** across pending.rs (5) + store.rs (2) + federation/registry.rs (6) + identity/registry.rs (3) + message/exchange.rs (1) + migration/state_machine.rs (1) + space/state.rs (15). (4) Contingency surfaces — `peer_urls` insert site at runtime.rs:181 needs owned `NodeXgid` per Surface #2 §4.3 step 6. (5) Parallel-milestone drift since J-124 — none (HEAD at `0bdb0b8`; no commits landed on main since runbook-shipping). (6) Test count baseline — **489 tests** (34 lib + 8 invariance + 447 xgen-core) matches J-122 exactly; clippy `-D warnings` clean. **AUDIT VERDICT: CLEAN.**

**Entry point for Clair: `tasks/XGID_RETROFIT_PASS_2_IMPL.md` §4 Commit 2.** Read this CLAUDE.md PLAY block + JOURNAL J-125 entry first per Rule 0, then runbook §4 in order, then design doc `tasks/XGID_RETROFIT_PASS_2_DESIGN.md` §2 Q-tables verbatim (Joe-lock checkpoint #2 requires the verbatim surface list be surfaced + approved by name before any production code lands).

**Joe-lock checkpoint #1 fires after this commit lands** per runbook §3.4. Four drift-detection points to confirm:
1. Design doc Status ACTIVE → COMPLETED + v1.0 → v1.1 ✅
2. ROADMAP version bump v1.34 → v1.35 + visual tree updated ✅
3. CLAUDE PLAY block flipped ✅
4. New design-doc §6.7 entry (Shape α, pointer-style) ✅

Once Joe confirms checkpoint #1, Clair proceeds to checkpoint #2 (verbatim surface list approval).

**Three-commit base + contingent Commit 2a sequence** (full detail at runbook §2):

- **Commit 1** doc-pass — ✅ SHIPPED at J-125 (this commit). Five files: design doc + ROADMAP + CLAUDE.md + JOURNAL + runbook header chain.
- **Commit 2** 🟢 NEXT — xgen-core algorithm-bearing retypes (6-10 files) — all five surfaces atomic per the shared Borrow<str> projection mechanic and lower internal coupling than Pass 1's lib + test split: Surface #1 validate_event + ValidationOutcome at exchange.rs; Surface #2 NodeRuntime::dispatch_event + DispatchOutcome + NodeRuntime struct partial retype (node_id + peer_urls retype now; six per-space HashMap keys defer to Pass 3 per design §4.1 Q2.8.c); Surface #3 PendingBuffer arrival hooks at pending.rs; Surface #4 IdentityRegistry + FederationRegistry method APIs; Surface #5 accept_message signature. Plus per-surface unit tests where the retype changes observable behaviour. **Joe-lock checkpoint #2 fires pre-Commit-2** (verbatim surface list from design doc §2).
- **Commit 2a [CONTINGENT]** test-fixture projection sweep — fires at Joe-lock checkpoint #3 if test-fixture error count > ~50; absorbs into Commit 2 if error count ≤ ~50. Honest per D-065 — single-commit shape is expected-default; split posture pre-locked but not forced.
- **Commit 3** milestone close per D-074 (6-8 files) — Status flips + ROADMAP + JOURNAL + CLAUDE PLAY flip + freeze J-NNN placeholders per J-108 codification + layered-B3 audit answer (expected null) + Pass 2 milestone-scope "honest longer work" final count.

**Three Joe-lock checkpoints** (mandatory STOP points per runbook §2.3):

- **#1 post-Commit-1 doc-pass drift check** — ✅ fires after this commit lands; Clair pauses and surfaces four drift-detection points enumerated above; Joe approves.
- **#2 pre-Commit-2 verbatim surface list** — Clair extracts the five-surface Q-decision tables from design doc §2 verbatim and gets explicit Joe approval before any production code lands in Commit 2.
- **#3 post-Commit-2 split-trigger decision** — Clair reports test-fixture error count from `cargo test -p xgen-common -p xgen-core --tests` after lib retypes verify clean; Joe locks single-Commit-2 (absorb sweep) or split (ship Commit 2 lib-clean + Commit 2a sweep atomic) per ~50 threshold heuristic.

**Verification rigour at Commit 2 milestone-bearing boundary**: 5 isolated runs (cargo clean between each) + 3 workspace runs of `cargo test -p xgen-common -p xgen-core` = 8 green runs minimum. Plus `cargo clippy -p xgen-common -p xgen-core --lib --all-features -- -D warnings` clean + `cargo clippy --tests -- -D warnings` clean. `cargo build --workspace` deliberately broken per Path A inherited from Pass 1 (xgen-node + xgen-client consume retyped types at Pass 3 + Pass 4; Pass 5 close restores).

**Borrow<str> projection mechanism is load-bearing** per design doc §3 + runbook §7.5 — Pass 1's Commit 4 implementation-kickoff Joe-lock (`Borrow<str>` additive API on `Xgid` + flavour wrappers) means `HashMap<NodeXgid, V>::get(&str)` and sibling lookups work without per-query wrapper allocation. Pass 2 retype is structurally cheap *because Pass 1 paid the structural cost up-front*; if Pass 1 had skipped the Borrow<str> addition, Pass 2 would be substantially heavier.

**What this commit does NOT close.** Pass 2 milestone stays PLAY (Commit 1 doc-pass is within-milestone). M6 (new) Node admin write path stays unblocked-but-not-selected (parallel-eligible Block 4 verb-by-verb walks remain available for Chat Claude + Joe at session-open selection-time). XGID Retrofit Passes 3 + 4 + 5 stay PENDING behind Pass 2 close.

**Next-active for Chat Claude.** Standby until Clair's Commit 2 closes affirmatively at Joe-lock checkpoint #3. Parallel-eligible items if Joe selects: M6 (new) Block 4 verb-by-verb walks at `docs/xgen_node_admin_ops_design.md` §6 (~35 verbs across 7 categories).

---

## 🟢 (historical, superseded by Commit 1 ✅ above) PLAY — XGID Retrofit Pass 2 implementation runbook SHIPPED at J-124

**XGID Retrofit Pass 2 implementation runbook SHIPPED at J-124 (2026-05-27).** Runbook landed at `tasks/XGID_RETROFIT_PASS_2_IMPL.md` Status: ACTIVE v1.0 (~43 KB, eight sections + §4a contingent). Sibling-in-shape to topo-sort + persistence-amendment + Pass 1 runbook precedents (eight-section shape + §7 discipline-notes + precedent-departure self-defense at §7.1) but lighter at ~43 KB vs Pass 1's ~65 KB vs trilogy's ~80-100 KB per Pass-internal-consistency framing.

---

## 🟢 (historical, superseded by Pass 2 design close above) PLAY — XGID Retrofit Pass 1 milestone CLOSED at J-122; standby for next-milestone selection

**XGID Retrofit Pass 1 milestone CLOSED at J-122 (2026-05-26, this commit).** Seven atomic commits on `main`: Commit 1 `403ef3f` canonical-form module move; Commit 2 `8a94dee` convenience constructors on flavour wrappers; Commit 3 `75e81b4` xgen-common data-structure retypes (+5 invariance tests A–E); Commit 4 `774fe9d` xgen-core data-structure retypes (lib-clean per Path A Joe-lock); Commit 4a `4895446` xgen-core test-fixture projection sweep (split from Commit 4 per Joe-lock); Commit 5 `096162e` Appendix C + Appendix I retypes; this Commit 6 milestone-close commit. Plus J-121 hygiene atom `1dd909e` shipped immediately before Commit 6 closing a lib-level `unused import: NodeXgid` clippy regression surfaced at the Commit 6 verification gate.

**Test count at close.** 489 (xgen-common 34 lib + 8 invariance + xgen-core 447). Negative delta vs the J-119 baseline of 627 is expected and honest per Path A — `xgen-node` + `xgen-client` consume retyped `xgen-core` types and don't build at workspace level until Pass 3 + Pass 4 (the missing ~140 tests live in those crates). Pass 5 close should restore the workspace test count to ≥ 627 plus the +5 Pass 1 invariance tests.

**What unblocks.** XGID Retrofit Pass 2 (xgen-core algorithm-bearing functions; runbook authoring is the next Chat Claude work-shape on the XGID retrofit track). M6 (new) Node admin write path stays unblocked-but-not-selected — opens after Joe selects the next-active milestone at session open. Pass 2 + M6 (new) are both ready for selection; sequencing is Joe's call.

**Two mid-implementation Joe-locks beyond the runbook's authored scope** (full detail at JOURNAL J-122 sub-sections 3 + 4):

1. **`Borrow<str>` additive API on `Xgid` + flavour wrappers** at Commit 4 implementation kickoff. Enables `HashMap<NodeXgid, V>::get(&str)` and sibling Borrow-driven lookups without per-query wrapper allocation. Soundness verified — derived `Hash` + `PartialEq` on `Xgid` forward to inner `String` / `str`, hash-consistent with `&str` per std docs. Preserves the newtype's flavour discipline (no `Deref<Target = str>`). Locked over per-site explicit-wrap-with-comment churn at hundreds of lookup sites. Flagged-not-promoted as candidate D-NNN per D-069 (one instance).
2. **Commit 4 → Commit 4 + Commit 4a split** at Commit 4 implementation halfway. The runbook's "data structures only" framing did not anticipate the ~296 test-fixture errors that surfaced when xgen-core lib retypes completed. Joe-lock locked the split rather than absorbing the test sweep into Commit 4 — preserves D-074 atomic discipline + gives Commit 4a its own clean scope. Discipline implication for next sibling milestone runbook author: runbook estimates for "data structures only" scope should include an explicit test-fixture-sweep estimate alongside the lib retype estimate. Flagged-not-promoted as candidate D-NNN per D-069 (one instance).

**Verification at close.** `cargo test -p xgen-common -p xgen-core`: 489 tests pass (34 + 8 + 447, 0 failed, 0 ignored). `cargo clippy -p xgen-common -p xgen-core --lib --all-features -- -D warnings`: clean. `cargo clippy -p xgen-common -p xgen-core --tests -- -D warnings`: clean. `cargo build --workspace`: deliberately broken per Path A — downstream Passes 2/3/4 own the xgen-node + xgen-client retype work. **J-NNN freeze guardrail** (J-108 codification): `grep -rn 'J-NNN' . --include='*.rs' --include='docs/*.md' --include='tasks/*.md'` returns ZERO matches post-staging.

**"Honest longer work over fast shortcuts" — first recurrence within Pass 1 scope at J-121** (hygiene atom catching the lib-level `unused import: NodeXgid` regression rather than papering it over with `#[allow]` or shipping with the warning intact). Count NOT incremented at this Commit 6 milestone-close commit (close-event-not-recurrence-event sibling-shape to J-101/J-108 milestone-close framing). Pass 1 milestone-scope final count: one recurrence.

**Layered-B3 audit answer per runbook §6.7 DoD**: no layered surface emerged — Pass 1's scope is data-structure shape, not algorithm validation; validators (`is_dag_root_type`, `validate_dag_structure`) consume the typed fields as `&str` projections through `Borrow<str>`, naturally type-clean. Sibling-shape to the persistence-amendment milestone close (J-108) which named layered-B3 closure as a load-bearing audit dimension; J-108 found one such surface; J-122 finds zero.

**Track 1 (Chat Claude + Joe): standby for next-milestone selection.** No active work until Joe picks next-active. Both Pass 2 + M6 (new) are ready and sized similarly for a Clair work session.

**Track 2 (Clair): stood down** until Joe picks next-active milestone.

**Entry point for next session: this CLAUDE.md PLAY block + JOURNAL J-122 entry first per Rule 0**, then whatever document Joe pointed the session at for milestone selection.

---

## ✅ DONE-IN-FLIGHT — Phase 9 Commit 3b-3 SHIPPED at J-112 (Compound C2 against extended harness); Commit 3b-3-pre SHIPPED at J-111 (harness extension + G2 retrofit); Commit 3b-2-equivalent SHIPPED at J-110

**Phase 9 Commit 3b-3 SHIPPED 2026-05-24 at J-112** in a five-file atomic commit per D-074 + Lock #3 per-commit cadence. Single new test file `xgen-node/src/tests/phase9_compound_c2_anti_transitivity_at_load.rs` (~280 lines, one test `three_node_anti_transitivity_at_load_100_messages`) implementing Compound C2 (F-5 anti-transitivity under push queue depth) against the J-111 extended harness. Three-Node setup mirrors Scenario 2 (A↔B + A↔C federated; B↔C explicitly NOT federated; Alice posts 100 MessageText events in rapid succession).

**Five honesty assertions** per the Pre-Commit-3b-3 lock + module doc-comment:

1. **Load-bearing per-event negative-existence F-5 regression lock** — for every Alice event_id, `node_b.push_attempt_count(event_id, &node_a.node_id) == 0` AND `node_c.push_attempt_count(event_id, &node_a.node_id) == 0`. This is what C2 adds over Sc2: per-event negative-existence catches partial-bypass regressions that Sc2's positive `logs_contain` (at-least-once) cannot.
2. Positive G2 trace via `harness_logs_contain("federation_push_skipped_origin")` (sibling to Sc2 but via harness facility instead of `#[traced_test]`).
3. Destination-side reach — every Alice event arrives on B and C within tighter 60s per-event budget (vs Sc2's 120s).
4. Structural anti-transitivity — B/C have no peer entry for each other (sibling-shape to Sc2).
5. Source-side positive of A's pushes — `node_a.push_attempt_count(event_id, &peer_node_id) >= 1` for each event (counter-wiring sanity; distinguishes "wired correctly" from "trivially zero").

**Workspace test count**: 599 → **600** (+1). 2 of 2 consecutive `cargo test --workspace` runs PASS; pre-existing flakes did NOT fire. Wall-clock ~12s isolation, ~30s workspace (longer under parallelism contention is acceptable per Sc2's budget pattern).

**Five files in this atomic commit per D-074 + Lock #3 per-commit cadence**:

1. `xgen-node/src/tests/phase9_compound_c2_anti_transitivity_at_load.rs` NEW
2. `xgen-node/src/tests/mod.rs` — one new module declaration
3. `JOURNAL.md` — J-112 entry + header chain
4. this `CLAUDE.md` PLAY block flip + header chain
5. `docs/ROADMAP.md` v1.26 → v1.27 + Past entry + header chain

**D-074 Lock #3 per-commit cadence applies** — not a milestone-close so the milestone-close tally does NOT increment.

**Next-active for Clair**: Phase 9 Commit 3b-4 per `tasks/FEDERATION_PROPAGATION_PHASE_9.md` §3.0a table + §3 Commit 6. Five NodeRuntime-level tests:

- **Scenario 4** — validation asymmetry regression (30 forgery cases: 6 forgery variants × 5 event families per findings §2.4 sub-item E)
- **Compound C5** — validation asymmetry under load (100 mixed valid+forged events)
- **Compound C7** — `continue_from` pagination at boundary (4 test cases: N=999, 1000, 1001, 2000 events)
- **Compound C9** — F-3 drain-time approximation hazard (federation event from defederated peer drains via missing-predecessor path)
- **Compound C10** — Identity-replicate hook serialisation under lock contention (3 concurrent federation peers × 3 concurrent identity-replicates)

These are NodeRuntime-level (direct `dispatch_event` calls without TCP) per Phase 9 §3 Commit 6; they live under `xgen-core/src/tests/` or `xgen-core/src/node/tests/` per Clair's call based on existing xgen-core test organisation.

**Pre-Commit-3b-4 Joe-lock checkpoint #4 MUST trigger before any code lands** per Phase 9 §3.0 — Joe approves Scenario 4's 30-test-case enumeration by name explicitly (largest scope of any single commit in the 3b arc, most likely to surface unanticipated validator-side findings).

**CounterLayer infrastructure reusable for Commit 3b-4 if needed** — the J-111 counter mechanism wired through `apply_federation_push` (xgen-node layer) is available if any 3b-4 NodeRuntime-level test needs cross-Node push-attempt observability. Most 3b-4 tests are likely direct `dispatch_event` checks that don't need it; runbook authoring will surface whether any do.

After Commit 3b-4 ships, Commit 3b-5 (milestone close per D-074) remains in the Phase 9 §3.0a five-commit sequence — that's the milestone-close commit that flips Phase 9 milestone PLAY → DONE.

**What stays paused/pending.** Phase 9 milestone stays PLAY. Federation Event Propagation milestone stays PLAY. M6 (new) + XGID Retrofit Pass 1 stay PENDING.

**Track 1 (Chat Claude + Joe): no active work** until Clair's Commit 3b-4 arc closes. Possible parallel work: M6 (new) Block 4 verb-by-verb walks; future-walk of candidate D-NNN "ingest path invariant encoding" if Joe locks it.

**Track 2 (Clair): pickup at Commit 3b-4** per `tasks/FEDERATION_PROPAGATION_PHASE_9.md` §3 Commit 6.

**Entry point for Clair: `tasks/FEDERATION_PROPAGATION_PHASE_9.md`** (read CLAUDE.md PLAY block + J-112 entry first per Rule 0, then §3 Commit 6 for the five NodeRuntime-level test scopes).

**Entry point for Chat Claude's next session (no active work): standby** until Clair's Commit 3b-4 arc closes; parallel-eligible items above.

---

**Phase 9 Commit 3b-3-pre SHIPPED 2026-05-24 at J-111** in a ten-file atomic commit per D-074 + Lock #3 per-commit cadence. The two-commit shape for Phase 9 Commit 3b-3 is locked at Pre-Commit-3b-3 Joe-lock checkpoint #3 walk (Joe-lock Option Q3-alt-a + six Q-locks at the harness-extension shape walk + three components shipped at this 3b-3-pre commit). Three components:

- **G2 trace-event retrofit** — `apply_federation_push` gains `local_node_id: &str` parameter; all four G2 federation-push trace events (`federation_push_skipped_origin`, `federation_push_sent`, `federation_push_dropped_full`, `federation_push_dropped_unregistered`) gain `local_node_id = %local_node_id` field. Production observability addition (per-Node attribution on federation traces). Type stays `String` per Joe-lock Q1; XGID Retrofit Pass 1 will sweep all four trace fields together.
- **Push-attempt counter on `InProcessNode`** per Joe-lock Q4 — `push_attempts: Arc<StdMutex<HashMap<(String, String), u64>>>` keyed by `(event_id, peer_node_id)`; public accessors `push_attempt_count` + `push_attempts` (returns clone, not reference, because Mutex guard cannot outlive function call); counter Arc registered in process-global `REGISTRY` keyed by `node_id` so the tracing Layer can route increments.
- **Tracing-Layer composition** per Joe-lock Q3-alt-a — `CounterLayer` listens for `federation_push_sent` trace events and increments the matched Node's counter via `local_node_id` field lookup; sibling `LogBufferLayer` buffers each event as text for `harness_logs_contain(needle)`. Per-test installation via `install_harness_subscriber() -> HarnessSubscriberGuard` (NOT global — see honest finding below).

**Honest finding during implementation — `tracing-test 0.2.6 set_global_default` exhaustion.** Initial implementation used `set_global_default` via `OnceLock`. Counter unit tests passed in isolation, failed under `cargo test --workspace`. Cause: `#[traced_test]` macro uses `tracing::dispatcher::set_global_default` via a shared `INITIALIZED: Once` in the tracing-test crate (verified at `tracing-test-macro-0.2.6/src/lib.rs:67-86`); once any `#[traced_test]` test fires in a workspace process, `set_global_default` is one-shot exhausted. Resolution: switched to per-test `tracing::subscriber::set_default` returning a `DefaultGuard` that auto-drops at end of test scope (scoped per-thread). Tests that read the counter call `install_harness_subscriber()` as first line + bind to a named guard like `_sub` (NOT `_` which drops immediately). Tests using `#[traced_test]` continue unchanged — different mechanisms operating on the same dispatcher slot at different scopes.

**Three counter unit tests added** at `phase9_harness::counter_unit_tests`:
- `counter_starts_empty_on_fresh_node` — sanity baseline (no subscriber install required for static-value reads).
- `counter_increments_on_federation_push_sent_for_matching_local_node` — load-bearing unit test for Layer routing. Asserts A's counter for `(alice_msg_id, B.node_id)` increments to 1 within 500 ms AND B's counter for `(alice_msg_id, A.node_id)` stays at 0 (F-5 short-circuit on B).
- `log_buffer_captures_federation_push_skipped_origin_trace` — unit test for LogBufferLayer; dispatches event on B with ReceivedViaFederation origin, asserts `harness_logs_contain("federation_push_skipped_origin")` within 500 ms.

**Q3 architectural-gap retrospective** (sub-section 3 of J-111). At implementation start, `grep -rn 'apply_federation_push(' xgen-node/src/` (D-077 backward-coherence audit before signature change) surfaced four production call sites + three test call sites. The original Q3 lock's "wrapper layer in phase9_harness where apply_federation_push calls are routed through InProcessNode" reading covered only ONE of those seven sites (the `submit_locally` wrapper). Without the grep + willingness to stop and surface the gap, the implementation would have produced a fabricated F-5 regression lock — the counter would have only recorded A's outbound pushes, missing B's hypothetical leak entirely. Joe walked four resolution options; locked Q3-alt-a (tracing-Layer-fed counter + emitter-field G2 retrofit). Sibling-shape origin pattern to D-077 itself (J-107 cross-milestone Phase 7 B3 amendment dependency surfaced at Clair's Commit 2 forcing Y-lock revert) — both instances: lock made on best-available info at lock time; implementation surfaced an unanticipated dimension; lock amended rather than papered over.

**Workspace test count**: 596 → **599** (+3). 2 of 2 consecutive `cargo test --workspace` runs PASS; pre-existing flakes (precedence env-var race; `reconnect_with_existing_tip_small_delta_delivered`) did NOT fire.

**Ten files in this atomic commit per D-074 + Lock #3 per-commit cadence**:

1. `xgen-node/src/federation_session.rs` — G2 retrofit (parameter + 4 trace fields)
2. `xgen-node/src/app.rs` — 3 production call-site updates (pass `&home_node_id`)
3. `xgen-node/src/tests/federation_push_integration.rs` — 3 test call-site updates (pass `&pubkey_uri(&local_key)`)
4. `xgen-node/src/tests/phase9_harness.rs` — counter + Layers + installer + 3 unit tests (~250 lines added)
5. `xgen-node/src/tests/phase9_drop_and_recover.rs` — 1 call-site update
6. `xgen-node/src/tests/phase9_three_node_anti_transitivity.rs` — 1 call-site update
7. `xgen-node/src/tests/phase9_two_node_smoke.rs` — 1 call-site update
8. `JOURNAL.md` — J-111 entry + header chain
9. this `CLAUDE.md` PLAY block content updated + header chain
10. `docs/ROADMAP.md` v1.25 → v1.26 + Past entry + header chain

**D-074 Lock #3 per-commit cadence applies** — not a milestone-close so the milestone-close tally (eleventh at J-108, twelfth at J-109) does NOT increment. JOURNAL inclusion satisfies D-074's per-commit JOURNAL discipline unambiguously.

**Next-active for Clair**: Phase 9 Commit 3b-3 — Compound C2 test against the extended harness, single new file `xgen-node/src/tests/phase9_compound_c2_anti_transitivity_at_load.rs`. Three sub-assertions per locked shape: (i) tighter destination-side timing budget per event; (ii) positive G2 assertion `harness_logs_contain("federation_push_skipped_origin")` (sibling to Sc2's `logs_contain` but using harness facility); (iii) negative assertion `b.push_attempt_count(event_id, &node_a.node_id) == 0` for all 100 Alice event_ids — the load-bearing F-5 regression lock now actually catchable thanks to Component 3's CounterLayer routing.

**"Honest longer work over fast shortcuts" count inherited at eighth from J-104**, NOT incremented at this per-commit cadence ship.

**Two discipline data points recorded** for next sibling milestone runbook author (J-111 sub-section 7): (a) tracing-test global-default exhaustion forbids `set_global_default` composition — use per-test `set_default` with explicit guard binding; (b) Q3 architectural-gap detection via D-077 backward-coherence audit at implementation time prevented a fabricated F-5 regression lock.

---

**Phase 9 Commit 3b-2-equivalent SHIPPED 2026-05-24 at J-110** in a seven-file atomic commit per D-074 + Lock #3 per-commit cadence. Four new tests added across two new modules at `xgen-node/src/tests/`:

- `phase9_unknown_signer_first_contact.rs` — Scenario 5 — one test, `unknown_signer_first_contact_releases_on_identity_arrival_hook`. Owns F-10 (Phase 6 HeldPending generalisation). Bob's event from federated peer X (where Bob's Identity is NOT yet on B) goes HeldPending on the F-10 unknown-signer trigger; the production identity-arrival hook (`register_identity` → `drain_pending_by_identity` serial under the same runtime lock, mirroring `xgen-node/src/app.rs::handle_identity_replicate_msg`) drains the buffer within a 100 ms wall-clock distinguishing window (sub-100 ms proves the synchronous hook fired vs the 5 s sweep). Bob's event then lands in B's DAG within 200 ms.
- `phase9_federation_relationship_rejection.rs` — Scenario 6 + Phase 7.5 §5 + narrowness regression — three tests:
  - `unfederated_peer_event_defers_via_held_pending_and_recovers_on_federation_add` — main F-3 defer + recovery; Alice's room_create pushed by unfederated peer X defers via HeldPending (Phase 7.5 §6 held-not-bypassed posture); X's subsequent bootstrap `state.federation_add` via federation (Lock B1 + B3 cooperate; D-075 vantage derivation puts the drain pair on (X, S)) fires `dispatch_event` Step 7's `drain_pending_by_federation_relationship` hook; the buffered room_create re-dispatches and lands in B's DAG within 200 ms.
  - `extended_skip_set_does_not_trigger_f3_deferral` — Phase 7.5 §5 extended-skip-set negative assertion across `StateFederationAdd` + `StateSpaceCreate` + `StateDmSpaceCreate` (the discriminator is "creates the Space it references"; structural assertions confirm none lands on the F-3 third-trigger buffer).
  - `room_create_still_triggers_f3_deferral_narrowness_regression` — narrowness regression for the skip set (state.room_create is NOT in it; a regression that widened the set would silently weaken F-3 for room_create).

**Workspace test count**: 592 → **596** (+4). 2 of 2 consecutive `cargo test --workspace` runs PASS; pre-existing flakes (precedence env-var race; `reconnect_with_existing_tip_small_delta_delivered`) did NOT fire.

**Discipline note — cross-crate trace-event assertion gap, per Lock #2 honesty.** Findings v1.2 §2.6 sub-item C step 3 names the G2 `event = "f3_reject"` trace event with `disposition = "held_pending"` as a load-bearing observability surface; the trace fires from `xgen-core/src/node/runtime.rs:537` at warn-level. `tracing-test 0.2`'s per-crate scope (default `xgen_node=trace`) does NOT capture cross-crate events from `xgen-core` — `logs_contain("f3_reject")` returns false from an xgen-node test even when the trace fires. **Resolution**: trace assertions dropped, replaced with stronger structural assertions (`DispatchOutcome::HeldPending` outcome + `PendingBuffer::pending_federation_relationship_count() == 1` on the target Space — a regression that broke F-3 buffering would fail both). Trace-event regression coverage falls to Phase 7.5 NodeRuntime unit tests at `xgen-core/src/node/runtime.rs:1486+`. Documented in module doc-comment §4 of `phase9_federation_relationship_rejection.rs`. Promoting `f3_reject` capture to the deployment level would require structural surfaces requiring Joe-lock per HANDOFF `tasks/HANDOFF_PHASE_9_COMMIT_3B_2.md` §7 Trigger 1/3. Deliberately not done at this commit.

**Recovery-path setup discovery worth recording.** Phase 7 B3 amendment (J-088) skips F-4 step 11 (sender registration + sender membership) ONLY for federation_add via the federation channel; locally-submitted federation_add enforces full validation. First attempt at the Scenario 6 recovery used `submit_locally` for the bootstrap federation_add — Step 11 rejected because the Node's pubkey wasn't a Space member. Restructured to dispatch via `ReceivedViaFederation` with X as the wire-authenticated peer (X signs its own federation_add per F-1a tip-exchange shape; D-075 vantage derivation puts the (peer=X, space=S) pair on the drain hook), with X's Node-pubkey pre-registered (modelling production F-10 identity-replication). B3 skip kicks in, Step 7 fires the drain hook, the previously-buffered room_create lands in B's DAG.

**Seven files in this atomic commit per D-074 + Lock #3 per-commit cadence**:

1. `JOURNAL.md` — J-110 entry chained ahead of J-108 + header chain
2. `xgen-node/src/tests/phase9_unknown_signer_first_contact.rs` NEW
3. `xgen-node/src/tests/phase9_federation_relationship_rejection.rs` NEW
4. `xgen-node/src/tests/mod.rs` — two new module declarations
5. `tasks/HANDOFF_PHASE_9_COMMIT_3B_2.md` Status ACTIVE → COMPLETED v1.1 + header chain
6. this `CLAUDE.md` PLAY block flip + header chain
7. `docs/ROADMAP.md` v1.24 → v1.25 — small Past entry for Commit 3b-2 ship + header chain

**D-074 application — Lock #3 per-commit cadence**: not a milestone-close so the milestone-close tally (eleventh at J-108, twelfth at J-109) does NOT increment. JOURNAL inclusion satisfies D-074's per-commit JOURNAL discipline unambiguously.

**"Honest longer work over fast shortcuts" — count inherited at eighth from J-104**, NOT incremented at this per-commit cadence ship (sibling-shape to J-108 / J-109 close-event-not-recurrence-event framing).

**Phase 9 Commit 3b-3 next-active scope** per `tasks/FEDERATION_PROPAGATION_PHASE_9.md` §3.0a table + §3 Commit 5: Compound C2 (F-5 anti-transitivity under push queue depth). Setup mirrors Scenario 2's `phase9_three_node_anti_transitivity.rs` harness pattern with a queue-depth dimension layered on. Pre-Commit-3b-3 Joe-lock checkpoint #3 may trigger if the harness setup requires divergence from Scenario 2's pattern (e.g. queue-depth measurement instrumentation). Single new file expected: `phase9_compound_c2_anti_transitivity_at_load.rs`. After Commit 3b-3 ships, Commit 3b-4 (NodeRuntime-level Scenario 4 + C5/C7/C9/C10) and Commit 3b-5 (milestone close per D-074) remain in the Phase 9 §3.0a five-commit sequence.

**What stays paused/pending.** Phase 9 milestone stays PLAY. Federation Event Propagation milestone stays PLAY. M6 (new) + XGID Retrofit Pass 1 stay PENDING.

**Track 1 (Chat Claude + Joe): no active work** until Clair's Commit 3b-3 arc closes. Possible parallel work: M6 (new) Block 4 verb-by-verb walks; future-walk of candidate D-NNN "ingest path invariant encoding" if Joe locks it.

**Track 2 (Clair): pickup at Commit 3b-3** per `tasks/FEDERATION_PROPAGATION_PHASE_9.md` §3.0a table.

**Entry point for Clair: `tasks/FEDERATION_PROPAGATION_PHASE_9.md`** (read CLAUDE.md PLAY block + J-110 entry first per Rule 0, then §3.0a table + §3 Commit 5 for Compound C2 scope).

**Entry point for Chat Claude's next session (no active work): standby** until Clair's Commit 3b-3 arc closes; parallel-eligible items above.

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
