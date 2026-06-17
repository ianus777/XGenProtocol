# Bootstrap-Client — Design (D-071 arc, design phase)
> **Status**: COMPLETED  
> Version: 1.0  
> Date: May 2026  
> **Last updated**: 2026-05-31  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Arc position

Design phase of the **bootstrap-client** D-071 arc (third of the four post-M6
subsystem arcs to ship; A1 + A2 closed). Entry artifact is
`tasks/M6_BOOTSTRAP_CLIENT_AUDIT.md` (ACTIVE) — read it first; this doc does not
restate the evidence. Per D-069 the arc runs audit → design → impl. Implementation
runbook: `tasks/M6_BOOTSTRAP_CLIENT_IMPL.md` (ACTIVE).

## Verdict carried in

**GAP IDENTIFIED — HIGH (whole-subsystem).** All 5 verbs (`show`, `register`,
`deregister`, `set-info`, `set-tiers`) are absent-backed. The gap is wider than
"just the send-path": the send-path is a placeholder **and** there is no
`[bootstrap]` config section, no registrations store, no self-info store. The
server-side directory/reputation machinery is real but is the orthogonal
Bootstrap-Node-*server* role. Scope is **client-only** in this arc (A3-D1).

## Locked decisions

| ID | Decision | Status | Rationale |
|---|---|---|---|
| BC-D1 | Runtime bootstrap state lives in a **sibling JSON store** (not the TOML config). `[bootstrap]` config holds **operator seed intent only** (`#[serde(default)]`). | **LOCKED** (J-190) | `register`/`deregister`/`set-info`/`set-tiers` mutate at runtime via the admin pipe. TOML-as-truth would force the verbs to rewrite the operator's config file (clobbers comments/edits). The three prior D-071 arcs all landed on sibling JSON stores for this reason (`pending_queue` / `federation_policy` / `module_registry`). |
| BC-D2 | **Config seeds, store is truth.** Config provides defaults (bootstrap nodes to auto-register at startup; initial self-info/tiers). Store holds the runtime-mutable registrations record + self-info record. | **LOCKED** (J-190) | The runtime-mutable fields (which bootstrap nodes we are registered with; current advertised info/tiers) must survive verb writes without touching operator config. Mirrors the auth-module-registry config/store discipline. |
| BC-D3 | **register / keepalive / deregister are framed `BootstrapMessage` exchanges over the normal transport — NOT HTTP.** The only HTTP in bootstrap is the **directory-fetch** path (D-051), which is **out of A3 scope**. | **LOCKED** (J-190) | Code-trace: `http.rs`/`client.rs` are explicitly the directory endpoint (spec 3.14.2/3.14.4); D-051 states HTTP is the *only* place HTTP is used (the signed directory document). `BootstrapMessage` lives in `wire/types.rs` alongside every other framed control message. The send-path reuses the existing outbound-connect machinery (D-067 no-drift), not reqwest. Corrects the audit's "reqwest HTTP send-path for register/keepalive/deregister" framing. |
| A3-D1 | **Client-only** this arc — this Node registers *itself* with Bootstrap Nodes. Operating *as* a Bootstrap Node (the server role) is separately deferred. | LOCKED (design source §6.A3) | Carried from `docs/xgen_node_admin_ops_design.md`. |
| A3-D2 | **Best-effort re-advertise** — a local `set-info`/`set-tiers` update succeeds even if the re-advertise fan-out to registered Bootstrap Nodes fails. | LOCKED (design source §6.A3) | Honest per D-065/D-070 (sibling to force-eject Option B: best-effort after the local write). |
| — | **Scope boundary: directory-fetch is OUT of A3.** None of the 5 verbs fetch a directory; that is a discovery-time consumer (the HTTP/reqwest piece), a separate concern. | LOCKED (J-190) | Keeps the arc to register-self + manage-self-advertisement. |

Arc-local IDs (`BC-D#`) live in this doc per D-069; a call graduates to a global
`D-###` in DECISIONS.md only if it crosses an arc boundary (none currently does —
BC-D1/D2/D3 are arc-local).

## Wire shapes (evidence, from `wire/types.rs`)

- `Register { protocol_version, node_id, endpoint, region, capabilities: Vec<String>, timestamp, signature }` — signed by the registrant.
- `RegisterAck { protocol_version, node_id, directory_url, timestamp, signature }` — returns the `directory_url` the registrations store records per bootstrap node.
- `Keepalive { … node_id, timestamp, signature }` / `KeepaliveAck { … }` — TTL refresh.
- `Deregister { … node_id, timestamp, signature }` — explicit removal.

**Self-info = `endpoint` + `region` + `capabilities` + `auth_tiers_served`.** `set-info`
edits endpoint/region; **`set-tiers` edits `auth_tiers_served: Vec<u8>` (Tier 1–4), which
has NO field in the `Register`/`Keepalive` wire frames** — so it is **local-self-info
only** with no re-advertise (Checkpoint #1(d), Option A LOCKED, J-190). Propagating tiers
on the wire (Option B) is a wire-format change deferred to a separate protocol-design arc.

## Prime invariant

A Node with **no `[bootstrap]` config and an empty registrations store registers
with nobody and behaves byte-for-byte like today.** Sibling to "empty registry =
today" (A2) / "absent policy = permit-all" (A1-2b) / "require_approval=false =
today" (A1-2a). Trivially held early (nothing today sends bootstrap frames); a
mandatory explicit regression lands with the first wiring commit (D-065).

## Design agenda (targets — the runbook sequences these)

1. `[bootstrap]` config section on `NodeConfig` (D-035 convention, `#[serde(default)]`).
2. Local registrations store (per-bootstrap-node records) + self-info store.
3. Framed send-path: `register`/`keepalive`/`deregister` send + `*_ack` receive
   **and signature-verify**, reusing the existing outbound-connect path (BC-D3).
4. The 5 verbs in `admin_ops::*`.
5. Keepalive scheduler (mirror `reconnect.rs`, D-067) + best-effort re-advertise
   on `set-info`/`set-tiers` (A3-D2).

## Checkpoints (detail in the runbook)

- **#1 (before C1)** — pin data-layer type/store/path names against sibling
  precedents; `register` input shape (lean: bootstrap-node url + its pubkey or
  directory_url, for `register_ack` verification); the `set-tiers` field-mapping
  (capabilities-encoding vs local-only); one-vs-two store files.
- **#2 (at C2)** — the framed send-path mechanics (reuse federation's connect path
  vs a lightweight bootstrap client), `register_ack`/`keepalive_ack` signature
  verification, and how keepalive scheduling composes with `reconnect.rs`.

## Cross-refs

- Audit (entry artifact): `tasks/M6_BOOTSTRAP_CLIENT_AUDIT.md`.
- `docs/xgen_node_admin_ops_design.md` §6.A3 (A3-D1/A3-D2) + Appendix K.2.3.
- Spec §3.14.3 (register) / §3.14.4 (directory fetch — out of scope) / §3.14.7 (keepalive/deregister).
- `tasks/M6_BACKING_AUDIT.md` A3 row. D-051 / D-067 / D-069 / D-065 / D-070.
- Sibling design docs (worked examples): `tasks/M6_AUTH_MODULE_REGISTRY_DESIGN.md`,
  `tasks/M6_FEDERATION_POLICY_DESIGN.md`.

---

*Design LOCKED (BC-D1/D2/D3 + scope boundary). Implementation sequenced in
`tasks/M6_BOOTSTRAP_CLIENT_IMPL.md`. Checkpoint #1 fires before Clair Commit 1.*
