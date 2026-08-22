# M-SPACE-ADMISSION Leg D Runbook — the admission gate
> **Status**: ACTIVE — 🔒 LOCKED BY JOE 2026-08-22  
> Version: 1.1  
> Date: Aug 2026  
> **Last updated**: 2026-08-22  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — READ THIS FIRST

**Entry order:** `CLAUDE.md` PLAY block → `JOURNAL.md` J-761 → **`tasks/M_SPACE_ADMISSION_LEGD_PHASE0.md` v1.1 IN FULL** → this runbook. **The Phase-0 is item 3, not item 1.**

🛑 **§1 IS MANDATORY AND IS EXECUTED BEFORE §4 IS OPENED.** Derive the current join path from source, from `runtime.rs` and `exchange.rs`, and **write down what you derived before reading this document's account of it.** If your derivation and §2 disagree, **§2 is the suspect** — three of the last four legs found a defect exactly here.

🔒 **Rule 6: report deviations, do not absorb them.** 🛑 **Never push.**

---

## §1 — DERIVE FIRST (no reading ahead)

From source alone, answer in writing:
1. **What adjudicates a `membership.join` today?** Name every gate, with file and line, and say which pipeline step each belongs to.
2. **What happens to a join from someone with no pending invite?**
3. **What does `SpaceState.admission` currently affect?**
4. **What does `from_space_create` do with `{"admission": 5}`?**

**Then and only then open §2.**

---

## §2 — THE GROUND, MEASURED AT `1623eb6`

- **`MembershipJoin` is in `skip_membership`** — `exchange.rs:670-680` ⇒ **step 11 (`:681`) and step 13 (`:750`) are both skipped.** `validate_event` never adjudicates a join.
- **The only join gate is `runtime.rs:1580-1613`**, and the expiry check lives **inside** `if let Some(pi) = space.pending_invites.get(&event.sender)` (`:1586`) ⇒ ***no pending invite means the block never runs and the join applies.*** The comment at `:1563-1565` already states this.
- **`SpaceState.admission`** — `state.rs:284`, `String`; its doc comment at `:283` reads ***"Nothing reads this field until Leg D."***
- **Defaults:** regular Space ⇒ `DEFAULT_ADMISSION` = `ADMISSION_OPEN` (`wire.rs:776`, `:748`); DM ⇒ `ADMISSION_INVITE` (`state.rs:511`, `:627`).
- **`3047` / `3048` are RESERVED and not yet live** (Leg C's ch3 line). **`3046` is live in code and MISSING from the ch3 registry** — `C-8`.

---

## §3 — THE FOUR EDITS

| | edit | file |
|---|---|---|
| **D-1** | the admission gate, **before** the expiry check, inside the existing `origin == LocallySubmitted && room_id.is_empty()` block | `runtime.rs:1580` |
| **D-2** | `F-3`'s three-state parse + 64-byte char-boundary cap; **the false doc comment corrected, not deleted** | `state.rs:344-351` |
| **D-3** | `:1112` gates on `left_at.is_none()` (`D-154`) | `state.rs:1112` |
| **D-4** | `3047 admission_required` minted; ch3 §3.6.10.10 gains **its row AND `3046`'s** (`C-8`) | `exchange.rs`, `ch3` |

🛑 **D-1 AND D-3 ARE DIFFERENT LAYERS AND MUST NOT SHARE A TEST.** D-1 is a **Node dispatch gate**; D-3 is a **`SpaceState` applier** rule. ⚠️ **`M-1`'s species is precisely a check that exists only in the applier and is therefore a silent no-op on the answer path** — a unit test calling `apply_event` directly **cannot** see D-1, and a node-path test **can** pass while D-3 is absent.

### D-2 — the parse

Three states, named: **Absent** (key missing) ⇒ `DEFAULT_ADMISSION` · **Valid** (a string) ⇒ stored as-is · **Malformed** (present, non-string, or over-cap) ⇒ **stored as raw JSON text, truncated to 64 bytes on a char boundary.**
🔑 **The cap is not about size — every node must truncate IDENTICALLY or the stored value diverges.** Truncate on `char_indices`, never on a byte slice.
🛑 **The gate treats Malformed as `invite`** — fail-closed, matching `:1591`'s `unwrap_or(true)`.

### D-1 — the gate

Runs when `admission` resolves to **`invite`** (or Malformed) **and** the sender has **no pending invite** ⇒ `DispatchOutcome::Rejected(RejectInfo::coded(3047, "admission_required", …))`.
🔒 **`open` ⇒ no gate.** 🔒 **`ReceivedViaFederation` ⇒ SKIPPED, not rejected** — inherited from `:1567-1579`, not re-argued.

---

## §4 — VERIFICATION. **A NEGATIVE CONTROL PER GATE — THIS IS THE LEG'S POINT.**

🛑 **Three gates ship here. Each one is deleted in turn and MUST turn something red.** A gate whose removal leaves the suite green **was never tested**, and `E-0` spent a whole session on that lesson.

| | control | must fail |
|---|---|---|
| **V-3a** | delete D-1's arm | the **node-path** invite-only join test |
| **V-3b** | delete D-2's Malformed branch | the `{"admission": 5}` test — **and it must fail CLOSED, not open** |
| **V-3c** | delete D-3's `left_at` guard | the rejoin test **and** the retained-ban test |

🔑 **V-3c's second half is the one that matters:** with D-3 absent, a retained banned member is refused **`AlreadyMember` instead of `Banned`** — ***a green-looking refusal that is the wrong refusal.***

🛑 **Controls use FILE BACKUPS, never `git checkout`** — the work is uncommitted. **Assert the mutation changed something, and assert on CONTENT not on a remembered offset** — both guards fired in one session at J-760, and without them three controls would have run against unmutated source and reported clean passes. **Restore and verify by `sha256`.**

**Also required:** **V-4** — a `ReceivedViaFederation` join into an `invite` Space is **APPLIED** (proven, not asserted). **V-5** — floors re-measured from HEAD, cargo **detached**, log to file, `XGEN_EXIT_SENTINEL=` appended, summed over `^test result:` case-sensitively. **V-6** — every new test confirmed **by exact name**; a bare grep for "admission" will miss some.

**Floors to beat, carried and to be re-measured on the delivered tree:** cargo **1616 / 0 / 62 × 56 SUITES** · vitest **172 / 172 × 9 FILES** · svelte-check **0 / 34 / 15**. **Zero `ui/**`, zero `xgen-client` expected.**

---

## §5 — DoD

- [ ] §1 executed **from source, before §4 was opened**, and the derivation written down
- [ ] D-1 · D-2 · D-3 · D-4 shipped
- [ ] **V-3a/b/c all reproduced live**, each restored and `sha256`-verified
- [ ] V-4 · V-5 · V-6
- [ ] Deviations reported, not absorbed (Rule 6)
- [ ] Hand-back at `tasks/CLAIR_LEG_D_HANDBACK.md`

🛑 **Records, `Status: COMPLETED`, the commit and the push are Chat's and Joe's.** 📌 **"Commit pushed" is not a DoD item.**