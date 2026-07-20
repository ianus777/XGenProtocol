# M-RP-PROCESSOR-WIRE — the Text Processing row, composer wiring, and rule persistence
> **Status**: COMPLETED  
> Version: 1.4  
> Date: Jul 2026  
> **Last updated**: 2026-07-20  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — THE FENCE

This milestone does **three** things and nothing else:

1. **Leg C (first)** — user-owned substitution rules survive relaunch. Rust.
2. **Leg A** — a `Text Processing` plugin row whose settings pane is the existing `substitutions-editor`, plus the persist seam that makes its Apply durable.
3. **Leg B** — one processor spread at the composer's textarea.

**NOT in scope, named so nobody smuggles them in:**

- No new processor kind. Kinds 1–3 are built; kind 4 (`use:render`) stays deferred (D-065).
- No per-pair editor UI. The one-textarea shape is the Joe-lock from M-RP4.3 and is unchanged.
- No change to the `[substitutions]` grammar (D-100) or to `parseRules`/`applyRules`.
- No change to D-120's settings mount contract. `<C />` stays prop-less and generic.
- No processing on any input other than `composer-panel`'s textarea.
- No retirement of D-101. **D-101 stays whole and keeps wiping `xgen-client_config.toml`.**
- No `core` component change. The `Record<string, Component>` typing debt is a *separate* filed item and must not be folded in here.

---

## §1 — GROUNDING (read this before the design; it corrects the kickoff)

Every claim below was grepped at J-561, not remembered. Line numbers are as of `16eb555`.

### §1.1 — The subsystem is complete at both ends and connected to nothing in the middle

| Fact | Evidence |
|---|---|
| The client **loads** the user's rules at boot | `app_client.svelte:537` — `substitutions.setRules(await invoke('get_substitutions'))` |
| The Rust **read** path exists | `desktop.rs:410` `get_substitutions` → `app.rs:246` `load_substitutions_section` (fails soft to empty) |
| The Rust **write** path exists **and is tested** | `desktop.rs:421` `set_substitutions` (registered `:904`) → `app.rs:271` `write_substitutions_section` (strict read-mutate-write; round-trip / error / clearing tests at `app.rs:6371`–`6437`) |
| `textarea` is processor-**READY**, not processor-containing | `textarea.svelte:24`/`:28` comments, `...rest` `:52`, `{...rest}` `:73` |
| The composer wires **nothing** | `composer-panel.svelte` — **zero** matches for `processor`/`substitution` across the whole file |
| The editor is mounted **only in the sampler** | `plugins/registry.ts:100` states it in writing; no client mount exists |

⇒ **The client loads your substitution rules, has no screen to edit them, and no input that applies them.**

### §1.2 — ⚠️ THE FINDING THAT SETS THE SCOPE: D-101 is live in the shipping client

`desktop.rs:677` calls `app::clean_slate_config()` at launch. `app.rs:414` — if a config exists, `remove_file` it and regenerate from seed.

**So the write path works and the file is deleted at the next launch.** The editor's own note already says so: *"Changes apply this session; config resets on restart."*

Neither the kickoff nor J-560 knew this. A settings screen for user-authored data, in front of a config file that is deleted at every launch, is the sharpest possible instance of what D6 and D-065 exist to prevent — **and it would have shipped looking finished.**

### §1.3 — ⚠️ M-RP-SETTINGS IS CLOSED. The "second settingsComponent unblocks J-513" argument is DEAD.

`docs/ROADMAP.md` L931: Leg A ✅ J-535 · Leg B ✅ J-537 · **Leg C CLOSED J-540 → "the SETTINGS arc is CLOSED"**, with **D-B → D-120 minted** (`DECISIONS.md:4452`).

J-513 was decided at J-534 and minted at J-540. It was **never** waiting on a second `settingsComponent`. The `Text Processing` row is a **second tenant of a closed mechanism**, not a gate opener. Corroboration for D-120 at best, and D-120 does not need it.

**Do not restate the "one instance is a special case, two is a pattern" argument anywhere.** It appears in the CLAUDE.md PLAY block, `ROADMAP.md` L804 and the J-560 kickoff, and all three are corrected in the records leg.

### §1.4 — The store already fails safe, so `processor()` has NO crash path

`processor.ts:38` calls `assertSafeRules` **at render time** and **throws** on an invalid untrusted set. That looks like a crash surface for a persisted-and-gone-bad rules file. It is not:

`store.svelte.ts:setRules` validates with `{trusted:false}` and, on rejection, **keeps `_rules` empty** while still stashing the raw text in `_source`. So `substitutions.rules` is *always* a validated set and the composer's `processor()` re-validates something that already passed.

**⚠️ But the failure it DOES have is the one Leg C creates.** A bad rules file **silently disables every substitution**, and the only explanation is a `console.warn` stripped from release builds. Under D-101 this was near-impossible (config regenerated from seed every launch). Under persistence, a stale file — an older grammar, a hand-edit — survives into every launch and quietly does nothing.

**The surface for it already exists and is free.** `_source` is set on reject as well as success, and `substitutions-editor` seeds its draft from `source` with live inline validation. So a user opens **Settings ▸ Text Processing** and sees their actual rules *and the reason they were refused*. This has simply never been reachable in the client. → **V-C3.** It is a verification leg, not an assertion.

### §1.5 — The `trusted` flag: grounded, and the kickoff's premise was half right

- The sampler **does** pass `{ trusted: true }` — `app_sampler.svelte:586`. Kickoff correct.
- But `processor()`'s default is **already** `false` (`processor.ts:33`), and the store validates as Tier-2 regardless.

⇒ **The composer writes `processor(substitutions.rules)` with no options object at all.** See §3.2.

### §1.6 — The settings mount is prop-less by design

`ui/client/src/settings-dialog.svelte:193–195`:

```svelte
{:else if drill?.mode === 'settings' && drillPlugin?.settingsComponent}
  {@const C = drillPlugin.settingsComponent}
  <C />
```

*"generic mount — no per-plugin branch"*. `substitutions-editor` takes a host-injected `onApply` and lives in `$common`, so it **cannot** import `invoke` (W-3, the M-RP4.3 first-instance finding). Mounted as-is it type-checks, renders, and **silently never persists**. See §3.3.

### §1.7 — The clean-slate discriminator Leg C needs ALREADY EXISTS

`app.rs:414`:

```rust
pub fn clean_slate_config(config_path: &Path, keypair_path: &Path) {
    if config_path.exists() {
        let _ = std::fs::remove_file(config_path);
        let _ = write_fresh_config(config_path, keypair_path, None);
    }
}
```

The whole block is **already gated on `config_path.exists()`** — a genuine first run is left untouched so `run_startup`'s first-run SETUP detection still fires (`clean_slate_leaves_first_run_untouched`, `app.rs:6555`).

**That gate is exactly the J-438 discriminator Leg C needs**, and it is already the right one:

- **no config existed** → block skipped → first run seeds the starter pack (`DEFAULT_SUBSTITUTIONS_SEED`, `write_fresh_config` `app.rs:389`)
- **config existed with an empty rules string** → preserved as empty → **cleared pairs stay cleared**

*The code shape B-narrow needs is already the code shape that is there.* Leg C adds a capture and a re-inject inside an existing gate; it does not add a gate.

---

## §2 — THE FRAMING (Joe-locked)

**Leg C is not an exemption from D-101. It is the correction of a mis-filing.**

`app.rs:290–291` already drew this exact line, in writing:

> *"NOT config: the UI-state store is the project's first deliberately persistent user-facing state, so it is NOT touched by D-101 clean-slate-on-start (which wipes `xgen-client_config.toml` only)."*

The discriminator there was never *"this section is special"* — it was **persistent user-facing state vs. config**. `ClientConfig` holds five sections (`client`, `paths`, `ai`, `sync`, `substitutions`). The first four are machine and deployment config. **`substitutions` is the only one whose content a human authors** — `app.rs:69` calls it *"user-owned text-substitution pairs"* in the code's own words.

⇒ The sentence to use everywhere is **"user-owned content was mis-filed into the config file, and D-101 wipes config files."** Not *"exempt substitutions from D-101."*

**Why the wording is load-bearing:** it keeps D-101 crisp — *config is wiped, whole and undiminished; user-owned content is not config* — and stops Leg C becoming a precedent for arbitrary per-section exemptions later.

**Option E was named and not taken.** Moving `[substitutions]` to its own store beside `xgen-client_uistate.json` gives an outcome a user could not tell apart, at meaningfully higher cost (new file, new load/write path, both Tauri commands repointed, one-time migration). It remains available later as the tidier home **without reversing anything Leg C does**. Joe chose B-narrow on cost with E on the table, not by default.

---

## §3 — DECISIONS

### §3.1 — D-C: Leg C preserves the rules string across the wipe

> ⚠️ **AMENDED J-562 — THIS SECTION AS ORIGINALLY WRITTEN WAS SELF-CONTRADICTORY AND WOULD HAVE SHIPPED A SILENT DATA-LOSS PATH. Rule-6 flag 1, raised by the implementer, and it changed the code.**
> It named `load_substitutions_section` (fail-soft) for the capture **and** required an unreadable old config to "proceed with a fresh seeded config". **Incompatible:** that helper **collapses *could-not-read* and *user-cleared* into one empty string**, so re-injecting it would blank the starter pack just written — ***a user with a corrupted config loses their substitutions AND gets no starter pack, silently.***
> **CORRECTION AS SHIPPED:** the capture uses a new fallible sibling, `try_load_substitutions_section -> Option<SubstitutionsSection>`. **`None` (unreadable/malformed) → skip the re-inject, leave the seed standing. `Some("")` (the user cleared their pairs) → re-inject empty, so the clearing rides across.**
> 🔑 **The defect was that §3.1 conflated exactly the two states §3.6's J-438 discriminator exists to keep apart, one level down — written by the same seat, in the same document, and re-read before hand-over.** Recorded in D-101's amendment too, because it is the kind of thing a future reader otherwise re-derives by breaking it.

`clean_slate_config` captures `[substitutions].rules` **before** `remove_file`, then re-injects it **after** `write_fresh_config`. Both helpers already exist and are tested: `load_substitutions_section` (fail-soft, `app.rs:246`) and `write_substitutions_section` (strict, `app.rs:271`).

D-101's rationale survives intact: the regenerated file has whatever new **shape** we want; only the user's rule **text** rides across.

**Failure posture:** the capture is fail-soft (an unreadable/malformed old config yields empty, and the launch proceeds with a fresh seeded config) — the `load_substitutions_section` precedent. A launch must never be blocked by an unreadable old config. The re-inject is best-effort (`let _ =`), matching the two calls already in the function; a failed re-inject leaves a valid seeded config, never a broken one.

### §3.2 — D-B1: the composer writes `processor(substitutions.rules)` — no options object

**① User-visible impact: NONE.** Behaviour is identical with or without an explicit `trusted: false`, because the default is already `false` and the store has already validated as Tier-2. **Saying this plainly rather than inventing a UX rationale** (D-121: a manufactured rationale launders an internal preference as a user's interest).

**② Resource cost: zero**, and it removes a `trusted:` token from the call site. **An explicit `trusted: false` is a worse artifact than an absent one: it advertises a knob that must never be turned.**

The double validation (store, then `processor()`) is harmless and cheap on an already-validated set. **Do not "optimise" it away** — it is the guard that makes §1.4's no-crash claim true independently of the store.

### §3.3 — D-A1: the persist seam lives on the `$common` store

Three exits from §1.6's collision:

| | ① user-visible | ② cost |
|---|---|---|
| **(i) persist seam on the `$common` store** ✅ | Apply persists; nothing else moves | **Lowest.** One store method, one shell line beside the existing read at `app_client.svelte:537`, editor calls it instead of a prop. D-120's mount contract untouched — no tenant gets special handling |
| (ii) widen the mount to a uniform prop bag | none | Changes the settings contract for one tenant's benefit; every future tenant inherits a bag it ignores |
| (iii) a shell-side wrapper component | none | **Structurally impossible** — `$common/registry.ts` cannot reference a shell component (W-3) |

**(i) is taken.** It mirrors the read path exactly: the shell already injects across this seam at boot; Leg A adds the write half beside it.

**`onApply` is NOT removed from `substitutions-editor`.** The sampler mounts it without a host and must stay live-only (D-097 / W-8). The editor's `apply()` calls the store seam **and** `onApply?.()` — the seam is the client's channel, the prop is the sampler's. Neither is load-bearing on the other.

### §3.4 — D-A2: the row is `kind:'system'`, `surface:'none'`, no `component`

The first row that is **purely a settings surface** (Grid Backdrop minus the component).

`kind:'system'` ⇒ **no disable button** (`plugin-list.svelte:91`, `disabled: !isCustom`, W-13) ⇒ no data-loss path and no uninstall question. `kind:'custom'` would reopen both: every existing custom plugin owns **no** user data; this one owns a curated rule list the user authored by hand.

**Registry consequence (N-147, do not derive it — enumerate it):** the row is `surface:'none'` **with no `component`**, so `layout-default` does **not** pick it up (its derived `bgWidgets` requires `surface:'none' && component` — `registry.ts:212–219` is the grid-plate shape it must NOT match). The delta should therefore be **plugin-list row ids only, zero widget ids.** Enumerate both readers live; arithmetic agreement afterwards is a check, not a derivation.

### §3.5 — D-B2: exactly ONE call site is wired, and the other is refused on the record

**Measured across the whole client+common tree — two text-input call sites:**

- `composer-panel.svelte:94` → **PLUG IN**
- `substitutions-editor.svelte:98` → **NEVER.** Processing the textarea where you author substitution rules is a feedback loop: typing `:)` into the rule list rewrites the rule you are writing.

**The wiring policy (Joe-locked J-560): default OFF everywhere; plug in only where it matters.** Two gates:

1. the component must forward `{...rest}` — only `converter-field`, `number` and `textarea` do. **`password-field` and `textfield` do NOT, so processing them is structurally impossible.**
2. the consumer opts in per call site.

**The criterion — COMPOSING vs CONFIGURING.** Composing is prose written in flow where the transformation is visible and correctable as it happens. Configuring is a value stored and reused, where the rewrite lands silently and nobody is watching. Plus a **third, stronger class: values that must round-trip BYTE-EXACT** (XGIDs, tokens, passwords) — a correctness rule, not a preference.

### §3.6 — D-C2: J-438 seed-once resumes for the preserved section, AS AN INTENDED CONSEQUENCE

With Leg C, **cleared pairs stop reappearing.** That is correct behaviour — a user who deletes a rule expects it to stay deleted — but it is a **behavioural change**, and it is recorded here as **chosen, not discovered**, with its own verification leg (**V-C2**), so it cannot surface later as a surprise.

D-101's written text says seed-once resumes *"when the client/node UIs are rewritten with persistent settings"*. Leg C is the first instalment of that exit condition, scoped to the one section that holds user-authored content. **D-101 is otherwise unretired** and `write_fresh_config` still seeds the starter pack at config birth.

---

## §4 — LEGS, IN ORDER

**Order: C → A → B.** Rationale, recorded because it was Joe's call on a question left open: Leg C is the only leg with a Rust floor, and it is the only one whose absence makes the other two **ship a promise we break**. Building it last would mean A+B exist for a window in exactly the state §1.2 rejects.

### Leg C — rule persistence (Rust). Own commit. ✅ **CLOSED (J-562, code `1932474`).**

> ✅ **CLOSED.** Shipped as specified except where flagged. `clean_slate_config` captures before the wipe and re-injects after the regen. **⚠️ §3.1 BELOW IS WRONG AND WAS CORRECTED IN IMPLEMENTATION** — see the amendment there. Two stale comments corrected (`app.rs:300` **and `desktop.rs:671–676`**, the latter omitted from §5). Tests as flagged: T-C1 + T-C2 as specified, T-C3 **repurposed** (the specified one was a duplicate), and `clean_slate_wipes_and_reseeds_existing_config` **retargeted + renamed** to `..._wipes_and_regenerates_...`. **cargo 1549/0/62 × 56 — exactly the predicted +3.** V-C1 / V-C2 / V-C3a / V-C4 all driven twice, independently, every one with a positive control. **D-101 amended in place (Joe's option A), no new D-number.**

**🔒 STATUS NOTE — THE MILESTONE IS NOT DONE.** This header stays `ACTIVE`. §6's `Status: COMPLETED` is the **milestone's** signal, not a leg's, and **Legs A and B remain unbuilt.** Flipping it here would mark a milestone done with two thirds of it missing.

**⚠️ THE SINGLE-WRITER WINDOW IS NOW CLOSED.** Leg C leaves **exactly one** writer of `[substitutions]` (the Rust startup path). **Leg A deliberately adds a second** (editor Apply → store seam → `set_substitutions`). The Leg C legs were therefore re-driven at `1932474` and **must not be deferred to the milestone close** — after Leg A, re-driving V-C2 means first proving the UI did not write during launch.

- `app.rs::clean_slate_config` — capture before wipe, re-inject after regen (§3.1).
- Tests (three, all in `app.rs`'s existing test module, beside `clean_slate_wipes_and_reseeds_existing_config` at `:6531`):
  - **T-C1** a pre-existing config with user pairs → pairs survive the clean slate, other sections regenerate.
  - **T-C2** a pre-existing config with an **empty** rules string → stays empty (**the J-438 leg**, §3.6).
  - **T-C3** genuine first run (no config) → untouched; the existing `clean_slate_leaves_first_run_untouched` must still pass unmodified.
- Doc comments on `clean_slate_config` and `write_substitutions_section` updated: the *"session-only this phase"* phase-note at `app.rs:300` is now **false** and must be corrected in the same commit.

**`cargo` floor MOVES on this leg** (+3 tests). Predict, then decompose any miss — never adjust.

### Leg A — the row + the persist seam. No Rust.

- `store.svelte.ts` — add the persist seam (a settable host callback + its invocation point). Keep `setRules`'s fail-safe semantics **exactly** as they are; §1.4's whole no-crash argument rests on them.
- `app_client.svelte` — fill the seam once at boot, **directly beside** the existing read at `:537`.
- `substitutions-editor.svelte` — `apply()` calls the store seam **in addition to** `onApply?.()` (§3.3). Do **not** delete `onApply`.
- **⚠️ The editor's closing note must change.** `"Changes apply this session; config resets on restart."` is false after Leg C and would be a lie shipped in the settings pane. Replace with functional, PROVISIONAL copy (§7).

  > ⚠️ **AMENDED J-563 — THE INSTRUCTION ABOVE ASSUMES ONE TRUTH, AND THERE ARE TWO. Grounded, not inherited: `xgen-sampler/src/main.rs:113` STILL CALLS `clean_slate_config`, and Leg C deliberately touched zero sampler code.** So one `$common` component, with one note, is mounted in **two hosts whose persistence behaviour is now OPPOSITE**: in the **client** the note is about to become **false**; in the **sampler** it remains **true**. A single replacement string is wrong in one host whichever way it is written.
  > **🔒 DECIDED UNDER THE GRANT — the note DERIVES FROM THE PERSIST SEAM, it is not a constant and not a new prop.** The seam's presence *is* the fact: **seam filled ⇒ "your rules are saved" · seam unfilled ⇒ the existing session-only wording.** The client fills it (§3.3); the sampler does not and keeps `onApply` (D-097/W-8).
  > **① USER-VISIBLE IMPACT — this is why, and the cost is not the reason.** A constant string tells **one of the two users something false about whether their work survives**: either a sampler user is promised persistence that will be wiped at next launch, or a client user is warned their curated rules will be lost when they will not. *A settings screen that misdescribes whether it saves is the same failure D6 and D-065 exist to prevent as §1.2's — just quieter.* **② RESOURCE COST: lower than the alternative** — no new prop, no host branch, and **the note cannot drift out of true**, because it is derived from the mechanism rather than maintained alongside it. *(Recorded second, and not leading: the correctness argument stands on its own even if the costs were reversed.)*
  > ⚠️ **The wording itself is PROVISIONAL and Joe's to judge live** (§7) — only the derivation is locked here.
- `plugins/registry.ts` — the `Text Processing` row (§3.4), with `settingsComponent: SubstitutionsEditor` and the import that entails.

### Leg B — the composer spread. No Rust. ✅ **CLOSED (J-563, code `bf4a530`).**

> ✅ **CLOSED.** One file, +20/−0. `{...processor(substitutions.rules)}` on the `<Textarea>` at `:94`, **no options object**, with the §3.5 policy recorded as a comment at the call site that carries it. Nothing else in the file moved. **cargo identical** — the direct zero-Rust proof. **vite client 201 → 202**, predicted from reachability (`processor.ts` reachable from the client for the first time) and landed exactly. **Registry 149 identical — an attachment is not an element.** V-B1/V-B2 re-driven by both seats: same token, same rules, same session, composer **morphed**, editor **unmorphed**. §3.5's feedback-loop refusal is true in fact.

- `composer-panel.svelte` — import the store + `processor`, add `{...processor(substitutions.rules)}` to the `<Textarea>` at `:94`. **One spread.** No options object (§3.2).
- Nothing else in the file moves. The `sendMessage` path, the room latch, the echo store and lock #12 (the textarea is never disabled) are all untouched.

### ⚠️ MILESTONE CLOSED — AND THE DEFECT THIS RUNBOOK'S VERIFICATION COULD NOT SEE

> **`-->` IS UNREACHABLE BY TYPING, AND EVERY §9 LEG IN THIS DOCUMENT PASSED ANYWAY.** The seed holds **both** `--` and `-->`; typing `-` `-` completes the shorter rule immediately, so the `>` lands after a figure dash. **`<--` survived only because `<-` was not a rule.**
> ⇒ **A RULE WHOSE PROPER PREFIX IS ALSO A RULE IS UNREACHABLE BY SEQUENTIAL TYPING.**
> **🔑 THE MISS IS THIS RUNBOOK'S, AND IT IS A METHOD DEFECT: §9 NEVER SAID *HOW* TO TYPE.** Both seats drove every probe by **setting the value wholesale** — one assignment, one dispatched `input` — so `-->` always arrived as a **complete string** and always matched. ***A live edit-side transformer is a function of the INPUT SEQUENCE, not of the final string; a probe that skips the sequence cannot see a defect that only exists in the sequence.*** Found by Joe in two minutes of using the app. → **N-154.**
> **→ BINDING ON EVERY FUTURE KIND-1 RUNBOOK: the verification leg must type CHARACTER BY CHARACTER — one `input` event per character, asserting after each — and must say so BY NAME.** Carried into M-RP-PROCESSOR-SEED leg ①.

### §7 — PART-DISCHARGED. DO NOT READ THIS AS AN APPEARANCE PASS.

> Joe viewed the pane and the composer live at the close and returned **two items**: the seed defect above, and **Send should be an icon button rather than a text button** (⚠️ **not doable honestly today — `icons.ts` holds 17 glyphs and none is a send glyph, and D-108 forbids fabricating a Material `d` path**; needs a verified path first → **M-RP-SKIN**).
> **⚠️ THE NOTE WORDING STILL HAS NO VERDICT.** *"Changes are saved and applied on the next start."* remains **PROVISIONAL**; only its **derivation from the persist seam** is locked. **M-RP-SKIN owes it.** *Nothing in this document should be read as Joe having approved those words.*

### Leg R — records. Travels with the leg it closes, per D-074.

Not a code leg. See §6 `[CHAT]` and §10.

---

## §5 — FILE LIST (COMPLETE — grep-verified, not recalled)

> ⚠️ **This section is where J-560 failed.** Its §5 omitted `plugins/registry.ts`, *without which the composer would never have mounted at all* — a file list that omits the row mounting its own widget produces a component nobody can see. This list was built by grep and re-read against the code before hand-over.

**Leg C (Rust):**

1. `xgen-client/src/app.rs` — `clean_slate_config` body; the `write_substitutions_section` phase-note at `:300`; three new tests.

**Leg A (frontend):**

2. `ui/common/lib/components/processor/store.svelte.ts` — the persist seam.
3. `ui/client/src/app_client.svelte` — fill the seam at boot (beside `:537`).
4. `ui/common/lib/components/widgets/substitutions-editor.svelte` — `apply()` + the closing note.
5. **`ui/common/lib/plugins/registry.ts`** — the `Text Processing` row + its import. **Without this file there is no row and no settings pane; the milestone is invisible.**

**Leg B (frontend):**

6. `ui/common/lib/components/widgets/composer-panel.svelte` — imports + one spread at `:94`.

**Possible, to be confirmed by the implementer, NOT assumed:**

7. `ui/assets/skin.css` — only if the editor's new note needs a rule. If the existing `.subs-note` rule covers it, **do not touch skin.css.**

**Explicitly NOT touched — a diff outside this list is a Rule-6 flag, not a tidy-up:**

- Any `ui/core/**` file. (The `Record<string, Component>` debt is a separate filed item — §11.)
- `ui/sampler/**`. The sampler's `trusted: true` cell stays exactly as it is.
- `layout-default.ts` — the row has no `component`, so nothing derives from it (§3.4).
- `settings-dialog.svelte` — the mount stays prop-less and generic (§3.3).
- `transform.ts`, `processor.ts`, `clamp.ts`, `configs.ts` — the engine does not move.
- Any `.rs` file on Legs A and B. **`cargo` IDENTICAL is the direct proof, not a corroboration.**

---

## §6 — DEFINITION OF DONE

### IMPLEMENTER

- [ ] Leg C: capture-before-wipe + re-inject-after-regen landed; T-C1/T-C2/T-C3 written and green; `clean_slate_leaves_first_run_untouched` still green **unmodified**.
- [ ] Leg C: the stale *"session-only this phase"* phase-note corrected in the same commit.
- [ ] Leg A: persist seam on the store; shell fills it at boot; `apply()` calls seam **and** `onApply?.()`; `onApply` **not** removed.
- [ ] Leg A: the editor's closing note no longer claims settings are lost on restart.
- [ ] Leg A: the `Text Processing` row present in `CLIENT_PLUGINS` with `kind:'system'`, `surface:'none'`, **no** `component`, `settingsComponent` set.
- [ ] Leg B: exactly **one** spread added, at `composer-panel:94`, with **no** options object.
- [ ] `substitutions-editor:98` **not** wired. Confirmed by reading the file, not by intent.
- [ ] `svelte-check` errors at **0**. Warning delta reported either way.
- [ ] Scope proven by `git show --stat` against §5 — per leg, not in aggregate.
- [ ] Every §9 leg driven, with its measured value recorded. A leg that could not run is reported **inconclusive**, never omitted.
- [ ] Rule-6: every deviation from this runbook flagged to Joe, **not absorbed**.

### [CHAT]

- [ ] Re-drive every non-destructive §9 leg independently (Rule 5). **A number Chat has not measured does not enter a canonical record.**
- [ ] Enumerate the registry delta from **both** readers (§3.4, N-147). Arithmetic agreement is a check, not a derivation.
- [ ] Floors re-measured after a **full reload — dev server gone, proven by ports, not names** (N-140).
- [ ] JOURNAL J-561 · CLAUDE.md PLAY · ROADMAP · this doc → `Status: COMPLETED`, in ONE commit (D-074).
- [ ] **The §1.3 correction landed in all three places** — the PLAY block, `ROADMAP.md` L804, and J-560's record.
- [ ] The seventh self-contradiction recorded per §10.
- [ ] **D-101 AMENDED IN PLACE at Leg C close (Joe-locked 2026-07-20, option A).** The scope discriminator — *config is wiped; user-owned content is not* — is stated in D-101 itself, and D-101's **exit condition** updated to record Leg C as a **partial instalment** (seed-once resumes for the preserved section only). **No new D-number**; B (a standalone filing rule) stays available later and promoting reverses nothing. *Reason it is a records item and not a chat note: the discriminator was already decided once and written only as a doc comment at `app.rs:290–291`, which is why this session had to re-discover it by grep.* **Lands in Leg C's records commit, never before the code exists** — a decision document must not describe behaviour the binary does not have.

*No "commit pushed" item. `Status: COMPLETED` in this header is the signal (Joe pushes; Claude never does).*

---

## §7 — APPEARANCE

**PROVISIONAL, shipped WITH the mechanics, judged by Joe live, discharged at M-RP-SKIN** (the M-RP-SHELF-FRAME pattern; Ms Design is retired, J-555).

The only new copy is the editor's replacement closing note (§4 Leg A). It is **functional and provisional** — it must state what is true after Leg C and nothing more. Final phrasing is Joe's at M-RP-SKIN.

No new tokens. No new rules unless §5 item 7 proves necessary. The Text Processing row inherits `plugin-list`'s existing row skin unchanged; its icon follows the M-RP6.2 D8 discipline — **a Material `d` path is never fabricated from memory** (Rule 5 / D-108). If no verified glyph exists in-repo, leave `icon` **unset** and let `plugin-list` fall back to its documented placeholder.

---

## §8 — CLOSE

Written **last**, after the legs. If the work moves after §8 is written, **re-read it rather than append** (N-143 — the rule is about sequence, not vigilance).

---

## §9 — VERIFICATION LEGS

CDP client **9222**. `.\cdp-debug.ps1 -App client -Mode eval`. Single-expression `JSON.stringify(...)` only; DOM reads in a **separate** eval after any mutation (N-099).

⚠️ **EVERY probe that gates a conclusion needs a POSITIVE CONTROL.** *"The bad thing is absent"* and *"nothing happened"* are the same string. Confirm a run produced its summary line before concluding anything from what is missing.

**Leg C:**

- **V-C1** — user pairs survive a relaunch. Set rules → relaunch → rules present on disk **and** in the store (`__XGEN_SUBS__`). *Positive control: confirm the config file was actually regenerated (other sections fresh), or you have proved only that nothing ran.*
- **V-C2** — **the J-438 leg.** Clear a rule → relaunch → **still cleared.** Under D-101 today it reappears, so this is a real before/after, not an assertion.
- **V-C3** — **the §1.4 leg. ⚠️ SPLIT ACROSS LEGS — only the FIRST part is runnable at Leg C** (caught on a v1.1 re-read: at Leg C there is no settings pane and no wired composer, so the original single-leg wording asked for a verification that could not run):
  - **V-C3a (Leg C, runnable now)** — write a deliberately invalid rules file → relaunch → **the invalid text SURVIVES the clean slate** (on disk, the whole point of Leg C) **and the store fails safe to empty**: `__XGEN_SUBS__` reads `rules.length === 0` with `source` holding the raw invalid text. *Positive control: the same probe on a VALID file must read a non-empty `rules`, or you have proved only that the handle returns nothing.*
  - **V-C3b (Leg A)** — Settings ▸ Text Processing shows that raw text **with its warning**.
  - **V-C3c (Leg B)** — the composer morphs nothing while such a file is loaded.
  ***Together these are the whole answer to the hazard persistence creates; if V-C3b does not show it at Leg A, the hazard is unmitigated and that is a Joe decision, not a defect to paper over.***
- **V-C4** — genuine first run (no config) → starter pack seeded, unchanged.

**Leg A:**

- **V-A1** — registry delta, **both readers enumerated** (§3.4). Expect plugin-list row ids only, **zero** widget ids.
- **V-A2** — the row renders in Settings ▸ Plugins with **no disable button** (W-13) and an **enabled** `[settings]` button.
- **V-A3** — `[settings]` drills into the editor; it seeds from the live rules; Apply persists; **churn returns to baseline EXACTLY** on close.
- **V-A4** — the sampler still mounts the editor live-only and unbroken (D-097; `onApply` path intact). ⚠️ **AND IT MUST STILL SAY SO:** with the seam unfilled, the sampler's note keeps the **session-only** wording — because `xgen-sampler/src/main.rs:113` still clean-slates. **The paired client read (V-A3) must show the OPPOSITE wording on the same component.** *One of these two passing alone proves nothing; the pair is the verification.*

**Leg B:**

- **V-B1** — type a seeded token in the composer → it morphs **on the painted DOM**, and `bind:value` agrees.
- **V-B2** — the **negative** leg: type the same token into the **editor's** textarea → **no morph.** The feedback loop is refused in fact, not only in the runbook.
- **V-B3** — send still works end-to-end; send-status still renders. Lock #12 held (the textarea is never disabled).
- **V-B4** — an empty rule set is a clean no-op: no caret churn, no synthetic `input`.

---

## §10 — FLOORS

Measured at J-560. **Predict, then decompose any miss. Never adjust.**

| | floor at open | expected to move |
|---|---|---|
| `cargo` | 1546 / 0 / 62 across 56 terminator lines | **Leg C only** (+3 tests). **IDENTICAL on A and B** — the direct zero-Rust proof |
| `svelte-check` | 0 errors / 34 warnings / 15 files | errors gate at **0** |
| `npm` | 142 | possible on A |
| `vite` | 200 CLIENT / 170 sampler | A and B. ⚠️ **N-149 — a count can move by REACHABILITY, not new files.** Ask *"what does this newly reach?"*, not only *"what does this add?"* |
| catalogue | 419 | unchanged (no `core`, no sampler) |
| client registry | **143 quiescent** | A. ⚠️ **N-148 — five axes:** quiescence · store · selection · saved-state count · **echo count.** 143 means empty store, no selection, nothing folded, zero saved states, **zero echoes** |

⚠️ **N-117** — the dev client HOLDS THE EXE: exit 101, zero terminator lines, 0/0/0 reads exactly like a clean run. Grep case-**sensitive**, or `FAILED|panicked` matches `"0 failed"`.
⚠️ `cargo test` **exceeds the MCP timeout** — run detached, poll the PID in separate short calls. A long `Start-Sleep` kills the shell and takes the run with it.
⚠️ **N-140** — a baseline read after an app restart alone is under-specified. **The dev server process must be gone**, proven by the **ports**, not the names.
⚠️ **N-144** — do not write a literal `<style>` tag inside a `//` comment. Four files remain one style block away: `color-picker`, `shelf-face`, `shelf`, `region-tile` (re-verified J-561: each has 1 comment-`<style>` and 0 real style blocks).
⚠️ **N-145** — the gate prints three error-shaped lines before every green run. **HARMLESS. DO NOT "FIX" IT.**

---

## §11 — OPEN / NOT SMUGGLED IN

- **`core`'s `Record<string, Component>` prop-less typing** — the same root cause M-RP-TYPECHECK fixed in `registry.ts`. ⚠️ **Re-grepped J-561: SIX live declaration sites, not the two previously named** — `message.svelte:56` · `message-stream.svelte:62` · `mounts.ts:52` · `region-node.svelte:44` · `region-shell.svelte:49` · `region-shell.svelte:60`. `send-status.svelte:36` already documents the problem in a comment. **Whether all six are defects is that milestone's Phase-0 question** — some sockets may legitimately mount prop-less. Joe wants this taken early. **Not here.**
- **M-RP-COMPOSER-SCROLL** — auto-scroll on the user's own send. Lock #10 unmet at Leg D2; Joe accepted the deviation. Small `core` leg.
- **Option E** — `[substitutions]` as its own store. Named, not taken (§2). Remains available without reversing Leg C.
- **M-RP-SKIN** — owes Leg D2/D3's provisional appearance (re-verified J-561: `send-status.svelte:47`, `composer-panel.svelte:47`, `skin.css:2994`), plus this milestone's editor note.
- **Three Svelte versions** — re-verified J-561 from the **lockfiles** (`package.json` says `^5` in all three — the J-497 range trap): client **5.55.5** · node **5.55.5** · sampler **5.56.4**. The sampler verifies the catalogue on a different compiler than ships.
- **D-101's full retirement** — not attempted. The other four config sections remain ephemeral by design.
