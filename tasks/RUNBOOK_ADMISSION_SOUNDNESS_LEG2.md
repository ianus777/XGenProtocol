# RUNBOOK — M-ADMISSION-SOUNDNESS Leg 2: the two situations that need eyes
> **Status**: PENDING  
> Version: 1.0  
> Date: Aug 2026  
> **Last updated**: 2026-08-27  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — WHAT THIS LEG IS

The second leg of `M-ADMISSION-SOUNDNESS — driving the ruled admission mechanism as one lived story`. Leg 1 handed back at `ed8f789` + one untracked file; this leg takes the two situations Leg 1 could not answer.

- **S-3** — she returns and looks at the gap. `D-154`④: **content closed, membership structure open.**
- **S-4** — she was in rooms; she returns. `D-154`⑤: **rooms are not restored.**

🔑 **THEY ARE HERE BECAUSE THE RULED BEHAVIOUR IS AN EXPERIENCE, AND THE HARNESS SEES THE WIRE AND NEVER THE SCREEN.** Leg 1 proved the harness can reach the payload. ***What a person meets is not in the payload.***

🛑 **THIS LEG DOES NOT TEST AND IT DOES NOT ASSERT.** Leg 1's discipline carries: `call()` not `ok()`, print and move on, one plain-English `READING` per situation. ⚠️ **Leg 1's own `D-3` is the standing warning: all four `READING` lines were drafted BEFORE the run and one was wrong — *predictions wearing an observation's clothes*. Every `READING` in this leg is written AFTER the output exists, or it is not written.**

---

## §1 — 🛑 WHAT LEG 1 CHANGED, AND IT RESHAPES S-4 COMPLETELY

**Leg 1 measured, incidentally and while looking at something else, that `rooms` is unreadable by a joined member — before AND after leaving.** The reply is not an empty list; it is `no known Space with ID …`.

**The cause, opened and read:** `ops::join` **never writes client state.** The five `write_client_state` sites in `ops.rs` are `register` · `create_space` · `create_room` · `create_dm_space` · `leave`. **`join` is not among them** ⇒ ***a member's own client records his departure and never his arrival.***

### And it reaches the GUI, which is the whole of this leg

`desktop.rs:629 get_spaces` calls `crate::ops::spaces(&mut ctx)` — and `ops::spaces` reads **`xgen-client_state.json`**, the same local state `rooms` reads and `join` never writes.

⇒ 🔑 **S-4 IS NOT THE QUESTION IT WAS FILED AS.** It was filed as *she had rooms and does not get them back*. **The observation to make is now: does a Space she JOINED appear in her client at all — before she ever leaves?**

🛑 **THIS IS A PREDICTION AND IT MUST BE OBSERVED, NOT ASSUMED.** The reasoning is a code read across three files; **the GUI may sync from the node by another path this runbook has not found.** ⚠️ **If the staged client shows her Spaces, the prediction is wrong and THAT is the finding** — Leg 1's `D-3` in a new costume, and the reason nothing here is written as an assertion.

📌 **A creator is NOT a joiner:** `create_space` writes state, so alice's GUI is expected to work and bob's is the question. **Stage both, side by side, or the comparison is missing.**

---

## §2 — GROUNDING

| # | site | what is there |
|---|---|---|
| **A** | `xgen-client/src/desktop.rs:988` | data root precedence `--data-dir` > `XGEN_DATA_DIR` > platform default (`D-067`) ⇒ **the GUI can be pointed at a dir the harness drove.** |
| **B** | `xgen-client/src/desktop.rs:629` | `get_spaces` → `ops::spaces` → **local client state**. |
| **C** | `xgen-client/src/ops.rs` | `write_client_state` at `:466` `:732` `:816` `:1050` `:1892` — `register`, `create_space`, `create_room`, `create_dm_space`, `leave`. **No `join`.** |
| **D** | J-740 | **the desktop client has NO route to `ops::join`** ⇒ every join and rejoin in this leg is driven by the harness, never by clicking. |
| **E** | `xgen-mptest/tests/mp_admission_soundness.rs:69-75` | ports 8592-8595 taken; **next file starts at 8596.** |
| **F** | `xgen-mptest/Cargo.toml` | *"Not a shipped artifact; never depended on by a binary."* 📌 See §3's instrument decision. |
| **G** | `cdp-debug.ps1` | client CDP base **9222**; `-Debug` launch; ⚠️ **the CDP port opens BEFORE Svelte mounts `window.__XGEN_DEBUG__` — retry `snapshot()` until non-null.** |

---

## §3 — 🔧 THE INSTRUMENT: A STAGING HARNESS, NOT A TEST

🛑 **A `cargo test` tears its rig down at the end and leaves nothing to attach to.** This leg needs the node **alive** while a GUI connects to it.

🔒 **RULED (Chat's seat, tooling — `D-123`): `xgen-mptest/examples/stage_admission.rs`, run with `cargo run -p xgen-mptest --example stage_admission`.**

⚠️ **Why an example and not `src/bin/`:** `xgen-mptest`'s own header says *never depended on by a binary*, and a `[[bin]]` would contradict it in the crate that exists to be black-box. **An example is conventional for exactly this, adds no shipped surface, and stays out of `cargo test`.** 📌 If `examples/` turns out to drag the crate into a build path it should not be in, **that is a deviation to REPORT, not to work around by adding a bin.**

**What it does:** builds the rig on **port 8596**, drives alice and bob to the staged state, then **prints the two data dirs, the node URL and the exact GUI launch line, and BLOCKS on stdin until Enter.** On Enter it tears down cleanly. 🛑 **It must print its teardown, and it must not leave orphan processes** — Leg 1 left zero and that is the bar.

---

## §4 — THE TWO SITUATIONS

### `S-3` — the gap, on the wire and then on the screen

alice creates a Space and a room. She invites bob; bob joins. **While bob is away**, alice invites carol and dave, **removes carol** (`ban`), and sends **three messages** into the room. bob rejoins.

**Wire half — print verbatim:** bob's `members` after rejoining · bob's `history` for the Space and for the room · **explicitly, whether the three messages sent during his absence are present or absent** · whether carol's removal is visible to him, **and whether any reason for it is.**

🔑 **`D-154`④ says the gap is closed to CONTENT and open to MEMBERSHIP STRUCTURE.** ⇒ **the expected shape is: he can see that carol was removed and cannot see one word of why, and cannot read the three messages.** **Observe it; do not assert it.**

**Screen half:** stage and look. **What does the members list show him? Is there any mark that he was away? Does the room's history show a gap, or does it show a continuous conversation with a hole in it that reads like nothing happened?**

### `S-4` — the Space that may never have appeared

alice creates the Space and **three rooms**. bob joins, is added to the rooms, leaves, rejoins.

**Wire half — print verbatim at four points** (after join · before leave · after leave · after rejoin): bob's `spaces` · bob's `rooms` · and **alice's same two verbs at the same four points, as the control.**

**Screen half:** 🔑 **stage BOTH clients and look at them side by side. Alice's should show the Space; the question is whether bob's shows anything at all.**

---

## §5 — 📋 THE VIEWING SESSION. **CHAT DRIVES; JOE RULES.**

🛑 **Joe is not asked to run commands.** Chat self-drives the whole loop (`D-123`, and the standing CDP practice):

1. `cargo run -p xgen-mptest --example stage_admission` detached from a **`.ps1` file** (`N-206`), sentinel + log.
2. Poll for the staged-and-waiting marker in the log. **Do not proceed on elapsed time.**
3. Launch the GUI **detached, `-Debug`**, once per identity, with `XGEN_DATA_DIR` set to that identity's dir (site A).
4. Poll CDP **9222**; ⚠️ **retry `snapshot()` until non-null** — the port opens before Svelte mounts.
5. Capture the registry snapshot **and a screenshot** per identity.
6. Send Enter to the example; confirm teardown; **verify zero orphan `xgen-*` processes and zero ports left open.**

📌 **What Joe receives: alice's screen and bob's screen, side by side, plus the wire transcript.** 🔓 **The verdict on both situations is his and is not written for him.** ⚠️ **If the capture fails, this leg hands back with the wire half and says the screen half is missing — it does not substitute a description of what the screen probably showed.**

---

## §6 — VERIFICATION

🔒 **Chat re-drives every number on a FORCED REBUILD (Rule 5), `Compiling xgen-mptest` present before any result is read.**

| id | what it proves | how |
|---|---|---|
| **V-1** | The example builds and adds nothing to the suite | `cargo test --workspace` ⇒ **1667 / 0 / 68 × 58 SUITES, UNCHANGED.** 🛑 **An example must not appear as a test. Any movement at all is a finding.** |
| **V-2** | The wire halves run | The two situations added to `mp_admission_soundness.rs` ⇒ **+2 ignored, +0 passed, +0 SUITES** (same binary). Then `-- --ignored` ⇒ **6 passed**, transcript captured whole. |
| **V-3** | The staging harness stages | The example reaches its wait marker, prints both data dirs and the node URL, and exits 0 on Enter. |
| **V-4** | The screens were seen | Two CDP snapshots and two screenshots, **non-null**, captured and handed to Joe. |
| **V-5** | Nothing leaked | Zero orphan `xgen-*` processes; ports 8596 and 9222 free after teardown. |
| **V-6** | Scope | `git diff --stat` ⇒ **one modified test file and one new example, nothing else.** 🛑 **A change under `xgen-client/` or `ui/` is a deviation to REPORT** — this leg observes the client, it does not fix it. |

🔒 Carried by scope: vitest **172 / 172 × 9 FILES** · svelte-check **0 / 34 / 15**. 🛑 **Catalogue UNMEASURED.**

---

## §7 — WHAT THIS LEG MUST NOT DO

1. 🛑 **It must not fix `ops::join`'s missing state write.** ⚠️ **That is the single most tempting thing in this milestone** — it is a five-line change, obviously right, and **it would destroy the observation.** It is a finding for Joe, and if he schedules it, it belongs to its own arc.
2. 🛑 **It must not assert the Space is absent from bob's client.** §1's prediction is a code read, not a measurement.
3. 🛑 **It must not touch `ui/`, `skin.css`, or any appearance surface** — Joe's exclusively (`D-123`).
4. 🛑 **It must not add a `[[bin]]` to `xgen-mptest`.**
5. 🛑 **It must not describe a screen it did not capture.**
6. 🛑 **It must not write a `READING` before the run.** Leg 1's `D-3`, and it is the only defect in this arc so far that came from the implementing seat rather than the specifying one.
7. 🛑 **It must not drive S-5, S-6 or S-10** — no emitter; `M-ADMISSION-SURFACE`'s.
8. 🛑 **Never push.**

---

## §8 — DoD

- [ ] `xgen-mptest/examples/stage_admission.rs` exists; port 8596; prints both data dirs, the node URL and the GUI launch line; blocks on stdin; tears down on Enter
- [ ] S-3 and S-4 wire halves added to `mp_admission_soundness.rs`, `#[ignore]`, ports 8597-8598 with a collision comment
- [ ] Both situations print alice's control alongside bob's observation
- [ ] `V-1` — workspace floor **UNCHANGED at 1667 / 0 / 68 × 58**
- [ ] `V-2` — 6 passed under `--ignored`, transcript captured whole
- [ ] `V-3`, `V-5` — staging reached, clean teardown, zero orphans, ports free
- [ ] `V-4` — two CDP snapshots and two screenshots, non-null
- [ ] Every `READING` written **after** the output existed
- [ ] Deviations **reported, not absorbed** (Rule 6)

📌 **"Commit pushed" is deliberately not a DoD item.**

---

## §9 — 🔓 OPEN DECISIONS INSIDE THIS RUNBOOK

**None for the implementing seat.** 🔓 **Three findings await Joe's ruling and are deliberately NOT gated on this leg** — the open door (every Space is `open` and cannot be otherwise) · the spent invite (a lapsed invite leaves you worse off than no relationship at all) · the silent demotion. **Leg 3 is where they are put to him, with the screens in hand.** 📌 Stated as a count so a reader can tell *nothing was open* from *nobody looked*.
