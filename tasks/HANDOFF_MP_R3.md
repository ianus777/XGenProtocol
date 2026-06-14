# HANDOFF — Multiparty-tests MP-R3 (capstone) + at-completion ledger deliverable

> **Status**: COMPLETED  
> Version: 1.3  
> Date: Jun 2026  
> **Last updated**: 2026-06-14  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. What this file is

A cross-session HANDOFF note for the **Multiparty-tests** milestone. It survives until the whole
milestone closes (after MP-R3). Read at session-open per the mandatory reading order (CLAUDE.md PLAY
→ latest JOURNAL → ACTIVE HANDOFF notes in `tasks/` → pointed doc). Two jobs:
1. The **immediate pickup** — open MP-R3 (the capstone round).
2. The **standing deliverable** — produce the consolidated R1+R2+R3 ledger at FULL completion.

---

## 1. State at handoff (J-348)

- **MP-R1 ✅ CLOSED (J-340)** — deterministic correctness floor. Criterion: all-green-except-MP-C-06
  (→M10), MP-C-07 harness-green-with-boundary.
- **MP-R2 ✅ CLOSED (J-348)** — scale + real-clock. Criterion: all-green-except-{MP-C-16, MP-A-01(ii)},
  both R3-routed. Spawn-scale floor = MP-C-05 GREEN to 64 clients (no break-point).
- Latest JOURNAL entry: **J-348**. Latest doc versions: `MP_findings.md` v1.17, ROADMAP v3.38,
  `MULTIPARTY_TEST_MATRIX.md` v1.19.
- The Multiparty-tests milestone (unnumbered, `docs/ROADMAP.md`) **stays 🟢 PLAY** — R3 is still ahead.
  Only the R1 and R2 sub-passes are ✅ within it.

---

## 2. Immediate pickup — MP-R3 (capstone)

Next-active. The capstone round: max the box bears (~1,562-process ceiling, chaos overlay stacked),
inheriting the loop-to-green BOUNDED-gate rerun character (R1 J-322 → R2 J-344 → R3). Opens its own
**D-071 Phase-0** (Clair's seat) — ground first, no code until the runbook is Joe-locked.

**Named inbound dependencies (must be on the R3 Phase-0 radar):**
- **MP-F11** — regular-Space content catch-up onto a late-federating third node, F-3 gated.
  Mechanically MP-F1b/Design-Z (D-091 invariant E + the repopulate hook +
  `drain_pending_by_federation_relationship`) — solved for DMs, needs generalizing to a regular Space
  late-federating onto a third node. MP-A-01(ii) is its first witness row. (J-333 lesson: an
  unconditional F-3 skip would be a hole — F-3 changes are non-trivial.)
- **MP-F13** — Space `home_node` holds a WS URL, not a node pubkey id (NodeXgid contract violation;
  J-278 / F1B-D5 family). Root = the client only ever learns the node's WS URL, never its pubkey id.
  MP-C-16 is its first witness row. Same root as the production identity→home-node discovery arc.
- **MP-A-08** — partition + reconnect storm (always was R3; orchestrator link control).
- **MP-A-06** — equivocation / fork (re-routed R2→R3; needs a two-node / multi-target injector +
  convergence-on-winner oracle — the same multi-node-adversary class as MP-A-08).

**Also relevant to R3 scope (from the R2 record):** MP-A-07 flooding intensity *curve* (the curve, not
the liveness witness, → R3); residents-multiplexing (deferred to R3 at the MP-R2 design lock).

**Standing, NOT R3 (do not pull in):** MP-C-06 re-home → M10. MP-F6 (swallowed apply-error) → M10.
MP-F12 (departed-signer re-dispatch) → its own home (peer/identity-discovery space). Production
identity→home-node discovery (F1B-D5, now joined by MP-F13) → its own arc.

---

## 3. THE STANDING DELIVERABLE — consolidated R1+R2+R3 ledger at FULL completion

**Joe-directed (2026-06-11); DELIVERED at the MP-R3 close (J-356, 2026-06-12).** With MP-R1, MP-R2,
and MP-R3 all green/closed, here is the consolidated ledger: every scenario row (`MP-C-##` + `MP-A-##`)
across all three rounds with its FINAL status, plus the complete findings table (`MP-F#`).

**Completion legend:** ✅ GREEN/witnessed · ⏸️ DEFERRED → M10 (set aside on a named future-milestone
dependency; **not a defect**; re-run when that dependency lands). The in-process states (🔴 RED, 🚧
BLOCKED, PENDING) were correct *during* the rounds; at completion every row is either ✅ or ⏸️.

#### Cooperative family (`MP-C-##`)

| ID | Scenario | Round | Status | Resolution / note |
|---|---|---|---|---|
| MP-C-01 | Multi-client local fan-out | R1 | ✅ | single-node fan-out; both views agree, both posts present |
| MP-C-02 | Invite & join | R1 | ✅ | true cross-node A↔B; G-6 bootstrap (MP-R1-D1a) |
| MP-C-03 | Concurrent send under conflict | R1 | ✅ | cross-node; both retained + converge (byte-order = M8) |
| MP-C-04 | Federation topology, transitive path (3-node) | R2 | ✅ | R2 RUN (J-344) |
| MP-C-05 | Sustained n×n chat (scale floor) | R2→R3 | ✅ | R2: 64-client floor; R3: 8 rungs 8→64, break_point=None |
| MP-C-06 | Identity re-home | R1 | ⏸️ **DEFERRED → M10** | dep: keypair-relocation + re-home-notify; **not yet run** (unauthorable today); not a defect |
| MP-C-07 | DM private space across nodes | R1 | ✅ *(boundary)* | MP-F1a + MP-F1b/Design-Z; no production witness (F1B-D4), discovery deferred to M10 |
| MP-C-08 | Multi-room + per-room overrides | R1 | ✅ | `room_update` verb (J-338); per-room Deny enforced + converged |
| MP-C-09 | Ban → converge → post-rejected | R1 | ✅ | `ban` verb (J-337); assert-the-reject 4000 |
| MP-C-10 | Leave & rejoin | R1 | ✅ | cross-node lifecycle round-trip converges |
| MP-C-11 | Membership churn under load | R2→R3 | ✅ | MP-F7 fixed (J-348); R3 4 rungs, break_point=None |
| MP-C-12 | E2E content-blindness | R2 | ✅ *(boundary)* | injector-path node-blindness (ciphertext-only); client-decrypt half D3-gated |
| MP-C-13 | Thread create/resolve/archive | R1 | ✅ | `thread×3` verbs (J-339); transcript-asserted + ChangeInfo-teeth witness |
| MP-C-14 | 4–5 node star + mesh | R2→R3 | ✅ | **this close** — MP-F14 fixed; R3 RUN#1 RED→fix→stable 5/5 |
| MP-C-15 | Node restart mid-chat + replay | R2 | ✅ | 160→160 Spaces off disk (J-344) |
| MP-C-16 | Live space migration | R2→R3 | ⏸️ **DEFERRED → M10** | dep: MP-F13 home-node discovery (J-278); **ran to its expected reroute reason** (`MIG_6010`); not a defect |

#### Adversarial family (`MP-A-##`)

| ID | Scenario | Round | Status | Resolution / note |
|---|---|---|---|---|
| MP-A-01 | Expired-invite federation replay | R1 / R3 | ✅ | (i) PASS R1; (ii) GREEN R3 (MP-F11 resolved, 3/3) |
| MP-A-02 | Over-ceiling invite at submission | R1 | ✅ | MP-F5 assert-the-reject (3045) |
| MP-A-03 | Tier-gate join refusal | R1 | ✅ | auth-tier verb + MP-F5 (3030) |
| MP-A-04 | Unauthorized / non-member send | R1 | ✅ | MP-F5 (4000) |
| MP-A-05 | Signature / identity forgery (wire) | R1 | ✅ | injector; step-12 sig check |
| MP-A-06 | Equivocation / fork (wire) | R3 | ✅ | re-routed R2→R3; 4/4 convergence-on-winner |
| MP-A-07 | Flooding / DoS (volume) | R2→R3 | ✅ | R3 flood curve, break_point_pace=None |
| MP-A-08 | Partition + reconnect storm | R3 | ✅ *(boundary)* | 4/4 relationship-heal; reconnect-deadlock half routed (R3-D2) |
| MP-A-09 | Duplicate-event_id replay / dedup | R1 | ✅ | MP-F3 fixed; exactly-once fan-out |
| MP-A-10 | Causal gap / missing-parent | R1 | ✅ | HeldPending, never applied out of order |
| MP-A-11 | Oversized payload | R2 | ✅ | node live, no OOM (J-344) |
| MP-A-12 | Malformed / truncated frame | R1 | ✅ | parse-reject; node stayed live |
| MP-A-13 | Anti-transitivity probe | R2 | ✅ | pairwise model holds (J-344) |
| MP-A-14 | Ban-evasion via new identity | R1 | ✅ *(half + breadcrumb)* | same-identity refusal green; fresh-identity = pseudonymity by design |
| MP-A-15 | Clock-skew timestamp (wire) | R1 | ✅ | MP-F2; 3046 on wire |
| MP-A-16 | Forged invite "never issued" | R1 | ✅ | injector; HeldPending, grants nothing |
| MP-A-17 | Wrong-space_id confusion | R1 | ✅ | MP-F5 (4000), no cross-space leak |
| MP-A-18 | Connect / disconnect storm | R2→R3 | ✅ | R3 100 churned conns, node live |
| MP-A-19 | Slow-loris / held connections | R2 | ✅ | node not exhausted (J-344) |
| MP-A-20 | Privilege escalation | R1 | ✅ | role-gate path; MP-F5 (4000) |
| MP-A-21 | Stale / rollback MLS commit | R2 | ✅ | no epoch regression (J-344) |

#### Findings (`MP-F#`)

| ID | What | Final state |
|---|---|---|
| MP-F1a | DM facet-2 (message.text not transcript-observable) | ✅ RESOLVED (J-328) |
| MP-F1b | DM facet-1 (membership-driven DM federation) | ✅ SHIPPED/CLOSED (J-333, Design Z) |
| MP-F2 | Reject-path wire-code (3046 / generic-4000) | ✅ RESOLVED (J-324); residual **MP-F2-followon** → M10 (see §3.1) |
| MP-F3 | Duplicate re-fan-out | ✅ RESOLVED (J-326) |
| MP-F4 | `get_dag_tips` true-frontier anchor | ✅ RESOLVED (J-331) |
| MP-F5 | Assert-the-reject oracle migration | ✅ RESOLVED (J-336) |
| MP-F6 | Swallowed join apply-error / no `banned` pre-check | ↪ routed → M10 (open) |
| MP-F7 | Churn / leave→rejoin convergence | ✅ RESOLVED GREEN-on-rerun (J-348) |
| MP-F8 | Migration not aicontrol-exposed | ✅ CLOSED (J-347) |
| MP-F9 | Late-fed in-session ordered identity delivery | ✅ terminal (J-346) |
| MP-F10 | Director deadlock | ✅ terminal (J-346) |
| MP-F11 | Regular-Space late-third-node F-3 catch-up | ✅ RESOLVED (R3 RUN#1, J-351) |
| MP-F12 | Departed-signer breadcrumb | ↪ routed to own home (open) |
| MP-F13 | home_node WS-URL vs pubkey-id | ↪ routed → M10 (open; the dependency MP-C-16 is deferred on) |
| MP-F14 | Regular-Space pre-join-message backfill | ✅ RESOLVED GREEN-on-rerun (**this close**, J-356) |

#### Net summary

**37 scenarios** — **35 ✅ green/witnessed** (4 carry a named boundary: MP-C-07, MP-C-12, MP-A-08,
MP-A-14), **2 ⏸️ DEFERRED → M10** (MP-C-06 identity re-home; MP-C-16 live migration / MP-F13 home-node
discovery) — both named future-milestone dependencies, re-run when those land, **not defects**.
**Close criterion: all-green-except-{MP-C-06, MP-C-16}, both deferred to M10.** Findings: **14 total**
— 11 resolved/closed/terminal + MP-F14 just resolved (this close); 3 routed-open (MP-F6 → M10, MP-F12
own home, MP-F13 → M10). **What the green campaign certifies (honest):** the deterministic correctness
floor (R1) + the spawn-scale floor and every drivable protocol property under real-clock (R2,
MP-C-05 to 64 clients) + the box-measured process wall (~1384) under sustained stacked chaos +
multi-node-adversary properties (R3) — NOT multiplexed logical scale (R3-D1, routed), NOT the
transport-level reconnect-deadlock half (R3-D2, routed), NOT identity→home-node discovery (M10).

**ADDENDUM (J-375, M10.5+M10 close) — the 2 ⏸️-M10 rows discharged → 37/37 ✅, 0 deferred.** The ledger above was delivered at the MP-R3 close (J-356) with **MP-C-06** and **MP-C-16** as the two `⏸️ DEFERRED → M10` rows. **M10.5 closed exactly those two:**
- **MP-C-16 → ✅** at M10.5 C1a (`55a308e`, J-374): the M10.4 `AuthOk.node_id` namespace fix proven end-to-end on real binaries (home_node-flip-on-both + state-equality, stable 3/3); **MP-F13 → RESOLVED**.
- **MP-C-06 → ✅** at M10.5 C1c (`fd630fa`, J-375): full-mesh A↔B↔C replicate-convergence re-home witness (stable 3/3); the deferred `home_changed` broadcast proved a version-stale-by-construction no-op (CP-4 re-lock, J-374) — MP-C-06 converges via the existing re-registration `push_identity_to_peers`, so the broadcast was dropped, not built.
- **MP-F6 → RESOLVED** at M10.5 C1b (`7a2ff89`, J-374): the dispatch-level `banned` pre-check (the `MP-F6 → routed M10` row above is now closed).

**Net at full close: 37 scenarios — 37 ✅, 0 deferred** (the prior "35 ✅, 2 ⏸️-M10" tally is fully discharged). **Findings: MP-F6 + MP-F13 now RESOLVED** (were routed-open → M10); **new finding MP-F16** (federation_initiate advertises `config.node.listen` raw, not `--port`-corrected `effective_endpoint`; low-sev, routed to a future identity-replication/federation-endpoint arc) surfaced + routed during the M10.5 C1c loop-on-fault. Still routed-open: MP-F12 (departed-signer, own home); MP-F2-followon (7 unmapped wire-codes, M10-era). **M10 (Auth Module Reference Set) CLOSED at J-375.** This HANDOFF stays COMPLETED; the addendum records the downstream discharge for ledger fidelity.

### 3.1 Breadcrumb sweep at the close (final dispositions — nothing closed silently)

- **MP-F2-followon** — the 7 unmapped event-validation wire-codes (the `reject_code=4000`
  pinned-to-observed family; `tasks/MP_findings.md` ~L170). Wire-code hygiene, NOT a multiparty
  protocol gap → **re-homed to M10 explicitly** (the auth-module / wire-code pass). FINAL: routed-open
  → M10.
- **D-091 mis-file tidy** — the J-340 housekeeping note (invariant E members→parties promotion home).
  Verified at the close: the D-091 promotion is recorded in `DECISIONS.md` and cited from MP-F1b
  (J-333) + MP-F11 (J-346); the mis-file is reconciled. FINAL: **done** (no open action).

Full carried/standing register: §2 above + `tasks/MP_findings.md` (findings MP-F1…F14) +
`docs/tests/MULTIPARTY_TEST_MATRIX.md` §6.

---

## 4. Discipline reminders (unchanged)

- Surface-and-route (D-065 / D-084); **pin-by-observation BEFORE routing** (three falsifications across
  the MP-R2 stretch earned this bar — a DECISIONS-promotion candidate, alongside the round-close
  discipline, Joe's call).
- No self-close. Clair's code + arc-docs commit FIRST (pushed), then Chat's doc-bridge as a SEPARATE
  commit. Joe pushes (PowerShell: `cd` → explicit `git add <file>` per file → `git status` →
  `git commit` multi-`-m` → `git push`). Chat never pushes.
- Mandatory `.md` header on every file (Status / Version / Date / Last updated [date-only YYYY-MM-DD] /
  Language / Author JozefN / Credits / License BSL 1.1), two trailing spaces per `>` line.
- ALWAYS `Filesystem:*` (or Windows-MCP) for `E:\` — NEVER `create_file` (Claude sandbox `/mnt/`).
  Verify new files via `get_file_info`. `edit_file` needs exact char-level `oldText`.
- GitHub Projects #6 board is **empty** — not a live mirror; the local `.md` files are the sole source
  of truth. No board action needed at closes unless the board is later populated.

---

## 5. Entry point (Rule 0) for the next session

CLAUDE.md PLAY (the J-348 MP-R2-CLOSED head) → JOURNAL J-348 → this HANDOFF → `tasks/MP_findings.md`
(fix-phase note, CLOSED) → `docs/tests/MULTIPARTY_TEST_MATRIX.md` §6 → `docs/ROADMAP.md` Multiparty
node. Then: open MP-R3 Phase-0 (relay to Clair on request).
