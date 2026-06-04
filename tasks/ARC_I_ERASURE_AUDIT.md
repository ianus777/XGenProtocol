# XGen Protocol — Arc I (GDPR Erasure / Right-to-be-Forgotten, PG-02) Phase-0 Audit
> **Status**: COMPLETED  
> Version: 1.1  
> Date: Jun 2026  
> **Last updated**: 2026-06-04  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## §1 — Scope and the D-071 gate

Arc I is the Round-1 D-071 Phase-0 audit for **PG-02** (uniform GDPR-erasure Event across federation, S1) — the §2.1 banked tension and the hardest catalogued gap. Spec home: ch1 L879–883; Appendix D §3.3 ("Right-to-Erasure Problem in Federated Systems").

**This is a DESIGN-ONLY arc** (locked in discussion 2026-06-04). The deliverable is the canonical XGen erasure *architecture* — the written, defensible answer to "how does XGen satisfy Art.17 in a no-anonymity, append-only, federated system" — plus a cross-arc DECISIONS stance. No C1/C2 code; the *implementation* is substrate-gated on PG-05 (§3, AI-A2). PG-02 closes to **design-locked / impl-deferred**, NOT ✅ DONE (D-065 — never mark shipped what is not built).

## §2 — Spec anchor (what PG-02 demands)

ch1 L879–883: a **uniform, tier-graded deletion/erasure Event** that **propagates + confirms + audits** across federation. Appendix D §3.3 + L101: the append-only DAG forbids selective Event deletion without breaking the hash chain; identity is a public key replicated across replica Nodes, each holding a self-certifying cache (Appendix D L49). Today only `message.redact` (display redaction, `exchange.rs:703`) exists.

## §3 — As-built findings (AI-A#)

**AI-A1 (GAP-CONFIRMED).** Only `MessageRedact` exists — display redaction, not destruction. No content-erasing event, no identity-erasure mechanism, no federated erasure propagate/confirm/audit path. GAP-CONFIRMED, S1.

**AI-A2 (the substrate finding — why this is design-only).** `Event.content: Value` is stored **plaintext-inline** (`wire.rs:437`); `canonical.rs` excludes only `event_id` + `signature` from the canonical bytes, so **both the EventXgid and the signature cover `content`**. Content erasure therefore has exactly two mechanism families:
  - **(a) Crypto-shredding** — content encrypted under an erasable per-subject/item key; the DAG keeps ciphertext; the signature signs ciphertext (stays valid); erase the key ⇒ content unrecoverable, **chain + signatures fully intact, nothing mutated**. This is the correct, regulator-accepted answer. **It requires an encryption substrate that does not exist** — exactly the PG-05 (E2E/MLS) boundary. → erasure implementation is **substrate-gated on PG-05**.
  - **(b) Blank-at-rest** — physically overwrite `content`. Breaks signature re-verification (content changed) and forces a protocol-wide **verify-skip-on-erased** exception to the "every event is signature-verifiable" invariant, and fights D-076 byte-determinism. **Rejected as the default** — it is the wrong substrate: a permanent integrity scar to solve a problem crypto-shred solves cleanly, going vestigial the day PG-05 lands.
  
  Disposition: design the erasure architecture around crypto-shred-over-PG-05; do not build the integrity-weakening fallback now.

**AI-A3 (identity erasure = orphan the binding; touches no events).** Identity erasure does **not** touch the DAG. The pubkey persists in the log as `sender`; every signature keeps verifying; the chain is untouched. Erasure = removing the **binding record** (pubkey → natural person: display name, Tier-claims / legal attestation) from the registry cache. The residue is an **orphaned pseudonymous key** — "some entity signed these; no longer known who" — which is anonymized data, outside Art.17's scope (Art.17 erases data identifying a *natural person*). **No verify-skip is needed for identity** — that was only ever a content concern. This collapses the integrity-scar worry: identity erasure is clean and available today in principle.

**AI-A4 (the keystone — the AuthTier ladder *is* the erasability gradient).** `tiers.rs` defines T1 (cryptographic identity only, no TTL) → T2 (organisational, 365d TTL) → T3 (180d) → T4 (legal/government, 90d). Erasability grades with it:
  - **T1** — the binding is a self-asserted handle on a bare key; fully **orphan-able** on request. GDPR erasure honoured.
  - **T2** — protocol-side binding orphan-able; the external Auth Module's own records are out of protocol scope (operator/legal).
  - **T3 / T4** — **anonymization is refused by design.** The entire purpose of legal-identity verification is permanent, non-repudiable attribution; retaining the binding rests on a **lawful basis** (Art.17(3) legal-obligation / public-interest carve-outs). The protocol *correctly refuses* to orphan a T4 (likely T3) key — compliant **by exemption**, not in violation.
  
  This makes "tier-graded erasure" (ch1 L879–883) precise: **whether anonymization is permitted grades by tier**, not merely the propagate/confirm path. It closes the §2.1↔§2.2 knot — the no-anonymity pillar's strongest expression (T4 legal identity) is exactly where the right-to-be-forgotten yields to legal accountability, as a conscious, lawful tradeoff (the §2.2 residual-by-design, now a precise tier property).

**AI-A5 (federated propagate/confirm/audit — designable now, reuses infra).** The erasure Event flows through the existing federation push path (Arc F/G pattern); tier-graded confirm = a delivery-ack (Tier 3+); audit via the shipped M6 protocol-audit-log. **Genuine residual:** a non-complying / offline / out-of-jurisdiction replica still holds the cached binding — the protocol cannot compel another operator (identical shape to §2.2). Stated honestly; not solvable in-protocol.

**AI-A6 (complementarity).** Identity-orphaning and content-crypto-shred are **complementary, not independent**: an orphaned key still links all of a person's events, so un-erased content or out-of-band correlates can re-identify. Complete erasure needs both mechanisms. The design states this.

## §4 — Scope fence (named homes, STOP on drift)

- **OUT — content-destruction implementation:** deferred to PG-05 crypto-shred (AI-A2); this arc designs it, does not build it.
- **OUT — blank-at-rest fallback as default:** rejected (AI-A2b). Named residual: **legacy plaintext already committed pre-PG-05 is not crypto-shreddable** — erasing it would need the rejected blank-at-rest, which the design specifies as an explicit, audited, last-resort operator action *with its integrity consequence stated*, but does **not** build now.
- **OUT — compelling external Auth Modules / non-complying replicas:** operator/legal, §2.2-shape (AI-A5).
- **OUT — code:** design-only arc; no C1/C2.

## §5 — Verdict & design-deliverable framing

PG-02 = **GAP-CONFIRMED** (AI-A1), and its reputation as "unsolved" is half a myth: append-log-vs-Art.17 is solved in principle (crypto-shred), the substrate (PG-05) is what's missing; and the hard residuals are **stance**, not code. The arc therefore closes the *intellectual/architectural* gap now and honestly defers the build:
  - **Content** → crypto-shred over PG-05 (no integrity scar).
  - **Identity** → orphan the binding; pubkey persists as an anonymous token (touches no events).
  - **Erasability is tier-graded** — T1/T2 orphan-able, T3/T4 non-anonymizable by lawful basis (AI-A4).
  - **Residual** = federated compliance + re-identification complementarity (AI-A5/A6), stated, not pretended-away.

The DESIGN doc produces this as the canonical architecture + AI-D# locks + a candidate **DECISIONS.md** entry (the erasure stance is a protocol-wide cross-arc invariant, not arc-local). It also carries a **deferred-implementation sketch** (what the eventual C1/C2 become once PG-05 lands) so the future work is teed up. **Sequencing finding:** PG-05 (Arc H) is a prerequisite for the erasure *implementation* — Arc H should precede the erasure build.

No DECISIONS change at audit stage (the stance is proposed in DESIGN, D-069/D-074). Doc-only — suite unchanged at J-252's 1131/0/2, not re-run.
