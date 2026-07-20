# M-RP6.3 Leg D2 — R6 composer + the outbound echo store
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jul 2026  
> **Last updated**: 2026-07-19  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — WHAT THIS MILESTONE IS

**The milestone that makes sending exist.** Design is `tasks/M_RP6_3_COMPOSER.md` §9.11 (Joe-locked,
J-552). This runbook is the build order, not a new design.

**⚠️ AND IT IS LITERALLY TRUE, MEASURED, NOT A FIGURE OF SPEECH.** `sendMessage()`
(`app_client.svelte:532`) is called from **exactly one place — the DEV bridge**. Verified against a
real production build: `send_message`, `sendMessage` and `__XGEN_SEND__` are **absent from all six
bundle files**, while `invoke` / `Messages` / `room` are present (so the probe was not blind).
**In a release build today the client cannot send a message at all.** Leg B shipped the seam
deliberately; D2 is what makes it reachable.

**Tier: `$common` + one shipped `core` consumer.** ZERO `.rs` — §9.11.4's Rust work was **Leg D1 and
is CLOSED** (J-553, `53dab37`).

---

## §1 — RE-GROUND (J-559, against HEAD after M-RP-TYPECHECK)

§9.11 was grounded earlier on 2026-07-19 — **before** M-RP6.9 and the type gate landed. Substance
holds; **three line references have gone stale and one section is written in the wrong tense.**

| §9.11 claim | verdict at HEAD |
|---|---|
| Four-way `accepted`/`rejected`/`timed_out`/`failed` | ✅ `resident.rs` **743–761** — cited as `:706` |
| Author excluded from fan-out **by identity, not connection** | ✅ `fanout.rs` **303 · 1067 · 1116** (EV-D2) — cited as `:305` |
| `message.svelte` resolves `widgets?: Record<string, Component>`, drops unknown (W-13), `isOwn` ships | ✅ now **`:56` / `:63`** — cited as `:71` |
| `bodyExtras` declared but never rendered | ⬛ **SUPERSEDED — M-RP6.9 BUILT IT** (J-556) |
| Gate covers where the echo store lands | ✅ `ui/tsconfig.json` includes `common/lib/**` |
| A composer exists | ✅ **it does not** — built from scratch here |

**⚠️ §9.11.4 IS WRITTEN IN THE PRESENT TENSE ABOUT A CLOSED BUG.** It reads *"that is precisely the
silent queue D6 forbids, **live at HEAD**"*. **It is not live.** Leg D1 fixed it —
`desktop.rs:363` now calls `await_send_outcome(reply_rx, SEND_QUEUE_TIMEOUT)`, and
`SEND_QUEUE_TIMEOUT` is **derived** (`SEND_ACK_TIMEOUT + PENDING_SWEEP_INTERVAL + 5`) with a test
asserting it outlasts the drain's worst case. **Do not re-fix it.**

**🔑 THE FINDING THAT DECIDED Q1 — THERE ARE TWO LATCHES, AND THE NAME HIDES IT.**
`rooms-panel.svelte:23` latches the **Space** (`latchedSpaceId`). `stream-panel.svelte:55` latches
the **room** (`latchedRoomId`), with a stale-latch guard producing `effectiveRoomId` (`:68`) that the
projection filters on (`:79`). *Two latches, near-identical shape, different subjects — and "the
latch" refers to whichever one you happen to be reading.*

---

## §2 — THE TWO DECISIONS TAKEN AT J-559 (D-121 applied)

### §2.1 🔒 Q1 — THE COMPOSER SHARES R5's ROOM LATCH. It does NOT read the bus.

**User-visible impact — this is why the answer is what it is, and it reversed Chat's first
recommendation.**

- **Reading the bus (REJECTED):** you are reading room X. You click a Space in the tree. The stream
  keeps showing room X — it is latched. **The composer greys out.** You are looking at a conversation
  you can no longer type into, on ordinary navigation, **with nothing on screen explaining why.**
- **Sharing the latch (LOCKED):** the composer is enabled exactly when the stream is showing a room,
  and disabled exactly when the stream says *"select a room."* **The message always goes to the
  conversation you are looking at.**

**Resource cost:** one small `$common` store; **one edit to a shipped, CDP-verified component**
(`stream-panel` consumes the store instead of owning the state). No new dependency, no bundle growth
beyond the module itself.

**⚠️ Chat's first recommendation was the bus, on the argument "do not lift a deliberate
component-local workaround into a shared store." The argument was sound and answered a question that
ranks below the one that decides.** It also aimed at the wrong latch: the objection was about R2's
**Space** latch, which this does not touch. **→ D-121.**

**Third option considered and rejected: duplicate the latch logic in the composer.** No edit to
`stream-panel`, but two copies of a rule that must agree forever — a D-067 surface whose drift
produces **exactly the divergence this decision exists to prevent.**

### §2.2 🔒 Q2 — THE LEG B DEV BRIDGE STAYS. Retire it at Leg D3 close.

**User-visible impact: NONE, in either direction — and that is a legal answer (D-121).** Guarded by
`import.meta.env.DEV` (`app_client.svelte:139`) and **verified absent from a production bundle**. No
user ever sees it. *A UX rationale could have been invented here; there isn't one.*

**Resource cost of keeping: zero.** The composer calls `sendMessage()` directly; the bridge only
wraps the same function. Nothing is duplicated.

**Decided on the internal axis, stated as such:** the bridge is the only way to drive the send path
**without** the composer — i.e. to tell whether a fault is in the new UI or underneath it.
**Removing it in the milestone that adds the UI removes the control exactly when it becomes useful.**
Note its repo reference count is **1 — its own definition** — because its callers are CDP evals that
live outside the tree. *Retiring on a reference count would delete something whose users the count
cannot see.*

---

## §3 — 🔒 RESOLVED BY JOE (J-559): OPTION B — D3 IS A PRECONDITION OF D2's CLOSE

**Lock #6 gives send-status three visual states. The widget that draws them is D3.** A D2 that renders
echoes without one produces:

> Your message appears in the stream. It looks exactly like a delivered one — same component, same
> `MessageDescriptor` shape, same styling. **It may have been `rejected`, refused by the node, never
> arriving, ever — and the screen says nothing.**

**⚠️ AND IT NEVER SELF-CORRECTS.** The node excludes the author from fan-out **by identity**
(`fanout.rs:303`), so the message never returns from the server. **There is no later moment when the
truth arrives and the row updates** — §9.11.2: *the real event never arrives.* The echo is not a
placeholder awaiting confirmation; **it is the only record the message ever existed, and it asserts
success purely by being there.** In an ordinary chat client a failed message eventually goes red or
never gets its tick. Here **nothing ever happens** — the row is finished, and one time in four it is
finished wrong.

**🔒 LOCKED: D3 SHIPS WITH D2. D2 DOES NOT CLOSE ALONE.** They may be built as separate legs and
separate commits, but **`Status: COMPLETED` on this doc requires the send-status widget to exist.**

**Why B and not "accept the gap for one milestone":** XGen has no users, so the exposure is not
strangers — it is that **D2 would close as an honest milestone while its most likely observed state
is the lie.** `failed` fires on any outage, which during development is routine. Every verification
done in that window would be done against a UI that cannot report when a send did not land — *an odd
instrument with which to debug a send path.* **Resource cost of B: none.** It removes an option
(stopping between D2 and D3) rather than adding work.

**Fallback if the two must ever be separated: option A** — D2 ships a minimal honest failure signal,
not D3's widget: a row state distinct for `rejected` / `failed` / `timed_out`. **No build that can
send may ever render all four outcomes identically.**

---

## §3.1 — 🔒 RESOLVED BY JOE (J-559): LOCK #7 IS NARROWED FOR D2 — NO RETRY ON `timed_out`

**How this was decided, recorded because it touches the no-anonymity core.** Joe's *"go as you
recommend"* covered **both** open questions — §3 **and** lock #7. Chat later queried whether a
delegation could legitimately cover an identity-bearing item and **briefly retracted the lock; Joe
confirmed the delegation was intended for both.** *The query was right to make and the answer is yes:
a decision Joe makes by delegating is still Joe's decision.* **Locked.**

⚠️ **THIS NARROWS A JOE-LOCKED ITEM AND IS RECORDED AS SUCH.** §9.11.3 lock #7 reads *"`timed_out` →
retry only behind an explicit warning"*. **D2 offers no retry affordance on `timed_out` at all.**

| status | D2 behaviour |
|---|---|
| `failed` | **retry freely** — never reached the wire, so a retry cannot duplicate anything. This is also the common outage path |
| `rejected` | **no retry**, as locked — it will be refused again |
| `timed_out` | **no retry button.** Honest copy stating the node may hold it; the user decides, manually |

**User-visible impact.** With a retry button, one click under uncertainty can place a **second copy of
the same sentence on the federated network, permanently, attributed to the user's identity** — the
node may already be holding the first. Without it, the user must act deliberately. *The friction is
the point: the app declines to guess on the user's behalf about something it cannot take back.*

**Resource cost: negative** — no retry control, no warning dialog, no confirm copy to write.

**Why narrowed rather than built as locked.** `timed_out` is the **only** status where a UI
affordance can produce an irreversible, identity-bearing wire effect. **Not building it is
reversible; shipping it and letting the habit form is not.** And the decision is far better made once
a user can actually SEE the three states — which is D3. → filed as a named successor, not dropped.

**⚠️ Joe may reinstate the full lock at any time; this narrowing is a D2 scope decision, not a
reversal of §9.11.3.**

---

## §4 — LOCKS CARRIED FROM §9.11.3 (all twelve; the ones with build consequences annotated)

1. A local echo exists (C-4 amended, §9.11.2).
2. **The echo lives in a `$common` store**, not the widget — *a lost outage row is an omission; a lost sentence you just typed is the app eating your words in front of you.*
3. **Keyed by a client-minted local id; `event_id` stitched on at outcome.** ⚠️ The largest type surface on this arc — **and the first written under the type gate.**
4. The echo's timestamp is **client-minted and stays that way** (§9.11.5).
5. **Self is special-cased** — the user never sees their own six-char hash tail. `isOwn` ships.
6. **THREE visual states, not two** — sent (`accepted`) · **unresolved** (`timed_out`) · not sent (`rejected` + `failed`, same state, different copy). *Collapsing `timed_out` either way is the D6 lie verbatim.* → drawn at **D3**, see §3.
7. **Retry by status — 🔒 NARROWED FOR D2 AT J-559, see §3.1:** `failed` → retry freely (never reached the wire) · `rejected` → **no retry** · `timed_out` → **no retry affordance in D2 at all** (the lock's "explicit warning" path deferred to a named successor). *The only status where a click can put a permanent, identity-attributed duplicate on the federated network.*
8. The echo dies at **exactly one stated moment**: session end / reload. **The C-6 head marker must cover the user's OWN sends** — *they never read the messages they lost; they wrote the one they lost.*
9. Echoes are real `MessageDescriptor`s ⇒ **grouping and dividers come free.**
10. **Auto-scroll on your own send, always** — the one action where it is unambiguous.
11. **N windows, one device** — stated as N so nobody special-cases a pair.
12. **No room latched ⇒ typing yes, sending no.** → mechanism locked at §2.1.

---

## §5 — FILES

**Created**
- `ui/common/lib/stores/echo-state.svelte.ts` — the outbound echo store. Session-mortal,
  **never persisted, never federated.** Follows the `self-state` / `spaces-state` precedent.
- `ui/common/lib/stores/room-latch.svelte.ts` — the lifted room latch exposing `effectiveRoomId`,
  including the **stale-latch guard** (§2.1).
- `ui/common/lib/components/widgets/composer-panel.svelte` — R6. The `stream-panel` / `rooms-panel`
  widget pattern.

**Edited**
- `ui/common/lib/components/widgets/stream-panel.svelte` — ⚠️ **a shipped, CDP-verified component,
  twice:** consumes the lifted latch instead of owning it, and **merges echoes into the projection**
  (lock #9). C-4 governs **inbound** unchanged.
- `ui/common/lib/plugins/registry.ts` — ⚠️ **v1.1 CORRECTION (Clair, Rule 6): v1.0's list OMITTED THIS
  FILE, WITHOUT WHICH THE COMPOSER NEVER MOUNTS AT ALL.** The `CLIENT_PLUGINS` row is what the shell
  reads to place R6. *A file list that omits the row that mounts the milestone's own widget is not an
  incomplete list, it is a list that would have produced a component nobody could see.* And it is the
  reason the registry delta was +9 rather than +3 — the descriptor's second reader (→ N-147).
- `ui/client/src/app_client.svelte` — ⚠️ **v1.1: "mount R6 in app_client" was the GENERATION-STALE
  ANCHOR** N-116/J-541 already corrected once; the mount is driven by the registry row, not by a hand
  edit at a remembered line.
- `ui/assets/skin.css` — PROVISIONAL block → discharges at **M-RP-SKIN**.

**Must NOT be touched**
`**/*.rs` (Leg D1 is closed) · `rooms-panel.svelte` (its Space latch is not this) · `ui/assets/skin.css`
beyond a PROVISIONAL block · the `details` socket · `types.ts` `bodyExtras` (D3's) · `__XGEN_SEND__`.

---

## §6 — LEGS

- **D2-A — the echo store.** Local id minted at press; `{localId, spaceId, roomId, text, sentAt, status, eventId?, code?, reason?}`; **stitch `eventId` at outcome**; session-mortal death stated in the file.
- **D2-B — the room latch, lifted.** ⚠️ **Verify R5 is unchanged before anything consumes it** — this is a shipped component and the lift must be behaviour-neutral on its own.
- **D2-C — the composer**, plus locks #10 and #12.
- **D2-D — projection merge** (lock #9) and the **lifecycle guard** (§9.11.8): the guard is the nicety, **D1's bound is the guarantee**.
- **D2-E — verification + records.**

**⚠️ CARRY §9.11.8, DO NOT RE-DERIVE IT:** outage send latency is **re-anchor + `SEND_QUEUE_TIMEOUT`**,
measured **19 155 ms** = **3.143 s failed re-anchor + 16.013 s bounded wait** (the bound itself
accurate to 13 ms). **≈19 s is NOT a bug in the bound.** D2's lifecycle guard should normally stop the
call reaching that path at all.

---

*(§7 is deliberately absent: it held the unresolved lock-#7 collision and was **resolved at J-559**,
so it moved up to **§3.1** beside the other resolved decision. Nothing was dropped.)*

---

## §8 — FLOORS, PREDICTED BEFORE DRIVING

| floor | predicted | why |
|---|---|---|
| `cargo test` | **1546 / 0 / 62 across 56 terminators — IDENTICAL** | zero `.rs`; identity is the DIRECT proof |
| `svelte-check` | **0 errors** / warnings **34 + n**, n stated and justified | the gate now covers every file this milestone writes |
| `npm test` (sampler) | **132** unless tests are added, then stated exactly | |
| `vite build` client | **193 + n** — ⚠️ **PREDICT n DELIBERATELY**: 3 new modules + any `<style>` block (**N-141: a Svelte `<style>` block is exactly one vite module**) | |
| `vite build` sampler | **170** — unchanged; R6 is not a sampler tenant | |
| sampler catalogue | **419** | |
| client registry | **134 + n** — ⚠️ the composer registers; **predict n, do not read it off** | |

⚠️ **N-140: read every baseline after the DEV SERVER is gone, not just the app.** ⚠️ **N-117: the dev
client holds the exe** — a held exe gives exit 101, zero terminator lines, 0/0/0, which reads exactly
like a clean run. Grep **case-SENSITIVE**.

---

## §9 — VERIFICATION

| # | leg | evidence |
|---|---|---|
| V1 | **R5 unchanged by the lift** | latch behaviour identical before/after D2-B, on its own |
| V2 | **Lock #12** | no room latched ⇒ **typing works, sending refused**; composer enabled iff the stream shows a room |
| V3 | **🔑 The id stitch** | echo created with a local id, `event_id` present on the same row after outcome — **the row is the same row** |
| V4 | **🔑 All four outcomes reach the store distinctly** | `accepted` · `rejected` · `timed_out` · `failed`, each landing as itself. **Drive via the dev bridge (§2.2) — that is what it is for** |
| V5 | **Lock #9** | echoes group and divide like any other row |
| V6 | **Lock #10** | own send auto-scrolls |
| V7 | **Session-mortality** | reload ⇒ echoes gone, and the **C-6 head marker covers own sends** (lock #8) |
| V8 | **Zero Rust** | `git diff --stat` has no `.rs` **AND** `cargo test` identical |
| V9 | **Every floor predicted then measured**, differences DECOMPOSED not adjusted | N-108: mark SEEN vs DERIVED |

**⚠️ V4 IS THE ONE THAT MATTERS.** A composer that renders `accepted` is a text box. **A composer that
distinguishes `rejected` from `timed_out` is the milestone** — that distinction is the whole of D6.

---

## §10 — DEFINITION OF DONE

**— IMPLEMENTER (Clair) —**

- [x] Echo store · room latch · composer created; `stream-panel` edits behaviour-neutral for R5 (V1) — **and V1 proved §2.1 live: the bus held a space while stream and composer both kept `rmA`**
- [x] All twelve locks satisfied or explicitly deviated with a reason — **eleven met; ⚠️ LOCK #10 (auto-scroll on own send) NOT MET, measured not assumed, flagged not absorbed → Joe accepted the deviation → `M-RP-COMPOSER-SCROLL`**
- [x] V1–V9 driven; **V4 driven for all four outcomes** (`rejected` code 3041 · `failed` no `eventId` · unknown → `timed_out`, live)
- [x] Every §8 floor **predicted then measured**; SEEN vs DERIVED marked; **three misses all DECOMPOSED, none adjusted** (npm 142 · vite 200 · registry 143 → N-147/N-149)
- [x] `cargo test` **1546/0/62 × 56 identical**, stated as the direct proof of zero Rust
- [x] `svelte-check` errors **0**; **warning delta ZERO** (34 → 34)
- [x] Any behaviour change outside the locks surfaced as a **FINDING**, not folded in — **`core`'s `widgets?: Record<string, Component>` flagged, NOT fixed in a `$common` milestone**
- [x] No probe artifact left on disk; `git status` clean of unintended files; **machine left quiescent, ports free (N-140)**
- [x] Deviations **flagged, not absorbed** (Rule 6) — **four flags, TWO of them against this runbook (§5, §11 item 2)**
- [x] 🔒 **D3 EXISTS — no build that can send renders all four outcomes identically (§3)**
- [x] 🔒 **No retry affordance on `timed_out` (§3.1)** — enforced **in the store as well as the button**, verified with the UI bypassed

**— RECORDS (Chat) —**

- [x] **[CHAT]** JOURNAL · CLAUDE.md PLAY · ROADMAP · this doc → `Status: COMPLETED` (D-074, one commit) — **J-560**
- [x] **[CHAT]** §9.11.4 re-tensed in `M_RP6_3_COMPOSER.md`; the three stale line refs corrected — **done at J-559 (v2.0)**
- [x] **[CHAT]** the M-RP-TYPECHECK DoD tick corrected — **done at J-559 (v1.2); both since measured, 419 and 134**
- [x] **[CHAT]** any new N notes — **N-147 · N-148 · N-149** (notes v1.4)
- [x] **[CHAT]** §5 and §11 item 2 corrected in this doc (v1.1), the two Rule-6 flags raised against it

*(No "commit pushed" item — unflippable inside the commit that performs the push. `Status: COMPLETED`
is the signal. Joe pushes.)*

**⚠️ THE SEAT SPLIT IS EXPLICIT BECAUSE M-RP-TYPECHECK's DoD GOT IT WRONG** — it assigned the
canonical records to the implementer while the kickoff assigned them to Chat, and Clair caught the
contradiction (J-558). Copied correctly here.

---

## §11 — FOR CLAIR (Rule 6)

**Flag, do not absorb.** Five runbooks on this arc were caught only because the implementer read them
whole first (J-499, J-548, J-553, J-556, J-558) — **four of them Chat's.**

**Known-weak spots in this document, named so you check them first:**
1. **§2.1's lift touches a shipped component.** If R5 changes behaviour at all, stop — the lift was
   supposed to be neutral.
2. ⚠️ **v1.1 — THIS ITEM WAS STALE WHEN YOU READ IT, AND YOU CAUGHT IT.** It said *"§3 is unresolved
   and Joe's — if it is still unresolved when you reach the render, ASK; do not pick"* while **§3's own
   header already read RESOLVED BY JOE.** A document disagreeing with ITSELF — the
   J-499/J-548/J-553/J-556/J-558 class, **sixth on this arc, all Chat's**, and findable only by reading
   the runbook whole before starting. *§3 and §3.1 are both RESOLVED; nothing in §3 needed asking.*
3. **§8's registry and vite deltas are predictions.** If a measured number differs, **decompose it** —
   do not adjust the prediction.
4. **The twelve locks were written before `bodyExtras` existed.** If one of them now reads oddly
   against the built socket, say so.

⚠️ **Do not assert a count, a file list, or a "there are N places that do X" without running the
grep.** The J-556 class has now recurred **three times on this arc, all from Chat**, twice inside the
milestone built to prevent it.

---

## §12 — NOT THIS MILESTONE

**Multi-device self-visibility.** Author exclusion is by identity and the echo is process-local, so
**you type on the laptop and the sentence does not exist on the phone — not "until reload," at all.**
Not solvable here (§9.11.7). **The node DID persist the event: fan-out is a live-delivery
optimisation, not the record.** ⚠️ **BINDING ON M-RP6.4: backfill reads the EVENT STORE and must
return the requesting identity's OWN events. If backfill were ever built by replaying fan-out, the
hole would silently persist.**

**Own-row ordering.** `accepted` carries no authoritative timestamp, so a user's own rows keep a
client-minted time and **may order differently for them than for everyone else in the room.** Real,
permanent within a session, filed, not Leg D's.

**D3 — send-status widget** (second tenant of `bodyExtras`) · **M-RP-REACTIONS** · **M-RP-A11Y** ·
**M-RP-SKIN** (this milestone's appearance is PROVISIONAL and discharges there).
