# Leg D §1 — derivation from source, written BEFORE reading the runbook's §2

> **Status**: ACTIVE
> Version: 1.0
> Date: Aug 2026
> **Last updated**: 2026-08-22
> Language: EN
> Author: JozefN
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.
> License: BSL 1.1 (converts to GPL upon project handover)

Measured at `7abd341`, tree clean. Runbook §2 NOT yet read at the time of writing.

---

## Q1 — What adjudicates a `membership.join` today?

Entry point `NodeRuntime::dispatch_event` — `xgen-core/src/node/runtime.rs:1120`.

| # | gate | file:line | pipeline step | applies to a join? |
|---|---|---|---|---|
| 0 | `space_id_of` resolves | `runtime.rs:1145-1148` | pre-step-1 | yes — reject `event missing event_id` |
| 1 | Space exists | `runtime.rs:1169-1174` | Step 1 structural | yes (join is not a create) — reject `space not found` |
| 2 | F-3 federation relationship | `runtime.rs:1180-1272` | Step 2 | **only if `peer_node_id.is_some()`**. `MembershipJoin` is NOT in `skip_f3` (`:1203-1208`) ⇒ peer absent from `federation_nodes` ⇒ `HeldPending` |
| 3 | validation core `validate_event` | `runtime.rs:1312` → `exchange.rs` | Step 3 | partially — see breakdown below |
| 4 | dedup-at-dispatch | `runtime.rs:1392-1396` | post-validation | yes ⇒ `Duplicate` |
| 5 | AI capability | `runtime.rs:1408` | Step 4 semantic | yes (no-op for human senders) |
| 6 | **banned pre-check** | `runtime.rs:1521-1529` | Step 4 semantic | yes ⇒ `PermissionDenied` (4000-unmapped) |
| 7 | **tier gate (PG-13)** | `runtime.rs:1530-1538` | Step 4 semantic | yes ⇒ 3030 `tier_mismatch`. Honest T1 no-op today |
| 8 | **invite-expiry gate (3044)** | `runtime.rs:1584-1610` | Step 4 semantic | **conditionally** — see Q2 |
| 9 | `apply_join` | `state.rs:1094-1131` | Step 5 applier | yes — but its `Err` is swallowed by `ingest_event`'s `let _ = state.apply_event` (the M-1 species, named in the comment at `runtime.rs:1505-1517`) |

**Inside `validate_event` (`exchange.rs`), for a join:**

- Sender-registration check `exchange.rs:650` — `MembershipJoin` is not in `node_authored`, so an unregistered sender ⇒ `HeldPending { missing_identity }`. **Applies.**
- **Step 11 sender membership — SKIPPED.** `skip_membership` at `exchange.rs:670-679` lists `EventType::MembershipJoin` first. Guard at `:681`.
- Step 12 signature — `exchange.rs:706`. **Applies.**
- **Step 13 permission — SKIPPED.** Same `skip_membership` flag, guard at `:750`.

⇒ **`validate_event` never adjudicates a join's authority to join.** It checks structure, DAG, timestamp bounds, sender registration and signature. Nothing more.

---

## Q2 — What happens to a join from someone with no pending invite?

The gate at `runtime.rs:1584` opens:

```
1584:  if origin == EventOrigin::LocallySubmitted && event.room_id.as_str().is_empty() {
1586:      if let Some(pi) = space.pending_invites.get(&event.sender) {
```

**The 3044 check is INSIDE the `if let Some(pi)`.** No pending invite ⇒ the `if let` body never runs ⇒ nothing after it in that block ⇒ the join falls through.

`apply_join` then takes its own explicit no-invite branch — `state.rs:1117-1120`:

```
let (role, invited_by) = match self.pending_invites.remove(joiner) {
    Some(pi) => (pi.role, pi.invited_by),
    None => (Role::Member, None),
};
```

⇒ **The join is APPLIED and the sender becomes `Role::Member`.** Any registered Identity holding a valid signature and knowing the `space_id` may join any Space. **There is no admission adjudication anywhere on the path** — this is not a gap in an enforcement mechanism; there is no enforcement mechanism.

---

## Q3 — What does `SpaceState.admission` currently affect?

**Nothing.** Census over all four crates:

- **Written:** `from_space_create` `state.rs:377` (parsed) · `from_dm_space_create` `state.rs:511` (pinned `ADMISSION_INVITE`) · a second DM constructor `state.rs:627` (pinned) · `algorithm.rs:439` (test fixture) · `apply_space_admission` `state.rs:857` (Leg C mutation).
- **Read:** only `assert_eq!` in tests. `apply_space_admission`'s own guards read `dm_constraints_active` and the actor's `Role` — **not `admission`**.
- **No production branch anywhere reads the value.**

The doc comment at `state.rs:283` asserts this (*"Nothing reads this field until Leg D."*) — confirmed by measurement rather than taken on trust.

---

## Q4 — What does `from_space_create` do with `{"admission": 5}`?

`state.rs:348-351`:

```
let admission = content["admission"]
    .as_str()
    .map(str::to_string)
    .unwrap_or_else(|| DEFAULT_ADMISSION.to_string());
```

`Value::as_str()` returns `None` for **any** non-string — number, bool, array, object, null. So `5` takes the `unwrap_or_else` arm and the field is set to `DEFAULT_ADMISSION` = `ADMISSION_OPEN` = `"open"` (`wire.rs:748`, `:776`).

⇒ **`{"admission": 5}` is indistinguishable from an absent key, and both yield `open`.** `{"admission": "banana"}` by contrast is stored verbatim (test `from_space_create_unrecognised_admission_is_stored_verbatim`, `state.rs:2455`).

**And the comment sitting on that line says the opposite.** `state.rs:346-347`:

> *"A validator here would move that judgement into parse and collapse 'absent' and 'present but unrecognised' into one case, which the spec keeps apart."*

The line beneath it performs exactly that collapse for every present non-string value, **and collapses toward the permissive value.**

---

## Derivation vs runbook §2 — reconciliation, recorded after opening §2

*(filled in below once §2 was read)*

**Read after writing everything above.** §2 and the derivation **agree on every point**, and every §2 citation was opened before being accepted (`D-153`):

| §2 claim | opened at | verdict |
|---|---|---|
| `MembershipJoin` in `skip_membership`, `exchange.rs:670-680` | `:670-679` list, guards at `:681` / `:750` | ✅ exact |
| the only join gate is `runtime.rs:1580-1613`, expiry inside `if let Some(pi)` at `:1586` | `:1580` opens the block, `:1586` the `if let` | ✅ exact |
| the comment at `:1563-1565` already states it | *"an open join (no pending invite at all) is untouched"* | ✅ verbatim |
| `SpaceState.admission` at `state.rs:284`, doc at `:283` | `pub admission: String,` / *"Nothing reads this field until Leg D."* | ✅ exact |
| defaults: regular ⇒ `ADMISSION_OPEN`, DM ⇒ `ADMISSION_INVITE` | `wire.rs:748`/`:776`, `state.rs:511`/`:627` | ✅ exact |
| `3046` live in code and missing from the ch3 registry | `exchange.rs:155`; table `:2185-2193` runs 3040→3045 then 3049 | ⚠️ **narrower than stated** — `3046` has no ROW, but the paragraph beneath the table did name it (*"3046 is assigned outside this table"*). The correction is still right, because the TABLE is the instrument; and adding the row makes that sentence false, so both had to move together. |

**Nothing in the derivation contradicted §2.** The one refinement is the `3046` nuance above, folded into the ch3 edit rather than reported as a disagreement.
