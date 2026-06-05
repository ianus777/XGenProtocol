# R2-F06 — Operator-Terminology Correction — Phase-0 Audit
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-05  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. What this is

Phase-0 audit (D-071) for the **R2-F06 fix-arc** — the operator-terminology correction, the
larger of the two open Round-2 findings, next-active before M10. **Doc-only, no code.**

The headline finding reframes the arc: **this is not a semantic-design arc.** The semantics were
locked nine days ago in **D-082** (2026-05-29). R2-F06 is the *execution* of D-082's already-defined
Sense-D rename across the corpus the J-150 sweep did not reach, plus a correction to the Round-2
register's one-line summary, which contradicts D-082.

---

## 2. The governing decision (D-082) — four senses, only one renames

D-082 partitions every occurrence of "operator" into four senses and renames exactly one:

- **Sense A — AI-operator role** (the reserved sense): **keep "operator".** Markers:
  `resolve_operator`, `operator_known`, `StateAiOperator*`, `ai delegate` / `ai revoke`,
  "operator role for an AI Identity". The moderator-parallel role (D-059 / D-064 fall-upward).
- **Sense B — wire field names** (`operator_display_name` in the Node Announcement signing order;
  `bootstrap_info.operator`): **keep verbatim — untouchable** (wire-format invariance D-081 +
  signing byte order).
- **Sense C — infrastructure operator** (Node operator / deployer / custodian / GDPR data
  controller; woven through Appendix D legal language): **keep "operator".** "administrator" is
  explicitly a *poorer* fit for a data controller. Disambiguate **inline** where a line is
  genuinely ambiguous ("Node operator (the entity running the Node)") — never by re-equating to
  owner/admin.
- **Sense D — runtime admin principal** (whoever drives the `--batch` admin write surface): **the
  only rename** → **"administrator"** (prose) / **"admin"** (code identifiers, CLI tokens,
  error-code namespaces, config keys — matching the existing `admin_ops` / `AdminContext` /
  `AdminError` vocabulary).

D-082 §1 is explicit: "operator" MUST NOT be introduced as a *new* owner/admin alias. The
classifier is locked; R2-F06 applies it — it does not re-litigate it.

**Prior coverage (J-150).** The J-150 sweep already applied Sense-D to the M6 admin-ops design doc
(10 hits) + the `xgen_aicontrol_implementation.md` "Space/Room operator actions" mirrors. R2-F06
finishes Sense-D across the *remaining* corpus.

---

## 3. Scope — Joe-confirmed (2026-06-05)

**Write scope = active spec/design docs + live code only.** Frozen historical records are
excluded (no retroactive rewrites): `JOURNAL.md`, `DECISIONS.md`, ARCHIVED docs, `docs/backup/*`,
and COMPLETED/closed `tasks/`. The arc records itself in JOURNAL/ROADMAP/CLAUDE as usual (D-074)
but does not rewrite old "operator" mentions inside them.

**Why this is load-bearing.** Raw corpus grep returns **3,399** doc occurrences; the **active**
doc corpus is **~384**. The 8.8x gap is almost entirely frozen records. Getting scope wrong upward
rewrites history; wrong downward misses live docs. This boundary decides the arc's real size.

### Active-doc corpus in scope (per-file `operator` counts)
| File | count | dominant sense (pre-classification read) |
|---|---|---|
| `docs/xgen_ch3_specification.md` | 105 | mixed A/C/D — the main classification work |
| `docs/xgen_appendix_f_en.md` | 36 | TBD |
| `docs/xgen_ch6_client_design.md` | 35 | A-heavy (AI client operator surface) |
| `docs/xgen_ch2_architecture.md` | 34 | mixed |
| `docs/xgen_appendix_d_en.md` | 32 | **C-heavy (GDPR / data controller) — keep** |
| `docs/ROADMAP.md` | 29 | milestone/feature refs (canonical; updated anyway) |
| `docs/xgen_federation_propagation_design.md` | 28 | TBD |
| `docs/xgen_appendix_i_en.md` | 19 | TBD |
| `docs/xgen_aicontrol_implementation.md` | 15 | J-150 already swept Sense-D mirrors |
| `docs/xgen_ch4_implementation.md` | 14 | mixed |
| `docs/xgen_lifecycle_states.md` | 9 | TBD |
| `docs/xgen_ch1_philosophy.md` | 7 | TBD |
| `docs/xgen_node_admin_ops_design.md` | 6 | residue after J-150's 10-hit sweep |
| `docs/xgen_appendix_e_en.md` | 6 | TBD |
| `docs/xgen_appendix_l_en.md` | 4 | TBD |
| `docs/xgen_appendix_c_en.md` | 3 | TBD |
| `docs/xgen_appendix_g_en.md` · `_j_en.md` | 1 · 1 | TBD |

≈ **384 active-doc occurrences.** Most are A/B/C (keep); the Sense-D rename subset is a fraction.

### Code corpus in scope
**388** raw occurrences across `xgen-common` / `-core` / `-node` / `-client`. Of these:
- **Sense-A** (`AiOperator*`, `resolve_operator`, `operator_known`, "operator role for an AI
  Identity", `operator_delegations`): **~117+ — keep.** (Bigger than the proxy count; the `wire.rs`
  `StateAiOperator*` doc-comments are all Sense-A.)
- **Sense-B** (`operator_display_name`, `bootstrap_info.operator`): **14 — keep, untouchable.**
- **Candidate Sense-D / Sense-C** (the residue: `precedence.rs` "operator passed `--flag`",
  `module.rs` "operator-facing label", `state.rs` "operators detect / reading", `space_local.rs`
  "operator"): a **few dozen**, many of which resolve to **Sense-C** on inspection (observer /
  infra), not D. D-082 §2 already ties code-side "admin" to `admin_ops`/`AdminError`, so most
  admin-surface code is *already* correct — the rename residue is small.

The register's "~133 code (excl. ai_operator)" reconciles as 388 minus the ai_operator cluster; but
within that 133, A/C still dominate, so the genuine Sense-D code set is smaller still. **The risk
is classification accuracy, not volume.**

---

## 4. Findings

- **F06-A1 — the register one-liner is wrong; correct it.** `ROUND_2_AUDIT.md` §3 line 127 / §5
  row R2-F06 say *"old node-custodian sense → owner/admin."* This contradicts D-082: the
  infra-custodian sense (C) **keeps "operator"**, and only the runtime-admin sense (D) renames — to
  **administrator/admin**, never **owner**, and never as an alias (D-082 §1). The arc's first
  edit corrects this row to the D-082 wording.
- **F06-A2 — scope trap (quantified).** 3,399 raw vs ~384 active doc occurrences (§3). Frozen
  records dominate and are out of scope.
- **F06-A3 — Appendix D is Sense-C-heavy → mostly keep.** Its 32 hits are GDPR / data-controller
  language; D-082 §C says keep "operator" there. The audit flags Appendix D as a *low-rename,
  high-disambiguation* file: any genuinely ambiguous line gets the inline facet specifier, not a
  rename.
- **F06-A4 — most of the corpus keeps "operator".** Sense A (the AI-operator role) + B (wire) + C
  (infra/GDPR) are the large majority on both code and doc sides. The Sense-D rename surface is a
  minority — bounded and rule-driven.
- **F06-A5 — no DECISIONS change.** D-082 is the governing decision; R2-F06 executes it. No new
  D-NNN; arc-local decisions are F06-D#.

---

## 5. The crux + the design questions for Phase 1

There is no semantic fork (D-082 settled it). The residual judgment is **classification accuracy**
on the genuinely ambiguous lines — the A-vs-C-vs-D edge cases where context alone doesn't decide.
The Phase-1 design resolves:

- **Q1 — ambiguous-call handling.** Build the per-file A/B/C/D ledger; for each line where the
  sense is not unambiguous from local context, **list it for individual Joe-lock** rather than
  guessing (mirrors F01's "surface, don't guess"). Expected small (the `--batch`-vs-infra "operator
  runs the node" lines are the likely ambiguous cluster).
- **Q2 — inline-disambiguation policy for Sense C.** Confirm the facet-specifier form
  ("Node operator (the entity running the Node)") and whether to apply it proactively or only on
  ambiguous lines (D-082 says the latter).
- **Q3 — code residue confirmation.** Confirm no Sense-D code identifier is load-bearing on a wire
  token / error-code namespace before any code rename (D-082 §2 suggests most is already `admin`).

---

## 6. Proposed roadmap

1. **Phase 0 — audit** (this doc). Scope locked, classifier confirmed, findings surfaced.
2. **Phase 1 — design** (light): produce the per-file A/B/C/D ledger; resolve Q1–Q3; list any
   ambiguous lines for Joe-lock. Lock the rename mechanics (prose→administrator / code→admin).
3. **Phase 2 — runbook + execute**: split by surface — C1 code residue (one writer per file),
   C2 active-doc sweep (per-file, header-bump each), C0 the register one-liner correction (F06-A1).
   Doc-only close flips **R2-F06 ✅** → register Open 1/9 → **M10 unblocked → UI**.

No code this arc beyond the small Sense-D code residue; the bulk is doc edits. No DECISIONS change.

---

## 7. Status & next-active

Audit complete (Joe scope-confirmed 2026-06-05). **Next-active: Phase-1 design** — the per-file
ledger + Q1–Q3 + the ambiguous-line Joe-lock list. No code until the runbook locks.

*Tooling note:* the classification grep (PowerShell MCP) timed out during the first authoring
attempt; the §3/§4 distribution is from per-file counts + representative sampling. The exhaustive
per-line ledger is built in Phase 1/2.

Per Rule 0 + D-065 + D-069 + D-071 + D-074 + D-082 + the two-round audit principle.
