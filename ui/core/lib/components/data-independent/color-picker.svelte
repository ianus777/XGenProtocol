<script lang="ts">
  // color-picker — data-independent COMPOSITE (M-RP2.29): 7th di composite, 24th `core`.
  // The themeable, COMPACT answer to native <input type=color> — whose popup dialog is
  // OS/Chromium-painted and unreachable by CSS *or* JS past the swatch (N-047, reconfirmed).
  // Combobox-shaped OWNED-POPUP (N-063): an anchor `textfield` (`__hex`) carrying the live
  // canonical value, with a PALETTE icon in the chevron slot; expanding reveals a dense
  // picker surface (SV square + hue + alpha + model selector + eyedropper + 8 recents).
  //
  // PASSIVE di, not a `widget` (N-059): owns exactly one UI flag, `open` (combobox order).
  // Composite-registration (N-054): root registers one aggregate getter `{value}`; the
  // children self-register — `textfield __hex`, `range __hue`, `range __alpha` (suffixes
  // collision-safe vs `__field`/`__input`/`__filter`). The SV surface, model row, recents
  // and eyedropper are RAW elements (no atomic) so the matrix stays +4/cell.
  //
  // VALUE = canonical `#rrggbbaa` (8-digit, lowercase, always-valued; more capable than
  // native, which emits no alpha). Internal source of truth = HSVA (`h` 0–360, `s`/`v`
  // 0–100, `a` 0–255); `value` is derived on every change. Alpha units 0–255 (1:1 `aa`).
  // Two guarded effects (commit hsva→value/hexDraft/lastHexa; parse user hex edits) keep
  // the anchor field and the model inputs in sync without a feedback loop (`lastHexa` gate).
  //
  // Model selector (HEXA/RGBA/HSVA) swaps the popup NUMERIC ROW only — the SV/hue/alpha
  // surface is HSV-native and identical across models. Eyedropper recycles the native
  // `EyeDropper` API (returns `#rrggbb`; keeps current alpha); hidden when the API is absent.
  //
  // Deferred (N-047 lineage / D-065): colorspace attr; persistence of recents + model;
  // alpha-as-% toggle; keyboard nav on the SV surface (pointer-only v1, as combobox has no
  // key subsystem). No component <style>: all appearance is `.color-picker*` in skin.css (L2).
  import { envelope } from '$common/components/base/envelope';
  import Textfield from './textfield.svelte';
  import Range from './range.svelte';

  type Model = 'hexa' | 'rgba' | 'hsva';

  let {
    value = $bindable('#000000ff'),
    disabled = false,
    id,
    name,
  }: {
    value?: string;
    disabled?: boolean;
    id?: string;
    name?: string;
  } = $props();

  const cid = (s: string) => (id ? `${id}__${s}` : undefined);

  // ── pure colour helpers ───────────────────────────────────────────────────
  const clamp = (n: number, lo: number, hi: number) => Math.min(hi, Math.max(lo, n));
  function hsvToRgb(h: number, s: number, v: number) {
    s /= 100; v /= 100;
    const c = v * s, x = c * (1 - Math.abs(((h / 60) % 2) - 1)), m = v - c;
    let r = 0, g = 0, b = 0;
    if (h < 60) { r = c; g = x; }
    else if (h < 120) { r = x; g = c; }
    else if (h < 180) { g = c; b = x; }
    else if (h < 240) { g = x; b = c; }
    else if (h < 300) { r = x; b = c; }
    else { r = c; b = x; }
    return { r: Math.round((r + m) * 255), g: Math.round((g + m) * 255), b: Math.round((b + m) * 255) };
  }
  function rgbToHsv(r: number, g: number, b: number) {
    r /= 255; g /= 255; b /= 255;
    const mx = Math.max(r, g, b), mn = Math.min(r, g, b), d = mx - mn;
    let h = 0;
    if (d) {
      if (mx === r) h = 60 * ((((g - b) / d) % 6 + 6) % 6);
      else if (mx === g) h = 60 * ((b - r) / d + 2);
      else h = 60 * ((r - g) / d + 4);
    }
    // Keep FLOATS (no rounding) so rgb->hsv->rgb is lossless; round only at display (N-066).
    return { h, s: mx ? (d / mx) * 100 : 0, v: mx * 100 };
  }
  const byteHex = (n: number) => clamp(Math.round(n), 0, 255).toString(16).padStart(2, '0');
  const toHexa = (h: number, s: number, v: number, a: number) => {
    const { r, g, b } = hsvToRgb(h, s, v);
    return `#${byteHex(r)}${byteHex(g)}${byteHex(b)}${byteHex(a)}`;
  };
  const HEX_RE = /^#?([0-9a-fA-F]{6}|[0-9a-fA-F]{8})$/;
  const validHex = (x: string) => HEX_RE.test(x.trim());
  function normHexa(x: string) {
    let s = x.trim().replace(/^#/, '').toLowerCase();
    if (s.length === 6) s += 'ff';
    return `#${s}`;
  }
  function parseHexa(x: string) {
    const s = normHexa(x).slice(1);
    const r = parseInt(s.slice(0, 2), 16), g = parseInt(s.slice(2, 4), 16),
          b = parseInt(s.slice(4, 6), 16), a = parseInt(s.slice(6, 8), 16);
    return { ...rgbToHsv(r, g, b), a };
  }

  // ── state (HSVA truth) ─────────────────────────────────────────────────────
  const init = parseHexa(validHex(value) ? value : '#000000ff');
  let h = $state(init.h), s = $state(init.s), v = $state(init.v), a = $state(init.a);
  let hexDraft = $state(toHexa(init.h, init.s, init.v, init.a));
  let lastHexa = $state(hexDraft);
  let model = $state<Model>('hexa');
  let open = $state(false);
  let recents = $state<string[]>([]);

  const rgb = $derived(hsvToRgb(h, s, v));
  const rgbaCss = $derived(`rgb(${rgb.r} ${rgb.g} ${rgb.b} / ${(a / 255).toFixed(3)})`);
  const solidCss = $derived(`rgb(${rgb.r} ${rgb.g} ${rgb.b})`);
  const hueCss = $derived(`hsl(${h} 100% 50%)`);

  // commit: HSVA -> value + anchor field + gate. Reads hsva only (no cycle through here).
  $effect(() => {
    const x = toHexa(h, s, v, a);
    lastHexa = x;
    if (x !== value) value = x;
    hexDraft = x;
  });
  // parse: user edits to the hex field. Gated by lastHexa so reflected writes are no-ops.
  $effect(() => {
    const d = hexDraft;
    if (validHex(d) && normHexa(d) !== lastHexa) {
      const p = parseHexa(d);
      h = p.h; s = p.s; v = p.v; a = p.a;
    }
  });
  // recents: commit the value on popup CLOSE (open true->false), dedup, most-recent-first, cap 8.
  let prevOpen = false;
  $effect(() => {
    const isOpen = open, cur = value.toLowerCase();
    if (prevOpen && !isOpen && validHex(cur)) {
      recents = [cur, ...recents.filter((c) => c !== cur)].slice(0, 8);
    }
    prevOpen = isOpen;
  });
  // outside-pointerdown closes (robust for a rich popup; no blur-close race with sliders/SV).
  let rootEl: HTMLDivElement;
  $effect(() => {
    if (!open) return;
    const onDoc = (e: PointerEvent) => { if (rootEl && !rootEl.contains(e.target as Node)) open = false; };
    document.addEventListener('pointerdown', onDoc, true);
    return () => document.removeEventListener('pointerdown', onDoc, true);
  });

  // ── mutators ───────────────────────────────────────────────────────────────
  function setRGB(nr: number, ng: number, nb: number) {
    const p = rgbToHsv(clamp(nr, 0, 255), clamp(ng, 0, 255), clamp(nb, 0, 255));
    h = p.h; s = p.s; v = p.v;
  }
  function pickRecent(c: string) { const p = parseHexa(c); h = p.h; s = p.s; v = p.v; a = p.a; }

  // SV surface: pointer x -> S, y -> V (top = max V). Pointer-captured drag.
  let svEl: HTMLDivElement;
  function svPoint(e: PointerEvent) {
    const r = svEl.getBoundingClientRect();
    s = clamp(((e.clientX - r.left) / r.width) * 100, 0, 100);
    v = clamp((1 - (e.clientY - r.top) / r.height) * 100, 0, 100);
  }
  function svDown(e: PointerEvent) {
    if (disabled) return;
    svEl.setPointerCapture(e.pointerId);
    svPoint(e);
  }
  function svMove(e: PointerEvent) {
    if (disabled || !svEl.hasPointerCapture(e.pointerId)) return;
    svPoint(e);
  }

  const hasEyeDropper = typeof window !== 'undefined' && 'EyeDropper' in window;
  async function eyedrop() {
    if (disabled) return;
    try {
      // @ts-ignore — EyeDropper is Chromium/WebView2 native, not in lib.dom yet.
      const res = await new window.EyeDropper().open();
      const p = parseHexa(res.sRGBHex); h = p.h; s = p.s; v = p.v; // keep current alpha
    } catch (_) { /* user dismissed */ }
  }

  function onFocusIn() { if (!disabled) open = true; }
  function onKey(e: KeyboardEvent) { if (e.key === 'Escape') open = false; }
  function onPalDown(e: PointerEvent) {
    e.preventDefault();
    if (disabled) return;
    open ? (open = false) : rootEl?.querySelector('input')?.focus();
  }
  const keep = (e: PointerEvent) => e.preventDefault(); // retain field focus on popup chrome

  const debug = () => $state.snapshot({ value });
</script>

<div
  class="color-picker"
  bind:this={rootEl}
  use:envelope={{ name: 'color-picker', id, debug }}
  data-open={open || undefined}
  aria-disabled={disabled || undefined}
  onfocusin={onFocusIn}
  onkeydown={onKey}
>
  <span class="cp-swatch" style="background: {rgbaCss}" aria-hidden="true"></span>
  <Textfield bind:value={hexDraft} {disabled} {name} id={cid('hex')} pattern="#?([0-9a-fA-F]{'{6}'}|[0-9a-fA-F]{'{8}'})" />
  <span class="pal" aria-hidden="true" onpointerdown={onPalDown}></span>

  {#if open}
    <div class="color-picker-pop" role="dialog" aria-label="Colour picker">
      <div
        class="cp-sv"
        bind:this={svEl}
        role="slider"
        aria-label="Saturation and value"
        aria-valuetext="S {Math.round(s)} V {Math.round(v)}"
        style="--cp-hue: {hueCss}"
        onpointerdown={svDown}
        onpointermove={svMove}
      >
        <span class="cp-sv-thumb" style="left: {s}%; top: {100 - v}%"></span>
      </div>

      <div class="cp-hue" onpointerdown={keep}>
        <Range bind:value={h} min={0} max={360} step={1} {disabled} id={cid('hue')} />
      </div>
      <div class="cp-alpha" style="--cp-solid: {solidCss}" onpointerdown={keep}>
        <Range bind:value={a} min={0} max={255} step={1} {disabled} id={cid('alpha')} />
      </div>

      <div class="cp-models" role="radiogroup" aria-label="Colour model" onpointerdown={keep}>
        {#each (['hexa', 'rgba', 'hsva'] as Model[]) as m}
          <button
            type="button"
            role="radio"
            aria-checked={model === m}
            class:active={model === m}
            onclick={() => (model = m)}
          >{m.toUpperCase()}</button>
        {/each}
      </div>

      <div class="cp-fields" onpointerdown={keep}>
        {#if model === 'hexa'}
          <input class="cp-num cp-hex" value={hexDraft} oninput={(e) => (hexDraft = (e.currentTarget as HTMLInputElement).value)} spellcheck="false" />
        {:else if model === 'rgba'}
          <input class="cp-num" type="number" min="0" max="255" value={rgb.r} oninput={(e) => setRGB(+(e.currentTarget as HTMLInputElement).value, rgb.g, rgb.b)} />
          <input class="cp-num" type="number" min="0" max="255" value={rgb.g} oninput={(e) => setRGB(rgb.r, +(e.currentTarget as HTMLInputElement).value, rgb.b)} />
          <input class="cp-num" type="number" min="0" max="255" value={rgb.b} oninput={(e) => setRGB(rgb.r, rgb.g, +(e.currentTarget as HTMLInputElement).value)} />
          <input class="cp-num" type="number" min="0" max="255" value={a} oninput={(e) => (a = clamp(+(e.currentTarget as HTMLInputElement).value, 0, 255))} />
        {:else}
          <input class="cp-num" type="number" min="0" max="360" value={Math.round(h)} oninput={(e) => (h = clamp(+(e.currentTarget as HTMLInputElement).value, 0, 360))} />
          <input class="cp-num" type="number" min="0" max="100" value={Math.round(s)} oninput={(e) => (s = clamp(+(e.currentTarget as HTMLInputElement).value, 0, 100))} />
          <input class="cp-num" type="number" min="0" max="100" value={Math.round(v)} oninput={(e) => (v = clamp(+(e.currentTarget as HTMLInputElement).value, 0, 100))} />
          <input class="cp-num" type="number" min="0" max="255" value={a} oninput={(e) => (a = clamp(+(e.currentTarget as HTMLInputElement).value, 0, 255))} />
        {/if}
        {#if hasEyeDropper}
          <button type="button" class="cp-eyedrop" aria-label="Pick from screen" onclick={eyedrop}></button>
        {/if}
      </div>

      <ul class="cp-recents" role="listbox" aria-label="Recent colours">
        {#each Array(8) as _, i}
          {@const c = recents[i]}
          <li
            role="option"
            aria-selected={c === value.toLowerCase() || undefined}
            class:empty={!c}
            style={c ? `background: ${c}` : undefined}
            title={c || 'empty'}
            onpointerdown={keep}
            onclick={() => c && pickRecent(c)}
          ></li>
        {/each}
      </ul>
    </div>
  {/if}
</div>
