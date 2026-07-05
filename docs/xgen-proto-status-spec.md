# XGen — PROTO-STATUS.1: Self-Set Status Spec
> **Status**: ACTIVE  
> Version: 1.0  
> Date: Jul 2026  
> **Last updated**: 2026-07-05  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Track A spec for self-set status. Builds on PROTO-STATUS.0. Design-only.

---

## 1. Key namespace

`state.status/<identity_xgid>` — one object per identity, owner-writable only (only the identity may write its own status).

## 2. Object shape

```
state.status {
  emoji?:      string      // exactly one grapheme cluster
  text?:       string      // <= 128 bytes UTF-8, trimmed
  updated_at:  Timestamp
  expires_at?: Timestamp    // now+60s .. now+30d
}
```

## 3. Caps (validation)

- `emoji` — exactly one Unicode grapheme cluster; reject multi-char or empty-string.
- `text` — max 128 bytes UTF-8, trimmed; empty after trim → treat as absent.
- `expires_at` — min `now + 60s`, max `now + 30d`; out-of-range rejected.

## 4. Clear semantics

Clearing status = **delete the object**, not write all-null. Absence = no status. No tombstone-vs-empty ambiguity.

## 5. Expiry

Lazy: readers treat `expires_at < now` as absent. No active sweep; writers/readers enforce at access time.

## 6. Propagation

Existing `state.*` resolution + per-object `update_version`. Owner-authored, self-scoped. Public visibility.

## 7. Deferred

Per-space status, visibility gating, presence (online/typing). Named for later, not in this spec.

## 8. Open (PROTO-STATUS.2 handoff)

- Appendix I entry + spec section number.
- Reference-impl surface: `state.status` type, resolution wiring, validation, tests.

## 9. Roadmap

| milestone | scope |
|---|---|
| PROTO-STATUS.0 | object decision + shape + propagation ✅ |
| **PROTO-STATUS.1** | this doc — namespace, caps, clear, expiry ✅ |
| PROTO-STATUS.2 | reference impl + Appendix I + tests |

Gates status-bearing `entity-avatar` variants (Track B, M-RP5.2).

---

*Companion to `xgen-status-gap-phase0.md`, `xgen-proto-status-phase0.md`.*
