# M-RP-ROOM-HEADER — R4 exists as a tile with nothing in it
> **Status**: PENDING  
> Version: 1.1  
> Date: Jul 2026  
> **Last updated**: 2026-07-26  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — WHY THIS DOCUMENT EXISTS

Opened 2026-07-26 during the M-RP-MEMBERS Leg B design walk. Joe asked for R4 to be *tuned*:

> *"room header – this is worth to think about, when the time will come for tuning. yes, some info about opposite identity, at the first display name, then some stylized avatar? for now let's there put the display name + random background graphic"*

🔑 **THERE IS NOTHING TO TUNE. R4 HAS NO WIDGET.** Measured 2026-07-26 at `2814fec`:

- `room-header` is in `REGION_IDS` (`layout-default.ts:18`)
- it has a `REGION_NAMES` entry — `'R4 · Room header'` (`:29`)
- it is a **leaf in the default layout** (`:105`)
- it appears in the layout tests (`mutate.test.ts:23`, `resolve.test.ts:25`)
- **there is no `room-header-panel.svelte`.** Every other region has one.

⇒ R4 is an **empty tile holding a reserved slot**. This is not a tuning pass; it is building the region from nothing.

📌 **Filed rather than absorbed into Leg B.** Leg B moves the `svelte-check` floor for the members panel. Folding an unbuilt region into it would make a regression unattributable — the same reason Phase-0 §8 splits Leg A from Leg B.

---

## §1 — WHAT JOE ASKED FOR, AND THE ONE WORD THAT MUST CHANGE

**v1 content:** the opposite identity's **display name**, plus a **background graphic**.
**Later:** a stylised avatar. Explicitly deferred by Joe — *"when the time will come for tuning"*.

⚠️ **"RANDOM" IS THE WORD TO FIX, AND IT IS NOT PEDANTRY.** A graphic randomised **per render** changes every time the room is opened. It would then be:
- **useless as an identity signal** — it tells you nothing about who you are talking to
- **actively misleading** — a changing image beside a fixed name reads as *something changed*
- **untestable** — no verification leg can assert a value that is different every time

⇒ **DERIVED, not random.** The same XGID must always produce the same graphic. That is an **identicon**: deterministic, seeded by the identity, stable for the life of the identity.

🔑 **AND DERIVED IS THE WHOLE POINT, NOT A CONSOLATION.** A deterministic graphic is a **weak identity signal** — a second channel beside the display name that an attacker cannot cheaply match. Under the no-anonymity core that is worth having. A random one is decoration and would need replacing the moment anyone asked it to mean something.

📌 **THIS BELONGS TO THE D-126 FAMILY, NOT BESIDE IT.** D-126 (the humane pubkey label) is already the project's answer to *how do we render an identity a human can recognise*. A background graphic derived from an XGID is the **same question in a different medium**. ⇒ it must **consume** D-126's canonical-vs-cosmetic decision and its seeding, not mint a parallel scheme. **Two derivations from one XGID that disagree is a D-067 drift surface.**

---

## §1a — 🔒 TWO MODES, LOCKED — AND THEY HAVE VERY DIFFERENT COSTS

🔒 **Joe, 2026-07-26:** *"it is about normal room, not dm. if so, the info will be about the room. this will be set by the room's owner"*.

| Room kind | R4 shows | Source |
|---|---|---|
| **DM** | the opposite identity's display name (+ derived graphic) | address book / `members` — **available today** |
| **Normal room** | information about the room, **set by the room's owner** | ⚠️ **does not reach the client** — see below |

🔑 **MEASURED 2026-07-26, AND THIS IS THE FINDING THAT RESHAPES THE MILESTONE.**

- ✅ **The concept exists in core.** `RoomState.topic: Option<String>` (`xgen-core/src/space/state.rs:117`), event-sourced and parsed from event content at `:284`. It is real, not hypothetical.
- ✅ **The owner role exists.** `KnownSpace.role` is `"owner" | "admin" | "moderator" | "member"` (`xgen-common/src/state.rs:189`). *"set by the room's owner"* has a role model to hang on.
- ❌ **IT DOES NOT CROSS TO THE CLIENT.** `KnownRoom` — the shape the client actually holds — is **`room_id · name · joined`** (`state.rs:195-199`). **No topic. No description. No owner-set field of any kind.**
- ❌ **`xgen-client` contains ZERO occurrences of `topic`.**
- 📌 And the UI already says so in its own comments: `spaces-panel.svelte:38` — *"`secondary`/`meta` ship UNFED (D6/D-065) — no faked topic"*. **The slots were left unfed precisely because this datum does not exist client-side.**

⇒ 🔑 **THE DM HALF OF R4 IS A UI MILESTONE. THE ROOM HALF IS NOT.** It needs owner-set room information carried to the client — a **state/wire change**, and a **write path** so an owner can set it. That is architecture, not appearance, and it is **Joe's**.

⚠️ **CONSEQUENCE, STATED SO IT IS NOT DISCOVERED MID-BUILD:** R4 cannot ship whole as a frontend milestone. Either it splits (DM half first, room half after the wire change), or it waits for the wire change and ships once. **Chat recommends the split** — the DM half is buildable today and is the half Joe described in the most detail; the room half then arrives with real data instead of an unfed slot. 🔓 **The sequencing is Joe's.**

📌 **AND THE UNFED-SLOT TRAP APPLIES HERE TOO.** Shipping R4's room mode against a field that is always `None` would light a region from a constant — the **N-097** shape that stranded `entity-item.selected`, and the exact reason `secondary`/`meta` are unfed today. **R4's room mode must not ship before its data does.**

---

## §2 — OPEN, AND WHOSE

🔓 **JOE'S (appearance and taxonomy):**
1. ✅ **RESOLVED — what R4 shows in a group room.** See §1a. What remains is the **sequencing** question it opened, above.
2. **The graphic's form** — geometric, gradient field, glyph grid. D-126's canonical-vs-cosmetic split governs.
3. Whether the avatar (deferred) **replaces** the graphic or **sits on** it.

🔓 **CHAT'S, PENDING MEASUREMENT (not Joe's to answer):**
4. Whether R4 scopes off `roomLatch.effectiveSpaceId` like R5/R6/R7, or off the room rather than the Space.
5. Whether an existing component already renders a derived graphic anywhere (checking before building).
6. What R4 renders when **nothing is latched** — R7 solved this with the self fixture; R4 has no equivalent and may need a genuine empty state.

---

## §3 — NOT IN SCOPE, STATED SO IT IS NOT ASSUMED

- ❌ The stylised avatar — Joe deferred it explicitly.
- ❌ Presence. **Layer ④ is unbuilt.** Nothing in R4 may put a dot beside a name, for the same reason §7 of M-RP-MEMBERS forbids it.
- ❌ Any claim about the identity's **current** state. The address book stores observations, not truth (M-RP-MEMBERS §3) — a header asserting *"Bob, verified"* would be lying in the exact direction the no-anonymity core exists to prevent.

---

## §4 — SEQUENCING

**After** M-RP-MEMBERS closes. R4 and R7 both render identity, and R7 lands first — so R4 should **consume whatever R7 establishes** about resolving a display name from the book versus `selfState`, rather than deciding it again.

📌 **Not scheduled. `PENDING`, not `PLAY`.** Opened so the requirement is recorded while it is fresh, not so it starts.
