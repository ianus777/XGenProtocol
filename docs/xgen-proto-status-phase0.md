# XGen — PROTO-STATUS.0: Self-Set Status State Object
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-05  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Track A Phase-0 (D-071) for the self-set status gap. Design-only; no code until locked and a runbook is authored.

---

## 1. Decision — dedicated state object (A2)

Self-set status is **not** a field on `IdentityRecord`. It is a dedicated `state.status` object with its own version stream.

Rationale: status changes often and is low-stakes. Folding it into `IdentityRecord` pollutes the identity version history (3.6.8) and forces heavyweight identity federation for a mood line. A separate object keeps identity records stable.

## 2. Shape (locked)

```
state.status {
  emoji?:      string   // one grapheme, capped
  text?:       string   // description line, length-capped (128)
  updated_at:  Timestamp
  expires_at?: Timestamp // optional auto-clear
}
```

- **scope** — global (identity-wide). Per-space status deferred.
- **visibility** — public. Status is inherently broadcast; gating deferred.
- **presence** (online/typing) — excluded; ephemeral, separate arc.

## 3. Propagation (locked)

Reuse the existing `state.*` resolution machinery + per-object `update_version`. No new sync primitive — status is just another versioned state object. Owner-authored, self-scoped (only the identity may write its own status).

## 4. Open (next locks)

- Exact `state.*` key/namespace (e.g. `state.status.<identity>`).
- Caps: emoji grapheme rule, `text` max length, `expires_at` bounds.
- Empty/clear semantics (all-null vs delete object).
- Appendix I entry + spec section number.

## 5. Roadmap

| milestone | scope |
|---|---|
| **PROTO-STATUS.0** | this doc — object decision + shape + propagation locked |
| PROTO-STATUS.1 | full spec: key namespace, caps, clear semantics, Appendix I entry |
| PROTO-STATUS.2 | reference impl (`state.status` type + resolution wiring + tests) |

Gates the status-bearing `entity-avatar` variants (Track B, M-RP5.2).

---

*Companion to `xgen-status-gap-phase0.md`. Track B (`entity-avatar` core) proceeds in parallel, unblocked.*
