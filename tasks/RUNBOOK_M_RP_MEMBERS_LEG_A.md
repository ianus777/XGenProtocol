# M-RP-MEMBERS Leg A — the address-book read surface: two Tauri commands
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jul 2026  
> **Last updated**: 2026-07-25  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — Seat and scope

**Clair implements. Chat verifies. Joe locks and pushes.** Phase-0 is `tasks/M_RP_MEMBERS.md` **v1.4 ACTIVE** — read §1 (grounding), §3 (what may be shown), §4a (which latch) and §5 (fill timing) before starting. This runbook does **not** restate the Phase-0's reasoning; where the two disagree, **the Phase-0 wins and the disagreement is a defect to report**.

**In scope:** two `#[tauri::command]`s in `xgen-client/src/desktop.rs`, their registration, one `Serialize` derive, and one piece of managed state.

**Out of scope — do NOT build these here:** any `ui/**` file · any store · any widget · the `CLIENT_PLUGINS` row · `skin.css` · any `app.emit` push · anything from M13 · **the S2 self-exclusion fix** (⚠️ Joe has not ruled; if he locks it, it becomes Step 4 of this runbook, not a silent addition).

---

## §1 — ⚠️ THE PRECEDENT SPLITS. THE TWO COMMANDS ARE NOT THE SAME SHAPE.

This is the single most likely thing to get wrong, so it is first.

`desktop.rs` already builds an `OpContext` in **two** places — `get_self_state:549` and `get_spaces:608` — and **both use a throwaway `SessionState` with an EMPTY `home_node`**, because `whoami` and `spaces` never touch `session`; they read the on-disk state file.

⚠️ **`fill_from_space` DOES touch `session` — it calls `ensure_connected`.** Copying the throwaway pattern into Step 2 will compile and then fail at runtime against an empty node endpoint.

| | Step 1 — read | Step 2 — fill |
|---|---|---|
| Precedent | `get_spaces` (`desktop.rs:600-616`) | `reanchor_space` (`desktop.rs:369-381`) |
| Session | **none needed at all** — `AddressBook::load` takes `&Path` | **real**: `app::resolve_node` + `ensure_identity` |
| Network | no | yes |
| Managed state | `DataDir` | `DataDir` **and** `ConfigPath` |

---

## §2 — Grounding facts (measured 2026-07-25 at `a2ef058`)

Build against these; do not re-derive them.

- **`AddressBook`** — `#[serde(transparent)]` over a **private** `BTreeMap<String, SeenRecord>` (`address_book.rs:172-175`). It serialises as a JSON **object keyed by XGID**, not an array. `SeenRecord` derives `Serialize, Deserialize` (`:77`).
- **`AddressBook::load(&Path)`** (`:313-321`) — **file absent ⇒ `Ok(Self::default())`**, an honest empty book. The **only** `Err` is a corrupt file, and the corrupt path deliberately leaves the file untouched for inspection.
- ⚠️ **`FillReport` derives `Debug, Clone, Default, PartialEq` — NO `Serialize`** (`ops.rs`, above `pub struct FillReport`). Returning it across the Tauri boundary **requires adding `Serialize`**. That is an additive one-line change to closed-milestone code; it is expected, in scope, and must appear in the diff.
- **`fill_from_space(ctx, &mut book, space)`** does **not** load or save. The caller owns both.
- 🔒 **`fill_from_space` is re-entrant BY DESIGN (J-586, expensive to learn).** It clears `ctx.session.conn = None` on **every** exit, including `?`-skipped error paths. **Add no caller-side connection management, and do not touch those clears.**
- **`ConfigPath` and `DataDir` are already managed state** — `get_substitutions` takes `tauri::State<ConfigPath>`, `get_spaces` takes `tauri::State<DataDir>`.
- **D-129 checked and does NOT fire here:** each command builds a fresh `SessionState` and drops it. Nothing becomes persistent across ops, so `ensure_connected` is untouched. *Recorded so the next arc knows it was checked, not skipped.*

---

## §3 — Steps

### Step 1 — `get_address_book`

```rust
#[tauri::command]
fn get_address_book(data: tauri::State<DataDir>) -> Result<AddressBook, String>
```

- Body: `AddressBook::load(&data.0).map_err(|e| format!("{e:#}"))`. **No `OpContext`, no `SessionState`** — `load` takes a `&Path`.
- ⚠️ **Return `Result`, NOT `unwrap_or_default()`.** `get_spaces` uses `unwrap_or_default()` because there `Err` means *"state file absent = unregistered"*, which is an honest empty render. **Here, absent is ALREADY `Ok(empty)`** — so the only `Err` is **corrupt**, and swallowing it would render a corrupt book as an empty one. That is the exact "absence renders as fine" failure the Phase-0 §3 display rule exists to prevent. The `set_substitutions` precedent (`Result<(), String>`) is the shape.

### Step 2 — `fill_space_records`

```rust
#[tauri::command]
async fn fill_space_records(
    space_id: String,
    data: tauri::State<'_, DataDir>,
    config: tauri::State<'_, ConfigPath>,
) -> Result<FillReport, String>
```

1. Add `Serialize` to `FillReport`'s derive list (§2).
2. Build a **real** session, the `reanchor_space` way: `app::resolve_node(None, &config_path)` → `SessionState::new(node, data_dir)` → `session.ensure_identity(&app::resolve_keypair_path(&config_path))?`.
3. `let mut book = AddressBook::load(&data_dir)?;`
4. `let mut ctx = OpContext { session: &mut session, data_dir: &data_dir, node_override: None };`
5. `let result = ops::fill_from_space(&mut ctx, &mut book, &space_id).await;`
6. ⚠️ **`book.save(&data_dir)` UNCONDITIONALLY, before propagating `result`.** Rationale, and it is a deliberate call rather than an oversight: `fill_from_space` takes `&mut book` and applies **touches and absorbed fetches as it goes**, so on a mid-loop error the book already holds **real observations**. Discarding them would throw away work that actually happened, and the next fill would re-fetch identities it had already resolved. A save on an early error (drain failed, nothing mutated) is a harmless no-op write.
7. Map the error with `format!("{e:#}")` — the alternate form, so the `anyhow` context chain survives to the webview.

### Step 3 — serialise concurrent fills

Add managed state — a `tokio::Mutex<()>` — and hold it across **load → fill → save** in Step 2.

**Why (D-121):** ① *user-visible* — without it, two fills overlapping (the user clicking through Spaces quickly) both `load`, both `save`, and **the loser's resolved names are silently discarded**, so rows sit as pubkey stubs longer than they should and the cause is invisible. ② *cost* — one managed type, one `.lock().await`, roughly five lines.

📌 **This is Chat's call, not Joe's, and it is cheap to strike.** If it is removed, the load-modify-save race must be recorded as a known limitation rather than left unstated.

### Step 4 — register both commands

Add both to the `invoke_handler![...]` list. ⚠️ **Easy to forget and it fails silently from the webview's side** — the invoke rejects at runtime with no compile error.

---

## §4 — Definition of Done

- [ ] Both commands exist, are registered, and `cargo build` is clean
- [ ] `FillReport` derives `Serialize`; the diff shows it
- [ ] `cargo test` — floor is **1585 / 0 / 62 across 56**; report the new totals and **explain every delta**. ⚠️ Run **detached** and poll (`cargo test` exceeds the MCP timeout), sum `test result:` lines **case-sensitively**, and **56 is the completeness check**
- [ ] Scope-clean: `git show --stat` shows **zero `ui/**`**, zero `skin.css`, zero new crates
- [ ] `svelte-check` **not re-measured** — held by scope, and say so rather than implying it was run
- [ ] **Live proof on the real client 9222** (Chat drives): `invoke('get_address_book')` → `{}` on a cold book → `invoke('fill_space_records', {spaceId})` → a `FillReport` with non-zero `fetched` → `get_address_book` again → **non-empty** → 🔑 **`xgen-client_address_book.json` EXISTS ON DISK.** That is the milestone's one-line proof and it is reachable at Leg A
- [ ] Corrupt-book path **exercised, not asserted** (N-095): write malformed JSON into the file, invoke, and confirm the command **rejects** rather than returning `{}`
- [ ] Re-entrancy re-confirmed: **two consecutive fills** on the same Space, second returns `touched > 0, fetched 0`, no connection error

*(Per the standing rule, "commit pushed" is deliberately NOT a DoD item — `Status: COMPLETED` in the header is the shipped signal.)*

---

## §5 — ⚠️ RE-OPEN-ON-BUILD CLAUSE (mandatory, D-122)

If building this surfaces anything that contradicts the Phase-0 or this runbook — a wrong precedent, a missing derive, a signature that does not fit — **stop and report it; do not absorb it**. Four defects in the previous arc's runbook were all one class: **prose under-specifying a contract the Phase-0 had already made clear**, and every one was caught by a second reader working from the source rather than by care.

📌 **Specifically expected to fire on:** the §1 precedent split (the empty-`home_node` trap) and the §2 `FillReport` derive.

---

## §6 — Handoff

✅ **CLOSED 2026-07-25. Every DoD leg green, every number Chat-measured on the real client 9222.**

**Static:** `cargo build` clean · `cargo test` **1585 / 0 / 62 across 56 terminators**, `FAILED|panicked` = 0, `error[` = 0 — **identical to floor**, which is the honest signal for a milestone that adds a derive, two commands and a newtype · scope `desktop.rs` +89/−1 · `ops.rs` +5/−1, **zero** `ui/**`, `skin.css`, `Cargo.*` · `svelte-check` **not re-measured**, held by scope.

**Live (9222, node on 8080):** cold `get_address_book` → **`{}`** · `fill_space_records` → `{candidates:1, fetched:1, not_found:0, touched:0}` · 🔑 **`xgen-client_address_book.json` EXISTS — 392 B, 14:00:45** · second fill → `{candidates:0, fetched:0, touched:1}`, no connection error (re-entrancy through the new caller) · multi-record on a second Space → `BobLegB` fetched + `Joe` touched, 2 records · corrupt path **exercised**: rejected with the full `anyhow` chain, `returnedInstead: null`, damaged file **byte-untouched** (18→18), restored · client registry **149**, the at-rest floor exactly ⇒ **zero frontend effect, measured not argued**.

🔑 **THE FIRST FILL IN THE PROJECT'S HISTORY WROTE JOE INTO JOE'S OWN ADDRESS BOOK** — Engineering has one member, and the single record was self (`isSelfInBook: true`, checked against `get_self_state`). Every §3 wire prediction held live: `display_name` present · **`revoked: false`** (the constant-false trap, which is why `flags.revoked` ships unfed) · `update_version: 0` · `trust_assertion` absent · `registered_at` correctly trimmed.

**Deviations, flagged not absorbed (D-122 outcome):**
- The re-open clause **did not fire** — Clair read the runbook against the Phase-0 and the code first, and confirmed every §1/§2 claim at source. She additionally located the trap-① bail at `session.rs:134-136`; Chat reproduced it.
- **One Clair number does not reproduce, and it is not her fault:** she reported both edited files uniformly CRLF. `desktop.rs` is `w/lf`. But `address_book.rs` and `session.rs` are also `w/lf` and untouched — **the `.rs` working tree was already mixed**, index is `i/lf` throughout, committed bytes identical, and the project convention covers `.md` only. **Conclusion stands, number does not.**
- **One undocumented-in-runbook choice she documented herself:** if `save` fails *and* the fill failed, the save error wins and the fill error is lost. Reasoned in the doc comment; accepted.
- ⚠️ **Chat's own orphan, N-165 shape:** `tauri dev` found 5173 held by Joe's 226-minute Vite and started its own on **5174**, which survived the client kill — found in the post-run port sweep and killed. The webview therefore rendered the **stale 5173 bundle**. Immaterial here (Rust-only leg; registry read 149 as expected); **a trap for Leg B**.

**Next:** Leg B (store + `members-panel` + the 7th `CLIENT_PLUGINS` row). ⚠️ Now carries three binding constraints from Phase-0 **§4c**: the self row is a **fixture** (always present · always first · filter-immune), the roster crosses in **`Option`-shaped, never a bare array**, and any member **count derives from the roster, never from rendered rows**.

**Leg B** (store + `members-panel` + the 7th `CLIENT_PLUGINS` row) is **blocked on `M_RP_MEMBERS.md` §4b — the DM Space question, open for Joe.** Leg A is not: neither command cares whether a Space is a DM.

**Leg A moves the cargo floor; Leg B moves svelte-check.** They are separate runbooks for that reason — one commit spanning both would make a regression ambiguous.
