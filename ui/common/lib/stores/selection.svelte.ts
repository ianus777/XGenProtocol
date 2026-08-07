// selection.svelte.ts — the selection bus (M-RP6.1f, D3 · region-dock §5). ONE active selection across
// the whole layout: `{ regionId, entity } | null`. R1/R2 rows WRITE it on activation; R8 (inspector) and
// `entity-context-menu` READ it. It is a `$common` store — GROUNDED, not preferred (D3): both eventual
// consumers are widgets in `ui/common/lib/components/widgets/`, and W-3 forbids a `common` widget importing
// a shell dep, so a shell-local bus would be structurally UN-consumable. A `.svelte.ts` module so its
// module-level `$state` participates in Svelte 5 reactivity (the `processor/store.svelte.ts` precedent).
//
// ⚠️ CORRECTED, ANNOTATED NOT REPAIRED (D-131, M-RP-MEMBER-ACT Leg A). The line above says
// `entity-context-menu` READS this bus. Measured 2026-08-06: it does NOT. It takes
// `descriptor: EntityDescriptor` as a PROP, is gesture-agnostic, and does not import this
// module. R8 (inspector) is the only consumer. J-591 carries the same claim.
//
// 🛑 CORRECTED AGAIN, ANNOTATED NOT REPAIRED (D-131, M-RP-SELECT-ORIENT, J-692). THE ANNOTATION
// DIRECTLY ABOVE IS ITSELF FALSE: "R8 (inspector) is the only consumer" is wrong, and it was
// written one arc earlier while correcting a different false claim in this same comment block.
// Measured 2026-08-08 (grep "stores/selection" over ui/**) — THERE ARE FIVE READERS:
//   inspector-panel.svelte:40  — R8, the acknowledged one
//   self-panel.svelte:55       — its own `selected` highlight
//   rooms-panel.svelte:25,:44  — an $effect (scope) AND its `selected` highlight
//   spaces-panel.svelte:45     — its `selected` highlight
//   app_client.svelte:196      — the bus→latch bridge ($effect → roomLatch.note)
// 🔑 AND THE CONSEQUENCE IS A SHIPPED DEFECT, not bookkeeping: because R1/R6 derive their
// highlight by matching `entity.kind`, ANY identity selection (R3's self card today, every
// member click once M-RP-MEMBER-ACT Leg C lands) extinguishes BOTH panel highlights while the
// room latch — and therefore R5's content and the composer's target — keeps the room. Driven on
// the live client: roomsSel room-id → null, latched UNCHANGED.
// ⇒ M-RP-SELECT-ORIENT moves R1/R6 off the bus and onto the latch. Until it lands, this comment
//   is the honest record: FIVE readers, and the count has been wrong twice.
//
// Shape is EXACTLY as locked (D3) — ONE meaning, do not widen. Joe killed the shelf minus-button precisely
// so a second widget/leaf-selection bus is never needed (`xgen-widget-surfaces-phase0.md` S-6); this is the
// single selection primitive.
//
// The `EntityDescriptor` import is TYPE-ONLY from `$core` (§0.2 — `entity-context-menu` already imports
// this type from `$core` on shipped precedent; erased at build, so no layering line is crossed).
//
// FIRST WRITER (M-RP6.1g, D5): the `self-panel` widget's identity `entity-item` calls `selection.set()` on
// activation — R3 is the bus's first real producer, retiring the 6.1f "no UI writer" phase-limit. R1/R2 add
// more writers as they land; R8 (inspector) + `entity-context-menu` are the readers (6.1h+). The DEV
// `__XGEN_SEL__` handle below stays for CDP-driving the bus directly.

import type { EntityDescriptor } from '$core/components/data-dependent/types';

export interface Selection {
  regionId: string;
  entity: EntityDescriptor;
}

let _current = $state<Selection | null>(null);

export const selection = {
  /** The one active selection, or null. Reactive — R8 / entity-context-menu read this. */
  get current(): Selection | null {
    return _current;
  },
  /** Replace the selection (ONE selection, not a list — a second `set` overwrites). */
  set(regionId: string, entity: EntityDescriptor): void {
    _current = { regionId, entity };
  },
  /** Clear to null. */
  clear(): void {
    _current = null;
  },
};

// DEV-only CDP handle (mirrors __XGEN_SUBS__ / __XGEN_PROC__, N-024) — for driving/reading the bus at
// verify. Dead-code-eliminated in a production build.
if (import.meta.env.DEV && typeof window !== 'undefined') {
  (window as unknown as { __XGEN_SEL__?: unknown }).__XGEN_SEL__ = selection;
}
