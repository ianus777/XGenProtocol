# M10.2 — Tier-1 Reference Auth Module (`xgen-auth-module`) — Design
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-13  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## 0. Status & lineage

Second M10 sub-arc; the first with a real new binary. Decisions M10.2-D1..D5 **Joe-LOCKED (J-363)** off the
Phase-0 audit (`tasks/M10_2_REFERENCE_BINARY_AUDIT.md` v1.0, `629b30c`, grounded vs `main` @`a86db48`). The two
framing calls (M10-A-02 = registry→policy, M10-A-06 = operator-CRUD) were locked at J-362; this design locks the
four shape questions the audit surfaced. Production arc (`xgen-core`/`xgen-node` + a new binary). **Next-active
= Clair: author the runbook** `tasks/M10_2_REFERENCE_BINARY_IMPL.md`. No code until the runbook lands.

## 1. Scope

**In:** the `xgen-auth-module` binary (own keypair; offline-signs Tier-1 `TrustAssertion`s as itself); the
trust-source rewire (gate live-reads `AuthModuleRegistry`); the config→registry bootstrap-seed; the end-to-end
witness (module issues → identity registers with the assertion → node validates against the registry-trusted
issuer → accept; revoke → reject).

**Out (explicit, recorded boundaries):** `accepted_tiers` *enforcement* → **M10.3** (D4); the parameterized
T2–T4 mock + dormant-tier paths → M10.3; MP-F13 → M10.4; a module-presented registration manifest/handshake →
flagged future (M10-A-06); erasure *enforcement* (the `module_policy.erasability` consumer stays D3-gated).

## 2. Locked decisions

**M10.2-D1 — issuance = offline-signed token.** The binary signs a Tier-1 `TrustAssertion` (issuer field =
`AuthModuleXgid::from_pubkey(pk).to_string()`, signed with the module's key — the exact shape the synthetic test
issuer produces today). The identity attaches it to its register message; the node verifies the signature
**offline** against the trusted issuer pubkey. **No live endpoint** in M10.2 (`endpoint_url` is only the
`auth-module test` connectivity-probe target — the node never calls the module to validate). Challenge/response
issuance = flagged future. Spec-aligned: §3.8.7 Auth-Module registration is out-of-band/operator-mediated.

**M10.2-D2 — trust source = `AuthModuleRegistry`, live-read; registry to `run_node` top-level.** The gate's
trusted-issuer set flips from the startup config snapshot (`app.rs:745–749`) to a **live read of the registry**
at validation time (the sole production caller is `validate_assertion` via `accept_registration`,
`registration.rs:486`, reached from `handle_identity_msg`, `app.rs:2842` — a single seam). The `AuthModuleRegistry`
`Arc<Mutex<>>` moves to **`run_node` top-level** so the gate and the CRUD verbs share one live instance — the
**first runtime consumer of the registry, which structurally closes AMR-D1**. A `revoke` therefore takes effect
immediately (no restart). `register`/`revoke` become enforcement-bearing.

**M10.2-D3 — config `trusted_auth_modules` bootstrap-seeds the registry: add-only + idempotent + revoke-wins.**
At startup, each config issuer is **inserted only if absent** (add-only), the seed **re-runs safely every boot**
(idempotent), and it **never un-revokes** an issuer a CRUD `revoke` has marked dead (operator revoke is
authoritative over the config seed). **Prime invariant:** empty config + empty registry = today's behaviour,
byte-for-byte (no trusted issuers → the baseline/`local_mode` path is untouched, Fork 1). Config is neither
deprecated nor a second live source — it seeds, the registry rules.

**M10.2-D4 — `accepted_tiers` enforcement deferred to M10.3.** M10.2 ships trust on/off + live revoke (all a
Tier-1-only world needs). The per-issuer tier check is a `validate_assertion` shape change with **nothing to
enforce until T2–T4 exist** — so it lands in M10.3 alongside the parameterized mock that issues higher tiers and
can witness it. This **narrows the literal M10-A-02 wording** ("register/revoke/accepted_tiers") on *timing*
only; the (b) registry→policy *direction* is untouched. **M10.3 owns making `accepted_tiers` enforcement-bearing**
(recorded, not dropped).

**M10.2-D5 — the binary = pure offline signer (M10-A-06).** `xgen-auth-module` reuses the existing keypair
surface (`xgen-core/src/identity/keypair.rs`: generate/save/load, encrypt-at-rest), holds its own keypair, signs
Tier-1 assertions as itself, and **populates the M10.1 descriptor** on issued assertions (`module_kind:
reference` + a `module_policy` with `erasability`). The operator registers it via the existing 5 CRUD verbs
(trust stays operator-controlled). No module-presented manifest/handshake.

## 3. Impl surface (audit-grounded; Clair confirms file:line at runbook)

- Trust-source flip + live-read: `app.rs:745–749` (the snapshot to replace); the gate read inside
  `accept_registration` (`registration.rs:486`); the node path `handle_identity_msg` (`app.rs:2842`).
- Registry relocation: from the pipe-block load (`app.rs:~1164`) + the read-only storage-floor load (`app.rs:~528`)
  to a single `run_node` top-level `Arc<Mutex<AuthModuleRegistry>>` shared by gate + CRUD verbs.
- Config seed: where `trusted_auth_modules` is read at startup → an add-only/idempotent/revoke-wins seed into the
  registry.
- The binary crate: new workspace member `xgen-auth-module`; reuses `keypair.rs` + the `TrustAssertion` signer
  (`SignedPrimitive` is a **concept, not a trait** — A5; canonical-bytes signing) + `AuthModuleXgid` (D-083).
- Descriptor population: `module_kind`/`module_policy` accessors on `TrustClaims` (shipped M10.1, `01ea770`).

## 4. Proof obligations (RED-on-revert; Clair builds in the runbook)

1. **End-to-end accept** — `xgen-auth-module` issues a Tier-1 assertion → an identity registers with it → the
   node validates against the **registry-trusted** issuer → accepted. RED on revert (un-trust the issuer → reject).
   Extends the in-process `non_local_registration_with_valid_assertion_accepted` shape.
2. **Live revoke** — a trusted issuer's module is `revoke`d → a subsequent registration with its assertion is
   **rejected without restart** (proves D2 live-read, not a snapshot). RED on revert.
3. **Config-seed** — a config `trusted_auth_modules` issuer is honoured after a fresh boot (seed works); re-boot
   is idempotent; a CRUD-revoked issuer stays revoked across a re-seed (revoke-wins). RED on revert.
4. **Empty-baseline invariant** — empty config + empty registry ⇒ behaviour byte-for-byte as today (no
   regression to the `local_mode`/baseline path).

## 5. Design-close details (Clair confirms at runbook; Joe-call only if non-obvious)

- The exact live-read shape at the gate (lock the `Arc<Mutex<>>` per-validation vs a cheap snapshot-per-message)
  — must honour D2 (revoke is live) without a contention footgun.
- Seed reconciliation precedence when config and a CRUD record name the same issuer (D3 = add-only/revoke-wins;
  confirm the exact merge).
- Binary CLI surface (`issue`/`keygen`?) — minimal for the witness; not a product CLI in M10.2.

## 6. Close deliverables

- Appendix F entries for any operator-visible surface (the binary's CLI; the registry-as-trust-source change to
  `auth-module register`/`revoke` semantics).
- ch2/ch4 reconcile if the "demonstrator over baseline" framing needs a doc note; §3.8.7 already aligns.
- Findings flips at close: **M10-A-02 RESOLVED** (registry→policy wired) + **M10-A-06 RESOLVED** (operator-CRUD
  binary shipped); M10.2-A1 carried → M10.3 (accepted_tiers).
- DECISIONS: candidates only, arc-local (D-069) — none pre-decided. Matrix as applicable.

## 7. Next-active

Clair: author `tasks/M10_2_REFERENCE_BINARY_IMPL.md` (the binary crate + the trust-source rewire + registry
relocation + config-seed + the four witnesses), confirming the §3 groundings + §5 details to file:line →
implement → Chat doc-bridge → close. The one design-close detail to surface if non-obvious: the live-read shape
(§5).

**CLOSED J-364** — shipped runbook `e824844` + C1 `113504f` / C2 `b87f6e3` / C3 `6a3f972` + Cargo.lock `8a6024c`; D1–D5 as locked; the §5 detail resolved by Clair (live-read = NodeRuntime field + lock-per-validation, no Joe-fork); 1382/0 (+8), clippy clean, all four witnesses RED-on-revert; AMR-D1 structurally closed; M10-A-02 + M10-A-06 RESOLVED, M10.2-A1 carried → M10.3. D-065 close note: config-removal no longer un-trusts a seeded issuer ("registry rules") → Appendix F §F.10.
