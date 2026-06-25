# M-RP2.11 — display-di `image` (root <img>, src + required alt)
> **Status**: COMPLETED  
> Version: 1.0  
> Date: Jun 2026  
> **Last updated**: 2026-06-25  
> Language: English  
> Author: JozefN  
> Credits: Concept, philosophy, requirements, project direction: Jozef Nižnanský. Technical assistance and implementation support: AI-assisted development tools.  
> License: BSL 1.1 (converts to GPL upon project handover)  

---

## Goal

Author + skin (one pass) the third and final **display-kind di**: `image` — root `<img>`, `src` value + required `alt` (N-032). Read-only, value-carrying. **Completes the display-di trio** (label / paragraph / image).

## Locks (design walk, Joe-locked 2026-06-25 "go")

1. **Value prop = `src`** (the locked display-di semantic name — label/paragraph = `text`, image = `src`). Required: `src: string`, no default.
2. **`alt` required** — `alt: string`, no default (force the decision; `alt=""` for decorative is valid and conscious). DEV-only `console.warn` if `alt === undefined` to catch runtime omission. No prod throw (an image with missing alt should still render).
3. **Getter `{ src, alt }`** — unlike label/paragraph's single `{text}`, image's contract is two fields; `alt` being required makes it part of the component's meaningful state, and snapshotting it lets verify confirm the required-alt landed. Precedent: a display-di getter carries the fields the semantic demands.
4. **Skin `.image` = `border-radius: var(--rad)`** — assembled from existing vocabulary, **no new token**. Sizing (width/height) is a layout/consumer concern, not the atomic's skin.
5. **`use:envelope` unchanged** — content-agnostic. **Structural novelty:** `<img>` is a **void element** — first display-di whose value lives in an **attribute** (`src`), not a text-node body.
6. **Demo src = bundled placeholder asset** `ui/assets/img-placeholder.svg` (Joe-approved graphic: neutral grey square, light-grey frame + sun + two peaks). Reusable neutral placeholder, not just demo throwaway.

**Deferred off the atomic** (consumer/composite concerns, same spirit as association off `label`): load/error states (broken-src placeholder), `width`/`height` (CLS). Add when a consumer needs them.

## Phases

**Phase 1 — asset** `ui/assets/img-placeholder.svg` (DONE — the approved graphic).

**Phase 2 — author** `ui/core/lib/components/data-independent/image.svelte`: `src: string` + `alt: string` (both required, no default) + `id`; `use:envelope={{ name:'image', id, debug }}`; `debug = () => $state.snapshot({ src, alt })`; DEV guard on `alt`; void `<img src={src} alt={alt} use:envelope />`; zero `<style>`; deferred concerns documented.

**Phase 3 — skin** `.image { border-radius: var(--rad); }` appended to `skin.css`.

**Phase 4 — wire demo, both shells** (`app_client.svelte` + `app_node.svelte`): `import Placeholder from '$assets/img-placeholder.svg'`; mount `<Image src={Placeholder} alt="Image placeholder" id="demo" />` after the paragraph demo. No `$state` var (read-only).

**Phase 5 — CDP verify both apps:** `image#demo` → `{src, alt}` (src = resolved asset URL, alt = "Image placeholder"); computed-style `.image` → `border-radius` + `display:block`; screenshots; clean teardown.

**Phase 6 — records (D-074 atomic):** notes N-037; components Built row + detail (v0.15→0.16); ROADMAP RP node + frontier (v3.95→3.96) — **display-di trio complete**; CLAUDE PLAY → M-RP2.11 CLOSED / Next first composites / pointer J-415→J-416; this task → COMPLETED; JOURNAL J-416. All `.md` `Last updated` bumped.

## Definition of Done

- [x] `img-placeholder.svg` asset created
- [x] `image.svelte` authored (`src` + required `alt`, envelope, `{src,alt}` getter, DEV alt-guard, void `<img>`, zero `<style>`)
- [x] `.image` skinned (`--rad`, no new token)
- [x] demo wired both shells (imports the asset)
- [x] CDP both apps: `image#demo` `{src,alt}` + computed-style radius/block (real output in JOURNAL)
- [x] screenshots both apps eye-checked
- [x] records updated (N-037, components v0.16, ROADMAP, CLAUDE PLAY, JOURNAL J-416, task COMPLETED) — trio complete
