# M-RP-IDENTITY-RESOLUTION Leg F — Phase-0: how the seven obligations are produced
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Aug 2026  
> **Last updated**: 2026-08-04  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — What this is, and what it is NOT

Leg F is the milestone's FIRST behaviour verification. Legs A, B, C and D are compile-, type- or computed-style-verified only. This document exists because `M_RP_IDENTITY_RESOLUTION.md` §8 prices Leg F as *"two clients, a real join, a real `not_found`"* — and that sentence hides the whole leg.

**This is NOT the runbook.** It settles WHAT CAN BE PRODUCED AT ALL, so the runbook is written against measured levers rather than against a plan. Two §§ are open and are Joe's.

**It is also not a re-derivation of the seven obligations.** They are locked at `M_RP_IDENTITY_RESOLUTION.md` §8 (Leg F) and appear here only as the rows of §5's matrix.

---

## §1 — Grounding (measured 2026-08-04 at `3bde951`, HEAD = origin/main, tree clean)

Gate `.\xgid-slot-gate.ps1` re-run clean-tree: **PASS 74** (65 BOUNDARY / 5 DESCRIPTIVE / 3 INTERNAL / 1 UNREAD). Ports 5173/5174/5175/9222/9322/9422 free; zero XGenProtocol processes. `git diff --name-only 9901036 HEAD -- '*.rs'` EMPTY and `git diff --name-only 03c92cc HEAD -- 'ui/*'` EMPTY, so cargo **1596/0/62 × 56** and svelte-check **0/34/15** hold by scope. 📌 *Catalogue 435 is inherited from `03c92cc` and was NOT re-read live this session — stated, not claimed.*

**G1 — MULTI-INSTANCE IS A DESIGNED, EXERCISED CAPABILITY, NOT SOMETHING TO INVENT.** `bin/instances/` holds `m3-alice · m3-bob · m3-carol · m4-alice · m4-bob` (dated 17 May 2026), and **each client instance dir carries its own `xgen-client_keypair.enc`** ⇒ distinct identities per instance, on disk, already.

**G2 — ROOT PRECEDENCE, GROUNDED AT THE PRODUCER.** `xgen-client/src/main.rs:40-61` — `resolve_data_dir` takes the `--data-dir` flag > `XGEN_DATA_DIR` env > platform default via `xgen_common::data_dir::resolve_data_root`, then `--instance <label>` rebases under that root (`instance_path`). Fails fast, no silent `exe_dir()` fallback (M12.2b F9/D5). `main.rs:81` reads `XGEN_DATA_DIR` again for the legacy notice, and `:155` passes the resolved dir into `desktop::run` ⇒ **the env var reaches the GUI path.**

**G3 — D-101 DOES NOT THREATEN IDENTITY STABILITY.** `xgen-client/src/app.rs:413` states it in the declaration, not in a summary: clean-slate-on-start *"wipes `xgen-client_config.toml` only"*. The keypair and `xgen-client_state.json` survive ⇒ an instance keeps ONE STABLE XGID across launches. *(Window used: the doc comments at `app.rs:388-413` and `:522-543`, read whole.)*

**G4 — 🔑 THE JOINER NEEDS NO GUI AT ALL.** `main.rs` dispatches `ClientCommand::` `Init · Whoami · Status · Spaces · Rooms · Register · CreateSpace · CreateDmSpace · SelfThread · CreateRoom · **Invite** · Ban · RoomUpdate · Thread · **Join** · **Leave** · Send · History · Fetch · Redact · **Members**`. The kickoff's asymmetry lead is stronger than it was stated: **only the OBSERVING client needs the harness.**

**G5 — AND THE CLI JOIN IS THE SAME EVENT, PROVEN AT THE PRODUCER RATHER THAN INFERRED FROM D-092.** `app.rs:3032 cmd_join` is a thin shim: it builds a `SessionState`, calls `session.ensure_identity`, and delegates to **`ops::join`** (`ops.rs:1579`), which signs **`EventType::MembershipJoin`**. The GUI arm reaches the same `ops::` function. ⇒ **a headless join produces the identical `membership.join` the live router consumes.** *This was the item named at open as most likely wrong; it is measured, and it held.*

**G6 — ⑥'s LEVER EXISTS AND COSTS NOTHING.** `ops::identity_get` (`ops.rs:539`) opens with `ctx.session.ensure_connected(...)` before `identity_get_on`. ⇒ **a node that is down at fetch time makes the Tier-1 fetch fail on the ordinary path.** No code, no fault injection, no harness addition — stop the node process.

**G7 — 🛑 OBLIGATION ⑤ HAS NO PRODUCT VERB BEHIND IT, ANYWHERE.** `xgen-core/src/identity/registry.rs` exposes exactly `new · register · get · contains · apply_update · revoke · is_revoked · set_trust_expiry · len · is_empty · all · upsert · save · load`. **There is no `remove` and no `erase` in any crate** (searched across `xgen-node/src`, `xgen-core/src`, `xgen-common/src`; every `erase` hit is M12.4 blob redaction). `identity.not_found` is emitted at `xgen-node/src/app.rs:3567` on exactly one condition — `registry.get()` returned `None`. ⇒ **a real erasure cannot be produced by any verb this product has.** That is what §3 exists to answer.

**G8 — THE BINARY TRAP IS WIDER THAN THE KICKOFF STATES.** `bin/xgen-client.exe` is 27,426,816 B dated **21 May 2026** — it predates the entire milestone. ⚠️ **AND IT IS THE SAME FILENAME THE RUNNING DEV APP LOCKS** (the J-511 finding: `cargo test` dies on *"failed to remove file …xgen-client.exe"*). Separately, `run-client.ps1 release` emits **`bin\xgen-client-app.exe`** — a DIFFERENT file. Two names, one trap each.

**G9 — `run-client.ps1` CANNOT LAUNCH A SECOND DEV INSTANCE, BY DESIGN.** Two parameters only (`-Mode`, `-Debug`). Guarantee 1 (`:85-99`) REFUSES to start when 5173 is held, and names the holder. That refusal is `M-RP-DEVSERVER-GUARD` working, not a defect to route around.

---

## §2 — 🔓 OPEN, DECISION 1: THE JOINER VEHICLE (Joe)

The joiner must hold a distinct stable identity, accept an invite, join, and — for ⑤ — be absent from the node registry. **It renders nothing that Leg F reads.**

| | what it is | ① USER-VISIBLE IMPACT | ② RESOURCE COST |
|---|---|---|---|
| **J1** | headless CLI joiner — `XGEN_DATA_DIR=<root>\bob` + `register` + `join` | **NONE.** The CLI paints no pixels; the observing client is the entire subject of ①–⑤ | one warm `cargo build`; **zero** launcher change, zero second Vite, zero second webview, no port contention. ⑦ becomes N processes in a loop |
| **J2** | second GUI from a FRESH release build, `XGEN_DATA_DIR` set | **NONE** — nothing on the joiner's screen is an obligation | a full `cargo tauri build`; and per G8 the rebuild must be STATED in the run, or it is the 21-May trap wearing a green tick |
| **J3** | second `cargo tauri dev` | **NONE** | blocked twice (G9 port refusal + G8 exe lock). Changing the launcher is its own milestone, never a rider |

**🔑 CHAT RECOMMENDS J1.** It is the only option whose cost is a warm debug build, and G5 proves it is not a weaker stimulus: the event is identical because the code path is identical. J2 buys nothing that ①–⑦ reads.

**⚠️ THE HONEST LIMIT OF J1, RECORDED RATHER THAN DISCOVERED AT RUN TIME.** J1 verifies the OBSERVER's rendering of a remote identity. It does NOT exercise a second GUI's own behaviour — nothing in ①–⑦ asks it to, but if a later milestone wants two-panel observation, J1 does not deliver it and must not be cited as if it had.

---

## §3 — 🔓 OPEN, DECISION 2: HOW ⑤'s `not_found` IS PRODUCED (Joe)

G7 is the constraint: **the product cannot erase an identity.** So the state is produced some other way, or not at all.

| | route | ① USER-VISIBLE IMPACT | ② RESOURCE COST |
|---|---|---|---|
| **E1** | remove the joiner's row from `xgen-node_identities.db` out of band, between the join and the observer's fetch (node down → edit → node up) | **NONE at production time** — this is test setup. The impact under test is the observer's row, which is the obligation | minutes. The file already exists under `bin/instances/*/` |
| **E2** | two federated nodes — joiner registers on B, Space homed on A; A holds the DAG event and no registry record | **NONE** | a second node instance, federation config, a longer run. Real setup cost |
| **E3** | declare ⑤ unproducible; record as a FINDING, not a failure | ⑤ ships unverified **and says so out loud** | zero |

**🔑 CHAT RECOMMENDS E1, AND THE REASON IS THAT E1 AND E2 EXERCISE THE SAME CLIENT PATH.** `M_RP_IDENTITY_RESOLUTION.md` §9 already records that the client **cannot distinguish erased-here from never-replicated-here**. E2 is the never-replicated case; E1 is the erased case; both reach the observer as one `identity.not_found`. ⇒ **E1's cheapness costs no fidelity for what Leg F is actually testing**, which is the observer's render.

**🛑 AND THE RECORD MUST SAY WHAT E1 IS.** The state is produced by FILE SURGERY, not by a verb. Writing it up as *"an erased identity"* without that qualifier would claim the protocol has an erasure story this milestone verified. **It does not, and Leg F must not be the place that implies otherwise.** 📌 *M13 §3c already records the related defect — erasure is invisible to anyone holding a cached record. Read alongside.*

---

## §4 — ⑥ AND ⑦ ARE MEASUREMENTS, NOT GATES

Locked at §8, restated because it changes how the runbook is written: **⑥ and ⑦ produce NUMBERS that re-price a deferred decision. They are never written as ticks and they cannot fail.**

- **⑥ — the fetch that fails or times out.** Lever: **G6 — the node is down at fetch time.** Expected: the row STAYS ④ and **nothing retries it**. That residue is `D-126`'s T3. The number is *how visible the stuck row is*, after which T3's bounded retry returns as a live option or becomes a defect.
- **⑦ — join concurrency.** A3 shipped one-shot `identity_get` (one connect/auth/`goodbye` per joiner). The question is a COUNT: how many joins land at once in real use. 🔑 **J1 is what makes this measurable at all** — N CLI processes launched together is a scripted loop; N GUIs is not. If N-at-once is common the batched form returns as a live option; **if it is not, it is CLOSED WITH ITS REASON.**

---

## §5 — THE OBLIGATION → PRODUCTION MATRIX

Rows are `M_RP_IDENTITY_RESOLUTION.md` §8's seven. Columns say what produces the state **under J1 + E1** (the recommendation). ⚠️ *If Joe rules otherwise this table is rewritten before the runbook opens.*

| | obligation | produced by | producible? |
|---|---|---|---|
| ① | a real join producing `data-unresolved="unasked"` on a client row | observer invites; CLI joiner `join` (G5) | ✅ yes |
| ② | a real `not_found` producing the ③ filter AND §5a's E2 DM exception | E1, plus a DM Space for the exception half | ✅ under E1 |
| ③ | a populated roster giving `erasedHidden` something to count | rides on ② | ✅ under E1 |
| ④ | a joiner resolving — the name lands **and the AI badge lights** | joiner registered with the AI flag; Leg D's Tier-1 fetch on join | ⚠️ needs the AI-registration route grounded — §6 R3 |
| ⑤ | an erased live joiner reaching `_notFound`, or MARKED as DM counterpart | E1, DM variant | ✅ under E1 |
| ⑥ | a fetch that FAILS or TIMES OUT; row stays ④, nothing retries | G6 — node down at fetch time | ✅ yes, measurement |
| ⑦ | join concurrency — the count | N CLI joiners in a loop | ✅ yes, measurement |

**🛑 ④ IS THE ONE THAT MATTERS MOST AND IT IS THE ONE ROW THAT IS NOT YET GREEN.** Leg D's Phase-0 §2b found that `members-panel.svelte:101` tests `m.unresolved` BEFORE it reads the book ⇒ the badge is gated on the MARKER CLEAR, not on the record arriving. **That finding is ARGUED, never SEEN**, and only ④ can show it. ⇒ **grounding the AI-registration route is the runbook's first job, named here rather than met at the console.**

---

## §6 — SEQUENCING CONSTRAINTS THAT ARE NOT OBVIOUS

- **R1 — BUILD BEFORE LAUNCH, ALWAYS.** G8: the dev app locks `xgen-client.exe`. The joiner binary is built while nothing is running; the observer launches after.
- **R2 — 🛑 A JOIN REQUIRES A PRIOR INVITE, AND THE INVITE COMES FROM THE OBSERVER'S IDENTITY.** `ops::join` reads `crate::batch::get_invite_bootstrap` and sets `prev_events=[invite_id]` so the join is causally after the invite on the `membership:{space}:{invitee}` key. ⇒ **the invite must be issued BEFORE the observer GUI holds its data dir**, or the run deadlocks on its own file locks. A real ordering constraint, which is why it is written down here.
- **R3 — ④'s AI FLAG IS UNGROUNDED AND IS NOT CLAIMED.** `ClientCommand::Init` takes an `--ai` argument (`app.rs:509` cites *"`Some(..)` from `cmd_init --ai`"*), which is the likely route, **but it has NOT been read end to end.** Runbook item 1.
- **R4 — SWEEP AND ASSERT QUIESCENCE BEFORE EVERY COUNT.** N-105 / N-108 / N-112 / N-115: the registry breathes with menu state, store contents, selection AND saved-UI-state count. Every number in this leg states all four.
- **R5 — N-123.** A probe that persists a mutation OWES A CLEANUP CALL, and any session that touched inline styles ends with `location.reload()`.
- **R6 — HMR IS NOT A COLD START.** Restart rather than trust it.

---

## §7 — WHAT LEG F DOES NOT DO

- It does not fix `M_RP_MEMBERS.md` §6a's tail-8 lock-versus-build gap. ⚠️ Leg C-3 made it MORE visible, and ④ rows in this run will show it. **Filed, Joe's, gates nothing here.**
- It does not touch `entity-avatar.svelte:125`'s `isAi` third-state collapse (J-655, Joe's).
- It does not touch `N-169` — `roomLatch.effectiveSpaceId` is UNMEMOISED ON PURPOSE and Leg E's discharge depends on that cascade.
- It does not build an erasure verb. **G7 is a finding, not this leg's scope.**
- It does not change `run-client.ps1`. G9's refusal is the guard working.

---

## §8 — Legs

- **F-0** — this document. No code, no launch. Open §§: **§2** and **§3**.
- **F-1** — the runbook, written from the LOCKED §§ only. First job: R3, the AI-registration route. 🔑 *Written so Clair can refuse it: cite the producer, not the name; name where it is most likely wrong; and never present that list as a census of its errors.*
- **F-2** — **THE RUN. INTERACTIVE, CUSTODY TRANSFERS TO JOE UNDER `D-132`.** Start announced, end announced, hands off in between.
- **F-3** — the close: records, plus ⑥/⑦'s numbers feeding back into `D-126` and A3's filed option. **The milestone closes with this leg.**

---

## §9 — DoD

- [ ] §2 the joiner vehicle locked by Joe
- [ ] §3 ⑤'s production route locked by Joe
- [ ] R3 grounded before the runbook locks — the AI-registration route read end to end, **window stated**
- [ ] ①–⑤ observed on a real client row, EXERCISED not asserted, with an idle control in the same run
- [ ] ⑥ and ⑦ recorded as NUMBERS, never as ticks; `D-126` T3 and A3's batched form each re-priced or closed with a reason
- [ ] floors re-measured before the first edit if Leg F becomes a fix leg — **cargo BEFORE any `.rs` touch**
- [ ] Records: JOURNAL + CLAUDE.md PLAY + ROADMAP + `M_RP_IDENTITY_RESOLUTION.md` + this doc in one commit (`D-074`)

---

## §10 — Handoff

**Open for Joe: §2 (J1 recommended) and §3 (E1 recommended).** Everything else in this document is measured, or is Chat's under `D-123`. **Nothing is launched and nothing is built until §2 and §3 are locked** — a run improvised around a launcher that does not exist is how a leg produces numbers nobody can reproduce.
