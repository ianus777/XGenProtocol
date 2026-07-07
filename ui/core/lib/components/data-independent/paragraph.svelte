<script lang="ts">
  // paragraph — data-independent, DISPLAY-kind (N-032): a single paragraph of prose.
  // Second of the display-di trio (label/paragraph/image) — value-carrying but
  // READ-ONLY, the display half of the di model. Atomic (N-020): the root IS the
  // native <p> — chosen over a generic `text` (too primitive) and `textblock`
  // ("block" oversells a single paragraph; that name is parked for a future
  // multi-paragraph richtext renderer).
  //
  // Value prop is `text` (the display-di semantic value-name, shared with label;
  // image takes `src`). Plain, NOT $bindable — read-only.
  //
  // FORMATTER SEAM (reserved, NOT built — N-032). Today the body is a plain TEXT NODE
  // (`{text}`), never `{@html}` — safe by default. The future limited inline-mark
  // formatter (WordStar/markdown-lineage `_x_`/`*x*`, whitelist <strong>/<em>/<br>,
  // an escape char, links the one risky add) lands as a `common` `use:render` action
  // — the render-side counterpart to the edit-side `use:processor` earmarked for
  // textfield/textarea. That action will own the delimiter map + whitelist +
  // sanitisation and rewrite this node's content ONLY when applied; the paragraph
  // component itself never opens `{@html}`. Built separately, later.
  //
  // The type-class is supplied by `envelope` (mergeClasses), so no `class` is hardcoded
  // (N-023). No local CSS: a bare <p> is function-complete (xgen-normalize zeroes UA
  // margins + inherits flow typography); all appearance — size (--fs-2), colour (--t,
  // brighter than label's caption --t2 since prose is the content), line-height,
  // block spacing — is skin, keyed by `.paragraph` in the one skin file (N-025).
  import { envelope } from '$common/components/base/envelope';

  let { text = '', id, class: klass }: { text?: string; id?: string; class?: string } = $props();

  // N-024 opt-in. Read-only (no user delta); the registry stays uniform (N-030 §4).
  // Verify = render + computed-style, the display-di pattern founded at label (N-035).
  const debug = () => $state.snapshot({ text });
</script>

<p class={klass} use:envelope={{ name: 'paragraph', id, debug }}>{text}</p>
