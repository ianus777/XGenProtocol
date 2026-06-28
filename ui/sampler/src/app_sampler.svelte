<script>
  // app_sampler.svelte — SAMPLER matrix (M-RP3.1). Plain-JS app shell (bare $state,
  // no TS annotations — N-041). Mounts all 11 built `core` components live in a
  // semantic-group x state grid; each cell is a real `envelope`-registered instance
  // (`{type}#{state}`) so CDP `ids()` enumerates the matrix. The class x phase axes
  // (N-028) are deferred — degenerate while everything is di-A.
  //
  // State-map is RAGGED on purpose (honest, not a forced uniform grid):
  //   default  — all 10
  //   disabled — interactive only (display-di have none); NOTE `toggle` has no
  //              `disabled` prop (atomic gap, N-045) -> shown as `toggle#switch` instead
  //   invalid  — only textfield (bad email) + number (out-of-range); no faked columns
  //   variants — toggle checked/switch, button toggle-mode, textfield password, textarea \n
  // No focus column: focus is transient; a static focus cell would be a lie (verify live).
  import { onMount } from 'svelte';
  import Toggle from '$core/components/data-independent/toggle.svelte';
  import Button from '$core/components/data-independent/button.svelte';
  import TextField from '$core/components/data-independent/textfield.svelte';
  import Select from '$core/components/data-independent/select.svelte';
  import TextArea from '$core/components/data-independent/textarea.svelte';
  import NumberField from '$core/components/data-independent/number.svelte';
  import Range from '$core/components/data-independent/range.svelte';
  import Label from '$core/components/data-independent/label.svelte';
  import Paragraph from '$core/components/data-independent/paragraph.svelte';
  import Img from '$core/components/data-independent/image.svelte';
  import DateField from '$core/components/data-independent/date.svelte'; // not `Date` (global)

  // Runtime client<->node skin-swap (D-098): flipping [data-shell] re-aliases --accent*
  // live, so the whole grid re-themes at once. Replaces "run in both real shells".
  let shell = $state('client');
  function applyShell(s) { shell = s; document.documentElement.dataset.shell = s; }
  onMount(() => applyShell('client'));

  // Bound values (bare $state — plain-JS shell). One per interactive cell.
  let tglDefault = $state(false);
  let tglChecked = $state(true);
  let tglSwitch = $state(false);
  let btnPressed = $state(true);
  let tfDefault = $state('hello');
  let tfDisabled = $state('disabled text');
  let tfInvalid = $state('not-an-email');
  let tfPassword = $state('secret');
  let selDefault = $state('two');
  let selDisabled = $state('one');
  let taDefault = $state('line one\nline two');
  let taDisabled = $state('inert');
  let numDefault = $state(42);
  let numDisabled = $state(7);
  let numInvalid = $state(50); // outside [0,10] -> :invalid (rangeOverflow)
  let rngDefault = $state(50);
  let rngDisabled = $state(30);
  // date — string bind:value for every type (empty would be ''); seeded valid here.
  let dtDefault = $state('2026-06-28');
  let dtTime = $state('13:45');
  let dtDatetime = $state('2026-06-28T13:45');
  let dtMonth = $state('2026-06');
  let dtWeek = $state('2026-W26');
  let dtDisabled = $state('2026-06-28');
  let dtInvalid = $state('2030-01-01'); // outside [2026-01-01, 2026-12-31] -> :invalid (rangeOverflow)

  const selOptions = [
    { value: 'one', label: 'One' },
    { value: 'two', label: 'Two' },
    { value: 'three', label: 'Three' },
  ];

  // Inline data-URI so image#default renders with no file linkage (Joe's call):
  // the actual ui/assets/img-placeholder.svg artwork (mountains+sun glyph), hardcoded
  // here with explicit 72x72 so the sizeless-viewBox SVG renders tidy in the cell.
  const imgSrc =
    "data:image/svg+xml,%3Csvg viewBox='0 0 120 120' width='72' height='72' xmlns='http://www.w3.org/2000/svg' role='img' aria-label='Image placeholder'%3E%3Crect width='120' height='120' fill='%23c6c6c6'/%3E%3Crect x='30' y='40' width='60' height='43' rx='4' fill='none' stroke='%23e6e6e6' stroke-width='3.6' stroke-linejoin='round'/%3E%3Ccircle cx='43' cy='54' r='6' fill='%23e6e6e6'/%3E%3Cpath d='M33 79 L49 56 L60 68 L74 50 L87 79 Z' fill='%23e6e6e6'/%3E%3C/svg%3E";
</script>

<div class="sampler-bar">
  <span class="sampler-title">XGen Sampler</span>
  <div class="sampler-seg" role="group" aria-label="skin">
    <button class:active={shell === 'client'} onclick={() => applyShell('client')}>client</button>
    <button class:active={shell === 'node'} onclick={() => applyShell('node')}>node</button>
  </div>
</div>

<div class="sampler-body">
  <div class="s-section-title">Interactive</div>

  <div class="s-row">
    <div class="s-rowname">toggle</div>
    <div class="s-cells">
      <div class="s-cell"><span class="s-id">toggle#default</span><Toggle bind:checked={tglDefault} id="default" /></div>
      <div class="s-cell"><span class="s-id">toggle#checked</span><Toggle bind:checked={tglChecked} id="checked" /></div>
      <div class="s-cell"><span class="s-id">toggle#switch</span><Toggle bind:checked={tglSwitch} id="switch" shape="switch" /></div>
    </div>
  </div>

  <div class="s-row">
    <div class="s-rowname">button</div>
    <div class="s-cells">
      <div class="s-cell"><span class="s-id">button#default</span><Button label="Action" id="default" /></div>
      <div class="s-cell"><span class="s-id">button#disabled</span><Button label="Action" id="disabled" disabled /></div>
      <div class="s-cell"><span class="s-id">button#toggle</span><Button label="Toggle" id="toggle" mode="toggle" bind:pressed={btnPressed} /></div>
    </div>
  </div>

  <div class="s-row">
    <div class="s-rowname">textfield</div>
    <div class="s-cells">
      <div class="s-cell"><span class="s-id">textfield#default</span><TextField bind:value={tfDefault} id="default" placeholder="text" /></div>
      <div class="s-cell"><span class="s-id">textfield#disabled</span><TextField bind:value={tfDisabled} id="disabled" disabled /></div>
      <div class="s-cell"><span class="s-id">textfield#invalid</span><TextField type="email" bind:value={tfInvalid} id="invalid" /></div>
      <div class="s-cell"><span class="s-id">textfield#password</span><TextField type="password" bind:value={tfPassword} id="password" /></div>
    </div>
  </div>

  <div class="s-row">
    <div class="s-rowname">select</div>
    <div class="s-cells">
      <div class="s-cell"><span class="s-id">select#default</span><Select bind:value={selDefault} id="default" options={selOptions} placeholder="Pick one" /></div>
      <div class="s-cell"><span class="s-id">select#disabled</span><Select bind:value={selDisabled} id="disabled" options={selOptions} disabled /></div>
    </div>
  </div>

  <div class="s-row">
    <div class="s-rowname">textarea</div>
    <div class="s-cells">
      <div class="s-cell"><span class="s-id">textarea#default</span><TextArea bind:value={taDefault} id="default" rows={3} /></div>
      <div class="s-cell"><span class="s-id">textarea#disabled</span><TextArea bind:value={taDisabled} id="disabled" rows={3} disabled /></div>
    </div>
  </div>

  <div class="s-row">
    <div class="s-rowname">number</div>
    <div class="s-cells">
      <div class="s-cell"><span class="s-id">number#default</span><NumberField bind:value={numDefault} id="default" /></div>
      <div class="s-cell"><span class="s-id">number#disabled</span><NumberField bind:value={numDisabled} id="disabled" disabled /></div>
      <div class="s-cell"><span class="s-id">number#invalid</span><NumberField bind:value={numInvalid} id="invalid" min={0} max={10} /></div>
    </div>
  </div>

  <div class="s-row">
    <div class="s-rowname">range</div>
    <div class="s-cells">
      <div class="s-cell"><span class="s-id">range#default</span><Range bind:value={rngDefault} id="default" min={0} max={100} step={1} /></div>
      <div class="s-cell"><span class="s-id">range#disabled</span><Range bind:value={rngDisabled} id="disabled" min={0} max={100} step={1} disabled /></div>
    </div>
  </div>

  <div class="s-row">
    <div class="s-rowname">date</div>
    <div class="s-cells">
      <div class="s-cell"><span class="s-id">date#default</span><DateField bind:value={dtDefault} id="default" /></div>
      <div class="s-cell"><span class="s-id">date#time</span><DateField type="time" bind:value={dtTime} id="time" /></div>
      <div class="s-cell"><span class="s-id">date#datetime</span><DateField type="datetime-local" bind:value={dtDatetime} id="datetime" /></div>
      <div class="s-cell"><span class="s-id">date#month</span><DateField type="month" bind:value={dtMonth} id="month" /></div>
      <div class="s-cell"><span class="s-id">date#week</span><DateField type="week" bind:value={dtWeek} id="week" /></div>
      <div class="s-cell"><span class="s-id">date#disabled</span><DateField bind:value={dtDisabled} id="disabled" disabled /></div>
      <div class="s-cell"><span class="s-id">date#invalid</span><DateField bind:value={dtInvalid} id="invalid" min="2026-01-01" max="2026-12-31" /></div>
    </div>
  </div>

  <div class="s-section-title">Display</div>

  <div class="s-row">
    <div class="s-rowname">label</div>
    <div class="s-cells">
      <div class="s-cell"><span class="s-id">label#default</span><Label text="A field caption" id="default" /></div>
    </div>
  </div>

  <div class="s-row">
    <div class="s-rowname">paragraph</div>
    <div class="s-cells">
      <div class="s-cell"><span class="s-id">paragraph#default</span><Paragraph text="A single paragraph of prose, rendered read-only as a text node." id="default" /></div>
    </div>
  </div>

  <div class="s-row">
    <div class="s-rowname">image</div>
    <div class="s-cells">
      <div class="s-cell"><span class="s-id">image#default</span><Img src={imgSrc} alt="sample placeholder" id="default" /></div>
    </div>
  </div>
</div>
