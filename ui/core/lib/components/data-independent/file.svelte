<script lang="ts">
  // file — data-independent, interaction semantic: file selection, native picker button (N-022).
  // Atomic (N-020): root IS <input type="file">. OWN atomic, obvious — it binds a FileList, not a
  // string/number/boolean; no fold candidate (date/color differ entirely; no string/number
  // siblings). Applies D-096, no amendment.
  //
  // Thirteenth `core` component (M-RP2.18). THE HEADLINE is the binding shape: this is the FIRST
  // non-`value` binding in the library — `bind:files` (a FileList) — the 4th binding shape after
  // boolean-in (`checked`, toggle) / event-out (`onclick`, button) / string-in (`value`, the input
  // family). It is also the first value-type that `$state.snapshot` CANNOT serialise: a FileList is
  // a live host object, not a plain object/proxy.
  //
  // GETTER (the design point) — de-FileLists to plain metadata so the N-024 debug registry round-
  // trips over CDP returnByValue: `{ count, files: [{ name, size, type }] }`. The BINDABLE prop
  // carries the LIVE FileList; the getter is the serialisable view. (NOT `$state.snapshot(files)` —
  // that won't flatten a host FileList.)
  //
  // Prop surface: files (FileList | null, $bindable via bind:files, empty = null), accept (native
  // type filter), multiple (single vs multi), disabled, id, name.
  //   drop — value (UNSETTABLE programmatically — browser security; the consumer reads via the
  //          FileList binding), placeholder / pattern / readonly / min / max / step (n/a), type
  //          (fixed). `capture` (mobile camera) reserved, not built.
  // No processor seam (a file pick, not typed entry). A custom drag-drop file row (zone + selected-
  // file list + remove + progress) is the deferred `file-field` COMPOSITE, not this atomic.
  //
  // Selection fires `change` (NOT `input`) — `bind:files` updates on change. The type-class is
  // supplied by `envelope` (N-023). No local CSS: a bare file <input> is function-complete; the
  // `.file` skin styles the file-button pseudo (::file-selector-button / ::-webkit-file-upload-
  // button) to match `.button`. The surrounding "No file chosen" text is UA-rendered (accepted).
  import { envelope } from '$common/components/base/envelope';

  let {
    files = $bindable(null),
    accept,
    multiple = false,
    disabled = false,
    id,
    name,
  }: {
    files?: FileList | null;
    accept?: string;
    multiple?: boolean;
    disabled?: boolean;
    id?: string;
    name?: string;
  } = $props();

  // N-024 opt-in. De-FileList → a plain, serialisable view (count + per-file metadata).
  // The FileList itself is a live host object; map it rather than snapshot it.
  const debug = () => ({
    count: files ? files.length : 0,
    files: files ? Array.from(files).map((f) => ({ name: f.name, size: f.size, type: f.type })) : [],
  });
</script>

<input
  type="file"
  {accept}
  {multiple}
  {disabled}
  {name}
  bind:files
  use:envelope={{ name: 'file', id, debug }}
/>
