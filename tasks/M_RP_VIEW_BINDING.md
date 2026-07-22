# M-RP-VIEW-BINDING — two rooms side by side, and why that is not a windowing problem
> **Status**: PENDING  
> Version: 0.1  
> Date: Jul 2026  
> **Last updated**: 2026-07-22  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

⚠️ **THIS IS AN OPEN DISCUSSION, DELIBERATELY NOT CLOSED.** Joe asked for it to be recorded mid-thought
(2026-07-22) because it needs more discussion. **Nothing below is locked except D-122.** No Phase-0 has run,
no options have been chosen, and the questions in §3 are genuinely open.

## §1 — The want

**Joe: "if I want to watch two rooms at one time, parallel, side-by-side — what shall I do?"** His first
instinct was that this needs real OS windows rather than modal areas.

## §2 — Grounding: it is not a windowing problem

🔑 **Two stream tiles side by side would show the SAME room today.** The dock already provides the geometry
for free — split layouts, row/col nodes, resizable tiles. What is missing is not a viewport; it is that
**the stream has no room of its own.**

```
stream-panel.svelte:37   $props() = { regionId, id }         <- no room prop
stream-panel.svelte:70   roomLatch.effectiveRoomId           <- reads the singleton
room-latch.svelte.ts:39  let _latched = $state<string|null>  <- ONE value, single writer
```

**Opening a second OS window produces the identical result**, plus a second JS context with no shared store
(D-122). ***The blocker is binding, not windowing.***

**Three obstacles, in ascending order of difficulty:**

1. **The stream takes no room.** It would need a binding: *follow the active room* (today) or *pinned to room X*.
2. **The latch is a singleton** — one `_latched`, one writer. It would become *the active room*, not *the room*.
3. ⚠️ **W-12 forbids it outright** — *"a widget MAY declare exactly one of: `region` … at most one."* Two stream
   regions violate the locked model. **This is the real gate, and it is a taxonomy decision, not a coding one.**

**🔑 The data layer is already ready.** `echo.forRoom(effectiveRoomId)` is **room-keyed**, so the outbound side
would work with two rooms **unchanged**. Only the view is a singleton.

**🔑 And two filed milestones already need this exact mechanism.** **M-RP-SELF-SURFACE** (Saved Messages) is
*a stream bound to something other than the latched room*, and **M11 / D-021** (the `self` thread) is the same
shape. ***Per-view binding is not a new want — it is an unnamed prerequisite that two milestones already share.***

## §3 — Open questions (NONE answered)

1. **Does W-12 relax, or do we introduce widget *instances*?** A widget with N region mounts, each with its own
   instance id, is a different model from "a widget may declare two regions". **Affects every widget, not just
   the stream.**
2. **Does the latch become "the active room"?** If a pinned tile exists, what does the shell latch drive — the
   unpinned tiles only, or is there no global latch at all?
3. **What does a pinned tile show when its room disappears** — left, deleted, access revoked?
4. **Does the composer follow the latch or the focused tile?** Sending into the wrong room is an
   identity-attributed, permanent act. ⚠️ **This one touches the no-anonymity core and is the highest-risk
   question in the list.**
5. **Is there a per-tile unread/notification consequence?**

## §4 — Deferred, with the reason recorded

**Multi-window is DEFERRED (D-122), not refused.** The real case is a second monitor, which a modal cannot
serve. It is deferred because it is not the blocker for what was asked, and because it additionally requires a
second Vite entry, a second CDP target, typed Rust geometry (D-114/D-115) and a shared-state mechanism that
does not exist. ***Nothing is wasted by waiting: the binding work is a prerequisite for the window case anyway.***
