<script>
  // app_sampler.svelte — SAMPLER matrix. Plain-JS app shell (bare $state, no TS
  // annotations — N-041). Mounts all built `core` components live in a
  // semantic-group x state grid; each cell is a real `envelope`-registered instance
  // (`{type}#{state}`) so CDP `ids()` enumerates the matrix. 16 built `core` components.
  //
  // M-RP3.2: the matrix is now tabbed by class x arity (di/dd x atomic/composite).
  // ALL panels stay MOUNTED; inactive panels are hidden via CSS display:none
  // (class:hidden), NEVER {#if} — `envelope` registers only while mounted, so
  // unmounting a panel would drop its ids from window.__XGEN_DEBUG__ and break the
  // CDP matrix-count invariant (D-097). DI-atomic holds the current 44 cells with
  // INTERACTIVE / DISPLAY / NAVIGATION sub-headers; the other three panels are empty
  // placeholders (di-composite's first occupant is status-indicator, M-RP2.22).
  //
  // State-map is RAGGED on purpose (honest, not a forced uniform grid):
  //   default  — all
  //   disabled — interactive only (display/navigation di have none); NOTE `toggle` has
  //              no `disabled` prop (atomic gap, N-045) -> shown as `toggle#switch` instead
  //   invalid  — only textfield (bad email) + number (out-of-range) + date; no faked columns
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
  import ColorField from '$core/components/data-independent/color.svelte';
  import FileField from '$core/components/data-independent/file.svelte';
  import SelectMultiple from '$core/components/data-independent/select-multiple.svelte';
  import Led from '$core/components/data-independent/led.svelte';
  import Link from '$core/components/data-independent/link.svelte';
  import StatusIndicator from '$core/components/data-independent/status-indicator.svelte';

  // Processor (common infra, M-RP4.0/M-RP4.2): the kind-1 transformer attachment, fed from the
  // source-agnostic substitutions store. The atomic forwards {...rest}; processor(...) returns a
  // symbol-keyed attachment that lands on the inner <textarea>. Adds NO registry entry.
  // In the real client the store is hydrated from xgen-client_config.toml [substitutions] via the
  // get_substitutions Tauri command; here (no client config) the sampler seeds a literal list.
  import { processor } from '$common/components/processor/processor';
  import { substitutions } from '$common/components/processor/store.svelte';

  // One user-owned list (the only source) — the sampler seeds the SAME canonical starter pack the
  // client ships (DEFAULT_SUBSTITUTIONS_SEED in xgen-client/src/app.rs), so the workbench shows the
  // real shipped behaviour. Grammar (M-RP4.2): pairs on " | ", first space splits find|replace.
  substitutions.setRules('--> → | <-- ← | :) 🙂 | <3 ❤️ | :( 🙁 | -- ‒');

  // Runtime client<->node skin-swap (D-098): flipping [data-shell] re-aliases --accent*
  // live, so the whole grid re-themes at once. Replaces "run in both real shells".
  let shell = $state('client');
  function applyShell(s) { shell = s; document.documentElement.dataset.shell = s; }
  onMount(() => applyShell('client'));

  // Tab container (M-RP3.2): outer class x arity axis. Inactive panels are CSS-hidden,
  // NOT unmounted (class:hidden), so every component stays registered for CDP `ids()`.
  let activeTab = $state('di-atomic'); // 'di-atomic' | 'di-composite' | 'dd-atomic' | 'dd-composite'
  const tabs = [
    { id: 'di-atomic',    label: 'DI · atomic' },
    { id: 'di-composite', label: 'DI · composite' },
    { id: 'dd-atomic',    label: 'DD · atomic' },
    { id: 'dd-composite', label: 'DD · composite' },
  ];

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
  let taProcessed = $state(''); // M-RP4.0/4.2: processor-host cell; store-sourced user rules morph on input
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
  // color — always a string hex #rrggbb (never empty); seeded with the per-shell accents.
  let colDefault = $state('#9a6a30');
  let colDisabled = $state('#2a6090');
  // file — FileList | null; unsettable from markup (browser security), so all start null.
  let fDefault = $state(null);
  let fMultiple = $state(null);
  let fDisabled = $state(null);
  // select-multiple — string[]; empty = [] (NOT null). Arrays CAN seed from markup (unlike file).
  let smDefault = $state([]);
  let smSeeded = $state(['a', 'c']);
  let smDisabled = $state(['b']);

  const selOptions = [
    { value: 'one', label: 'One' },
    { value: 'two', label: 'Two' },
    { value: 'three', label: 'Three' },
  ];

  // select-multiple shares the N-034 options-prop shape (carried over from `select`).
  const smOptions = [
    { value: 'a', label: 'Alpha' },
    { value: 'b', label: 'Beta' },
    { value: 'c', label: 'Gamma' },
  ];

  // led — caller-supplied state→colour map (hex OR var(--token)); colour is data, not accent.
  const ledStates = { ON: '#22c55e', OFF: 'var(--t4)', ERR: 'var(--err)' };

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

<div class="sampler-tabs" role="tablist" aria-label="component class x arity">
  {#each tabs as t}
    <button
      role="tab"
      aria-selected={activeTab === t.id}
      class:active={activeTab === t.id}
      onclick={() => (activeTab = t.id)}
    >{t.label}</button>
  {/each}
</div>

<!-- DI · atomic — the current 44-cell grid; INTERACTIVE / DISPLAY / NAVIGATION sub-headers -->
<div class="sampler-panel" class:hidden={activeTab !== 'di-atomic'}>
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
      <div class="s-rowname">select-multiple</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">select-multiple#default</span><SelectMultiple bind:value={smDefault} id="default" options={smOptions} /></div>
        <div class="s-cell"><span class="s-id">select-multiple#seeded</span><SelectMultiple bind:value={smSeeded} id="seeded" options={smOptions} /></div>
        <div class="s-cell"><span class="s-id">select-multiple#disabled</span><SelectMultiple bind:value={smDisabled} id="disabled" options={smOptions} disabled /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">textarea</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">textarea#default</span><TextArea bind:value={taDefault} id="default" rows={3} /></div>
        <div class="s-cell"><span class="s-id">textarea#disabled</span><TextArea bind:value={taDisabled} id="disabled" rows={3} disabled /></div>
        <div class="s-cell"><span class="s-id">textarea#processed</span><TextArea {...processor(substitutions.rules, { trusted: true })} bind:value={taProcessed} id="processed" rows={3} placeholder="type --> <-- :) <3 :( -- to morph" /></div>
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

    <div class="s-row">
      <div class="s-rowname">color</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">color#default</span><ColorField bind:value={colDefault} id="default" /></div>
        <div class="s-cell"><span class="s-id">color#disabled</span><ColorField bind:value={colDisabled} id="disabled" disabled /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">file</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">file#default</span><FileField bind:files={fDefault} id="default" /></div>
        <div class="s-cell"><span class="s-id">file#multiple</span><FileField bind:files={fMultiple} id="multiple" multiple /></div>
        <div class="s-cell"><span class="s-id">file#disabled</span><FileField bind:files={fDisabled} id="disabled" disabled /></div>
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

    <div class="s-row">
      <div class="s-rowname">led</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">led#default</span><Led states={ledStates} state="ON" id="default" /></div>
        <div class="s-cell"><span class="s-id">led#off</span><Led states={ledStates} state="OFF" id="off" /></div>
        <div class="s-cell"><span class="s-id">led#pulse</span><Led states={ledStates} state="ERR" pulse id="pulse" /></div>
        <div class="s-cell"><span class="s-id">led#unknown</span><Led states={ledStates} state="???" id="unknown" /></div>
      </div>
    </div>

    <div class="s-section-title">Navigation</div>

    <div class="s-row">
      <div class="s-rowname">link</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">link#default</span><Link href="#settings" text="Settings" id="default" /></div>
        <div class="s-cell"><span class="s-id">link#external</span><Link href="https://xgen.example" text="xgen.example" external ariaLabel="XGen site (opens externally)" id="external" /></div>
        <div class="s-cell"><span class="s-id">link#disabled</span><Link href="#x" text="Unavailable" disabled id="disabled" /></div>
      </div>
    </div>
  </div>
</div>

<!-- DI · composite — first occupant: status-indicator (M-RP2.22) -->
<div class="sampler-panel" class:hidden={activeTab !== 'di-composite'}>
  <div class="sampler-body">
    <div class="s-section-title">Composite</div>

    <div class="s-row">
      <div class="s-rowname">status-indicator</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">status-indicator#default</span><StatusIndicator states={ledStates} state="ON" caption="Connected" id="default" /></div>
        <div class="s-cell"><span class="s-id">status-indicator#withlink</span><StatusIndicator states={ledStates} state="OFF" caption="Disconnected" linkHref="https://xgen.example/status" linkText="Status page" linkExternal id="withlink" /></div>
        <div class="s-cell"><span class="s-id">status-indicator#pulse</span><StatusIndicator states={ledStates} state="ERR" pulse caption="Error" linkHref="#logs" linkText="View logs →" id="pulse" /></div>
      </div>
    </div>
  </div>
</div>

<!-- DD · atomic -->
<div class="sampler-panel" class:hidden={activeTab !== 'dd-atomic'}>
  <div class="sampler-body">
    <div class="s-empty">
      <strong>No components yet</strong>
      <span>Atomic data-derived components land here (downstream of the di catalogue).</span>
    </div>
  </div>
</div>

<!-- DD · composite -->
<div class="sampler-panel" class:hidden={activeTab !== 'dd-composite'}>
  <div class="sampler-body">
    <div class="s-empty">
      <strong>No components yet</strong>
      <span>Composite data-derived components land here (downstream of dd atomics).</span>
    </div>
  </div>
</div>
