# XGen — Runbook: PROTO-STATUS.2 (self-set status reference impl)
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jul 2026  
> **Last updated**: 2026-07-05  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

Handoff for **Clair** (Code Claude). Reference impl of self-set status per PROTO-STATUS.0/.1. Chat Claude authored; Clair builds. Read PROTO-STATUS.0 + .1 first.

---

## Scope (locked)

1. **Module home** — new `xgen-core/src/status/` (identity-scoped, global; NOT under `space/`).
2. **Type** — `StatusRecord { emoji: Option<String>, text: Option<String>, updated_at, expires_at: Option<Timestamp> }` + validating constructor + `is_expired(now)`.
3. **Resolution** — register `state.status/<identity_xgid>` under existing `state.*` machinery + per-object `update_version`; owner-write guard (only the identity writes its own status). No new sync primitive.
4. **Tests** (D-078, grounded by symbol defs): caps, clear=delete, lazy-expiry, owner-write-only.

## Type surface

```rust
pub struct StatusRecord {
    pub emoji: Option<String>,       // exactly 1 grapheme cluster
    pub text: Option<String>,        // <= 128 bytes UTF-8, trimmed
    pub updated_at: Timestamp,
    pub expires_at: Option<Timestamp>, // now+60s .. now+30d
}
```

- Validating constructor rejects: multi-grapheme/empty emoji; text >128B or empty-after-trim; `expires_at` out of `[now+60s, now+30d]`.
- `is_expired(now) -> bool` = `expires_at.map_or(false, |e| e < now)`.
- Serde: XGID flavour = `IdentityXgid` for the key subject. `#[serde(transparent)]` conventions per Appendix I. `null` forbidden — omit absent optionals.

## Semantics

- **Clear = delete the object** (not all-null write). Absence = no status.
- **Lazy expiry** — readers treat `expires_at < now` as absent; no active sweep.
- **Owner-write** — write guard rejects non-owner writers.

## Test enumeration (grep symbol defs first, do not infer)

- emoji: 1 grapheme accepted; 2+ / empty rejected.
- text: 128B accepted; 129B rejected; whitespace-only → absent.
- expires_at: now+60s and now+30d accepted; now+59s / now+31d rejected.
- clear: delete removes object; subsequent read = absent.
- expiry: past `expires_at` read → absent (no sweep).
- owner-write: non-owner write rejected.

## Appendix I entry (insert atomically with impl, Part V runtime state)

### `StatusRecord`

Self-set identity status (PROTO-STATUS). State object at `state.status/<identity_xgid>`, owner-writable, public, global-scoped. Cleared by object deletion; expiry is lazy (readers treat expired as absent).

| Field | Wire key | Type | Req/Opt | Description |
|---|---|---|---|---|
| `emoji` | `emoji` | `Option<String>` / string | Opt | Exactly one Unicode grapheme cluster. |
| `text` | `text` | `Option<String>` / string | Opt | Description line; ≤128 bytes UTF-8, trimmed. |
| `updated_at` | `updated_at` | `Timestamp` / string | Req | RFC 3339 UTC. |
| `expires_at` | `expires_at` | `Option<Timestamp>` / string | Opt | Auto-clear time; range `now+60s .. now+30d`. |

## D-074 close (two commits)

1. **feat** — `status/` module + type + validation + resolution wiring + tests.
2. **docs** — Appendix I entry, JOURNAL J-NNN, ROADMAP (PROTO-STATUS.2 → DONE), CLAUDE.md PLAY, this runbook → COMPLETED.

## Roadmap

| milestone | scope |
|---|---|
| PROTO-STATUS.0 | object decision + shape ✅ |
| PROTO-STATUS.1 | spec (namespace, caps, clear, expiry) ✅ |
| **PROTO-STATUS.2** | this runbook — reference impl + Appendix I + tests |

Gates status-bearing `entity-avatar` variants (Track B, M-RP5.2).
