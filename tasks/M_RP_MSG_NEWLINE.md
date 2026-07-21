# M-RP-MSG-NEWLINE — Preserve typed line breaks in the rendered message
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-21  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — What this is

A **one-declaration correctness fix**, not a skin item. Shift+Enter put a newline in the composer,
the message rendered it as a space. First of the five sequenced at J-564.

This is a **task doc, not a runbook**. For a single CSS declaration, authoring a runbook and opening
a handoff costs more than the change — Joe approved that deviation explicitly, and it is recorded
here rather than left as a silent widening of "Chat does not implement".

## §1 — Diagnosis (measured, with controls — J-564+ and this session)

Every step driven live on CDP 9222 against a real send with a room latched:

| step | reading | verdict |
|---|---|---|
| composer textarea | `alpha\nbravo`, 11 chars | intact |
| `bind:value` | `draftLength` 11 | agrees |
| echo store | `"text":"alpha\nbravo"` | the `\n` is in the sent record |
| rendered DOM | `textContent` `alpha\nbravo` | the `\n` reaches the paint |
| computed style | `whiteSpace: "normal"` | **the culprit** |

⇒ **The data path is entirely innocent.** `composer-panel.svelte:67`'s `draft.trim()` strips
leading/trailing whitespace only and never touches interior newlines; `message.svelte` renders `body`
as a plain **text node** (never `{@html}`); no `white-space` rule reached the message body.

🔑 **The fix was proven before any file was edited** — setting `pre-wrap` on the live element measured
the body box at **18px → 36px → 18px on restore**. The restore is what rules out a wrong element or a
stale layout.

## §2 — The change

**One file, one hunk, +13/−0: `ui/assets/skin.css`, the existing `.message .message-paragraph` block.**

```css
	white-space: pre-wrap;
```

plus a twelve-line comment marking it NOT COSMETIC, naming the measurement and the M-RP-SKIN hazard.

⚠️ **`skin.css` is Joe's file and he made the edit himself.** It rides in a Chat-authored commit only
because J-564 ruled this a correctness defect that happens to live in the skin. The in-block comment
is what stops a future skin sweep from deleting it.

## §3 — Decision ① — scope: the BODY only (Joe-locked)

**The question turned out not to be about inheritance at all.** `message.svelte` renders three
**siblings** inside `.msg-content`, not a nest:

```
.msg-content
  ├─ <p class="paragraph message-paragraph">   ← the body
  ├─ <span class="msg-deleted">                ← tombstone, ::before content: var(--msg-deleted)
  └─ <span class="message-edited">(edited)</span>
```

A rule on `.message-paragraph` therefore **cannot** reach the tombstone or the `(edited)` marker.
They would only inherit if the declaration were hoisted to `.message` / `.msg-content`.

Options put to Joe with both D-121 lenses:

- **A — on `.message .message-paragraph` (TAKEN).** ① user-visible: typed breaks render; tombstone and
  `(edited)` unchanged. ② cost: one line in an existing block; **no floor moves** (CSS moves no module
  graph, measured J-550).
- **B — hoist to `.message` / `.msg-content`.** ① identical today. Becomes a trap later: any skin copy
  string with indentation or a run of spaces starts rendering it literally. ② same line, plus a latent
  constraint on all future skin copy.
- **A2 — a new `<style>` block in `message.svelte`.** ① identical. Protects the declaration from a
  skin sweep structurally rather than by comment. ② **moves two vite floors** (N-141: a Svelte
  `<style>` block is exactly one module; client 202→203, sampler 170→171).

**Recommendation A, Joe locked A.** A2's protection is real but costs two floor moves on a
one-declaration fix, and a comment is cheaper than a module.

🔑 **Named, not traded:** `pre-wrap` also preserves **runs of spaces and leading indentation**. On the
body that is *user content*, so preserving it is correct rather than a side effect. On skin-owned copy
it would be wrong — which is exactly why the rule stays off the tombstone and the marker.

**Deliberately NOT folded in:** `overflow-wrap: anywhere`. `pre-wrap` still soft-wraps at spaces, so a
long unbroken token behaves exactly as it does today — no regression, and adding it would be an
unmeasured change riding a measured one.

**Side effect stated rather than discovered later:** `.message[data-kind="system"] .message-paragraph`
also matches, so system notices gain `pre-wrap` too. Those strings are app-authored and contain no
newlines today — **no user-visible change**, and correct behaviour if one ever does.

## §4 — Decision ② — the wire leg is SEPARATE (Joe-locked)

The echo store proves the **client's own record** holds `\n`. Whether it survives out to a node and
back is **untested** and needs a running node.

- ① user-visible: shipping the CSS now fixes the case Joe reported. If the wire strips `\n`, remote
  messages still collapse and it presents as *my breaks work, theirs don't*.
- ② cost: identical total work either way; blocking merely defers it.

**Taken: ship the CSS, file the wire leg.** No wire outcome changes this declaration. If `\n` does not
survive the round trip that is a **protocol/serialisation defect with its own fix**, and finding it
after the CSS is in is strictly easier, because the renderer is no longer a confound.

⚠️ **Do not assume the protocol preserves it.** It is untested, not presumed working.

## §5 — Verification (Chat, driven live at 9222)

Client only, launched detached with an absolute `-File` path and explicit `-WorkingDirectory`; no
cargo run concurrently (N-117). Ports polled by number, not by process name.

- **V1 — the declaration reaches the element.** Computed `whiteSpace` on `.message-paragraph` =
  `pre-wrap`. Nothing shadows it: the pre-fix probe measured `normal`, i.e. **no author rule reached
  the element at all**, so a rule at specificity 0,2,0 cannot be out-specified.
- **V2 — the break paints, WITH ITS POSITIVE CONTROL.** `controlline` (11 chars, no newline) → **18px**;
  `alpha\nbravo` (11 chars, one newline) → **36px**. ***Identical character count is the whole point of
  the control*** — same font, same element, same width, so wrapping cannot explain the difference.
  DOM `textContent` reads `alpha\nbravo`, so the data path is intact end to end at this commit.
- 🔑 **Unplanned, and it lands on the strongest row:** the newline message arrived as a **grouped
  continuation** (same author within the window — avatar and name correctly suppressed, 3 registered
  ids instead of 5, D-106 behaving). So `pre-wrap` is proven on the row type that suppresses the most
  chrome, not merely on a fresh-header row.
- **V3 — decision ① holds IN FACT, not only in the selector.** A `.msg-deleted` and a
  `.message-edited` span injected into a real `.msg-content`, read, removed: both **`normal`**;
  `.msg-content` itself **`normal`** (nothing inherits downward); **positive control** — the real
  `.message-paragraph` in the *same subtree* reads **`pre-wrap`**. ***Without that control, "normal
  everywhere" is indistinguishable from a probe reading nothing*** (N-139/N-142 family). Revert
  confirmed: 0 probe nodes remain.
- **V4 — registry, DECOMPOSED not adjusted.** **164**, `count === unique === domCount`, 0 duplicates.
  From the 158 room-latched baseline: **−2** (the *"No messages in this room yet."* row leaves)
  **+5** (first echo: avatar · label · paragraph · send-status · message) **+3** (second echo, grouped:
  paragraph · send-status · message) = **164 exact**.

**Floors NOT re-measured, and the reason is stated rather than the numbers quoted:** the diff is
**zero `.rs`, zero `.ts`, zero `.svelte`** — one CSS block. cargo 1549/0/62 × 56 · svelte-check 0/34/15 ·
npm 142 · vite 202 client / 170 sampler · catalogue 419 are unchanged **by scope** (N-108: arithmetic
is not measurement, and quoting numbers nobody took is worse than saying which ones were not taken).

## §6 — Findings

- ⚠️ **N-155 — a registry baseline has a SEVENTH axis: WHETHER A ROOM IS LATCHED.** Measured this
  session on one fresh launch: **149** at rest (three spaces sitting in the store, nothing selected)
  → **156** with a space selected → **158** with a room latched. The J-563 kickoff carried **158** as
  *the* client baseline without naming the condition that produces it. ***A baseline quoted without
  its condition is not wrong, it is unreadable*** — the next Phase-0 must say which of the three it
  means. Extends N-148/N-152's list (quiescence · store · selection · saved-state count · echo count ·
  settings drill-in · **room latch**).
- ⚠️ **NOT decomposed, named rather than smoothed:** space-select moved 149 → 156, of which rooms
  account for **+3** (four room ids minus the vanished empty placeholder). **+4 unexplained.** Outside
  this milestone, no bearing on the fix, and no account was invented for it.
- 🔑 **The store residue the kickoff warned about is real and behaved exactly as described** — three
  spaces and two rooms persist across a full reload via D-114's UI-state store, the carve-out
  M-RP-PROCESSOR-WIRE leaned on. Joe ruled content-in-widgets acceptable for now. Quoted in every
  number above so nobody hunts a `+9` that was never a defect.

## §7 — DoD

**IMPLEMENTER (Joe, by hand — `skin.css` is his)**
- [x] `white-space: pre-wrap` + the NOT-COSMETIC comment in `.message .message-paragraph`
- [x] tabs preserved, no reformat of the surrounding block

**[CHAT]**
- [x] diff scope proven: `git diff --numstat` = `13  0  ui/assets/skin.css`, one file, one hunk, +13/−0
- [x] encoding verified from raw bytes at an absolute path (N-110): no BOM (`2F 2A 20`), **zero U+FFFD**
      across 161 841 bytes — the `—` and `→` in the comment survived the paste as valid UTF-8
- [x] V1 · V2 (with positive control) · V3 (with positive control, reverted) · V4 (decomposed)
- [x] apps and dev servers stopped, verified **by port** — 5173 · 5174 · 5175 · 8080 · 9222 · 9322 ·
      9422 all 0 listeners, zero `XGenProtocol` processes (`taskkill /T`, N-140)
- [x] JOURNAL J-565 · CLAUDE.md PLAY · ROADMAP v5.31 · this doc, all on disk before the commit command

## §8 — Owed, not smuggled in

- **`M-RP-MSG-NEWLINE-WIRE`** — does `\n` survive a real node round trip? Needs 8080 + 9322 up.
  Filed, **not assumed working**.
- 🔒 **The consequence that outlives this milestone:** `<br>` is **off kind-4's allowlist**
  (J-564, M-RP-PROCESSOR-RENDER). `pre-wrap` keeps the body a **text node with no `{@html}`, no
  sanitiser and no XSS surface**; `<br>` would require opening `{@html}` on every message body,
  **including ones that arrived over the wire from another identity**. *Every element removed from a
  sanitiser's allowlist is one less thing that has to be right forever.*
