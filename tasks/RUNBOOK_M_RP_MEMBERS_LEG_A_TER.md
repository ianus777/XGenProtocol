# M-RP-MEMBERS Leg A-ter — bound the authentication handshake, on both sides
> **Status**: PENDING  
> Version: 1.6  
> Date: Jul 2026  
> **Last updated**: 2026-07-25  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — Why this leg exists

Leg A-bis bounded the **dial** (`session::ensure_connected`) and **each `identity.get` reply** (`ops::identity_get_on`). Clair flagged, filed-not-fixed, that **the auth handshake after the dial is still unbounded**.

Chat measured it and it is **larger than the flag said**:

- `client_authenticate` has **TWO** bare `recv().await` — not one.
- **`server_authenticate` has one too** — the flag did not mention the server side at all.

🔑 **THE SAME DEFECT CLASS AS THE REST OF THIS ARC, FOR THE FIFTH TIME: a claim narrower than the thing it described.** Clair's flag was true and incomplete; Chat's report to Joe repeated it at *"two lines"* and was also incomplete. **Neither was caught by re-reading — it was caught by grepping the whole class instead of the two sites already named.**

**Scope:** `xgen-core` only. Moves the **cargo** floor. ⚠️ **Zero `xgen-client`, zero `ui/**`, zero `skin.css`, zero `Cargo.*`.**

---

## §1 — Grounding (measured 2026-07-25 at `2c3144b`; positive control: 257 `.rs` files excluding `target/` + `.claude/`)

### §1.1 — The six production `recv().await` in `connection.rs`, and which are bounded

| Line | Enclosing fn | Bounded? | By what |
|---|---|---|---|
| 207 | `send_event_confirmed` | ✅ | `tokio::time::timeout(timeout, drain)` at **:244** |
| 284 | `upload_blob` | ✅ | same pattern at **:303** |
| 339 | `fetch_blob` | ✅ | same pattern at **:365** |
| **504** | **`server_authenticate`** | ❌ | **bare** |
| **551** | **`client_authenticate`** (challenge) | ❌ | **bare** |
| **566** | **`client_authenticate`** (auth_ok / auth_fail) | ❌ | **bare** |

🔑 **THE PATTERN ALREADY EXISTS IN THIS EXACT FILE, THREE TIMES.** Every message-exchange verb wraps its recv loop in a caller-supplied `timeout: Duration`. **Authentication is the sole exception — and it is the only pre-auth, attacker-reachable path in the file.** ⇒ this leg is not "add a timeout", it is **"apply the file's own established pattern to the two functions that were missed."**

### §1.2 — Reproduced deterministically, with a positive control (N-163)

Throwaway harness appended to `connection.rs`, run, then reverted; tree verified clean at `2c3144b`.

```
ZZCONTROL  server speaks (unexpected msg) → client_authenticate returns in   692 µs
ZZSUBJECT  WS upgrade completes, server silent → did NOT return within       5.0 s
```

⚠️ **The control is the content.** Without it, the hang is indistinguishable from a dead rig.

### §1.3 — ⚠️ THE SERVER SIDE IS THE BIGGER HALF, AND IT IS NOT A UI CONCERN

`server_authenticate` has **exactly one production caller**: `xgen-node/src/app.rs:1524` (all others are tests). It is the **first thing the node does on an accepted connection** — before the revocation check (`:1539`), before the session context is built.

⇒ **A peer that completes the WS upgrade and then never sends its auth response holds that task and its socket indefinitely.** Unauthenticated, pre-identity.

**Measured, so this does not rest on a guess — there is nothing else bounding it:** `xgen-node/src/**` (production, excluding `tests/`) has **zero** hits for `max_connections` / `max_conn` / `idle_timeout` / `IDLE_TIMEOUT` / `Semaphore` / `connection_limit`. **No connection cap, no idle timeout, no accept-side semaphore.**

📌 This is stated as a **measurement, not a severity rating.** Rating it is Joe's.

### §1.4 — Existing timeout constants (checked BEFORE recommending a value — the §2a lesson applied forward)

| Constant | Where | Value |
|---|---|---|
| `WAIT_TIMEOUT_SECS` — *"max seconds to wait for the peer's response message"* (spec 3.4.3) | **`xgen-core/src/federation/handshake.rs:48`** | **15 s** |
| `CONNECT_TIMEOUT_SECS` | `xgen-node/src/reconnect.rs:80` | 15 s |
| `ACK_TIMEOUT_SECS` | `xgen-node/src/bootstrap_client.rs:56` | 15 s |
| `CONNECT_TIMEOUT` | `xgen-client/src/resident.rs:216` | 10 s |
| `SEND_ACK_TIMEOUT` | `xgen-client/src/resident.rs:790` | 10 s |
| `IDENTITY_GET_RECV_TIMEOUT` | `xgen-client/src/ops.rs:485` | 10 s (Leg A-bis) |

🔑 **THE CLIENT SAYS 10, THE NODE SAYS 15, AND `xgen-core`'s ONE HANDSHAKE CONSTANT SAYS 15.** A bound placed in `xgen-core` reaches node code, so importing the client's 10 s would be **exactly the §2a error in a new place** — a value taken from one precedent without checking whether one existed for *this* operation, in *this* crate. It does: `WAIT_TIMEOUT_SECS`, same crate, same shape (a handshake waiting on a peer's response).

📌 **Filed, not fixed:** `CONNECT_TIMEOUT` (client, 10 s) and `CONNECT_TIMEOUT_SECS` (node, 15 s) are **two homes for one policy across crates** — the D-067 shape A-bis was careful to avoid *within* a crate. Out of scope here; recorded so it is not rediscovered a fourth time.

---

## §2 — 🔓 OPEN FOR JOE — decisions this leg does not take

### §2a — 🔒 SCOPE: does the server side ride this leg? — **LOCKED S1**

- **S1 — bound all three (client × 2, server × 1).** ① *User-visible:* R7's state ③ becomes honestly bounded **and** a node stops accumulating stalled pre-auth connections. ② *Cost:* one extra wrap and one extra test over S2. Effectively free once the constant exists.
- **S2 — client only, file the server side.** ① *User-visible:* R7 is fixed; the node keeps the unbounded pre-auth path. ② *Cost:* zero now; the finding ages, and it is not a UI defect so no UI milestone will ever pick it up.

**Chat recommends S1.** ⚠️ **But it widens the leg from a client-UI concern into node hardening, which is structural ⇒ D-123 rider ② ⇒ Joe's.** Chat proposes; Chat does not take it.

🔒 **LOCKED — S1 (Joe, 2026-07-25).** ⚠️ **PROVENANCE: CONSIDERED, not delegated** — Joe asked for the option to be restated in plain terms, read the unauthenticated-connection consequence, and answered *"of course s1"*. **All three recvs are bounded, including `server_authenticate`.** ⇒ this leg changes `xgen-node`'s behaviour on an accepted connection, and `xgen-core` is where the change lives.

### §2b — 🔒 VALUE: 15 s, and what it does to state ③ — **LOCKED V1, as a DEFAULT**

**Chat recommends 15 s**, per §1.4 — `xgen-core`'s own handshake constant, same crate, same shape.

⚠️ **THIS CORRECTS TWO NUMBERS CHAT HAS ALREADY PUT IN FRONT OF JOE. BOTH WERE WRONG, AND THE SECOND WAS WRONG IN THIS DOCUMENT.** A-bis §2a stated the fill's worst case as **≈ 25 s**, omitting auth entirely. Chat then stated **≈ 85 s** here — which **dropped the second connect from its own arithmetic** while naming it in the parenthetical.

**The ceiling, derived rather than quoted.** The fill path pays connect + auth **twice** — once for `drain_space_events`, then again in `fill_from_events`, which sets `session.conn = None` and re-dials for the fetch loop. With auth bound `A`:

`connect 10 + 2A  +  drain 5  +  connect 10 + 2A  +  first fetch 10  =  35 + 4A`

| `A` | Ceiling |
|---|---|
| **15 s** (V1) | **95 s** |
| **10 s** (V2) | **75 s** |

📌 **Chat's earlier ≈85 s and ≈60 s were both arithmetically wrong.** *Same defect class as the rest of this arc: a figure carried forward without re-deriving it. It is recorded rather than quietly replaced, because the 85 was said out loud to Joe.*

⚠️ **AND THE CEILING IS NOT THE TYPICAL CASE — SAYING ONLY ONE OF THEM MISLEADS IN EITHER DIRECTION:**

- **Typical failure ≈ 10–15 s.** A run ends at the *first* bound that fires, and the stages before it complete at real speed. One dead stage costs one bound.
- **Ceiling ≈ 95 s.** Reached only by a node that is slow-but-alive at *every* stage — and note that such a run **SUCCEEDS**. The ceiling is therefore mostly a bound on how long a *legitimate* slow fill may take, not on how long a failure takes.

⇒ ① *User-visible:* **state ③ *"I am waiting for the others"* usually resolves in seconds, but may legitimately sit for up to ~95 s on a very slow link before either resolving or becoming ④.** That is bounded — which is what §4c-i requires, and it is the whole point — but *bounded* is not the same as *short*.

**Three ways to answer, and the choice is Joe's because it is a user-experience call:**
- **V1 — 15 s, ceiling ~95 s.** Consistent with the crate the bound lives in. ③ is truthful the whole time; it simply runs long on a genuinely slow link.
- **V2 — 10 s**, matching the client's other bounds ⇒ ceiling ~75 s. ⚠️ imposes the client's number on node code, against §1.4 — and buys only **20 s** of the ceiling, which is the point: *the ceiling is dominated by paying auth twice, not by the value.*
- **V3 — 15 s now, and let §4c-i's deferred progress note ("fetched k of N") become non-deferrable**, so a long ③ at least *shows* it is working.

**Chat recommends V1 and re-filing V3's trigger** — ⚠️ the §4c-i trigger is currently *"a Space large enough to notice"*, and this leg supplies **a second trigger that has nothing to do with Space size**. That trigger should be written down whichever value is chosen.

🔒 **LOCKED — V1, 15 s (Joe, 2026-07-25). PROVENANCE: CONSIDERED** — Joe read the corrected `35 + 4A` arithmetic, the ceiling-vs-typical split, and chose the crate-consistent value. ⚠️ **AND HE ADDED A CONSTRAINT THAT CHANGES §2d:** *"prepare it, that this can be a setting's value … no hardcoding."* ⇒ **15 s is a DEFAULT, not a fixed value.**

📌 **V3's trigger is re-filed as instructed:** §4c-i's progress note now has **two** independent triggers — ① a Space large enough to notice, and ② **a slow link, which is unrelated to Space size**. Recorded here so Leg B inherits both.

### §2c — 🔒 CONSTANT: mint, or reuse `WAIT_TIMEOUT_SECS`? — **TAKEN BY CHAT**

🔒 **MINT `AUTH_RECV_TIMEOUT_DEFAULT` in `connection.rs`** (renamed per §2d), citing `WAIT_TIMEOUT_SECS` as the **shape analogue** rather than reusing it — federation capability negotiation and transport authentication are distinct policies that merely share a value, and coupling them lets tuning one silently retune the other.

⚠️ **PROVENANCE: CHAT'S, NOT JOE'S.** This is a technical-consistency call with no appearance or structural consequence, so it sits inside Chat's autonomy under D-123 and is **taken rather than asked**. It is recorded here only because it is load-bearing for §2d — *if the value were shared, it could not be tuned per-role later.* **Named, not smuggled.**

📌 **RATIFIED by Joe, 2026-07-25 (*"as you recommend"*), after the reasoning was restated in plain terms.** ⚠️ **Recorded as a RATIFICATION of a Chat-taken decision, NOT as a delegated lock** — the distinction matters because §2c was never Joe's to decide under D-123, so *"as you recommend"* here confirms a call Chat had already made and owns. **If it turns out wrong, it is Chat's error, not a delegation Joe should have examined.**

📌 **This is Clair's own inverse-D-067 reasoning from A-bis, accepted there, applied again here.** `WAIT_TIMEOUT_SECS` is also private to `handshake.rs`, so reuse would mean widening it — the same *"two homes for one policy"* pressure in reverse.

### §2d — 🔒 CONFIGURABLE: **A NAMED CONSTANT AND A NOTE. NOTHING MORE THIS LEG.**

⚠️ **THIS SECTION HAS BEEN REVISED TWICE AND THE HISTORY IS KEPT DELIBERATELY** — a reader who sees only the conclusion cannot tell which parts were asked for and which Chat invented.

1. Chat first recommended a **plain constant**, on the measured ground that threading a `timeout` parameter would touch **~95 `client_authenticate` call sites**.
2. Joe: *"prepare it, that this can be a setting's value … no hardcoding."* Chat read this as **build the override seam now** and specified a default + static + setter + getter (P2), plus an `AtomicU64` shape decision.
3. Joe narrowed twice more — *"i meant in §2b. not over all values"*, then: ***"obvious we put value into named variable and occasionally some code note. we didnt now close them to special object. this will be a question of the future."***

🔒 **WHAT SHIPS THIS LEG — P1, and only P1:** a **named constant** holding §2b's 15 s, read by the three auth sites, **plus a short code note** saying the value is expected to become configurable. **That is the whole of it.**

⚠️ **WHAT DOES *NOT* SHIP, AND MUST NOT BE BUILT "WHILE WE ARE IN THERE":** no static override · **no `pub fn set_auth_recv_timeout`** · no getter indirection · no config plumbing · no `AtomicU64`. 🔑 **The seam is a FUTURE question, not a cheap addition** — *and a setter with no caller is dead public API on a `GPL-2.0-or-later` crate every binary links.*

📌 **CHAT'S ERROR, NAMED, AND IT IS THE SECOND WIDENING IN THIS ONE SECTION.** *"Prepare it so it can become a setting"* was read as *"build the mechanism that makes it a setting."* **It meant: give it a name and say so.** ⚠️ *Preparation was mistaken for construction — the same shape as the §2b→class widening one paragraph up, committed again before the ink was dry on the correction.*

**P3 — thread a `timeout` parameter** through the call chain. ② *Cost:* ~95 call sites. **Rejected on the measurement, and it stays rejected.**

⚠️ **SCOPE, NARROWED BY JOE:** the note and the naming apply to **§2b's auth-recv value ALONE**. **`CONNECT_TIMEOUT` (10 s), `IDENTITY_GET_RECV_TIMEOUT` (10 s), `WAIT_TIMEOUT_SECS` (15 s), `CONNECT_TIMEOUT_SECS` (15 s), `ACK_TIMEOUT_SECS` (15 s) and every other timeout are OUT OF SCOPE** — not renamed, not annotated, not touched.

⚠️ **NO AUTH CALL SITE MAY NAME A NUMBER** — `:504`, `:551` and `:566` read the constant.

#### 📌 §2d-i — THE STORAGE SHAPE: **DECIDED, BUT DEFERRED — NOT BUILT HERE**

⚠️ **Nothing in this subsection is built by this leg.** It records a ruling so the future milestone does not re-litigate it.

When an override eventually exists, how it is stored decides whether a Settings change needs a **restart**:

- **`OnceLock<Duration>`** — set once at startup, immutable thereafter. ① *A Settings change would NOT take effect until relaunch*, with nothing on screen explaining why. ② marginally simpler.
- **`AtomicU64` (seconds)** — readable and writable at any time. ① *A change takes effect on the next connection attempt.* ② **the same amount of code**; one atomic load per handshake, free next to a network round-trip.

🔑 **The two cost the same, so the only thing separating them is that one ships a setting that silently does nothing until you restart.** *A control that does not do what it says is the shape this project exists to refuse.*

🔒 **RULED — `AtomicU64` (Joe, 2026-07-25): *"atomic"*.** ⚠️ **A FORWARD BINDING ON THE FUTURE MILESTONE, NOT A BUILD ITEM.** Whenever the override is built, it is atomic and **no restart may be required**. 📌 *Recorded now because the reasoning was fresh; building it now was Chat's over-reach, corrected above.*

#### 🔓 §2d-ii — OPEN FOR JOE: THE FUTURE MILESTONE — the mechanism, AND which values get it

⚠️ **NOTHING HERE IS BUILT BY THIS LEG.** Recorded so the future work starts from a position rather than from scratch.

🔑 **JOE'S FRAMING, 2026-07-25, AND IT SETS THE SHAPE OF THE WHOLE FOLLOW-UP:** *"in the future we will need to make revision on other values (also not over all)."* ⇒ the future milestone is **selective on BOTH axes**: it builds **one mechanism**, and it applies that mechanism to **a chosen subset of values** — **never a sweep of every timeout in the workspace.**

**What it will own:**

- **The mechanism itself** — the *"special object"* Joe deferred: how a configured value reaches shared code. 🔒 **§2d-i's `AtomicU64` ruling binds it forward** — no restart may be required.
- **WHICH VALUES ADOPT IT.** ⚠️ **A named, justified list — not a category, not a grep.** *A milestone that says "the timeouts" will take all of them, which is what Joe has now refused twice.*
- **The config key(s).** `[sync] completion_timeout_seconds` is the naming precedent. **Joe's.**
- **One value or two for auth?** Client-side and node-side auth are **different roles** — *"how long I wait for a node"* vs *"how long I hold a socket for a stranger"*. ⚠️ They share a value **by coincidence of shape, not by policy** — the coupling §2c refuses elsewhere. **Chat's reading: they should be able to diverge.** A reading, not a design.
- **Config file only, or a Settings-pane control?** Appearance and taxonomy. **Joe's.**
- ⚠️ **A constraint the mechanism must satisfy, learned here:** `xgen-core` code such as `connection.rs` **holds no config path and must not acquire one** — whatever the mechanism is, the value arrives from the binary; the shared crate never learns to read a file.

🔓 **THE NAME IS UNRESOLVED AND GOES BACK TO JOE — CHAT HAS NOW PROPOSED TWO AND BOTH ARE WRONG.** `M-TRANSPORT-TIMEOUT-CONFIG — the transport timeouts become configured values` was locked under Chat's **too-wide** scope (all timeouts). Chat then proposed `M-AUTH-TIMEOUT-CONFIG`, which is **too narrow** — Joe's *"revision on other values"* means the milestone outlives the auth value. **Chat's third attempt, offered with low confidence: `M-CONFIGURABLE-VALUES — selected runtime values become configured settings`.** ⚠️ *The recurring failure here is Chat guessing the scope from a phrase and naming to the guess; the name should follow Joe's decision on the list, not precede it.* **The old locked name stands in the records until he rules.**

---

## §3 — Steps

### Step 1 — the value: ONE named constant, and a short note (§2d)

In `xgen-core/src/transport/connection.rs`, above `client_authenticate`:

- **`AUTH_RECV_TIMEOUT`** — the locked **15 s** (§2b). The three auth sites read it; **none of them names a number.**

⚠️ **THAT IS THE ENTIRE STRUCTURAL CHANGE.** No static, no setter, no getter, no `AtomicU64`, no config path (§2d). 🔑 **If the implementation finds itself adding a second item to this list, it has left the leg** — stop and report (§5).

**The doc comment — short, and it carries four things:**

1. the bound is **per-recv, never an overall handshake cap**;
2. **why it is minted rather than reusing `WAIT_TIMEOUT_SECS`** (§2c) — same value, different policy;
3. that **15 s is expected to become a configured value later**, under the milestone §2d-ii names once Joe locks it;
4. ⚠️ that `connection.rs` **holds no config path and must not acquire one** — recorded so a future reader does not solve the config question by reaching for a file here.

📌 **Proportionate, per Joe: *"occasionally some code note."*** A short paragraph, not an essay. **This constant earns one because it is known to be provisional; the neighbouring constants do not and are not touched.**

📌 **Not the only `15` in the workspace — three unrelated constants already hold 15** (`WAIT_TIMEOUT_SECS`, `CONNECT_TIMEOUT_SECS`, `ACK_TIMEOUT_SECS`), **none touched by this leg.**

### Step 2 — bound `client_authenticate` (`:551`, `:566`)

Wrap **each** `self.recv().await?` in `tokio::time::timeout(AUTH_RECV_TIMEOUT, …)`. ⚠️ **The constant, never a literal** (§2d). On elapse return a `TransportError` that **names the stage** — challenge vs auth-reply — so a log distinguishes *"the node never greeted us"* from *"the node never answered our signature."*

⚠️ **Two recvs, two bounds, not one bound around both** — consistent with the file's per-step discipline and with A-bis §2's "per-step, never an overall cap."

### Step 3 — bound `server_authenticate` (`:504`) — 🔒 **IN SCOPE (§2a locked S1)**

Same wrap, same constant. ⚠️ **Do not touch `xgen-node/src/app.rs`** — the bound belongs in the shared code where the wait lives, exactly as A-bis argued for T1. The node's caller already handles `Err` by logging and returning (`app.rs:1526-1529`), so a timed-out handshake drops the connection with no caller change.

### Step 4 — tests (this is where the floor moves)

Promote §1.2's throwaway harness into real tests in `connection.rs`'s existing test module:

1. **Control** — server sends an unexpected message; `client_authenticate` returns fast. *Without this the timeout tests prove nothing.*
2. **Subject, client** — WS upgrade completes, server silent ⇒ `client_authenticate` returns **Err at the bound**, not a hang.
3. **Subject, server** — WS upgrade completes, client silent after the challenge ⇒ `server_authenticate` returns **Err at the bound**.

🔑 **THE TIMING PROBLEM, AND IT IS SHARPER NOW THAT §2d DROPPED THE SEAM.** With no override, a test **cannot shorten the bound** — it either uses tokio's virtual clock or it really waits 15 s, twice.

⚠️ **Use `tokio::time::pause()` / `advance()`.** 🔑 **IF PAUSED TIME DOES NOT WORK AGAINST THE DUPLEX RIG, STOP AND REPORT — DO NOT ABSORB.** Real-time tests would add **~30 s to every `cargo test` run, permanently**, and that is a cost Joe should price rather than inherit. *A-bis's harness ran ~20 s of real waiting and was DELETED rather than kept; keeping one is a different decision.* → §5.

📌 **FLOOR PREDICTION, STATED BEFORE THE RUN (N-149 applied forward):** `1585 → 1588` — **three** tests (control, client subject, server subject). 📌 **Down from 1589: the fourth test verified the override seam, which §2d no longer builds.** **A delta that is not exactly this must be explained, not absorbed.**

---

## §4 — Definition of Done

- [ ] `cargo build --workspace` clean
- [ ] `cargo test --workspace` — floor from **1585 / 0 / 62 across 56**; report new totals and **explain every delta against §3 Step 4's prediction**. ⚠️ Detached, poll, sum `test result:` **case-sensitively**; **56 = completeness**. Never run cargo and the dev client together (N-117)
- [ ] 🔑 **NO NUMBER AT THE AUTH SITES, PROVEN BY GREP NOT BY ASSERTION** — `:504`, `:551` and `:566` read `AUTH_RECV_TIMEOUT` and **name no literal**. ⚠️ **NOT a global count of `15`** — the workspace already holds three unrelated 15s (`WAIT_TIMEOUT_SECS`, `CONNECT_TIMEOUT_SECS`, `ACK_TIMEOUT_SECS`) and **a workspace-wide claim would be false on arrival**. ⚠️ **Report the grep and its positive control**, not the conclusion
- [ ] 🔑 **NOTHING BEYOND THE CONSTANT WAS BUILT** — **zero** new `static` / `AtomicU64` / `OnceLock` / `pub fn set_*` in the diff (§2d). *The seam was explicitly deferred; a helpful extra here is a scope breach, not a bonus.*
- [ ] ⚠️ **THE CONTROL IS REPORTED WITH THE RESULT**, not separately (N-163)
- [ ] **`client_authenticate` bounded at both recvs** — a silent server is rejected at the bound, and the error **names which stage**
- [ ] **`server_authenticate` bounded** — a silent client is rejected at the bound, and `xgen-node/src/app.rs` is **NOT** touched to achieve it
- [ ] ⚠️ **NO EXISTING TEST DEPENDS ON UNBOUNDED AUTH** — the federation and `xgen-mptest` suites dial real sockets; if any test now trips the bound, **report it, do not raise the constant to make it pass**
- [ ] **Test-suite wall time reported** before and after — 🔑 **with no override seam, this is the ONLY guard against a permanent real-time tax** (Step 4)
- [ ] Scope-clean: `git show --stat` — **`xgen-core` only**; zero `xgen-client`, zero `ui/**`, zero `skin.css`, zero `Cargo.*`
- [ ] `svelte-check` **not re-measured**, held by scope — say so rather than imply it
- [ ] 🔑 **THE A-BIS CLAIM IS NOW TRUE AS WRITTEN** — re-run A-bis's black-hole leg with a harness that **completes the WS upgrade and then goes dark**, and show the command rejects. ⚠️ **The old harness (a bare listening socket) CANNOT see this — it trips the connect bound and passes green.** *That is how this hole survived A-bis's DoD.*

*(Per the standing rule, "commit pushed" is deliberately NOT a DoD item.)*

---

## §5 — ⚠️ RE-OPEN-ON-BUILD CLAUSE (mandatory, D-122)

If anything here contradicts the code — a bound that cannot sit where §3 puts it, a test that cannot fabricate the silent peer, paused time that the duplex rig will not honour — **stop and report; do not absorb**.

📌 **This leg exists because a filed-not-fixed flag turned out to be three sites instead of one.** Expect the clause to fire on **Step 4's paused time** (🔑 **the seam that would have been the fallback was deliberately not built** — so a real-time tax must be reported, never absorbed) and on **the federation suites in the DoD's sixth item**.

---

## §6 — Handoff

**Leg B** (store + `members-panel` + the 7th `CLIENT_PLUGINS` row) follows, against A-bis's measured `(FillReport, MembersResult)` signature and carrying Phase-0 §4c/§5's four bindings.

⚠️ **One Leg B input is decided here, not there:** §2b's answer determines how long state ③ can legitimately persist, and therefore whether §4c-i's progress note ships with Leg B or stays deferred.
