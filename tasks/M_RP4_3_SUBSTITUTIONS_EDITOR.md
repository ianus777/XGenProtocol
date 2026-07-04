# M-RP4.3 — `substitutions-editor` (the first `widget`)
> **Status**: ACTIVE  
> Version: 0.1  
> Date: Jul 2026  
> **Last updated**: 2026-07-04  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Runbook for the **first widget** — the in-app `[substitutions]` editor. Dogfoods + firms the widget-tier spec (`ui/docs/xgen-widget-tier.md` v1.0, D-102, first-instance-provisional). Phase-B; session-only write-back under D-101 (the honest phase-limit, W-8). Design Joe-locked 2026-07-04 (Phase-0).

## Locked design

1. **Shape** — one **textarea** holding the raw `" | "` string (D-100 1:1-with-TOML). No `stringifyRules` needed. Per-pair rows = additive follow-up (defers N-057's per-pair intent, logged).
2. **Apply model** — explicit **Apply** + **Revert**, gated on `dirty && valid`. No live-apply.
3. **Seed** — Step-A additive `substitutions.source` (raw string) on the store; editor draft inits from it. Store-mediated single source of truth (no extra `invoke`, no stringify).
4. **Persist seam** — `invoke('set_substitutions', { rules: string })` writes `[substitutions] rules` (symmetric with `get_substitutions`). Clair's Rust half.
5. **Getter** — aggregate `{ dirty, valid, count }` (task-state, never payload; W-4/1b).
6. **Home** — `ui/common/lib/components/widgets/substitutions-editor.svelte`; root `<div class="substitutions-editor" use:envelope data-tier="widget">`; first occupant of `widgets/`. Phase-B.
7. **Sampler** — 5th **WIDGET** tab (N-053 mounted-not-`{#if}`). Reuse the `textarea#processed` processor-host cell as the **live cross-widget morph proof** (edit → Apply → morph changes, no file I/O).

## Constraint conformance (W-1…W-11 self-check at close)

W-1 composes-down (textarea + button, core) · W-2 owns draft/dirty/valid · W-3 I/O via store + one `invoke` only · W-4 one aggregate getter, task-state not payload · W-5 clean mount/unmount · W-6 zero component `<style>` · W-7 `ui/common`, Phase-B · W-8 session-only write-back surfaced in-UI · W-9 Svelte component + `data-tier` + `widgets/`, static import · W-10 plugin contract honoured · W-11 no dd-slot this instance (none needed) — record as N/A, not a gap.

## Steps — Chat half (pure/presentational layer)

- **A. Store `source` additive.** `store.svelte.ts`: stash the raw text on `setRules` (success *and* reject), expose `get source()`. Additive, existing behaviour unchanged. Own commit.
- **B. Widget.** `substitutions-editor.svelte`: `draft` local `$state` (init `substitutions.source`); derived `parsed = parseRules(draft)`, `valid`/`errorMsg` via a guarded `assertSafeRules({trusted:false})`, `count = parsed.length`, `dirty = draft !== substitutions.source`. **Apply** (`dirty && valid`) → `substitutions.setRules(draft)` + `invoke('set_substitutions',{rules:draft})` wrapped `try/catch` (sampler no-op). **Revert** → `draft = substitutions.source`. Aggregate getter `{dirty,valid,count}` via `envelope`. Inline warning renders `errorMsg` when `!valid`. A one-line session-only note (W-8).
- **C. Skin.** `.substitutions-editor` in `skin.css` (L2 only; zero component `<style>`).
- **D. Sampler.** `app_sampler.svelte`: 5th tab **WIDGET** (mounted, CSS-hidden inactive — never `{#if}`); mount `substitutions-editor#demo`.
- **E. Pure-layer CDP verify** (sampler 9422, both accents): registry entry present; drive draft→`dirty:true`; a bad/looping pair (`a aa`) → `valid:false` + inline warning + Apply disabled; a good set → Apply → `__XGEN_SUBS__` rules update + the `textarea#processed` morph changes live; Revert → clean; skin in cascade; 0 orphans. Real output (Rule 2).

## Steps — Clair half (effect layer) + real-shell verify

- **F. Rust.** `set_substitutions(rules: String)` Tauri command → write `[substitutions] rules` to `xgen-client_config.toml` (client). Symmetric with `get_substitutions`.
- **G. Effect-layer real-shell verify** (client 9222, Rule 2): real Apply → command round-trip → file written; in-session the rules are live; **relaunch → clean-slate (D-101) wipes to seed** — session-only demonstrated honestly.

## Close (D-074)

Feat commits (A → B/C/D → F) then docs commit: N-068 (ui-notes) + components registry (first `widgets/` occupant + widget catalogued) + ROADMAP (RP node, M-RP4.3 ✅) + CLAUDE PLAY (next-active → M-RP4.1) + JOURNAL + this runbook → COMPLETED. Firms the spec (amend a W-clause if the instance surfaces one — D-065).

## Definition of Done

- [ ] A store `source` additive; existing tests/behaviour unchanged
- [ ] B widget built; getter `{dirty,valid,count}`; Apply/Revert; inline warning; session-only note
- [ ] C `.substitutions-editor` skin, zero component `<style>`
- [ ] D sampler WIDGET tab (mounted-not-`{#if}`); `substitutions-editor#demo` mounted
- [ ] E pure-layer CDP verified in sampler, both accents, real output, 0 orphans
- [ ] F Rust `set_substitutions` + client write path
- [ ] G effect-layer real-shell CDP verified; session-only-under-D-101 demonstrated
- [ ] W-1…W-11 conformance self-check recorded (W-11 = N/A)
- [ ] Records atomic per D-074; spec firmed/amended note
