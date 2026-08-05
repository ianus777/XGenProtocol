# RUNBOOK — M-RP-IDENTITY-RESOLUTION Leg F: live verification of the seven obligations
> **Status**: COMPLETED  
> Version: 1.7  
> Date: Aug 2026  
> **Last updated**: 2026-08-05  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 — What this is, and who holds it

🔒 **LOCKED BY JOE 2026-08-04 (v1.1 → v1.2 ACTIVE).** v1.0 went to Clair's adversarial read (J-642: the read runs BEFORE the lock). **IT WAS NOT CLEAN — see §12.** v1.1 carried the repairs; Joe locked it as read. ⚠️ *Joe was offered a second short read on the restructured R-3 and declined it — **§11-6 and §11-7 therefore ship as LIVE, ACCEPTED risks**, not as doubts awaiting a pass.*

**Locked inputs, from `M_RP_IDENTITY_RESOLUTION_LEGF_PHASE0.md` (Joe, 2026-08-04):** §2 = **J1**, the headless CLI joiner · §3 = **E1**, registry-row surgery, recorded as file surgery and never as *"an erased identity"*.

**Leg F is a VERIFICATION leg. It is expected to change no code.** 🛑 **If it finds a defect it becomes a fix leg — and then the floors are measured BEFORE the first edit, cargo before any `.rs` is touched.** Floors carried in: cargo **1596/0/62 × 56** · svelte-check **0/34/15**, both holding by scope at `3bde951`.

**CUSTODY: `D-132`. The run is INTERACTIVE.** Chat announces the start, Joe takes the running apps, Chat announces the end. Between those announcements the apps are Joe's.

---

## §2 — R3, GROUNDED AND GREEN — AND IT CLOSED A RISK NOBODY HAD NAMED

**R3-a — THE ROUTE EXISTS END TO END.** `InitArgs` (`app.rs:710`) carries `--ai` and repeatable `--cap KEY=VALUE`. Each link at its producer: `app.rs:220` `AiSection.is_ai` → `app.rs:2714` `if ai.is_ai` at init-write → **`ops.rs:342-343`** register branches on the `[ai]` section → **`ops.rs:439/457/469`** `FetchedIdentity.is_ai` off `identity.record` → **`address_book.rs:89`** `SeenRecord.is_ai`, set at **`:130`** inside `absorb_fetch` → TS `toDescriptor` → `flags.isAi`. 📌 *`address_book.rs:459` already asserts `"is_ai carries over from the wire"`.*

✅ **CONFIRMED BY THE SECOND READER, AND THE ONE GAP v1.0 ADMITTED IS NOW CLOSED.** v1.0's §11-4 flagged that `ops.rs:342` was read at the branch and not through `cmd_register`'s body — *"if the section is loaded from a path the joiner's data root does not own, Bob registers as a human."* **Closed:** `ops::register` takes `ai_section` as a **parameter** (`ops.rs:314`); the caller supplies it. `main.rs:76` resolves `data_dir` (flag > `XGEN_DATA_DIR` > default), `:93-96` derives `config_path` from it, `:254` `load_ai_section(&config_path)` reads **Bob's own config**, and `init --ai` (`:245`, `cmd_init(args, &data_dir)`) writes that same file. **Bob registers as an AI from his own data root.**

**R3-b — 🔑 AND THE D-101 RISK IS FORECLOSED, NOT MITIGATED.** `clean_slate_config` (`app.rs:566`) has **exactly ONE production call site: `desktop.rs:866`**, inside `desktop::run`'s GUI startup. All nine `app.rs` call sites (`:6765 · :6795 · :6835 · :6859 · :6886 · :6927 · :6966 · :6980 · :6999`) fall between the `#[cfg(test)]` markers at `:6492` and `:7008`. ✅ *Checked in both directions by the second reader.* ⇒ **the CLI never runs the wipe**; `[ai]` survives `init` → `register` by construction.

**🔑 J1 PAYING A SECOND TIME, AND THE PHASE-0 DID NOT SEE IT COMING.** J1 was chosen on cost. It is also the only vehicle where D-101 cannot eat the AI flag between two commands. *Recorded because it is luck, not design.*

---

## §3 — 🛑 ① AND ④ CANNOT BE THE SAME JOIN

Grounded at `ui/common/lib/components/widgets/members-panel.svelte:101`:

```
flags: m.unresolved ? {} : { isAi: rec?.is_ai ?? false },
```

**① wants a row RESTING at `data-unresolved="unasked"`. ④ wants that marker CLEARED with the badge lit.** `resolveMember` clears `unresolved` the moment the Tier-1 fetch returns ⇒ on a healthy node `unasked` exists for **one tick**.

**🔒 CHAT'S RULING (mechanical, `D-123`):**

1. **INSTRUMENT BEFORE STIMULUS** — a `MutationObserver` installed **before** the join. *A transient state is not unobservable; it is unobservable by a probe that arrives after it.*
2. **⑥ IS ALSO HOW ① IS SEEN AT REST.** A failed fetch parks the row at `unasked` indefinitely — there is no retry. Two obligations filed a week apart are one mechanism seen twice.
3. ⇒ **①+④ ride ONE join (R-1, healthy node); ①-at-rest+⑥ ride a SECOND (R-2, fetch fails).**

✅ **§2b IS NOW CONFIRMED AT THE SOURCE, AND IT IS STRUCTURAL RATHER THAN MERELY LIKELY (second reader).** Both the marker clear and the badge derive from the **same** `m.unresolved` flip, so they land in one reactive flush. ⇒ ***badge-before-clear is not just unobserved, it is unreachable.*** **R-1 still runs it** — the claim is now a prediction with a mechanism, and a prediction is worth one measurement. ⚠️ *But its status changes: R-1 CORROBORATES a structural property; it no longer DISCOVERS an unknown ordering.*

---

## §4 — THE RIG

| role | data root | vehicle |
|---|---|---|
| **OBSERVER** (Joe's identity, the subject) | its existing root — **unchanged** | `run-client.ps1 -Debug` → Vite 5173, CDP 9222 |
| **BOB** — resolves, AI, and the resolved CONTROL | `<legf-root>\bob` | CLI, `XGEN_DATA_DIR` |
| **CAROL** — fetch fails (⑥) | `<legf-root>\carol` | CLI |
| **DAVE** — erased (⑤, E1) | `<legf-root>\dave` | CLI |
| **N joiners** (⑦) | `<legf-root>\c01…cNN` | CLI, launched together |
| **NODE** | its existing root | `run-node.ps1` |

**CLI shapes, grounded (`app.rs:962/1008/1065/1113`):** `init [--passphrase] [--ai] [--cap k=v]` · `register --name <n> [--re-registration]` · `create-space --name <n> [--auth-tier]` · `create-room` · `create-dm-space` · `invite --space <s> --identity <id> --role <r> [--valid-for-days] [--note]` · `join --space <s> [--room <r>]` · `members --space <s>`.

### §4.1 — Order, and why each step is where it is

- **🛑 O1 — BUILD FIRST, WHILE NOTHING RUNS.** `cargo build` for the CLI binary. The dev app locks `xgen-client.exe` (J-511). `bin/xgen-client.exe` dated **21 May 2026** predates the milestone and **must not be used** — it would produce a green run that proves nothing. **The binary's timestamp is recorded and stated in the record.**
- **🆕 O1b — THE SPACES AND ROOMS EXIST BEFORE ANYTHING JOINS (F5).** The observer's identity creates: one **group Space + room** (R-1/R-2/R-3-group/R-4) and one **DM Space** with Dave as counterpart (R-3-DM, §5a's E2). ⚠️ *v1.0 omitted this entirely while resting obligation ⑤ on it — **a locked obligation must not rest on unstated setup.*** Owner and ids are written down before the run, not chosen at the console.
- **🛑 O2 — EVERY JOINER IS `init`'d AND `register`'d BEFORE THE OBSERVER GUI LAUNCHES.** Bob takes `--ai`; Carol, Dave and the ⑦ cohort do not.
- **🛑 O3 — THE INVITES ARE ISSUED BEFORE THE OBSERVER GUI LAUNCHES — AND THE REASON IS THE FILE LOCK, NOT THE PROTOCOL.** The invite is issued **by the observer's identity, from the observer's data dir, which the GUI will hold**. Issue every invite in the CLI phase or the run deadlocks on its own locks. ⚠️ **CORRECTED AT v1.1, KEPT NOT ERASED (`D-131`): v1.0 justified this as *"there is no join without a prior invite."* THAT IS FALSE.** `ops::join`'s match (`ops.rs:1615-1646`, read whole) has a `_ =>` arm — *already a member, a Room join, or the Node refuses* — falling to `get_dag_tips` and then `rejoin_anchor_or_root`, **a documented path that signs a `MembershipJoin` with no invite.** Chat read only the `Some` arm and generalised from it. 🔑 *The ordering survives; its stated reason did not, and a rule kept for a false reason is a rule that will be discarded the first time someone checks it.*
- **O4 — port sweep, then node, then the observer.** `run-client.ps1` refuses outright if 5173 is held (`:85-99`); that refusal is the guard working.
- **O5 — the joins fire one at a time, on Chat's cue, from Joe's console.**

---

## §5 — PRE-FLIGHT (before custody transfers)

- [ ] `git --no-pager status` clean; `rev-parse HEAD` recorded; `git ls-remote origin main` agrees
- [ ] `.\xgid-slot-gate.ps1` on the clean tree — **PASS 74** expected
- [ ] zero XGenProtocol processes; **5173 · 5174 · 5175 · 9222 · 9322 · 9422 all free**
- [ ] O1 build done, **binary timestamp recorded**
- [ ] O1b Spaces/rooms/DM created; O2 registrations done; O3 invites issued — all from the CLI
- [ ] node up; **node console shows NO `identity registry load failed` warning** (see R-3 F4 stop condition)
- [ ] observer up via `run-client.ps1 -Debug`
- [ ] `__XGEN_DEBUG__` **retried until non-null** — the CDP port opens before Svelte mounts; port-up is NOT ready
- [ ] baseline registry count read **QUIESCENT**, stating **all four axes** (N-105 menu · N-108 store · N-112 selection · N-115 saved-UI-state count)
- [ ] ids **ENUMERATED** via `querySelectorAll('[data-debug-id]')` — the pattern is never assumed

---

## §6 — THE RUNS

**🛑 R0 — EVERY OBSERVATION IN THIS LEG IS SINGLE-USE PER IDENTITY, AND SETUP MEASURED WHY (v1.4, 2026-08-05).** `partition_observed` touches held identities and never re-fetches them (the F1 mechanism), and the observer's book **persists to disk** at `xgen-client_address_book.json`. ⇒ ***the moment a joiner is fetched once, that identity can never again produce `unasked`, `not_found`, or a first-fill — it is spent.*** A misfired R-3 does not cost a retry; it costs Dave. 🔑 **MITIGATION, AND IT CONVERTS A ONE-SHOT INTO A REPEATABLE:** a **restore point** was taken before the run — `xgen-client_address_book.json` · `xgen-client_state.json` · `xgen-client_uistate.json` · `xgen-node_identities.db` · `xgen-node_state.json` · `xgen-node_space_local_metadata.json`, copied to `legf-restore-point-20260805-065013` under the data root, **hash-verified identical**. Restoring the book rolls a spent identity back to unfetched. ⚠️ *Restore with all apps DOWN — the client rewrites the book on save and would overwrite the restore.*

**🔒 STARTING STATE, MEASURED NOT ASSUMED (2026-08-05).** The observer's book holds exactly **two** records — `BobLegB` (…DS5Gqmv…) and **self, `Joe` (…VtLICfG…)** — both `is_ai: false`. ⇒ **`BobLegB` is a ready-made RESOLVED CONTROL ROW that no fill will disturb**, and every Leg F joiner is absent from the book and will therefore be fetched. 📌 *The node registry also holds an inert third record, a second `Joe` (…b33IZ7Z…, registered 2026-07-17), which appears in **no** Space, DAG, metadata or address book and cannot render a row — **filed as residue, not repaired** (`D-131`).*

**🔑 ④ HAS A DECISIVE PRE-CHECK THAT COSTS NOTHING AND RUNS BEFORE THE GUI EXISTS.** `IdentityRecord.is_ai` is `#[serde(default, skip_serializing_if = "is_false")]` (`registry.rs:36-38`) ⇒ **absent from disk when false**, which is why all three existing records show no `is_ai` at all. ⇒ after the AI joiner registers, **its record in `xgen-node_identities.db` MUST carry `"is_ai": true`.** If it does not, ④ is dead at setup rather than at the console. ⚠️ *Absence of the field is NOT evidence of a missing feature — it is the documented human-canonical form.*

Every run carries an **idle control** and a **falsification case**. 🔑 *A probe returning identical values for differentiated inputs cannot fail.* 🛑 **AND THE CONTROL MUST NOT MUTATE THE STATE IT IS CONTROLLING FOR — see §12 F1.**

**🔒 THE DOM, NAMED RATHER THAN GUESSED (second reader).** `data-unresolved` renders on **`<div class="entity-item">`** (`entity-item.svelte:126`); `[data-ai]` sits on the `EntityAvatar` **child inside it**. Containment: `div.members-panel` → `section` → `ul.entity-panel` → `li.entity-panel-listitem` → `div.entity-item`. Prop chain: `members-panel` `memberRows[].unresolved` → `EntityPanel` `items[].unresolved` (`:42`, `:162`) → `EntityItem`. ⚠️ *v1.0's §11-3 called the observer root a guess; it is now measured, and `div.members-panel` does contain the rows.*

### R-1 — ① and ④: Bob joins, resolves, and the badge lights
1. Install the `MutationObserver` on `div.members-panel`. 🛑 **CONFIG IS `{ childList: true, subtree: true, attributes: true, attributeOldValue: true }`, AND ADDED NODES ARE INSPECTED FOR `data-unresolved`.** ⚠️ **v1.0 SPECIFIED ATTRIBUTES ONLY — A DEFECT THAT WOULD HAVE LOST ①'s PRIMARY CAPTURE (F3).** A joiner's row **mounts with `data-unresolved="unasked"` already set**; that is a **childList** record, not an attribute mutation, so an attributes-only observer reports **nothing** and it reads exactly like the state never occurring. *The later clear IS an attribute mutation on the persisted keyed `<li>` and was always fine.*
2. **Assert the observer is attached and its subject is READABLE before the stimulus** (N-110).
3. Joe fires Bob's `join`. Read the recorded log.
- **Expect:** an added node carrying `data-unresolved="unasked"`; then that attribute REMOVED and `[data-ai]` appearing, **in the same flush, never badge-before-clear** (§3).
- **Control:** an existing resolved member row records **zero** mutations across the same window.
- **Falsification:** `[data-ai]` present while `data-unresolved="unasked"` still set ⇒ §2b is wrong and that is the headline.
- 📌 Also read `.ei-name` computed weight/colour on the `unasked` row — C-3 shipped `500` / `--t3` and verified them on the **sampler**; this is the first time they are seen on a **client** row.

### R-2 — ⑥ and ①-at-rest: Carol joins, the fetch fails
✅ **THE PREMISE IS CONFIRMED AT THE SOURCE (second reader), AND THE FEARED DEFECT DOES NOT EXIST.** `app_client.svelte:249-256`: `fetchJoinerIdentity` wraps `tauriInvoke('fetch_identity')` in `try { … resolveMember(record ?? null) } catch { /* the row stays ④ */ }`, and `desktop.rs:763` maps a fetch `Err` to `Err(String)` so the invoke **rejects**. ⇒ **a rejected invoke never reaches `resolveMember`; only `Ok(None)` (`identity.not_found`) coerces to null and routes to `_notFound`.** *A node blip cannot show a live person as erased.* 📌 *`address-book.svelte.ts:203-204` asserts the same in its own doc.*
1. Observer instrumented as R-1.
2. Joe fires Carol's `join`; **the node is killed inside the fetch window.** `ops::identity_get` opens with `ensure_connected` before `identity_get_on` — the fetch is a **separate** connect from the resident's socket.
3. Node stays down; the row is read at rest, then again after ≥60 s.
- **Expect:** the row RESTS at `data-unresolved="unasked"`, name = `tail()` fallback, **no badge**, **nothing retries** — the second read equals the first.
- **Control:** Bob's resolved row from R-1 is unchanged by the node going down.
- **Number for `D-126`:** seconds/attempts elapsed with the row stuck, and whether anything on screen explains it. **MEASUREMENT, not a tick.**
- ⚠️ *Timing is a race; retry is expected and is not a finding. If it proves unhittable, that is recorded as a limit of the lever and ⑥ reports on what was reached.*

### R-3 — ⑤, ② and ③: Dave is never fetched, then erased
🛑 **RESTRUCTURED AT v1.1. v1.0's SEQUENCE COULD NOT PRODUCE THESE STATES AT ALL — see §12 F1. The superseded steps are recorded there, not deleted (`D-131`).**

🔒 **THE GOVERNING FACT:** `partition_observed` (`ops.rs:2778-2786`, doc `:2767-2777`) splits observed identities into `(to_fetch, to_touch)` by `book.contains`, and **those already held are TOUCHED, NEVER RE-FETCHED**; `fill_from_events` (`:2916-2923`) fetches only `to_fetch`. ⇒ ***once Dave is in the observer's book, no fill will ever re-ask, and his erasure is invisible.*** This is `M_RP_IDENTITY_RESOLUTION.md` §5b, and the Phase-0's E1 wording (*"between the join and the observer's fetch"*) already had the timing right.

1. **Dave joins the group room WITH THE OBSERVER GUI DOWN.** 🔒 **THE PROTECTION IS STRUCTURAL, NOT PROCEDURAL (v1.5, measured 2026-08-05).** Every production write of the observer's address book is in **`desktop.rs` — `:710` and `:775`, the GUI path** (`ops.rs:2947`'s `absorb_fetch` mutates only in memory; every other `book.save` in the crate is inside a test). ⇒ ***no CLI command can write the observer's book at all.*** Registering, creating Spaces, inviting and joining from the CLI **cannot** burn Dave. ⚠️ *v1.1–v1.4 relied on the observer being "scoped elsewhere" so `addMember` would no-op — a behavioural argument (§11-6). **The file-level fact is stronger and does not depend on router state**: with the GUI down there is no writer.*
2. **Node down → remove Dave's object from `xgen-node_identities.db` → node up.**
   🛑 **THIS IS FILE SURGERY AND THE RECORD SAYS SO.** The product has no erasure verb — `IdentityRegistry` exposes `new · register · get · contains · apply_update · revoke · is_revoked · set_trust_expiry · len · is_empty · all · upsert · save · load` and **no `remove`/`erase`/`delete` in any crate** (✅ verified both directions by the second reader). `identity.not_found` fires at `xgen-node/src/app.rs:3567` on `registry.get() == None`.
   🛑 **THE FILE IS A PRETTY-PRINTED JSON ARRAY, NOT SQLITE (F4).** `registry.rs:240-248` writes `serde_json::to_string_pretty(&Vec<IdentityRecord>)`; `:252-256` parses it. ⚠️ ***`CLAUDE.md`'s file-placement table calls it SQLite and is WRONG*** — filed, not fixed here.
   🛑 **STOP CONDITION, AND IT IS THE MOST DANGEROUS THING IN THIS RUNBOOK.** On startup `xgen-node/src/app.rs:776-784` matches `IdentityRegistry::load`; on `Err` it **prints a warning and keeps `NodeRuntime::new`'s EMPTY registry**. ⇒ **a malformed edit does not fail loudly — it makes EVERY identity return `not_found`**, corrupting ①②③④⑤ at once into rows that all look erased. ***A broken rig would read exactly like a finding.***
   🔑 **STATE IT POSITIVELY — THE NODE BANNER COUNTS THE REGISTRY (v1.6, measured 2026-08-05).** The service banner prints **`Identities: N registered`**, and at setup it read **`Identities: 3 registered`**, matching the three records on disk. ⇒ **the check is `N` decremented by exactly one after the surgery — `2` — and `0` IS THE CORRUPTED STATE.** ⚠️ ***"No warning appeared" is a probe that cannot fail; a counter that must land on a predicted number can.*** ⇒ **delete ONE object, keep the array valid; read the banner on restart; if it is not exactly one less, STOP, restore from the restore point, restart. Never interpret a run whose banner disagreed.**
3. Observer scopes to the room **for the first time** → the first fill fetches Dave → `identity.not_found`.
- **Expect (②/③, group room):** Dave **absent from the rendered rows** — `memberRows` filters `notFound.has(id)` (`members-panel.svelte:127`) — while **`addressBook.roster` still contains him** (B-1: `_roster` stays complete, the filter is at render). `erasedHidden` has something to count.
- **Then the DM variant (§5a E2):** repeat in the DM Space. **Expect Dave VISIBLE and MARKED** — `:127`'s `|| m.identity_id === counterpart` is the exception, and `:130-134` gives `'erased'`, which outranks `'unasked'`. §5a-i's `inset 2px 0 0` selection bar must survive the strikethrough.
- **🔒 CONTROL — AND IT IS A DIFFERENT IDENTITY ON PURPOSE:** **Bob**, resolved in R-1, is the contrast row. ⚠️ **v1.0 used Dave-resolved-then-erased, which is the F1 defect: the control consumed the subject.**
- ⚠️ *Expected and NOT a defect: the struck name is `ed25519:…` clipped LEFT-anchored — `M_RP_MEMBERS.md` §6a's tail-8 gap. Filed, Joe's, gates nothing here.*

### R-4 — ⑦: join concurrency
1. **N = 5.** 🔒 **CHOSEN BEFORE THE RUN AND WRITTEN HERE (Chat, `D-123`; §11-5 made this binding).** Five is enough for one-shot `identity_get` to serialise visibly if it is going to, and small enough that a single failure stays readable. N CLI joiners fired together from one loop.
2. Read: how many `unasked` rows coexist; how long until all clear; whether any fails.
- **A NUMBER for A3's filed batched-`identity_get` option** — one connect/auth/`goodbye` per joiner today. **If N-at-once is common the batched form returns as a live option; if not, it is CLOSED WITH ITS REASON.** Never a tick.
- ⚠️ *A synthetic burst is not evidence about real usage. The number prices the MECHANISM under N and must not masquerade as a usage frequency.*

---

## §7 — POST-RUN

- [ ] `location.reload()` if any probe persisted a mutation (**N-123** — the cleanup is part of the probe)
- [ ] `MutationObserver`s disconnected
- [ ] all apps down, ports swept, **before** any static gate runs
- [ ] `xgen-node_identities.db` restored, and the node restarted **with no load warning**
- [ ] registry count re-read quiescent, all four axes stated, compared to pre-flight as a **TRANSITION in one session**

---

## §8 — WHAT THE RUN FOUND (2026-08-05, node `090246f`, client dev @ 5173/9222)

**ALL SEVEN OBLIGATIONS DISCHARGED.** Rig: J1 headless CLI joiners (`LegF-Bob` AI · `CAROL` · `DAVE` · `N1–N5`), E1 registry surgery, one observer GUI. **No code changed ⇒ floors untouched and deliberately not re-measured.**

### 🔑 R0 — the execution order is NOT the run numbering, and getting it wrong was unrecoverable

③'s first fill can only be observed **once**, and ① only exists on the live join path. ⇒ **Dave joined and was erased BEFORE the GUI ever launched**, making R-3 the FIRST thing the observer did. Executed order: **R-3 group → R-3 DM → R-1 → R-2 → R-4.** *Running the numbering as written would have filled the room before Dave was erased and spent him.*

### ① — CONFIRMED TWICE, AND F3 WAS RIGHT

At insertion: `ADDED · atInsert_unresolved="unasked" · atInsert_hasAi=false` (R-1, t=13362). **A row MOUNTS carrying the attribute** ⇒ v1.0's attributes-only observer would have recorded **nothing**, exactly as F3 predicted. Also at rest for 108 s in R-2.

### ②③ — CONFIRMED BY A COUNTER, NOT BY ABSENCE

`state:known · memberCount:2 · rowCount:1 · erasedHidden:1 · isDm:false · counterpart:null`. 🔑 **The roster stayed COMPLETE while the render dropped him** — B-1 exactly. *An empty screen alone would have been ambiguous between "filtered" and "never a member"; the counter distinguishes them.*

### ⑤ — CONFIRMED, AND THE CONTROL IS THE SAME IDENTITY IN TWO ROOMS

DM: `rowCount:2 · erasedHidden:0 · isDm:true · counterpart:DAVE`; row renders `data-unresolved="erased"`, `text-decoration:line-through`, weight 500. §5a-i's selection bar **survives**: `rgb(154,106,48) 2px 0 0 inset`. ⇒ *same identity, same `not_found`, opposite outcome — the `|| === counterpart` exception is implemented, not merely present.*

### ④ — CONFIRMED, AND §2b IS RESOLVED AT A FINER GRAIN THAN IT WAS ASKED

One flush at t=14513, **in this DOM-mutation order**: `aria-label` → `LegF-Bob` · **`data-ai` null→"true"** · avatar seed hue 73→35 · **`data-unresolved` "unasked"→null**.

🛑 **THE FALSIFICATION CRITERION AS WRITTEN WAS MET, AND IT WAS THE WRONG CRITERION.** §6 R-1 said *"if `[data-ai]` is present while `data-unresolved="unasked"` is still set ⇒ §2b is wrong."* **`data-ai` DOES land before the marker clears.** But all four mutations occur in ONE synchronous flush with no style/layout recalculation between them ⇒ **no frame is ever painted in that state.** 🔑 ***§2b is a claim about PAINT; the criterion tested ATTRIBUTE PRESENCE. Reading only the criterion would have reported a headline defect that does not exist; reading only the intent would have missed that the badge genuinely precedes the clear.*** **Forward rule: a falsification criterion must name the LAYER that decides it.**

### ⑥ — A NUMBER, AND THE RUNBOOK'S CLAIM WAS BROADER THAN WHAT HOLDS

Kill landed inside the fetch window on the first attempt. **108 s at rest, ONE log entry, zero mutations, row unchanged** — name `ed25519:7WGuWOqU…`, weight 500, `rgb(138,136,128)`. 📌 *C-3's skin seen on a CLIENT row for the first time (previously sampler-only).*

🛑 **THEN THE NODE RETURNED AND THE ROW HEALED.** At t=179649 **both** Bob's and Carol's rows were **re-ADDED with `atInsert=null`** — a whole-list rebuild, not an attribute change. ⇒ **nothing retried the failed fetch; a RE-FILL re-asked**, because `partition_observed` saw Carol unheld. **Two regimes, written as one claim:**
- **while connected** — no retry exists, indefinitely;
- **across a reconnect** — self-heals as a side effect of the re-fill, by no retry policy.

⚠️ **HONEST LIMIT: the reconnect itself was NOT instrumented.** Inferred from simultaneous whole-list rebuild + timing, then corroborated independently by Joe's screenshots (`Reconnecting` → `Ready` across the heal).

🛑 **AND A CHAT CLAIM RETRACTED IN-RUN.** Chat wrote *"nothing on screen explains why the row is stuck."* **False** — Joe's screenshot shows the self panel reading **`Reconnecting`** with the status bar echoing it. **The accurate residue: the ROW carries no failure affordance and never retries while connected, but the APP does report the disconnection globally.** *Weaker than "unreported", and it changes what a bounded retry would buy.*

### ⑦ — A NUMBER, AND IT IS DECISIVE FOR A3

N=5 fired together (all CLI processes returned in 1773 ms). **5 rows ADDED, every one `ins=unasked`; PEAK CONCURRENT UNASKED = 5**; adds spread t=10449→10779 (330 ms); all cleared by t=15908. `memberCount:9 · rowCount:8 · erasedHidden:1`.

🔑 **THE FETCHES SERIALISE.** Bob alone: **1151 ms**. Five: **5459 ms**. 5 × 1151 = 5755. ⇒ **cost is LINEAR IN N** — five sequential connect/auth/`goodbye` round trips, all five rows visibly unresolved throughout. ✅ **CORROBORATED AT A SECOND, UNRELATED LAYER:** the node DAG stamps the five joins at `14:13:03.123/.200/.291/.388/.453` — a **330 ms spread matching the DOM adds exactly**.

### ✅ THE JOINS WERE REAL — PROVEN AT THE DAG, NOT ASSERTED

Space event store: **`membership.join` × 10 · `membership.invite` × 8 · `state.space_create` · `state.room_create`**. Every join **signed** (138-char signature) by a **distinct sender key**, with real `prev_events` chains (Dave's: `prev=1` then `prev=2`, the invite-chain shape). **Nothing was injected into the client store.**

### 🔒 E1's LEVER WORKS, AND §11-8's OPEN QUESTION IS CLOSED

Registry 11 → delete one object → node banner **`Identities: 10 registered`** — exactly one less, on a **positive counter**. ⇒ **`IdentityRegistry::load` ACCEPTS the round-tripped JSON**, the claim §11-8 refused to make offline. *The banner count is a better stop condition than the absence of a warning.*

### 📌 FILED FINDINGS — NOT LEG F OBLIGATIONS, ANNOTATED NEVER REPAIRED (`D-131`)

1. **System feed entries never re-render on resolution.** `Bk9glk joined` persisted unchanged across three separate resolutions while the panel updated each time. `shortId() = id.slice(-6)`. → `D-130` / `M-RP-LIVEFEED-REFRESH`.
2. **`members-panel.svelte:11-14` OVERSTATES ITS OWN SOURCE.** It reads as permanent design law (*"NOT A SELECTION SURFACE … must NEVER call `selection.set()`"*); `M_RP_PANEL_INERT.md` §0's actual reason is *"R7 would ship six affordances promising interactions **Leg B does not wire**"* ⇒ **DEFERRED, not rejected.** 🔑 *The named defect class, in a comment: a claim narrower than the thing it describes.* ✅ **Inertness itself VERIFIED BY PROBE** — a click on a member row moved nothing (inspector, selection, focus all unchanged) while an identical gesture on a Spaces row flipped R8 to `spaces/space/Engineering`. Semantics agree: `role=list`/`listitem`, no `tabindex`, `cursor:default`.
3. **`entity-context-menu` is COMPLETE and never instantiated in the client** (`registry.ts:103`); the only consumer is the sampler. **Zero `oncontextmenu` handlers exist anywhere.** Seam reserved at `entity-avatar.svelte:101`; a third consumer waits at `xgen-dd-message-family-phase0.md:97`. ⚠️ **The DEFERRAL is recorded everywhere; its REASON (Joe: *"wait for the real needed menu items"*) is recorded NOWHERE.** *A deferral without its reason reads as an oversight — Chat misread it exactly that way during this run.*
4. **Candidate menu items, each earned by an observed state:** *Retry identity fetch* (⑥'s residue as a user action rather than a background policy) · *Copy identity ID* · *Show AI capabilities* (`dm_initiate`/`spontaneous_post` live on the node, invisible in UI) · an action for an erased DM counterpart.
5. **THREE inconsistent truncations of one identity render simultaneously:** feed `Bk9glk` (`slice(-6)`) · row `ed25519:7WGuWOqU…` (`tail()`) · avatar `ED` (initials). ⚠️ **The avatar initials derive from the literal string `ed25519`** ⇒ **every** unresolved identity shows `ED` — initials of the ALGORITHM, not of the person. 🔒 **Joe ruled: keep `tail()`.** `C-8` forbids fabricating a name map; it does **not** require three inconsistent truncations.
6. **Remembering names is built; forgetting them is designed and NOT wired.** `SeenRecord.display_name` persists across restarts (*"the whole point of the cache"*); `T1_DEFAULT_RETENTION_DAYS = 182` is **provisional (J-580)** and explicitly *"never `∞`"*; `E2` targeted erasure exists in the book API. Eviction is unwired in production (`M-RP-LIVEFEED-REFRESH`). 🔑 **Leg F EXERCISED M13 §3c rather than arguing it: Dave was erased on the node, and every client holding a cached record would still show his name.**
7. **The members panel has an outage affordance that CANNOT FIRE once a roster is loaded.** `:44-57` defines five PANEL states (numbered ①–⑤ independently of the seven obligations — do not read them as the same series) and distinguishes panel-④ `failed` *"I cannot reach the others"* from panel-⑤ `offline` *"I cannot see the others"* **by the connection, not the phase** — but only when `rosterKnown` is false. Measured during a live outage with a roster in hand: **`opacity 1`, `filter none`, names at weight 600, no `data-*` state on the panel** ⇒ panel-state ② `known` renders **fully confident about a roster it can no longer verify.** ⚠️ *Chat first reported "no outage affordance at all" — wrong at the panel level, corrected on reading `:44-57`.* 🔒 **Joe's `tail (display_name)` proposal was WITHDRAWN on measurement, correctly**: `tail()` is used *precisely when the book holds no `display_name`* (`:28-29`) ⇒ the two are **mutually exclusive by construction** and the parenthetical can never be populated.

---

## §9 — OBLIGATION LEDGER

| | obligation | run | verdict |
|---|---|---|---|
| ① | join → `data-unresolved="unasked"` | R-1 + R-2 | ✅ **at insertion AND at rest** |
| ② | real `not_found` → the ③ filter | R-3 | ✅ `rowCount 1`, roster complete |
| ③ | populated roster → `erasedHidden` counts | R-3 | ✅ `erasedHidden: 1` |
| ④ | joiner resolves, **AI badge lights** | R-1 | ✅ **badge precedes clear IN THE DOM, same flush ⇒ never painted** |
| ⑤ | erased joiner → `_notFound`, or MARKED as DM counterpart | R-3 DM | ✅ struck through, selection bar survives |
| ⑥ | fetch fails; row stays ④, nothing retries | R-2 | ⚠️ **NUMBER** — 108 s / 0 retries connected; **heals on reconnect** |
| ⑦ | join concurrency | R-4 | ⚠️ **NUMBER** — N=5, **serial**, ~1151 ms each, linear in N |

---

## §10 — RECORDS (`D-074`, one commit)

JOURNAL + `CLAUDE.md` PLAY + `docs/ROADMAP.md` + `M_RP_IDENTITY_RESOLUTION.md` + the Leg F Phase-0 + this runbook. ⑥ and ⑦'s numbers feed back into **`D-126`** and **A3's batched form** — each re-priced or **closed with its reason**. **The milestone closes with this leg.**

📌 **Also owed to the record, found by this read and NOT fixed here:** `CLAUDE.md`'s file-placement table describes `xgen-node_identities.db` as **SQLite**; it is a JSON array (`registry.rs:240-256`). Annotated, not repaired (`D-131`), and it belongs to whichever milestone next touches that table.

---

## §11 — WHERE THIS DOCUMENT IS MOST LIKELY WRONG

*The author's own doubts. **NOT a census of this runbook's errors** — v1.0 carried five and the second reader found five more, of which four were not on the list.*

1. **R-2's kill window may be unhittable by hand.** If several attempts fail, ⑥ reports what was reached and the lever's limit is recorded — it does not quietly become ①'s twin.
2. ⚠️ **SUPERSEDED, KEPT NOT ERASED (`D-131`). v1.0 read: *"R-3 step 1 assumes Dave resolves before the surgery. If he does not, the control is gone."*** **The doubt pointed the wrong way.** The hazard was that Dave **does** resolve first — which caches him and makes the erasure unobservable (F1). *A doubt aimed at the inverse of the real risk is worse than no doubt, because it reads like the question was already asked.*
3. ⚠️ **CLOSED — the MutationObserver root is now measured** (`div.members-panel`, §6). The residual risk moved into the observer's **config**, repaired at R-1 step 1.
4. ⚠️ **CLOSED — `cmd_register`'s config path is grounded** (§2, `main.rs:76/93-96/254`).
5. ✅ **CLOSED — ⑦'s N is fixed at 5, written into R-4 before the run** (v1.3, Chat).
6. ✅ **CLOSED, AND BY A STRONGER FACT THAN THE ONE DOUBTED (v1.5).** The doubt was that *"observer not scoped to that Space"* is a **read, not run** behavioural claim. It no longer carries the weight: **every production address-book write lives in `desktop.rs` (the GUI path), so with the GUI DOWN there is no writer at all** — a file-level fact, not a router behaviour. 🔑 *The original doubt was correct to distrust the behavioural route; the repair was to stop depending on it.*
7. ⚠️ **SUPERSEDED, KEPT NOT ERASED (`D-131`). v1.3 read: *"The E1 edit itself is unrehearsed — AND IS THEREFORE REHEARSED IN SETUP, ON A COPY."*** The rehearsal happened; **the word "rehearsed" overclaimed what it delivered.** See item 8.
8. 🆕 **The E1 rehearsal proves LESS than "rehearsed" suggests, and is recorded at its real strength.** Setup copied the live `xgen-node_identities.db`, deleted a record and reparsed it — **but reparsed it with PowerShell, which proves the JSON is WELL-FORMED and nothing more.** ***That `IdentityRegistry::load` accepts it is a different claim and was NOT tested offline*** (it would need a Rust test and a build). ⇒ **serde acceptance is checked at node start by F4's stop condition, and that is the only thing that checks it.** 🔑 *A probe that reparses its own output with its own parser cannot fail in the way that matters.*

---

## §12 — THE SECOND READER'S FINDINGS (Clair, 2026-08-04, v1.0)

**IT WAS NOT CLEAN, WHICH IS WHY THE READ RAN BEFORE THE LOCK (J-642).** Five findings; **four were absent from v1.0's own §11.** All re-driven by Chat at the producer before being absorbed.

- **F1 — WRONG / UNRUNNABLE, and it is the headline. `R-3` could not produce ②③⑤.** v1.0's step 1 had Dave *"join a group room and resolve normally (proves the row exists as a control)"*; resolving him writes his record to the observer's disk book, and `partition_observed` then routes him to `to_touch` forever. **Superseded sequence recorded here, deleted from §6 (`D-131`).** 🔑 **THE SHAPE, NAMED: A CONTROL THAT DESTROYS THE STATE IT IS CONTROLLING FOR.** Step 1 existed to prove the row could exist; proving it is what made the erasure unobservable. 🛑 **AND THE SPECIES IS WORSE THAN A NARROW READ — this is `M_RP_IDENTITY_RESOLUTION.md` §5b, a locked finding in Chat's OWN milestone document, which Chat had read; and the Phase-0 Chat wrote two hours earlier stated the correct timing (*"between the join and the observer's fetch"*).** *A source consulted, understood, recorded — and then not applied by the same author in the next document.*
- **F2 — IMPRECISE. O3's justification was false** (the no-invite `_ =>` arm, `ops.rs:1615-1646`). Ordering kept, reason corrected.
- **F3 — IMPRECISE, would have made ①'s primary capture a clean-looking nothing.** The attributes-only observer cannot see a node that mounts with the attribute already set.
- **F4 — IMPRECISE, a real E1 trap.** JSON not SQLite, and a malformed edit **silently empties the entire registry**.
- **F5 — OMISSION.** No Space, room or DM-Space creation anywhere in the rig, while ⑤ rests on the DM Space.

**✅ VERIFIED CLEAN BY THE READ — and a clean result is a result:** the fetch-error path (②, the most valuable question asked and the feared defect **absent**) · the `clean_slate_config` census, both directions · the `--ai` config path · the `data-unresolved` DOM chain, node named · the node's non-repopulation of the registry from the DAG · the absence of any erasure verb, both directions.
