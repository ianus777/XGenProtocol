# M8.5-B — INV Invitee Membership-Bootstrap (implementation runbook)
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-05  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. Framing & reading order

Clair implements the M8.5-B INV bootstrap from this runbook. The design is locked
(`tasks/M8_5_B_INV_BOOTSTRAP_DESIGN.md` COMPLETED v1.1, INV-D1..D6); do not
re-decide locked points — surface drift to Joe instead (D-065).

**Reading order (Rule 0):** CLAUDE.md PLAY → JOURNAL J-272 → this runbook §1–§2 →
design doc INV-D1..D6 → then the per-commit sections. Ground every named symbol
against live code before editing (D-078); the design's `file:line` anchors were
read at HEAD around `cecb5ee`/M8.5-A and may have shifted — re-confirm.

**The goal:** a one-shot invitee, connected to its home node, sources the invite
via a scoped structural fetch, chains its join `prev_events=[invite_event_id]`,
and becomes a member — bounded by a tier-graded `valid_until` (T1=14d live).

## 2. Sequence overview + checkpoints

Two code commits + a doc-only close. Confirm-at-pickup (CP-1..CP-5 from design §9)
are resolved **before** the commit that needs them, each a Joe-lock checkpoint.

| Commit | Scope | Gates |
|---|---|---|
| **C1 (node)** | scoped structural fetch + pending-invite authz + `valid_until` creation-clamp + join-acceptance enforcement + reject code + tests | CP-1, CP-2, CP-3 locked first |
| **C2 (client)** | `ops::join` chain + A2 fix + `ops::invite` `valid_until`/`note` + tests | CP-4, CP-5 locked first |
| **Close** | ch3 schema + scoped-fetch wire shape; audit M85-A1..A4 → resolved; INV-D# promotion eval; JOURNAL | doc-only |

**Joe-lock checkpoints:** **#1** pre-C1 — CP-1 (reject code, confirmed vs live
registry, NO guess — Arc-E lesson), CP-2 (fetch wire mechanism), CP-3 (structural
event-type set). **#2** post-C1 — node enforcement + read-gate verified. **#3**
pre-C2 — CP-4 (cascade compute site), CP-5 (`note` schema reuse). **#4** post-C2 —
end-to-end invitee-join green.

## 3. C1 — node (xgen-node + xgen-core)

1. **Scoped structural fetch (INV-D1, CP-2/CP-3).** New read path serving a
   requester who holds an **unexpired** `pending_invite`: returns only the
   structural set (Space create · Room create(s) · the membership chain incl. the
   invite naming the requester) — **no message content**. Lean wire mechanism: a
   **dedicated request** (leave `collect_sync_history` member-only). Authz =
   pending-invite presence AND `valid_until` not past (INV-D6 read-gate).
2. **`valid_until` creation-clamp (INV-D6).** When an invite is created/relayed,
   resolve `valid_for → node default → protocol default(14d)` bounded by the
   **invitee-tier ceiling** (`assertion_tier_of`; T1=14d the only live row).
   Over-ceiling → **reject at ingest** with wire **`3045 invite_validity_exceeds_max`**,
   never clamp. *(Corrected at C1 close: this is a distinct code from CP-1's
   `3044 invite_expired` — "exceeds max at creation" ≠ "expired at join". The
   three-code split is `1011` bootstrap read-gate refusal (transport) · `3044`
   join-acceptance expiry (identity) · `3045` invite-ingest over-ceiling
   (identity); see the J-273 JOURNAL entry.)*
3. **Join-acceptance enforcement (INV-D6).** On `membership.join` for a pending
   invitee: check `valid_until` vs node clock → CP-1 reject if past; the PG-13
   tier-gate stays. Convergence-neutral (a gate, no `derive_resolved` surface).
4. **Tests:** invitee scoped-fetch returns structural-only (no content); expired
   invite → read refused + join refused (CP-1); in-window join accepted; full
   invite→join→member happy path on the phase9 harness.

**DoD:** workspace green; clippy `-D warnings` both feature sets; the structural
fetch serves no message content (asserted); T1 14d path exercised end-to-end.

## 4. C2 — client (xgen-client)

1. **`ops::join` (INV-D3 + INV-D4).** Source the invite from the scoped fetch;
   set `prev_events=[invite_event_id]`. Fix the `:770` fallback so `Ok(empty)`
   is treated like `Err` (defensive; the invite-chain is the primary path).
2. **`ops::invite` (INV-D5 + INV-D6, CP-4/CP-5).** Stamp `valid_until` via the
   cascade; accept optional individual `valid_for`; carry optional `note`
   (`message.rich` body — reuse the message.rich content shape, do not reinvent).
3. **Bootstrap sequence.** connect home node → scoped fetch → find invite naming
   self → chain + send join → become member on accept.
4. **Tests:** join chains off the real invite id (not create-root); `Ok(empty)`
   no longer yields empty `prev_events`; `valid_until` stamped within ceiling;
   `note` round-trips as a rich body.

**DoD:** workspace green; clippy clean; the M85-A3 concurrency no longer arises
in a production-shaped (non-hand-chained) invite→join.

## 5. Close (doc-only)

ch3: add `valid_until` + optional `note` to the `membership.invite` content
schema; document the scoped-fetch wire shape (its message + the structural set).
Resolve audit M85-A1..A4 (→ fixed/closed). Evaluate INV-D# for DECISIONS
promotion (`valid_until`-as-credential-validity; exposure-graded tier ceiling) —
flagged candidates, Joe decides. JOURNAL close entry. Suite delta recorded.

## 6. Honest posture (carry into every commit)

Only the **T1 (14d)** path is exercisable until trusted Auth Modules exist
(`assertion_tier_of` → 1 for all). The tier-graded ceiling above T1 is
**wired-but-dormant** (PG-13 posture). The T1=14d constant becomes module-derived
(bounded ≤14d) at the Tier-1-auth-rebuild milestone — forward-note in design §4;
do not hardcode assumptions that block that transition.

## 7. Cross-references

Design `tasks/M8_5_B_INV_BOOTSTRAP_DESIGN.md` (COMPLETED v1.1, INV-D1..D6) ·
audit `tasks/M8_5_FINALIZATION_AUDIT.md` §3 (M85-A1..A4) · D-089 (federation
pairwise, sibling synchronize-the-record discipline) · TrustAssertion `valid_until`
ch3 §3.8.4 · PG-13 tier-gate (`assertion_tier_of`) · ch6 §6.9 (compose
substitution, UI build) · `message.rich` (ch2).

Per Rule 0 + D-065 + D-069 + D-071 + D-074 + D-076 + D-078.
