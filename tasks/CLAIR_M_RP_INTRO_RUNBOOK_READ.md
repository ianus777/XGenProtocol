# Clair's adversarial read — `RUNBOOK_M_RP_INTRO.md` v1.0: the build survives, the write half has no author, and the gate list is a census
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Aug 2026  
> **Last updated**: 2026-08-15  
> Language: EN  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 — VERDICT

**GO-WITH-FINDINGS.** The three §0 locks are coherent and implementable; §3 (Rust) and §4.1–§4.3 can be
built as written. **`F-1` must be resolved before the lock** — it is not absorbable by an implementer.

Seven findings, five pointer defects. **Two of the seven are gate-list holes and one is a specification
gap that has no honest closure available to Clair.** No finding touches a Joe-lock.

📌 **State measured at open, not inherited:** tree clean · `HEAD` `f9557ef` **= `origin/main` by
`git ls-remote`**. `git diff --name-only c3aa044..HEAD` returns **`tasks/RUNBOOK_M_RP_INTRO.md` only** ⇒
**every `B-` row was authored against source byte-identical to what I read.** No drift excuse is available
for any pointer defect below.

---

## §2 — `B-` ROW AUDIT

Every row opened at the cited file and line. **"Looks right" is not a verdict and does not appear.**

| row | verdict | what I opened |
|---|---|---|
| **B-1** | ⚠️ **CORRECTED** — count and file list both understate | 88 mention-lines repo-wide; **19 are `use` / doc / definition / comment** ⇒ **69 invocation lines, not "~60"**. The file list names **9** files; the real set is **14** — `xgen-node/src/tests/{arc_h_content_blindness,events_pipe_integration,m8_s6_e2e,m8_s7_privilege,smoke}.rs` are absent. **A census, not a partition.** ⇒ see `P-4` |
| **B-2** | 🛑 **CORRECTED — the claim is false** | The three named sites ARE production (`resident.rs:451` < its `#[cfg(test)]` at `:1098`; `ai_service.rs:489` < `:668`; `ops.rs:1996` < `:3082`). **But `xgen-client/src/app.rs` holds 15 more, ALL above its first `#[cfg(test)]` at `:6350`**, inside `pub async fn cmd_smoke_ph2` (`:1533`) · `cmd_smoke_test` (`:3399`) · `cmd_stress_test` (`:3737`) · `cmd_stress_complete` (`:4702`) — **shipped CLI subcommands.** ⇒ see `P-5` |
| **B-3** | ✅ **CONFIRMED** (material half) · ⚠️ one pointer wrong | `exchange.rs:945` *"Twin of [`build_message_text_event`]"* ✓ · `:974` *"Twin of [`build_message_file_event`]"* ✓. **Both carry a DIFFERENT `EventType`** — `MessageFile` (`:949`), `MessageRedact` (`:983`) ✓. **The "NEW sub-pattern, not the existing one" caution is correct and worth keeping.** ⇒ pointer `P-2` |
| **B-4** | ✅ field list CONFIRMED · ⚠️ **CORRECTED** on category | `resident.rs:711-728` declares exactly `{ space_id, room_id, text, reply }` ✓. **But `:711` is `pub struct OutboundRequest {` — a definition, not a construction site.** Constructions are **two**: `desktop.rs:355` ✓, `resident.rs:1565` ✓. ⇒ `P-3` |
| **B-5** | ✅ **CONFIRMED** | `derive.ts:69-97` — `projectEvent`, pure, exported, the only `message.text` → `MessageDescriptor` site ✓. `derive.test.ts` exists beside it ✓ **and exercises `projectEvent` 17 times** — which the row does not mention and which matters (`F-2`) |
| **B-6** | ✅ **CONFIRMED, verbatim** | `derive.ts:81` — `body: typeof e.content?.text === 'string' ? e.content.text : ''` ✓. **The `text` fallback is already correct.** `derive.test.ts:54-56` already pins both the non-string and absent cases |
| **B-7** | ✅ material half CONFIRMED · 🛑 **pointer WRONG, and it is an EDIT TARGET** | One literal, one tenant: `const widgets = { 'send-status': SendStatus }` ✓ — **at `:145`, not `:135`.** `:135` is a comment about timestamp interleaving. W-13 drop confirmed at `mounts.ts:50-60`. ✅ **And the row's conclusion is stronger than it claims:** `widgets` flows `stream-panel:278` → `MessageStream` → `message-stream:287` → `Message`, **so ONE entry serves every row, inbound included.** ⇒ `P-1` |
| **B-8** | ✅ **CONFIRMED, and the pointer is exact** | `types.ts:51-71` — `WidgetMount { widgetId; props?: Record<string, unknown>; mountKey? }` ✓. **`send-status.svelte:37` carries the warning verbatim**: *"`WidgetMount.props` is `Record<string, unknown>` — so NOTHING type-checks that a mount supplies this value"* ✓ |
| **B-9** | ✅ **CONFIRMED, verbatim** | `desktop.rs:313` — `if text.trim().is_empty() { return SendOutcome::failed("empty message"); }` ✓, inside `async fn send_message` at `:305` ✓. **The row's reading is right: this guard is what makes 1-bis enforceable at the seam** |

📌 **Also confirmed while checking, and worth not relitigating:** §4.1's **constant `mountKey` is correct and
has an explicit precedent** — `stream-panel.svelte:126-128` records that `resolveMounts` scopes keys PER ROW,
so one mount per row is already unique. And **the tombstone interaction is already right**:
`message.svelte:189` gates `bodyExtras` behind `{#if !deleted …}`, so a redacted intro drops its mount.

---

## §3 — FINDINGS

### 🛑 F-1 — HIGH, and it blocks §4.4: **the intro payload has no author.**

**What the runbook says.** §4.4: *"The first-send path … passes the intro alongside the text."* §2.1
scaffolds `xgen.intro.v1: { headline, blurb }`.

**What source says.** Nothing anywhere produces `headline` or `blurb`.

- `dm-draft.svelte.ts:45,47` — the draft store holds **`_counterpart` and `_texts` only**. No intro slot.
- `composer-panel.svelte:150` — `createDraftDm(counterpart: string, text: string)`. **Two parameters.**
- `:167` — `void echo.send(result.space_id, result.room_id, text)`.
- `echo-state.svelte.ts:162` — `send(spaceId, roomId, text, at)`.

The runbook specifies the write path **from `echo.send` downward** and specifies nothing above it.

**Consequence if implemented as written.** Clair reaches §4.4 holding a `text` and no intro, and has three
closures, **all of which the runbook itself forbids**:

1. **Derive the intro from `text`** — 🛑 forbidden by §2.1 in bold (*"`text` IS NOT DERIVED FROM THE KEY AND
   THE KEY IS NOT DERIVED FROM `text`"*).
2. **Ship `headline`/`blurb` unfed** — 🛑 forbidden by §7.6 and `N-182`. *A key nothing writes is a key
   nobody has round-tripped.*
3. **Invent a composer authoring UI** — and **by §2.3's own split this is a MECHANISM, which is Chat's**
   (§2.3 assigns *values* to Joe and keeps the mechanism with Chat). It is simply absent.

**Severity HIGH** because the milestone's whole content is the payload — §2.3 of the Phase-0 records
*"the render half is SOLVED and the milestone is not about it; the whole question is the payload"* — and the
payload's producer is the one step nobody wrote. **This is a specification gap, not a lock conflict**, and
it is exactly what §6 item 5 tells me to report rather than absorb.

📌 **Not proposing a design.** But note the fork is narrow: either the composer grows an intro-authoring
surface (a real chunk of Leg 3, currently unbudgeted) **or** v1 ships `xgen.intro.v1` with a single field
the composer already has — in which case §2.1's two-field scaffold is the thing that changes, not §4.4.

---

### 🛑 F-2 — HIGH, gate partition: **§5 has no `vitest` gate, and Leg 3 edits a unit-tested pure function.**

`projectEvent` is covered by `derive.test.ts` (**17 references**), including the two cases 1-bis rests on
(`:54-56`, non-string and absent `content.text` → `''`). `resolveMounts` — **the W-13 drop path that V-3
exists to prove** — is covered by `mounts.test.ts` (**28 references**). There are **9 `.test.ts` files under
`ui/`**; `ui/package.json` carries `vitest` as a devDependency.

**§5 lists cargo · svelte-check · CDP · a bespoke M1′ Vite eval. No vitest row.** Three consequences:

1. **An existing floor that this leg directly edits goes unmeasured.** §8 item 5 names cargo, svelte-check
   and the catalogue; the suite that actually tests the function being changed is not among them.
2. **V-3 and V-4 reach past a cheaper, deterministic, in-repo harness for a live one.** A Vite eval against
   a running dev server is reproducible only while that server is up; `derive.test.ts` is reproducible on
   any tree, by any seat, forever. **V-3 is the gate the whole (d) ruling was chosen for** (§8 item 4) —
   it is the one that most deserves the durable instrument.
3. **The leg would edit a tested function without extending its tests.** Nothing in §3, §4 or §8 asks Clair
   to add a case to `derive.test.ts`, so the intro branch ships uncovered by the suite that covers its
   neighbours.

---

### 🛑 F-3 — MEDIUM-HIGH, gate partition: **the `extras`-overwrites-`text` rejection has no gate.**

§3.1 requires rejecting an `extras` key literally named `text` (*"that is 1-bis violated at the lowest
level"*) and then **defers the mechanism to Clair** — *"whichever she picks she states in the deviation
report so Chat can gate on it."* ⇒ **no gate for it exists in §5.**

V-5 has two sub-claims and **neither reaches this case**:

- *"an event with the key and NO `text` never leaves the client"* — proven by `desktop.rs:313`, the
  **empty-text** guard. An overwrite arrives with `text` **non-empty** at the seam, so `:313` passes.
- *"one with `text` and no key is byte-identical to today's send"* — the **no-key** path. An overwrite is on
  the **with-key** path.

⇒ **the single branch that violates 1-bis at the lowest level is the one branch with no pre-declared gate,
in a runbook whose §8 item 3 says 1-bis is *verified, not asserted*.**

---

### ⚠️ F-4 — MEDIUM, `F9`: **V-0 cannot be run as written, and §5 and §8 contradict it on the seat.**

V-0 says the control *"belongs in the IMPLEMENTER's window, not the verifier's afterthought"* — and then
**sits in §5**, whose header reads *"Listed here so Clair can see the target — **not for her to run**"*, and
whose DoD item (§8.2) reads *"Every gate in §5 is driven by Chat."* **The document diagnoses the E-3b
sequencing defect by name and then reproduces its structure.**

Separately, **V-0's subject set is undefined**: *"the gates below fail on today's tree"* is

- **false for V-1** — a pre-edit `cargo` run is a **baseline**, not a failure;
- **false for V-6** — a pre-edit svelte-check must be **0/34/15**, i.e. it must *pass*;
- **impossible for V-2** — the optional param does not exist yet.

So V-0 meaningfully covers **V-3, V-4, V-5's second half, and V-7** and nothing else. As written, an
operator either runs it against gates that cannot fail (a control that passes for the wrong reason —
`N-194`) or guesses the membership.

📌 **It IS runnable once named:** Chat drives V-0 as the **last act before standing Clair up**, over an
explicit member list, and §5's header plus §8.2 say so.

---

### ⚠️ F-5 — MEDIUM, `F9`: **V-1 has no baseline, so it cannot fail.**

*"`cargo` floor moves in the expected direction"* — **the runbook states neither the current number nor the
expected direction.** Compare `M-RP-MEMBER-ACT` Leg B, where `1596 → 1597` was stated **before** the run.

And the expected direction is not obvious: **the delegating overload adds no tests**, so an **unchanged**
count may well be the correct result — the inverse signal this project has had to state explicitly before
(J-653: *"1588 unchanged is correct, not a failed run"*, written into the DoD **before** the leg ran so it
could not be misread afterwards). §5 should say which outcome is a pass, and `Compiling xgen-client`
appearing in the log is the check that distinguishes *"unchanged because nothing broke"* from *"unchanged
because nothing ran"*.

---

### ⚠️ F-6 — MEDIUM: **the sender never sees their own intro, and nothing says so.**

Own rows are not built by `projectEvent`. They are built by `echoToDescriptor`
(`stream-panel.svelte:119-133`), which **hardcodes** `bodyExtras: [{ widgetId: 'send-status', … }]` and
knows nothing about an intro. `projectEvent` builds **inbound** rows only.

⇒ after sending an intro, **the author's own row renders plain `text`; the recipient's renders the intro.**

That may well be the right product answer — but it is **unspecified and ungated**. V-7 (*"the intro renders
in message chrome on the live client, two identities"*) reads as covering the recipient. If it is a
decision, it should be stated; if it is not, it will be found by Joe looking at the screen.

---

### ⚠️ F-7 — MEDIUM: **§4.3 never says whether the widget takes `id` or registers.**

The mount contract is `<W id={x.id} {...x.props} />` (`message.svelte:190`), and `resolveMounts` computes
that id from the `cid('x-')` namespace (`mounts.ts:43-44`). The sibling tenant `send-status.svelte` imports
`envelope` (`:32`) and accepts `{ localId = '', id }` (`:42`).

§4.3 specifies props and markup and says nothing about `id` or `use:envelope`. Both branches have a cost:

- **If it registers** (the precedent), **every rendered intro row adds a registry entry** — and §5 has
  **no registry gate at all**.
- **If it does not**, the envelope id the host computed is discarded and the widget is unreachable from the
  debug bridge, which is the harness V-7 drives through.

The runbook should state which, because the answer changes what V-7 can see.

📌 **Adjacent, filed not raised as a finding:** `send-status.svelte:40-41` records that the prop-less
`Record<string, Component>` registry type is *"a `core` defect … FLAGGED, not folded into a `$common`
milestone."* `message-intro` becomes a **third** consumer of it. Not this milestone's to fix; named so it
is not discovered as one.

---

### 📌 Pointer defects — low individually, but three are edit targets

| # | runbook says | source says |
|---|---|---|
| **P-1** | `stream-panel.svelte:135` (B-7 **and** §4.2) | **`:145`.** `:135` is a comment on timestamp interleaving. 🔑 **The Phase-0's `G-9` cited `:143`** (the registry's doc comment — defensible) ⇒ **`:135` is a NEW error, not inherited.** §4.2 is an **EDIT TARGET** |
| **P-2** | `exchange.rs:983-989` for the redact twin's **doc comment** | the doc comment is **`:973-982`** (*"Twin of"* at `:974`); **`:983` is the `pub fn` signature.** A range that does not bound the thing it names |
| **P-3** | `resident.rs:711` as a **construction site** | `:711` is `pub struct OutboundRequest {`. **Two** constructions exist. *"Adding a field is a three-site change"* is right as an **edit** count and wrong as a **construction** count |
| **P-4** | B-1's 9-file list, *"~60 call sites"* | **14 files, 69 invocations.** Five `xgen-node/src/tests/*.rs` files omitted |
| **P-5** | B-2 *"only THREE call sites are production … the other ~57 are fixtures"* | **18 non-`cfg(test)` sites.** Honest partition: **3 user-facing send paths · 15 in shipped diagnostic subcommands · ~51 in `#[cfg(test)]`** |
| **P-6** | §3.3's file target `docs/ch3_*` | 🛑 **matches NOTHING.** The file is **`docs/xgen_ch3_specification.md`**, and the `meta_atts` namespace block §3.3 must diverge from is **`:340-357`** (`3.1.3` opens at `:332`). §3.3 calls itself *"a deliverable"* and its pointer does not resolve |

⚠️ **`P-4` and `P-5` do not move the plan** — §3.1's delegating overload churns zero call sites either way,
which is the row's real point and it stands. They are recorded because B-1/B-2 are the two rows an
implementer would lean on if the overload route were ever revisited, **and because "the other ~57 are
fixtures" is not true of 15 of them.**

---

## §4 — THE GATE LIST: **CENSUS, NOT PARTITION**

Asked of every gate: *what would this read return if the code were RIGHT?* — and *what can break that no
`V-` row would catch?*

**The likeliest defect was a failure mode nobody listed, and there are four.**

| missing | why it is not covered |
|---|---|
| 🛑 **the `extras`-overwrites-`text` branch** | `F-3`. V-5's two arms are the empty-text guard and the no-key path. **Neither reaches a non-empty `text` overwritten inside core.** This is 1-bis's lowest-level violation |
| 🛑 **the `vitest` suite** | `F-2`. `derive.test.ts` and `mounts.test.ts` cover the exact two functions Leg 3 touches. **No `V-` row runs them** |
| ⚠️ **the sender's own row** | `F-6`. V-7's *"two identities"* reads as the recipient's view. Nothing asserts what the author sees |
| ⚠️ **the registry / envelope delta** | `F-7`. §8 item 5 lists cargo, svelte-check and the catalogue. **A new registering widget moves the client registry and no gate reads it** |

**Also structurally weak rather than missing:**

- **V-0** — membership undefined and seat contradicted (`F-4`).
- **V-1** — no baseline, and the correct outcome may be *unchanged* (`F-5`).
- **V-5's second arm** — *"canonical bytes compared"* **names no surface**: compared where, against what,
  with which tool? Phase-0 §10 item 3 is explicit that *any DoD item saying "measured" MUST name its
  surface*. Every other gate in §5 names one; this half-row does not.
- **§3.3 has no gate at all.** It is called a deliverable and is covered only by §8 item 1's blanket
  *"every step in §3"*. Given `P-6` — its file pointer does not resolve — it is the row most likely to be
  quietly skipped.

✅ **Genuinely well-formed:** V-4 (table-driven, and it explicitly includes the oversized-blurb case that
§4.3's length bound needs) · V-6 (**runnable exactly as written** — `ui/package.json` `check` →
`svelte-check --tsconfig ./tsconfig.json`) · V-7's *"Chat cannot see PNGs — ASK JOE TO LOOK"*.

✅ **§7.6 check — nothing forbidden is reserved.** `OutboundRequest.intro` is fed in-milestone by §4.4; the
optional Tauri param is fed by the same path; §4.4 states outright that a no-intro send carries **no key at
all, not an empty one.** 🛑 **The one exception is `F-1`'s `headline`/`blurb`, which have no writer** — and
that is `F-1`, not a second finding.

---

## §5 — WHAT I COULD NOT CHECK

Stated rather than guessed.

1. 🛑 **V-2 — I could not settle it from source, and I decline to assert Tauri's semantics from memory.**
   What I established: **there is NO in-repo precedent.** The only `Option<` in a `#[tauri::command]` in
   `xgen-client/src/desktop.rs` is a **return** type (`fetch_identity`, `:739`), never a parameter. ⇒ **the
   runbook is right that this must be measured**, and my read strengthens that instruction rather than
   discharging it.
   ⚠️ **But V-2's framing is off in a way worth fixing:** `send_message` has **exactly ONE `invoke` call
   site** — `app_client.svelte:833`, `invoke('send_message', { spaceId, roomId, text })` — **and this
   milestone widens it.** So *"existing webview callers that omit it"* is a near-empty set. The real case is
   **the ordinary room send omitting the key**, which is worth measuring under that name.

2. 📌 **The `vitest` invocation.** `ui/sampler/package.json` declares `test: vitest run`, but **no `.test.ts`
   lives under `ui/sampler`** and its `vite.config.js` carries **no `test` block**; all 9 test files sit
   under `ui/common` and `ui/core`, and `vitest` is a devDependency of `ui/`. I could not determine from
   source which cwd reproduces the historical count. **`F-2` does not depend on the answer** — the suite
   exists and covers the edited functions either way.

3. 📌 **Whether `message-intro.svelte` should register.** `F-7` names both branches; **which one is correct
   is a Chat decision, not a measurement**, so I report the omission and take no position.

4. 📌 **The three §0 locks were read for internal contradiction and I found none.** (d) + (d1) +
   `xgen.intro.v1` compose: a versioned key inside `content` degrades to plain `text` on an ignorant reader
   exactly as claimed, and `G-4g`'s signature property makes stripping it detectable. **I did not re-derive
   the `G-` rows themselves** — the kickoff scoped me to the runbook — **except where a `B-` row leaned on
   one**, which is how `P-1` and `P-6` surfaced.

5. 🛑 **No floor was run and none is claimed.** This pass touched no `.rs` and no `ui/**`; svelte-check
   **0/34/15** is stated as carried, not measured. **The catalogue stays UNMEASURED** — its harness was not
   located, and a number not driven does not enter a record.

---

## §6 — THE ONE PROPERTY, CHECKED DIRECTLY

**Does any step let `text` become empty, derived, or decorative?**

**No step does — and two of them actively protect it.** §4.1 forbids touching `derive.ts:81` (verified
verbatim, and already defensive). §3.2 item 4 keeps `desktop.rs:313` (verified verbatim). §2.1 forbids
deriving either from the other. **The composition rule is right and the plan honours it.**

🛑 **But the property is guarded by argument in one place and by a gate in none:** the `extras`-overwrite
branch (`F-3`) is where a non-empty `text` can be silently replaced, and V-5 does not reach it. **1-bis is
protected at the seam and unprotected in core.**

---

📌 **Chat amends the runbook; nothing above was edited into it.** No code was written, nothing was pushed.
