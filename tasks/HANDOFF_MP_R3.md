# HANDOFF — Multiparty-tests MP-R3 (capstone) + at-completion ledger deliverable

> **Status**: ACTIVE  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-11  
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

**Joe-directed (2026-06-11).** When MP-R1, MP-R2, AND MP-R3 are ALL green/closed — i.e. at the close
of the whole Multiparty-tests milestone — Chat Claude produces a **consolidated R1+R2+R3 ledger**:
every scenario row (`MP-C-##` + `MP-A-##`) across all three rounds with its FINAL status, plus the
complete findings table (`MP-F#`). The format is the same as the R1+R2 ledger produced in the J-348
session (cooperative table, adversarial table, findings table, net summary). This is a milestone-close
deliverable — it does NOT exist yet because R3 isn't done. Do not produce it until R3 closes; carry
this obligation here until then. (Also recorded as memory #22.)

### 3.1 Breadcrumb sweep at the close (so nothing closes silently)

At the same R3-close moment, the consolidated ledger gives an explicit FINAL disposition to the two
tracked-but-not-arc-homed breadcrumbs (don't let them close silently):
- **MP-F2-followon** — the 7 unmapped event-validation wire-codes (the `reject_code=4000`
  pinned-to-observed family; `tasks/MP_findings.md` ~L170). NOT R3 scope (wire-code hygiene) →
  re-home to **M10** explicitly in the ledger.
- **D-091 mis-file tidy** — the J-340 housekeeping note. Verify **done-or-routed** and record the
  verdict in the ledger.

Both land in the ledger's findings / net-summary section. Full carried/standing register: §2 above +
`tasks/MP_findings.md` (findings MP-F1…F13) + `docs/tests/MULTIPARTY_TEST_MATRIX.md` §6.

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
