# XGen Debug Console — Q&A Record
> Date: 2026-05-04  
> Source: imagine-side question round before designing the debug/lifecycle window.  
> Status: User answers captured verbatim, with imagine-side gloss for handoff to Chat instance.  
> Author: JozefN  
> Next: Chat reviews, especially items flagged **OPEN — for Chat** below.  

---

## Premise (user's own framing)

> "i will try with ui at the first get the state, that ui window will be possible to run.
> the client cli has no life cycle contrary to the node cli and it is problem to logging
> his functions. every log is a log file. or 3 line. i want to use ui to define its cycle
> and than to do meaningfully debugging sessions."

Restated for Chat:
- The Node CLI has a lifecycle; logs hang naturally off start → run → stop.
- The Client CLI is shot-style — every invocation is a one-off. Logs fragment into
  one-file-per-call or three-line stubs. There is no continuity to debug against.
- The proposed UI window is the **lifecycle host the client doesn't have on its own**.
  Open the window → the client is brought up to a known state → many CLI calls happen
  within that window's lifetime → close window = session ends, logs archived.
- Goal: turn N stateless CLI shots into one coherent, navigable debug session.

---

## Q1 · Primary goal of the window

> Define a session lifecycle for the client (start → run N commands → stop,
> all logs grouped under that session).

**Locked.** Everything else is secondary surface.

---

## Q2 · What is a "debug session"

> One UI window open = one session. Close window = session ends, logs archived.

**Locked.** Implication: the window itself is the session boundary primitive.
No "Start session" / "End session" button needed in the chrome — opening and
closing the window does it. We should still surface the session ID + start
time so the user knows what they're inside.

---

## Q3 · Log surface

> Single chronological stream, dense, monospace (think `tail -f` with structure).

**Locked.** No collapsible groups, no three-pane breakdown in the base layout.
"With structure" means: timestamp column, level column, subsystem/tag column,
message column — visually aligned, but still one stream. Filter rail and
search are deferred (not rejected, just not in the first pass).

---

## Q4 · Command surface — OPEN, brainstorm with Chat

User: "this will be brainstormed with chat, what do you propose?"

**Imagine-side proposal for Chat to react to:**

The user's later answer (Q10) reframes this: *"our app are not native cli engine
with ui extension. so perhaps you are right that we will need console on the
both sides."* — i.e. **the console IS the app**, not an add-on to it. So the
command surface should not feel like a sidecar form; it should feel like a
terminal prompt at the bottom of the stream.

Concrete proposal:

1. **Bottom-anchored prompt line.** Looks like `xgen> _` or similar.
   Type a command, hit Enter, the command echoes into the stream above as a
   log line of its own (level=`cmd`), and its output streams in below it.
2. **History on Up/Down arrows.** Standard readline behavior — Gen-X muscle
   memory.
3. **Tab completion of known commands.** Discoverability without a separate
   form. A small ghosted hint to the right of the cursor showing the
   completion candidate.
4. **`?` or `help` lists commands.** No menu chrome.
5. **No buttons, no dropdowns in the command surface itself.** Buttons live
   in the chrome (clear log, export session, change colors), not in the
   command lane.

This keeps the design honest to "console on both sides" and avoids the
hybrid-form trap.

**OPEN — for Chat:** confirm or push back. Specifically:
- Should the prompt be the *only* way to issue commands, or do we also expose
  a small command palette (Ctrl-K) for users who don't remember names?
- For Node side: same prompt model, or different (Node has a real lifecycle,
  so maybe a tabbed command/log view is justified there)?

---

## Q5 · State to show at a glance — OPEN for client

User: *"logging will be able to do only owner or admin for the node side.
for the client it is a question to answer."*

**Captured constraint:** on the Node, the log/state surface is gated by role
(owner / admin only). The Node debug console must not show this surface to
non-privileged sessions.

**OPEN — for Chat:** what state does the *client* user have the right to see
about their own client? Imagine-side default if no answer comes back:
- Identity fingerprint (own pubkey, truncated)
- Connected node URL + reachability
- Current Space / room context
- Local DAG head (event ID + depth)
- Last error

These are all "about me / about my own client" — no privacy concern. But
this needs Chat's ruling because some of it (DAG head, cache size) edges
into protocol detail that the Ch1 philosophy may want to keep behind a
"context-on-demand" surface rather than always-visible chrome.

For now I'll render placeholder slots in the skeleton labeled clearly as
**TBD — pending Chat ruling**, so nothing leaks into a final visual.

---

## Q6 · Visual relationship to existing skeletons

> The console can be pure terminal design with choosable colors and fonts
> (green/black or orange/black or something else) something like win console.

**Locked, with implications:**

- This window is **not** part of the `xgen-ui-shared/` skin cascade. It is its
  own visual lane: a terminal emulator aesthetic with a small palette of
  classic phosphor schemes.
- Default schemes to offer (imagine proposal):
  - `green/black` — VT220 / classic terminal
  - `amber/black` — IBM 3270 / Hercules monochrome
  - `white/black` — generic VGA console
  - `black/white` — paper terminal (printer-style)
  - `xgen` — uses the active xgen-ui-shared skin tokens, for users who want
    visual continuity
- Font: a single monospace stack. JetBrains Mono is already on the project's
  spec list (Ch6); we use that as default and offer a "system mono" fallback
  toggle for users with their own preference.
- Window chrome: minimal. A thin top bar with session ID + clock + scheme/font
  selector. No tabs, no sidebar. The terminal fills the rest.

This is consistent with the "tool, not toy" stance and avoids dragging the
skin system into a debug surface that has different ergonomic needs.

---

## Q7 · Client lifecycle states — HIGHEST PRIORITY

User: *"some definitions are already in place. also there is the .md file to
introduce proposal, which will be discussed with chat at the first moment.
this is the highest priority of me, chat and code."*

**Captured:** there's an existing proposal `.md` somewhere in the project
(not in `skeleton/` or root markdown — likely in `docs/` outside the area
imagine has read). Imagine has not read it yet.

**Action for imagine:** in the skeleton, render the lifecycle as a single
labeled status indicator in the top chrome — but **leave the state set
unspecified** (use placeholder labels: `state-1`, `state-2`, …) so it does
not pre-commit to a vocabulary that the .md proposal owns.

When the .md is read and the proposal is settled, the skeleton's status
indicator binds to the agreed states with no other layout change.

**OPEN — for Chat:** point imagine at the proposal `.md` once Chat has
reviewed it.

---

## Q8 · Variations — clarification needed

User: *"which layout do you mean? logging/console layout or app layout?"*

**Imagine-side clarification (this is a record, not a re-question):**
I meant *layout of the debug console window itself* — different
arrangements of the prompt / stream / status chrome. Given the user's other
answers (single stream, terminal aesthetic, prompt at bottom) the layout
shape is now narrow enough that **one variation** is correct for the
first pass. We can fork later if the prompt-at-bottom assumption breaks
under real use.

Decision (imagine): **one layout, one skeleton.** Not three.

---

## Q9 · Fidelity

> Skeleton only — semantic HTML, no skin, fits the existing skeleton-first principle.

**Locked.** Same rule as the rest of the project: the DOM must read with no
CSS at all. The terminal aesthetic is applied as an opt-in skin file, not
baked into the markup.

---

## Q10 · Naming — OPEN, important

User: *"this is the question! because we want to be able to use cli commands
in ui environment. our app are not native cli engine with ui extension.
so perhaps you are right that we will need console on the both sides."*

**Captured insight (this is the design pivot):**
- XGen is **not** "CLI with optional UI". The CLIs are not the canonical app.
- XGen is "a thing that has both a UI surface and a console surface, and the
  console is first-class on both Node and Client".
- Therefore the debug window is not "the debug add-on to the CLI". It is
  **the console face of the application** — a primary surface, equal in
  status to the chat-style client UI and the dashboard-style node UI.

**Naming proposal for Chat:**
- `Console` — generic, accurate, matches user's framing ("console on the both
  sides"). Used for *both* node and client. Distinguish at the window level:
  "XGen Node Console", "XGen Client Console".
- Alternatives considered and rejected by imagine:
  - "Debug" — too narrow; this is not only for debugging, it's the canonical
    console face.
  - "Terminal" — collides with the user's host OS terminal conceptually.
  - "Inspector" — implies passive read-only, but we have a prompt.
  - "Workbench" — too aspirational; it's a console.

**OPEN — for Chat:** ratify "Console" or pick another. This name will
appear in window titles, the project filesystem, and the `index.html`
viewer toolbar.

---

## Q11 · Open notes

> "pls make a record of this questions. i will run it chat"

This file is that record. Hand to Chat for review.

---

## Summary of what imagine will produce next (first pass)

A single new file: `skeleton/console.html` — semantic HTML, no skin, no
script, that is:

- A `<header>` with: app label ("XGen Client Console" placeholder),
  session id, session start time, status indicator (placeholder states),
  scheme selector, font selector.
- A `<main>` containing the chronological log stream as a `<ol>` of `<li>`
  rows, each row carrying timestamp / level / subsystem / message in
  semantic spans. A handful of seeded entries so the structure reads.
- A `<footer>` with the prompt line: a `<label>` for the prompt
  (`xgen&gt;`) and an `<input>` for the command. A few seeded history
  entries so the relationship to the stream is visible.

Plus: register `skeleton/console.html` as a viewable app in the existing
`skeleton/index.html` toolbar, alongside Client / Node / Split.

No skin file. No JS behavior. No commitment to lifecycle vocabulary, log
schema, or command list — those are pending Chat review of the lifecycle
proposal `.md`.

---

## Items handed up to Chat (consolidated)

1. **Q4** — ratify console-as-prompt model; rule on Ctrl-K command palette
   and on whether Node uses the same prompt model.
2. **Q5** — define what state the *client* user is permitted to see about
   their own client (the node-side gating is already clear).
3. **Q7** — point imagine at the lifecycle states proposal `.md`.
4. **Q10** — ratify "Console" as the name; the design pivot in Q10 (XGen
   is not CLI-with-UI, it has dual primary surfaces) deserves Chat's
   explicit acknowledgment because it cascades into how Ch6 talks about
   the client.
