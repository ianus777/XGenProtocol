# Arc H — End-to-End Encryption (PG-05) — Phase-0 Design
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-04  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §0 — Scope & status

Design for **Arc H / PG-05**, grounded on `tasks/ARC_H_E2E_AUDIT.md` v1.0. Scope (Joe-locked): operationalise the Phase-2 epoch-key scheme onto the live `message.*` path + ship the server-blindness proof + make the **key-granularity decision** (envelope). **Fenced:** openmls / RFC 9420 real crypto (= D3); the no-E2E client indicator (UI milestone). **PG-05 closes interface-locked / impl-deferred behind D3** (PG-02 shape, D-065), with the named cascade `D-088 content-erasure → PG-05 real crypto → D3`.

Decisions below carry IDs **AH-D#**. Per D-069 they are **arc-local** *except* **AH-D1**, which is a **cross-arc D-088 amendment** (Joe-directed — see §1). No code until the runbook is Joe-approved.

---

## §1 — AH-D1: Envelope key granularity (LOCKED — D-088 AMENDMENT)

**Decision.** Message content is encrypted under a **per-message random content key `CK`**; `CK` is **wrapped under the current MLS epoch key**; the wrapped `CK` rides the DAG with the message. This replaces the as-built single-layer epoch encryption (`encrypt_message(epoch_key, …)`, `client_mls.rs:150`). The granularity unit is **one message = one erasable key**, which is what D-088 crypto-shred requires.

**Envelope format (`enc:` v2).** `enc:<base64url( version_1 ‖ wrapped_CK ‖ epoch_8le ‖ nonce_12 ‖ ciphertext )>`
- `CK` — 32-byte content key, **random per message** (`OsRng`), used with ChaCha20Poly1305 to encrypt the plaintext.
- `wrapped_CK` — `CK` encrypted under the epoch key (the MLS layer wraps the content key).
- `version` byte distinguishes v2 (envelope) from the v1 single-layer form, preserving the `is_encrypted_content` / `event_trace` blind-substitution convention (still keys off the `enc:` prefix — unchanged).

**The three constraints (Joe-locked), stated as invariants:**

1. **Envelope MUST NOT weaken MLS.** Confidentiality and forward secrecy still derive *entirely* from MLS: an attacker without the epoch key recovers neither `wrapped_CK` → `CK` nor the content. The envelope adds **only** an erasability layer on top; it removes no MLS property. *Invariant: removing the envelope layer (unwrap step) yields exactly the as-built epoch-confidentiality guarantee.*

2. **`CK` lifecycle is the whole game.** Erasure works **iff** `CK` is (a) random per message — **never KDF-derived from the epoch secret** (if it were re-derivable, destroying it would erase nothing); (b) wrapped under the epoch key; (c) the **wrapped form** is the only persisted form, riding the DAG; (d) read = unwrap-then-decrypt; (e) erase = destroy the wrapped `CK`. Because `CK` is random and only recoverable by unwrapping, **destroying the wrapped `CK` alone renders the content ciphertext permanently undecryptable** — the chain + signatures (which sign the ciphertext envelope) stay valid, nothing in the DAG mutates. That is crypto-shred at per-message granularity.

   **Testable invariant (threat-defended) — the entire erasure guarantee.** Destroying the wrapped `CK` MUST leave the content unrecoverable **even by a party who still holds the epoch secret.** This is the precise threat the random-`CK` rule defends: if `CK` were wrapped via an *epoch-derived KDF*, a future implementer could satisfy the wording while a holder of the epoch secret silently re-derives `CK`, and erasure becomes a no-op. **Test form:** hold the epoch secret, destroy the wrapped `CK`, attempt decryption → MUST fail. **AH-D5 assertion 5 is this same invariant, stated once** (do not state two near-duplicate properties — state the one invariant and reference it).

   **Arc-H ships steps (a)–(d)** (generate → wrap → store-wrapped → unwrap-to-read) — the *substrate*. **Step (e) (destroy-to-erase) is the fenced erasure-impl build** (deferred behind D3 per the cascade). Arc H proves the substrate exists and is sound; it does not ship the erase operation.

3. **§3.10.7 extension → full D-088 amendment.** ch3 §3.10.7 currently specifies single-layer epoch encryption; the envelope is an **extension**, so this is a formal **D-088 amendment via the audit→design→lock re-walk** that lives in this doc. Per **D-074 same-commit atomicity**, the DECISIONS.md D-088 amendment + the ch3 §3.10.7 envelope update ship **in the same commit as the envelope substrate (C1)**. The amendment records: (i) envelope granularity chosen; (ii) *why* — D-088 crypto-shred needs a per-erasure-unit key, which epoch-only keys cannot give; (iii) the non-weakening-of-MLS invariant (constraint 1).

**Promotion note.** Unlike the other AH-D#, AH-D1's substance is **promoted into D-088** (it is a cross-arc erasure invariant, not arc-local). The arc-local residue (the `enc:` v2 byte layout) stays in this doc.

---

## §2 — AH-D2: Per-Space encryption mode (LOCKED — AH-F2)

**Decision.** Add `SpaceState.e2e_encryption: bool`, **set-once at Space creation**, **default `false` (OFF)**, **uniform** (no DM exception). Mirrors Arc G's `jurisdiction` shape exactly (AG-D3/D4).

- **Type.** `bool` for Arc H. (A future `EncryptionMode` enum is the forward-extension if more modes appear; not minted now — promotion discipline.)
- **Set-once / immutable** per §3.10.8 ("immutable after Space creation; a Space cannot be retroactively encrypted or decrypted"). No mutation event, no applier, **no `state_key` arm** — fixed at Space birth.
- **Default OFF (honest dormant-but-correct, D-065).** Because PG-05 closes interface-locked (real crypto = D3), defaulting Spaces *on* would stake the default Space's security posture on crypto that does not yet exist. The flag + mechanism are real and **server-blind-proven on the Phase-2 path**; turning it on is **opt-in until D3**. The **default-flip is a D3 decision, not Arc H's.**
- **Constructors.** Read by all three Space constructors via `build_space_create_event` (the same threading Arc G used, AG-D4); DM create path threads it too (DM Spaces are *not* auto-ON — uniform, per the locked sub-call).
- **Convergence (M8).** Set-once field, no applier, rides `SpaceState` `PartialEq`/`Eq` — **zero M8 work**, identical to AG-D3.
- **Spec.** ch3 §3.10.8 field schema + Appendix C Space class.

**Rationale recorded.** Option A (encrypt-all) was rejected: it contradicts §3.10.8 (which specs the non-E2E mode), forces one global answer where the tier-graded compliance philosophy delegates to the module/deployment, and forecloses lawful-retention deployments. The erasure-vs-content-audit tension (E2E strengthens erasure but constrains T2–T4 content retention) is **correctly localised to the Space/deployment**, not the protocol.

---

## §3 — AH-D3: Group-init DAG anchor (LOCKED — AH-F3)

**Decision.** Add the **`state.mls_group_init` EventType** (§3.10.10 registers it; audit AH-A6 confirmed it is unbacked). It anchors "Room R has an MLS group, genesis epoch 0" on the DAG. It is a **`state.*` event — Node-readable, NOT E2E-encrypted** (only `message.*` content is encrypted, per the §3.10 scope; the Node must read group genesis to route DS messages).

- Emitted by the creating client when a Room is created **in an E2E Space** (gated on `space.e2e_encryption == true`). Non-E2E Spaces emit no group-init.
- **Node-side state.** A minimal opaque epoch anchor per Room (`group.rs` already tracks an opaque epoch counter — wire it to a `RoomState`-level `mls_epoch: Option<u64>`, genesis 0). **No key material Node-side** (the §3.10.1 invariant).
- **Applier.** `apply` arm sets the genesis epoch; epoch advances are driven by AH-D4. Carries a `state_key` (per-Room group anchor) so a re-init cannot fork — confirm key shape at runbook (candidate CP).

---

## §4 — AH-D4: Epoch-advance ↔ membership (LOCKED — AH-F4)

**Decision.** Epoch advances are a **client-side consequence of resolved membership**, correlated to the DAG via `mls.commit`.

- When a `membership.{join,leave,kick,ban}` event commits in an E2E Space, the affected client produces an `mls.commit` (Phase-2: advance the epoch secret per `ClientMlsGroup::add_member/remove_member`; Phase-3: a real RFC 9420 Commit). The Node DS routes the `mls.commit` / `mls.welcome` as **opaque bytes** (`delivery_service.rs`).
- **Interaction with M8 `derive_resolved` (the key clarity).** Encryption is **client-side**; the DAG carries plaintext `membership.*`/`state.*` (which M8 already resolves) + opaque ciphertext + opaque `mls.commit`. The epoch advance is **downstream of the resolved membership state** — it introduces **no new conflict domain and no new `state_key`**. `derive_resolved` is untouched by Arc H.
- **Fenced to D3 (honest):** the MLS **commit-race** under *concurrent* membership changes (two members committing different epoch advances simultaneously) is a real RFC 9420 problem solved by openmls's Commit ordering. Phase-2 demonstrates the **single-committer happy path**; the concurrent-commit resolution is **D3**, recorded not papered.

---

## §5 — AH-D5: Server-blindness proof (LOCKED — AH-F5)

**Decision.** The arc's headline deliverable test (the analogue of Arc F's two-node e2e). In-process: one E2E Space, one member sends one `message.text`, assert that **a Node hosting the Space cannot recover the plaintext** while a member client can. Concretely assert:

1. The Node stores the `enc:` blob **unchanged** (`handle_encrypted_content` pass-through; byte-identical in == out).
2. `event_trace` **blind-substitutes** encrypted content (the existing `enc:`-prefix rule; `encrypted_content_not_logged` already pins the convention — extend to the live path).
3. The DS routes `mls.welcome`/`mls.commit` as **opaque bytes** (no inspection).
4. A Node-side read of the content field yields **only ciphertext**; a member client unwraps `CK` and decrypts to the original plaintext.
5. **(Envelope-specific — = AH-D1 invariant, stated once)** the wrapped `CK` is present in the envelope, and the **erasability invariant of AH-D1 constraint (2) holds**: destroying the wrapped `CK` leaves content unrecoverable *even by a party holding the epoch secret*. The Node, holding no epoch secret at all, is strictly weaker than that adversary, so it can recover neither `CK` nor content. This and AH-D1(2) are **one invariant**, not two.

**Fence — content-blind ≠ blind (explicit, do not over-claim).** This is a **content-blindness** proof and nothing more. It does **NOT** claim metadata or traffic-analysis blindness: who-talked-to-whom, when, message volume and timing, and which-Space all remain **fully legible** to the Node (Layer-1 metadata, required for routing / signature-validation / audit) — and are frequently the *more* sensitive signal. Metadata-blindness is far beyond Arc H and is **not delivered and not claimed**. The deliverable is named precisely: *content-blindness*, fenced like every other Arc-H boundary.

---

## §6 — AH-D6: Commit split (LOCKED — AH-F6)

- **C1 — the boundary + the guarantee.** `SpaceState.e2e_encryption` (AH-D2) + `state.mls_group_init` (AH-D3) + client **envelope-encrypt send-path** (AH-D1, `enc:` v2: generate/wrap/store-wrapped/unwrap-to-read) + Node DS **blind-route** wiring + the **server-blindness proof** (AH-D5). **Ships the D-088 amendment + ch3 §3.10.7 envelope extension in this same commit (D-074).** This is the commit that makes the guarantee true on the wire.
- **C2 — lifecycle & distribution.** KeyPackage upload/distribute handlers binding the backed `mls.*` transport messages to `KeyPackageStore` (AH-A5) + epoch-advance-on-membership choreography (AH-D4, Welcome/Commit routing) + the ≥3-pool / expiry discipline (§3.10.3 MUSTs).
- **close — doc-only (D-074).** ch3 §3.10 reconcile (§3.10.7 already updated at C1) + Appendix C/I reconcile + gap-audit §5 **PG-05 → interface-locked/impl-deferred** + the **cascade note** + ROADMAP + JOURNAL + AH-D# promotion eval (AH-D1 → D-088; rest arc-local).

---

## §7 — Error codes & spec touchpoints (grounded)

- §3.10.11 error band **5001–5005** (`mls_key_package_not_found`/`_expired`/`mls_epoch_mismatch`, +5003/5004 reserved). `MlsClientError {DecryptionFailed, NotAMember, EpochMismatch}` maps to 5005 for epoch mismatch; KeyPackage handlers (C2) return 5001/5002. **Wire-code verification at runbook** (the 6007→6009 lesson — do not guess; grep the band first). Candidate CP.
- ch3 touchpoints: §3.10.7 (envelope extension, C1), §3.10.8 (`e2e_encryption` field, C1), §3.10.10 (`state.mls_group_init` EventType, C1). Appendix C (Space class + KeyPackage), Appendix I (EventType registry).

---

## §8 — Fences & honest residue (D-065)

- **D3 (openmls):** real RFC 9420 key schedule, TreeKEM, HPKE, real Welcome/Commit, concurrent-commit resolution. Arc H's interface is built to equal it (D-052) — a swap, not a rewrite.
- **UI milestone:** the §3.10.8 no-E2E **client indicator** (Round-2-gated).
- **Erasure-impl arc:** the destroy-to-erase operation (AH-D1 step e) — the substrate ships here, the erase build stays gated behind D3.
- **Phase-2 honesty:** deterministic-nonce simplification (`encrypt_message` note) — Phase-2 demonstrates single-message-per-epoch; per-message nonce counters are D3. With the envelope, each message has its own random `CK`, so the Phase-2 nonce simplification is **less load-bearing** than before (distinct `CK` per message ⇒ distinct keystream), but proper counter management is still D3.

---

## §9 — Candidate confirm-at-pickup (for the runbook, D-078)

- **CP-1** — `enc:` v2 byte layout + the `version` discriminator vs the v1 form; `EncryptedContent` API shape for wrap/unwrap.
- **CP-2** — `state.mls_group_init` `state_key` shape (AH-D3) + `RoomState.mls_epoch` field.
- **CP-3** — error-code mapping to the 5001–5005 band (grep the band; do not guess).
- **CP-4** — the send-path insertion point in `xgen-client` (where plaintext `message.*` content becomes `enc:` before dispatch) + the Node DS wiring site in `xgen-node` (`app.rs` connection/fan-out).
- **CP-5** — `build_space_create_event` + DM-create threading of `e2e_encryption` (mirror AG-D4).

---

## §10 — Locks summary

| ID | Decision | Status | Scope |
|----|----------|--------|-------|
| AH-D1 | Envelope per-message `CK` wrapped under epoch key; 3 constraints | LOCKED | **D-088 amendment (cross-arc)** |
| AH-D2 | `SpaceState.e2e_encryption` bool, set-once, default-OFF, uniform | LOCKED | arc-local |
| AH-D3 | `state.mls_group_init` EventType (Node-readable anchor) | LOCKED | arc-local |
| AH-D4 | Epoch-advance downstream of resolved membership; no new M8 surface; commit-race → D3 | LOCKED | arc-local |
| AH-D5 | Server-blindness proof (5 assertions) | LOCKED | arc-local |
| AH-D6 | C1 boundary+guarantee / C2 lifecycle / close | LOCKED | arc-local |

**Next (gated on Joe-lock of this design):** write `tasks/ARC_H_E2E_IMPL.md` (runbook; resolve CP-1…CP-5 at pickup). No code until the runbook is approved.

**v1.1 (2026-06-04):** folded two Joe honesty-tightenings into the locks (no scope change) — AH-D1 constraint (2) restated as a threat-defended testable invariant (erasure defeats a holder of the epoch secret); AH-D5 metadata/traffic-analysis fence made explicit (content-blind ≠ blind) and assertion 5 unified with the AH-D1 invariant.

**Design complete (v1.1).**
