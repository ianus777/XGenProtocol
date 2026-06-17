# R2-F06 — Operator-Terminology Correction — Design + Close (zero-rename)
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-05  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. What this is

Phase-1 design + close for the **R2-F06 fix-arc**, built on the Phase-0 audit
(`tasks/R2_F06_OPERATOR_TERMINOLOGY_AUDIT.md` v1.0) and the governing decision **D-082**. The
classification pass resolved the arc to **zero renames**: the active corpus is already
D-082-compliant. The arc's only edits are two non-rename corrections. Joe confirmed the shape
2026-06-05.

---

## 2. The classification result (grounded)

The per-file A/B/C/D ledger was built by grep + full-context read across all in-scope active
docs (~384 occurrences) and live code (388). The distribution:

- **Sense A — AI-operator role** (`resolve_operator`, `StateAiOperator*`, `ai delegate`/`revoke`,
  the ch3 §3.6.10.6 cluster): the large majority of substantive hits. **Keep.**
- **Sense B — wire field names** (`operator_display_name`, `bootstrap_info.operator`,
  `new_operator_identity_id`, `capability.operator`, `operator_notes`): **keep, untouchable.**
- **Sense C — infrastructure operator** (Node operator / deployer / GDPR data controller —
  the ch3 Auth-Module section, Appendix D legal language, the `federation_policy` /
  `pending_queue` / `node_policy` / `app.rs` code clusters): **keep.**
- **Sense D — runtime admin principal** (`--batch` admin write surface): the only rename target.
  **Already done by J-150** (M6 admin-ops design doc + aicontrol mirrors). **No remaining
  Sense-D in the active corpus.**

**Code identifiers** are already `admin` (`admin_ops`, `AdminContext`, `AdminError`); D-082 §2
is fully realized. `admin_ops.rs:31` even states the rule in-file. The ~90 code "operator" hits
are doc-comment prose, all A/B/C.

---

## 3. The zero-rename finding (why the candidates were not renamed)

The Phase-0 audit named three doc lines as Sense-D candidates (ch6:342, appendix_e:92,
lifecycle_states:141). Full-context reading reversed all three:

- **ch6:342** ("the operator-facing interface for managing a running Node") — ch6 uses
  "operator" *pervasively* for the Console / node-admin actor (`operator command surface` §6.11,
  `operator-configurable` §6.4, the AI operator panel §6.14.4). Renaming line 342 alone would
  call one actor two names *within the same chapter* — introducing inconsistency, not fixing it.
- **appendix_e:92** ("`MAINTENANCE` is operator-initiated") — Appendix E uses "operator"
  consistently throughout (its own design principles: "an operator needs to know"). Same
  inconsistency problem.
- **lifecycle_states.md:141** — identical text, but the file is a **superseded working draft
  slated for deletion** (Appendix E §E.4 relationship table). Out of scope by definition.

Combined with the console-operator keep-ruling (§4), the doc-side Sense-D surface collapses to
zero. This is a legitimate, honest outcome: the prior discipline (D-082 + the J-150 sweep) held;
R2-F06's job was to **verify** that, and correct the one stale record that misstated it.

---

## 4. Joe's rulings (2026-06-05)

- **Console operator = keep-sense.** The Ch1 / ch6 "Console operator" usage (the human-or-AI
  agent driving the command channel — "AI agents as first-class Console operators") is its own
  sense, not the `--batch` admin principal. Recorded as a **D-082 scope refinement** (R2-F06),
  mirroring the existing J-150 audit-refinement. Keep "operator"/"console-operator".
- **AI-client-runner lines = keep "operator"** (ch6:1433/1472/1476). Whoever runs the AI-client
  resident is not the node administrator; labelling them "ai-operator" would re-collide with
  Sense A. Plain "operator" (the AI-client runner) reads correctly.
- **Code doc-comment prose = leave as-is.** Identifiers already `admin`; the comments are
  Sense-C and read fine; churning them adds noise, not clarity.

---

## 5. The two edits (the entire arc)

1. **F06-A1 — register one-liner correction** (`tasks/ROUND_2_AUDIT.md` §3.6 + §5 R2-F06 row).
   The stale text ("repurpose operator → delegated AI-running user; old node-custodian sense →
   owner/admin") **contradicts D-082** (Sense C keeps "operator"; only Sense D renames, to
   administrator/admin, never an owner/admin alias). Corrected to the D-082 wording.
2. **Console-operator keep-sense** — appended to D-082's "Scope — the four senses" subsection in
   `DECISIONS.md` as an R2-F06 audit-refinement (mirroring the J-150 refinement already there).

No code change. No new D-NNN (D-082 governs; arc-local decisions are F06-D#, none cross-arc).

---

## 6. Close

R2-F06 🟪→✅ in the Round-2 register (now Open 1/9 — only R2-F09, D3-gated, remains).
**Next-active: M10 → UI** (R2-F09 is a D3-gated catalogue item, not a UI blocker). Suite
unchanged 1156/0/2 (no code). ROADMAP bumped. Audit + this doc → COMPLETED.

Per Rule 0 + D-065 + D-069 + D-071 + D-074 + D-082 + the two-round audit principle.
