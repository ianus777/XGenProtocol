# XGen UI — Notes
> **Status**: ACTIVE  
> Version: 0.1  
> Date: May 2026  
> **Last updated**: 2026-05-15  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Light chronological notes on UI design and adjacent topics. Lower ceremony than `xgen-ui-design-brainstorm.md` (deprecated, kept as inspiration): each entry dated, free-form, no fixed problem/direction/open-questions scaffolding. Notes graduate into Ch6, DECISIONS.md, or a proper instruction file when they mature. Resolved items are not deleted — they stay with a forward pointer (e.g. `→ D-NNN`) so the record remains readable.

---

## 2026-05-15

### N-001 — CLI-first binaries with UI envelopes

Review whether `xgenclient.exe` and `xgennode.exe` can be used as pure CLI binaries in addition to their normal UI-embedded mode. Today the CLI surface already exists (`whoami`, `status`, `--batch`, `--service` for headless Node) and library-first architecture is mandated (CLAUDE.md, D-037). Open question: is one binary serving both modes the right shape, or is a derivative CLI-only build preferable?

Analogy: FFmpeg has a stable CLI core with various UI front-ends built around it. Initial thought was that XGen UI extensions would be derivatives of the UI-embedded `.exe`. Refined thought: the same `.exe` may already be usable in pure CLI mode, given the library-first design — worth reviewing rather than assumed.

Likely implications if pursued:
- Explicit mode selection (`--cli` / `--headless` flag, or auto-detect from stdin/stdout redirection) so a CLI invocation in a script does not flash a Tauri window
- Clean exit codes and stdout/stderr discipline maintained across both modes
- Documentation: which subcommands are CLI-safe vs which assume an interactive session

Not urgent. For record now; review when UI work resumes.

### N-002 — Adversarial / misuse simulation suite (post-UI)

After the UI is built, build a simulation testing suite that goes beyond happy-path protocol correctness. Scope: node↔client interaction under irregular and hostile usage. Categories include:

- Privilege escalation attempts — regular user attempting actions reserved for admins or owners; spoofed sender fields; replay attacks on permission-changing events
- Out-of-context commands — commands valid in one state issued in another (e.g. send to a Room before joining; promote a DM that is already promoted)
- Malformed and adversarial inputs — overlong fields, control characters, unicode edge cases, malformed JSON, oversized batches, recursive structures
- Weird combinations — concurrent state-changing events that should not coexist; rapid join/leave/ban cycles; federation handshake interleaved with admin commands

Goal is hardening, not feature coverage. Separate from `stress-complete` (which is correctness under load) and from `smoke-ph2` (which is happy-path verification). Output is a list of error situations that the protocol handles cleanly and a list of failures that need fixing.

Depends on: UI track far enough along that a UI client can be driven adversarially, and a stable Auth Tier 1 implementation for the privilege model.

### N-003 — AI users in the XGen network — ACTIVE DISCUSSION

How does an AI agent participate as an identity in the XGen network? Several alternatives surfaced in earlier discussion (Cowork session, partial record):

- **Flag on a regular user** — AI is an attribute on top of an ordinary Identity. AI signs as itself with its own keypair; the `is_ai` (or similar) flag is visible in member lists and message decorators.
- **Dedicated Auth Tier** — AI gets its own tier on the existing Auth Tier axis (1–4), separate from human identity tiers. Implies different verification requirements and possibly different protocol privileges.
- **(further alternatives raised but not preserved verbatim — to be reconstructed during active discussion)**

This is the point being lifted to active discussion today. It has implementation impact (identity record shape, Trust Assertion content, possibly EventType additions for AI-specific protocol exchanges) and UI implications (visual distinction in member lists, message stream, avatar, status bar). Outcome of the discussion will graduate into a DECISIONS.md entry once a direction is chosen.

Cross-reference to existing project material:
- Ch1 — Human and Agent Operation (philosophical grounding for human + AI participation)
- D-036 — Module identity modes (`system` vs `user`) — relevant but distinct (modules are not the same as AI users)
- D-037 (Tier 1 identity precision: persistent accountable identity, not civil identity) — the framing of "what an Identity is" matters for the AI-user question

---

## How to use this file

- New notes go under the current date heading, indexed `N-NNN` continuing the numbering.
- A note that crystallises into a decision is marked with a forward pointer (`→ D-NNN`) and left in place — do not delete.
- A note that is superseded or no longer relevant is marked `SUPERSEDED` (or `DROPPED`) with one-line reason, and left in place.
- The file is append-only in spirit. The full history is the record.
