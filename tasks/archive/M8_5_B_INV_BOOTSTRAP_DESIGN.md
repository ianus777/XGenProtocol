# M8.5-B — INV Invitee Membership-Bootstrap (design)
> **Status**: COMPLETED  
> Version: 1.2  
> Date: Jun 2026  
> **Last updated**: 2026-06-06  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. Purpose & the problem

M8.5-B is the headline build of the finalization box: make a one-shot invitee
able to **join and become a member**. Phase-0 grounding is the M8.5 audit
(`tasks/M8_5_FINALIZATION_AUDIT.md` §3, findings M85-A1..A4) plus the additional
code grounding recorded here. No re-audit — design only.

**The defect (M85-A1..A3).** Sync is member-gated (`collect_sync_history`,
`fanout.rs` ~727: `if !space.is_member(requester) continue`). A pending invitee
sees **zero events**, so it cannot see the `membership.invite` that names it.
Its join therefore cannot chain off the invite; it falls back to the create-root
(via `ops::join` `:770`, whose fallback is `Err`-only so an `Ok(empty)` slips to
empty `prev_events` — M85-A2), landing **causally concurrent** with the invite
on `membership:{space}:{invitee}`. `derive_resolved` Layer 4 elects the Owner's
invite over the Member's join → **join dropped → invitee never joins** (M85-A3).
The test fixtures only pass because they hand-chain `join.prev=[invite_id]`,
which production cannot reproduce blind.

**The existing substrate (grounded for this design).** The invite already names
the invitee and is tip-chained: `membership.invite` content =
`{ target_identity, role }`, signed by the inviter, `prev_events=[tip]`
(`ops::invite`). The node already seeds `pending_invites={invitee}` (the DM flow,
`space/state.rs`). `ops::join`'s *intent* is already to chain off the invite
(its own comment). The invitee's tier is read via `assertion_tier_of` (the same
seam PG-13 uses). So the fix is wiring + two small schema additions, not a new
subsystem.

---

## 2. The fix in one frame

Let the invitee, connected to the home node, **source the invite via a scoped
structural fetch** (it is in `pending_invites`), read the invite's `event_id`,
and build its join with **`prev_events=[invite_event_id]`** — causally *after*
the invite. The invite→join chain is then a normal ordered membership chain;
`derive_resolved` sees no concurrency; the join is applied; the invitee becomes
a member. Validity is bounded by a tier-graded `valid_until` on the invite,
enforced at join-acceptance and on the read path.

---

## 3. Locked decisions (INV-D#, arc-local per D-069)

- **INV-D1 (Q1 read path = Bundle 2).** A **scoped structural fetch** serves a
  pending-invitee only the **structural** events needed to bootstrap — the
  Space/Room creates and the membership chain including the invite — **not**
  message content. `collect_sync_history`'s member-gate is **unchanged** (members
  still get full history); the invitee path is separate and structural-only.
  Privacy: an invitee who never accepts never sees Room content (§4 "public
  material only").
- **INV-D2 (Q2 event_id = served).** The invitee learns the invite `event_id`
  by reading it from the bootstrap events (the invite names `target_identity`=self).
  No out-of-band capability token. The only unavoidable out-of-band data is
  space_id + home-node endpoint, inherent to any invitation.
- **INV-D3 (join causal chain).** `ops::join` builds the join with
  `prev_events=[invite_event_id]` sourced from the bootstrap fetch (not
  `get_dag_tips`). This dissolves M85-A3 and makes `invited_by` flow correctly.
- **INV-D4 (A2 fix).** `ops::join`'s `get_dag_tips` fallback is corrected so an
  `Ok(empty)` is treated like `Err` (defensive; the primary path is INV-D3).
- **INV-D5 (Q1 note = `message.rich` body).** The invite content gains an
  optional **`note`** carrying a **`message.rich`-format body** (markdown /
  mentions / code blocks / emoji), rendered through the identical client path;
  it inherits ch6 §6.9 compose substitution at UI-build time. The note is opaque
  UTF-8 content on the invite event — convergence-neutral, space-visible (not a
  private channel; a private invite message would be a separate encrypted DM,
  out of scope).
- **INV-D6 (Q3 validity = `valid_until`, tier-graded).** See §4.

---

## 4. Validity model (INV-D6)

- **Wire field: `valid_until`** — an **absolute** timestamp on the invite content
  (named `valid_until`, matching TrustAssertion ch3 §3.8.4 semantics: a
  *credential-validity* deadline, identity-substantial — deliberately **not**
  `expires_at`, which is reserved for system/content-retention TTL).
- **Resolved at creation, client-side, in `ops::invite` at sign time**
  (corrected at C2 close — the invite is signed by the inviter's client, so
  `valid_until` must be in the content before signing; the Node cannot fill it
  post-hoc without breaking the signature. The earlier "by the inviter's node"
  wording was imprecise; §6's "ops::invite stamps via the cascade" is correct).
  Cascade (C2): **individual `valid_for` (`--valid-for-days`) → protocol
  default (14d)**; the **node-default tier is deferred** (the client has no
  source for the inviter-node's default — a future node→client config surface),
  with the Node's ingest ceiling (`3045`) as the backstop. All bounded by a
  **per-tier ceiling keyed on the invitee's tier** (`assertion_tier_of`),
  enforced Node-side at ingest. **Default-stamp-14d**: an invite with no expiry
  is the unbounded capability INV-D6 prevents, so 14d is the secure default.
- **Tier grading (rule + the one live number).** Ceiling **tightens as tier
  rises** — rationale is **exposure-window minimization** (a longer window is
  more time for an invite to be misdirected/manipulated; the most consequential
  credential, the highest tier, gets the tightest window; the lowest tier, least
  to exploit, tolerates the widest). **Only T1 is defined now: `T1 max = 14d`**
  (generous). T2/T3+ ceilings are **deferred to the tier/Auth-Module work** that
  will own per-tier policy (same dependency as PG-13/PG-03); the grading *rule*
  is recorded, the numbers land with the modules.
- **Default when nothing specified** = the invitee's tier ceiling (today: 14d).
- **Over-ceiling → reject at creation** (D-065 honest-fail — "max validity is
  N"), never silent clamp.
- **Enforcement** at join-acceptance against the home node's own clock (a gate,
  like PG-13 — convergence-neutral, no `derive_resolved` surface, no clock-skew
  problem). An expired `valid_until` → **a new membership-band reject code**
  (exact number = CP-1, confirmed against the live code-registry to avoid the
  Arc-E guessed-code collision). Expiry **also gates the INV-D1 read path** — an
  expired invite is a dead read capability, not just a non-joinable one.
- **Post-expiry handling = lazy.** The expired `pending_invite` is left inert in
  state (both join and read check `valid_until` at request time); no sweeper, no
  new `invite_expired` event (avoids a convergence surface). Renewal = the
  inviter issues a fresh `membership.invite`; an operator kick can hard-remove a
  stale pending-invite if desired.
- **Forward-note (tier-module transition, D-077).** The T1=14d ceiling is an
  interim protocol constant standing in until **Tier 1 is rebuilt as a proper
  Auth Module** (the planned Tier-1-auth-rebuild milestone). At that point the
  T1 ceiling becomes **module-derived** like every other tier — sourced from the
  Tier-1 module's policy, **bounded ≤ 14d** (14d is the inherited upper bound,
  not a floor). The grading rule, the 14d cap, the wire field, the cascade, and
  the enforcement are unchanged across the transition; only the *source* of the
  number moves from protocol-constant to module-policy.
- **Honest posture (D-065).** Until trusted Auth Modules exist, `assertion_tier_of`
  resolves every identity to **Tier 1**, so only the **T1 path (14d)** is
  exercisable end-to-end. The tier-graded ceiling above T1 is **wired-but-dormant**,
  same accepted posture as PG-13.

---

## 5. Node side (xgen-node)

- **Scoped structural fetch (INV-D1).** A path serving a pending-invitee the
  structural event set (Space create, Room create(s), the membership chain incl.
  the invite addressed to the requester). Authorization = the requester holds an
  unexpired `pending_invite` in the Space. **CP-2:** wire mechanism = a dedicated
  request vs a flag on `sync_request` — lean **dedicated request** (keeps
  `collect_sync_history` member-only; clean authorization boundary). **CP-3:** the
  exact structural event-type set served.
- **Join-acceptance enforcement.** On a `membership.join` for a pending invitee,
  the home node checks (a) `valid_until` vs its clock (→ CP-1 reject if past) and
  (b) the existing PG-13 join-tier-gate. Both already happen at this seam.
- **Creation-time ceiling.** When an invite is created/relayed, the node clamps/
  rejects per the invitee-tier ceiling (§4).

## 6. Client side (xgen-client)

- **`ops::join` (INV-D3 + INV-D4).** Source the invite event from the bootstrap
  fetch; set `prev_events=[invite_event_id]`; fix the `Ok(empty)` fallback.
- **`ops::invite`.** Stamp `valid_until` via the cascade (§4); accept an optional
  individual `valid_for`; carry the optional `note` (INV-D5).
- **Bootstrap sequence.** Invitee connects to home node → scoped fetch → find the
  invite naming self → chain + send join → on accept, becomes a member.

---

## 7. Convergence / M8 safety

The invite→join chain is causally ordered (INV-D3), so `derive_resolved` sees no
concurrency on the membership key — the M85-A3 conflict is dissolved structurally,
not by resolution tuning. `note` is opaque content (no `state_key`). `valid_until`
is an acceptance gate, not a resolved value. **No new `derive_resolved` surface;
convergence-neutral.** Round-2 client-side resolution (R2-F01) already handles the
invite/join chain (its fixtures were causal-linked at J-262).

## 8. Scope fence (OUT)

T2/T3+ ceiling numbers (tier-module work); node-max as a separate knob (the
tier ceiling *is* the max); private encrypted invite DM; ch6 §6.9 substitution
implementation (UI build); multi-device (D3-gated); real higher-tier teeth
(Auth-Module dependency).

## 9. Confirm-at-pickup (for the runbook / Clair)

- **CP-1** — exact membership-band reject code (confirm vs live registry; no guess).
- **CP-2** — scoped-fetch wire mechanism (dedicated request, lean) + its message shape.
- **CP-3** — the structural event-type set served by the fetch.
- **CP-4** — where the `valid_for→node→protocol(14d)` cascade is computed (inviter-node).
- **CP-5** — `note` content-schema reuse point (the `message.rich` body shape).

## 10. Commit plan + next

Standard lifecycle (runbook → Joe-lock → Clair → close, D-071). Likely:
- **C1 (node):** scoped structural fetch + pending-invite authorization +
  `valid_until` creation-clamp + join-acceptance enforcement + CP-1 reject code + tests.
- **C2 (client):** `ops::join` chain + A2 fix + `ops::invite` `valid_until`/`note` + tests.
- **Close (doc-only):** ch3 schema additions (`valid_until`, `note` on
  `membership.invite`; the scoped-fetch wire shape) + audit M85-A1..A4 →
  resolved + INV-D# evaluated for DECISIONS promotion + JOURNAL.

INV-D# are arc-local (D-069); `valid_until`-as-credential-validity and the
exposure-graded ceiling are **promotion candidates** — flagged, evaluated at close.

**Promotion eval (J-275 close, Joe-ruled): flag both, promote neither — all
INV-D# stay arc-local.**
- **`valid_until`-as-credential-validity** *instantiates* the established
  TrustAssertion §3.8.4 `valid_until` convention (also used by the Arc-H
  KeyPackage) — an application of an existing decision, not a new cross-cutting
  one. Arc-local; nothing to promote.
- **The exposure-graded tier ceiling** (ceiling tightens as tier rises) is a
  genuinely novel principle but at **one instance** (invites). Recorded as a
  **promotion-watch candidate** under the three-instance bar (D-077/D-078); it
  may promote if it recurs on another credential/capability (a second/third
  instance of "validity window narrows as assurance rises").

**Next after M8.5-B:** M8.5-C (S5 surfaces).

**DESIGN LOCKED at J-272 (2026-06-05).** INV-D1..D6 Joe-locked; runbook authored
at `tasks/M8_5_B_INV_BOOTSTRAP_IMPL.md`. Clair implements from the runbook.

Per Rule 0 + D-065 + D-069 + D-071 + D-074 + D-076 + D-078.
