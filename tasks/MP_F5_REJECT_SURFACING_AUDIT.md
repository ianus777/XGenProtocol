# Phase-0 Audit — MP-F5: client reject-surfacing + C6 reject-oracle reconciliation
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-10  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 1. What this is

The D-071 Phase-0 audit for **MP-F5** — the fifth loop-to-green finding-fix arc
(MP-F2 ✅ → MP-F3 ✅ → MP-F1a ✅ → MP-F4 ✅ → MP-F1b ✅ → **MP-F5** → ban →
room_update → thread×3 → R1 rerun). Surfaced empirically during the auth-tier
arc (J-334 arc 1); Joe-LOCKED as a named finding + sequenced ahead of ban
(2026-06-10), because ban's `MP-C-09`/`MP-A-14` witnesses inherit the identical
reject-oracle dependency — closing MP-F5 first lets MP-A-03 **and** ban land
green RED-on-revert witnesses instead of piling witness-debt onto the rerun.

**No code, no design lock pre-decided here.** This grounds the surfaces + frames
the forks. The D-9 amendment + the reject-body shape are Joe-locked at MP-F5's
design-lock.

---

## 2. The finding (grounded empirically)

The C6 batch reject-oracle is **broken on current `main`**. Running the existing,
matrix-PASS scenarios on HEAD (untouched by the auth-tier arc):

```
test mp_a_02_over_ceiling_invite_rejected ... FAILED  (alice.a4 reply has no `event_id`)
test mp_a_04_non_member_send_rejected     ... FAILED  (carol.c2 reply has no `event_id`)
```

The J-321 PASS rows (MP-A-02/04/17/20) are **stale** — recorded before
**MP-F1a** (await-confirm, J-328) + **MP-F2** (`reject_signal` wiring, J-324)
changed the reject path. This is exactly what the loop's final R1 rerun is meant
to catch; MP-F5 closes it ahead of time.

**Root-cause trace (node → client → harness):**

1. **Node (correct, MP-F2):** a locally-submitted rejected event gets an `Error`
   frame back — `reject_signal` ([app.rs:2388](../xgen-node/src/app.rs#L2388),
   sent at [app.rs:2762](../xgen-node/src/app.rs#L2762)) carries the **wire code**
   (e.g. 3045/3030) **and** the `event_id`. Gated locally-submitted-only (the J-081 §5 / D-070 fix).
2. **Client transport (correct):** `send_event_confirmed`
   ([connection.rs:168](../xgen-core/src/transport/connection.rs#L168)) matches the
   `Error` frame by `event_id` → `EventConfirm::Rejected { code, reason }` — the
   **structured wire code survives here** (but the variant drops the `event_id`,
   though the caller still holds the sent id locally).
3. **Client ops (the flattening — the defect):** `apply_single_event_confirm`
   ([ops.rs:116](../xgen-client/src/ops.rs#L116)) collapses
   `Rejected { code, reason }` into `anyhow::bail!("{verb} rejected by node (code
   {code}): {reason}")` — a free-text string. The structured `code` + the
   `event_id` are lost into prose.
4. **Client aicontrol (the surface):** the `anyhow` becomes
   `DispatchError::ClientVerb(String)` → `ErrorBody { code: "GENERIC_4000",
   category: Protocol, message: <text>, .. }` ([aicontrol.rs:88](../xgen-client/src/aicontrol.rs#L88)).
   So the reply is an **error envelope** with the wire code buried in `message`,
   `category` flattened to `Protocol`, and **no `event_id` field**.
5. **Harness (correct, blind by omission):** `wire::Reply` already has
   `Error { error: ErrorBody }` + `.error()` + `.is_ok()`
   ([wire.rs:96](../xgen-mptest/src/wire.rs#L96)), but the C6 oracle reads the
   **`Ok.data` path** (`reply_field(...,"event_id")`), which is `None` for an
   `Error` reply → **panic** ("reply has no `event_id`"). The mirror `ErrorBody`
   also has no `event_id` field.

**The favorable reframe (carry into the design-lock).** This resolves D-9 in the
**favorable** direction. The reject is *already* batch-observable — the node
sends code + event_id (reject_signal); the client merely **buries the wire code
in free text** instead of surfacing it as a field. MP-F5 is closer to *"finish
the MP-F2 surfacing into the client reply"* than *"redesign the oracle."* The
MP-R1-D9 premise ("category not batch-observable; the op is fire-and-forget, no
recv") is simply pre-MP-F1a/MP-F2 and now false.

---

## 3. Surfaces (grounded)

| Layer | File / anchor | State | MP-F5 touch |
|-------|---------------|-------|-------------|
| Node reject signal | `reject_signal` [app.rs:2388](../xgen-node/src/app.rs#L2388) | **correct** — sends code + event_id | none (MP-F2 already did it) |
| Client confirm | `EventConfirm::Rejected{code,reason}` [connection.rs:191](../xgen-core/src/transport/connection.rs#L191) | structured code present; event_id dropped from the variant (caller holds sent id) | maybe carry event_id (fork F1) |
| Client ops flatten | `apply_single_event_confirm` [ops.rs:116](../xgen-client/src/ops.rs#L116) | **defect** — flattens to anyhow text | propagate structured (code, event_id) up |
| Client aicontrol map | `DispatchError` [aicontrol.rs:77](../xgen-client/src/aicontrol.rs#L77) / `into_body` :88 | **defect** — `GENERIC_4000`/`Protocol`/no-event_id | add a structured verb-reject variant → ErrorBody fields |
| Shipping ErrorBody | [envelope.rs:106](../xgen-common/src/aicontrol/envelope.rs#L106) | `code`/`category`/`message`/`stage`/`hint`; **no event_id** | add `event_id` (+ maybe `reject_code`); fork F2 |
| Harness ErrorBody mirror | [wire.rs:75](../xgen-mptest/src/wire.rs#L75) | same shape, deser side | mirror the new field(s) (drift-lock test) |
| C6 oracle | `reply_field`/`assert_rejected_no_membership` [mp_r1_c6.rs:71](../xgen-mptest/tests/mp_r1_c6.rs#L71) | **broken** — reads `Ok.data`, panics on `Error` | rewrite to assert-the-reject |
| Matrix rows | A-02/04/17/20 [MULTIPARTY_TEST_MATRIX.md](../docs/tests/MULTIPARTY_TEST_MATRIX.md) | stale ✅ PASS | annotate (don't leave lying) |

**AC-D2/AC-D3d coupling (flag for the lock).** Today's client map is "ops::* →
`GENERIC_4000`/`Protocol`, message-only" (AC-D3d invariant — a control code can
never represent a verb error). MP-F5 introduces a *protocol wire code* (3030/3045)
into the reply, which is neither a control code nor cleanly `GENERIC_4000`. The
design-lock must decide whether the wire code rides an **additive** structured
field (keeping `code = GENERIC_4000` as the client-surface code — least
disruptive to AC-D2) or repurposes `code`. See fork F2.

---

## 4. Forks for the design-lock (none pre-decided)

- **F1 — how the structured reject reaches aicontrol.** A typed verb-reject
  carried up from `EventConfirm::Rejected` (a `DispatchError::VerbReject { code,
  event_id, reason }` sibling to `ClientVerb`) vs parsing the code back out of
  the anyhow text (rejected on sight — D-067 drift). *Audit lean:* typed path;
  `apply_single_event_confirm` returns the structured reject, the op surfaces it,
  aicontrol maps it. event_id source = the op's locally-known sent id (it has it)
  or widen `EventConfirm::Rejected` to carry it.
- **F2 — ErrorBody shape (the Joe-lock).** Add `event_id: Option<String>` + carry
  the wire code. Sub-fork: (a) additive `reject_code: Option<u32>`, keep `code =
  GENERIC_4000`, optionally map `category` → `Permission`/`Lifecycle`/etc. from
  the wire band (AC-D2-preserving — *audit lean*); or (b) repurpose `code` to the
  wire code. Both update the shipping envelope **and** the harness mirror (+ the
  drift-lock test).
- **F3 — C6 oracle rewrite.** `assert_rejected_no_membership` → assert-the-reject:
  (1) the op reply is an `Error` with the expected wire code/category recoverable
  as a **field**; (2) protected state unchanged (target not a member); (3)
  offending event absent everywhere (`rejection_verdict` — needs the event_id from
  the now-structured error body). Apply to A-02/04/17/20 + the new MP-A-03.
- **F4 — D-9 amendment.** "Reject IS batch-observable post-MP-F2 (node sends code
  + event_id; the client surfaces them structurally)." Blessed at design-lock.
- **F5 — stale-row annotation.** A-02/04/17/20 matrix rows re-grounded against the
  rewritten oracle (was-stale note), not left as bare PASS.

---

## 5. Scope guard

- **In:** the client reject-surfacing (xgen-client ops + aicontrol), the
  `ErrorBody` field addition (xgen-common envelope + xgen-mptest mirror), the C6
  oracle rewrite, the D-9 amendment, the A-02/04/17/20 re-grounding, and **MP-A-03's
  batch witness** (it greens here, on the rewritten oracle — auth-tier's deferred
  half). MP-F5 is a **production arc** (xgen-client/xgen-common) under full
  protocol-change discipline.
- **Out / breadcrumbs:** the node side (MP-F2 already correct — no node touch);
  the C7 wire-path category assertions (orthogonal — C7 recvs the raw `Error`
  frame directly); the broader AC-D2 client-error taxonomy redesign (only the
  reject path is in scope, not every `ops::*` anyhow). ban's `MP-C-09`/`MP-A-14`
  consume the rewritten oracle but are **ban's** arc, not MP-F5's.

---

## 6. Phase-0 DoD

- [x] Finding grounded empirically (mp_a_02/04 RED on HEAD, untouched by auth-tier).
- [x] Root-cause traced end-to-end (node correct → client flattens → harness panics).
- [x] Surfaces enumerated with live anchors.
- [x] Favorable-D-9 reframe stated (finish the surfacing, not redesign the oracle).
- [x] Forks framed for Joe-lock (F1 reject-threading · F2 ErrorBody shape · F3 oracle rewrite · F4 D-9 amendment · F5 stale-row annotation); nothing pre-decided.
- [x] Scope guard: MP-A-03 witness greens here; ban inherits the oracle; node untouched.

**Next:** design phase — lock F1–F5 (F2 ErrorBody shape + F4 D-9 amendment are the
crux), author the runbook, impl → close. MP-A-03's batch witness + the
A-02/04/17/20 re-grounding are hard close deliverables (the witness-debt this arc
retires). Appendix F unaffected (no CLI surface change).

---

Per D-065 + D-069 + D-070 + D-071 + D-074. MP-R1-D9 (oracle path-split — amended
favorably here) + MP-R1-D10 (loop-to-green) govern. MP-F5-D# arc-local (D-069).
