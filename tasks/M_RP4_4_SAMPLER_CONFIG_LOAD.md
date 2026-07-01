# M-RP4.4 — sampler real config-load path + clean-slate-on-start discipline (first instance: substitutions)

> **Status**: PENDING  
> Version: 0.1  
> Date: Jul 2026  
> **Last updated**: 2026-07-01  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. What this is

Config-backed components should run **in the sampler** through the **real**
`generate → file → load → command → setRules` chain — not a hand-synced frontend literal — so a
component drops into the rewritten client/node UIs with **zero reprogramming**. The first instance
is **substitutions** (M-RP4.2), which closes the two-hand-synced-seeds seam N-057 flagged. This arc
**sets the precedent** for every future config-backed component, so the shape is locked carefully.

Paired with it: a **clean-slate-on-start** discipline — every binary wipes any config it finds at
launch and regenerates from seed — appropriate while the settings logic is still in development and
the config file is deprecatable.

**Joe-locked decisions (this arc's design walk, J-441 session):**

1. **Real path where reasonable/cheap; direct-inject fallback where impractical.** The stable
   contract is the **component interface** (`substitutions.setRules(string)`), not the plumbing.
   **No client clone.**
2. **Clean-slate-on-start — EVERY binary, this phase.** `xgen-client`, `xgen-node`, and the
   `xgen-sampler` host each **wipe any found config at launch, before reading it, then regenerate
   from seed**. Config is ephemeral/deprecatable this phase. This **suspends J-438 seed-once** for
   the phase (see §1.1 — load-bearing). Retirement condition: the real client/node UIs are rewritten
   and settings become **persistent**.
3. **Sampler config = subset snippets.** The sampler generates only the sections it needs
   (e.g. `[substitutions]`), NOT the whole client/node config — the needed slice of what the real
   `.exe`s generate.
4. **"Real path" under D-098 = contract-shape fidelity, not code reuse.** The sampler host can't
   depend on `xgen-client`, so it reimplements a **minimal** read/write/delete of its subset config
   + a `get_substitutions` command. Same chain shape as the client; different (minimal) impl.

**Records deliverable:** **D-101** (written alongside this runbook — the clean-slate-on-start
discipline + the seed-once suspension + the exit condition) and, at close, **N-058** (the sampler
real config-load path). D-101 is referenced at the delete site in code.

---

## 1. Why this shape (for the N/D entry)

The component contract (`setRules`) is already source-agnostic (N-057); what this arc adds is
proving the **plumbing around it** in the workbench — the whole chain, not just the component. Once
the sampler runs the real generate→load path, a config-backed component needs no rewiring when it
lands in the real UI: the sampler exercised the exact contract the real shell provides.

**Clean-slate-on-start** is the enabling discipline: while settings are in flux, treating the config
as derived-from-seed (regenerated each launch) is crash-safe + self-healing — no binary inherits a
stale or another binary's file, and a corrupt config never wedges a launch.

### 1.1 Interaction with J-438 seed-once (MUST be findable — D-101 carries it)

J-438 built `cmd_init` to seed the client's starter pack **once at config-birth, never resurrected
after the user clears pairs**. Clean-slate-on-start **suspends** that guarantee this phase: because
the config is deleted + regenerated from seed every launch, cleared pairs **do** reappear. Intended
now (no persistent user-owned settings surface exists yet, so nothing durable is lost). Seed-once
resumes at the exit condition. The delete site in code AND D-101 both carry the *why* + the
*until-when* — a future session finding a vanishing/resurrecting config must reach both.

---

## 2. Phase-0 references (grounded; read + audit before authoring — D-071)

**xgen-client (Clair):**
- `xgen-client/src/app.rs` — `cmd_init` (~2109; config-birth branch ~2193 seeds
  `DEFAULT_SUBSTITUTIONS_SEED`); `load_substitutions_section` (~246); the config path
  `data_dir.join("xgen-client_config.toml")`.
- `xgen-client/src/desktop.rs` — `ConfigPath` managed state (~44/256); `get_substitutions` (~101);
  `run_startup` config read (~154/213). The delete-on-start insertion goes **before** the read.

**xgen-node (Clair):**
- The node's first-run generator (`default_config_toml()` / `maybe_write_default_config`, the J-080
  path) + its config read on start. The node has **no** substitutions consumer today, so its slice
  of this arc is **delete-on-start only** (no `get_substitutions`).

**xgen-sampler host (Clair — Rust; minimal host, D-098):**
- The sampler host crate (`tauri`+`tauri-build` only, no protocol deps). Gains: a subset-config
  generator (writes `[substitutions] rules` from a seed const), a minimal loader (reads that one
  string), a `get_substitutions` command, and delete-on-start.

**Sampler frontend + client boot precedent (Chat):**
- `ui/sampler/src/app_sampler.svelte` — currently seeds a literal (J-440); switches to
  `invoke('get_substitutions') → substitutions.setRules(...)` on mount.
- `ui/client/src/app_client.svelte` — the boot-hydration precedent (J-437) to mirror.

**Chat/Clair split:** Clair owns all Rust (client + node delete-on-start; sampler host subset-gen +
loader + command + delete-on-start) + Rust tests. Chat owns the sampler frontend invoke swap + the
sampler CDP verification.

---

## 3. Clean-slate-on-start (all three binaries)

At each binary's config read on start, insert **before the read**: if the config file exists, delete
it; then run the existing generator to regenerate from seed; then read. A comment at each delete site
points to **D-101** (the why + the exit condition).

- **client:** in `run_startup`, before the config read. Reuse `cmd_init`'s config-birth generator.
- **node:** analogous, before its config read. Reuse `default_config_toml()`/`maybe_write_default_config`.
- **sampler host:** new — wipe + write the subset config (§4) from seed on startup.

Rust unit tests (Clair): a pre-existing config on disk is wiped + regenerated to the seed on the
startup path (client + node). Sampler host: the subset config is (re)written on start.

---

## 4. Sampler subset config + minimal loader + command (Clair — Rust)

- **Subset config generator:** writes a minimal `xgen-client_config.toml`-shaped file containing
  ONLY `[substitutions] rules = "<seed>"` (the needed slice, decision 3) into the sampler's own
  instance data-dir.
- **Seed const:** a `DEFAULT_SUBSTITUTIONS_SEED` in the sampler host, **hand-synced** with the
  client's (the third copy of the seam — documented, NOT resolved here; a shared-const crate is
  explicitly out of scope, decision noted in N-058).
- **Minimal loader:** read the file → parse the one `[substitutions] rules` string → return it
  (contract-shape parity with `load_substitutions_section`, not code reuse — D-098).
- **`get_substitutions` command:** returns the loaded string (mirrors the client command's shape).

## 4a. Sampler frontend (Chat)

`app_sampler.svelte`: drop the literal seed; on `onMount`, `substitutions.setRules(await
invoke('get_substitutions'))` — mirroring `app_client.svelte` (J-437). Matrix stays **56** (no new
cell; the cell now sources from the real load path, not a literal).

---

## 5. CDP verification (Chat self-drives — sampler)

Launch detached (`run-sampler.ps1 -Debug`); poll 5175/9422; fresh launch; split dispatch from read
by a tick (J-433); teardown to 0 orphans. Quote actual output (Rule 2).

1. **Config regenerated on start:** the sampler subset config file exists after launch and contains
   the seed `[substitutions] rules` string.
2. **Command feeds the store:** `get_substitutions` returns the seed string; the store is hydrated
   from it (NOT a frontend literal) — read the store → the parsed config.
3. **Live morph from the loaded rules:** type `--> :) -- <3` into `textarea#processed`, dispatch
   input, tick, read → `→ 🙂 ‒ ❤️` in DOM AND registry (the rules came through the real load path).
4. **Delete-on-start proven:** pre-seed a *different* `[substitutions]` line into the config, launch,
   confirm it was wiped + regenerated to the seed (not inherited).
5. **Count unchanged:** `ids().length === 56`.

(The client/node delete-on-start is verified by Rust unit tests + Joe's live check; not
sampler-CDP'able.)

---

## 6. Records (D-074; after verification)

- `DECISIONS.md` — **D-101** (written now; the clean-slate-on-start discipline + seed-once
  suspension + exit condition). `Last updated` bump.
- `ui/docs/xgen-ui-notes.md` — **N-058** at close (the sampler real config-load path;
  contract-shape-not-code-reuse; subset config; the N-057 seam closed; the seed hand-sync still
  open, shared-const crate out of scope). Version bump.
- `docs/ROADMAP.md` — M-RP4.4 ✅; version bump; CLAUDE same-commit.
- `CLAUDE.md` — PLAY → M-RP4.4; next-active → M-RP4.3.
- `JOURNAL.md` — **J-NNN** (newest-first; real CDP + Rust output).
- `ui/docs/xgen-ui-components.md` — `textarea` source-note: sampler now loads via the real path
  (iff edited).
- `tasks/M_RP4_4_SAMPLER_CONFIG_LOAD.md` — Status → COMPLETED.

`.md` header rule: `> **Last updated**:` carries ONLY the date.

---

## 7. Commit plan (UI pattern; Joe pushes)

Likely staged (two agents):
- **Clair commit (feat):** client + node delete-on-start; sampler host subset-gen + minimal loader +
  `get_substitutions` + delete-on-start; Rust tests.
- **Chat commit (feat):** `app_sampler.svelte` invoke swap. CDP-verified in the sampler.
- **Records commit (docs):** D-101 (if not already committed with this runbook) + N-058 + ROADMAP +
  CLAUDE + JOURNAL + task.

Exact `git add` lists authored at close per the standing PowerShell discipline (one `git add` per
file; multiple `-m` flags; `$ProgressPreference='SilentlyContinue'`; Joe pushes).

---

## 8. Definition of Done

- [ ] D-101 written (clean-slate-on-start + seed-once suspension + exit condition). *(Authored with this runbook.)*
- [ ] client: delete-on-start before config read; comment → D-101; Rust test (pre-existing config wiped + regenerated).
- [ ] node: delete-on-start before config read; comment → D-101; Rust test.
- [ ] sampler host: subset-config generator (`[substitutions]` only) + minimal loader + `get_substitutions` + delete-on-start; Rust test.
- [ ] sampler frontend: `app_sampler.svelte` drops the literal; hydrates via `invoke('get_substitutions')`; matrix 56.
- [ ] CDP §5 run in the sampler — actual output captured (regenerated config, store-fed-from-command, live morph, delete-on-start, count 56).
- [ ] N-058 written; ROADMAP M-RP4.4 ✅ + CLAUDE same-commit; JOURNAL J-NNN (real output).
- [ ] Seed hand-sync seam documented as still-open (shared-const crate out of scope); the N-057 literal seam closed.
- [ ] Task Status → COMPLETED.

(`Status: COMPLETED` is the real signal — no "commit pushed" checklist item.)

---

## 9. Relationship / preview

- **M-RP4.3** (in-app TOML editor + write-back) benefits from the ephemeral-config discipline
  established here — the editor is authored knowing how config is treated this phase.
- **M-RP4.1** (kind-3 number-clamp) follows.
- **Exit** (out of scope, future): when the real client/node UIs are rewritten with persistent
  settings, delete-on-start is removed from all three binaries and J-438 seed-once resumes as real
  client behaviour (D-101 exit condition).
