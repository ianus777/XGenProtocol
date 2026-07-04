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
  import PasswordField from '$core/components/data-independent/password-field.svelte';
  import StarRating from '$core/components/data-independent/star-rating.svelte';
  import FileFieldComposite from '$core/components/data-independent/file-field.svelte';
  import Combobox from '$core/components/data-independent/combobox.svelte';
  import Chip from '$core/components/data-independent/chip.svelte';
  import TagSelect from '$core/components/data-independent/tag-select.svelte';
  import ColorPicker from '$core/components/data-independent/color-picker.svelte';
  import ConverterField from '$core/components/data-independent/converter-field.svelte';
  import Meter from '$core/components/data-independent/meter.svelte';

  // Processor (common infra, M-RP4.0/M-RP4.2): the kind-1 transformer attachment, fed from the
  // source-agnostic substitutions store. The atomic forwards {...rest}; processor(...) returns a
  // symbol-keyed attachment that lands on the inner <textarea>. Adds NO registry entry.
  // The store is hydrated on mount from the sampler host's get_substitutions Tauri command
  // (xgen-sampler_config.toml [substitutions], generated from the seed on start — the REAL load
  // path, mirroring app_client.svelte / J-437; M-RP4.4 / D-101). No literal seed.
  import { processor } from '$common/components/processor/processor';
  import { clamp } from '$common/components/processor/clamp';
  import { intlNumber } from '$common/components/processor/transform';
  import { substitutions } from '$common/components/processor/store.svelte';
  import SubstitutionsEditor from '$common/components/widgets/substitutions-editor.svelte';

  // Runtime client<->node skin-swap (D-098): flipping [data-shell] re-aliases --accent*
  // live, so the whole grid re-themes at once. Replaces "run in both real shells".
  let shell = $state('client');
  function applyShell(s) { shell = s; document.documentElement.dataset.shell = s; }

  // Hydrate the user-owned substitution pairs from the sampler host's REAL load path (M-RP4.4):
  // xgen-sampler_config.toml [substitutions] -> get_substitutions -> store. Mirrors
  // app_client.svelte (J-437). Async, so the processor-host cell starts with empty rules and
  // re-attaches when setRules lands (source-agnostic store + attachment lifecycle, D-099).
  onMount(async () => {
    applyShell('client');
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      substitutions.setRules(await invoke('get_substitutions'));
    } catch (_) {
      // Outside Tauri (plain-browser Vite preview) — store stays empty; the morph's canonical
      // home is the Tauri window (D-098), where CDP verifies it.
    }
  });

  // Tab container (M-RP3.2): outer class x arity axis. Inactive panels are CSS-hidden,
  // NOT unmounted (class:hidden), so every component stays registered for CDP `ids()`.
  let activeTab = $state('di-atomic'); // 'di-atomic' | 'di-composite' | 'dd-atomic' | 'dd-composite'
  const tabs = [
    { id: 'di-atomic',    label: 'DI · atomic' },
    { id: 'di-composite', label: 'DI · composite' },
    { id: 'dd-atomic',    label: 'DD · atomic' },
    { id: 'dd-composite', label: 'DD · composite' },
    { id: 'widget',       label: 'WIDGET' },
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
  let numClamped = $state(5); // clamp-host (M-RP4.1): kind-3 guard coerces to [0,10] on change
  // converter-field (M-RP4.5): kind-2 bridge. Typed value = number; display = Intl-formatted string.
  // Stable converter identity (defined once, NOT inline) so the prop doesn't churn per render.
  const numConv = intlNumber({ maximumFractionDigits: 2 });
  let cvDefault = $state(1234.5); // shows "1,234.5"; edit raw, blur reformats
  let cvDisabled = $state(99.9);
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
  // password-field (M-RP2.23) — string bind:value; the child textfield is redactValue'd.
  let pwDefault = $state('hunter2');
  let pwDisabled = $state('inert');
  let pwRevealed = $state('shown');
  // star-rating (M-RP2.24) — numeric bind:value; 0 = unrated. readonly cell shows a fixed rating.
  let srDefault = $state(0);
  let srRated = $state(3);
  let srReadonly = $state(4);
  // file-field (M-RP2.25) — FileList|null; unsettable from markup (browser security), all start null.
  let ffDefault = $state(null);
  let ffMultiple = $state(null);
  let ffDisabled = $state(null);
  // combobox (M-RP2.26) — rich owned-popup; string bind:value = shown label. preset seeded.
  let cbDefault = $state('');
  let cbPreset = $state('Online');
  let cbDisabled = $state('');
  // chip (M-RP2.27) — display token, no bind. onRemove spy for CDP.
  let chipRemoved = $state(0);
  // tag-select (M-RP2.28) — string[] bind:value. default seeds 4 (proves +N overflow at
  // COLLAPSE_AT=3); max caps at 2 (3rd pick no-ops + data-full dim); create allows freeform.
  let tsDefault = $state(['important', 'work', 'personal', 'urgent']);
  let tsMax = $state(['important', 'work']);
  let tsCreate = $state([]);
  let tsManaged = $state(0); // gear spy (onManage) for CDP
  // color-picker (M-RP2.29) — string bind:value #rrggbbaa; seeded with the per-shell accents (padded ff).
  let cpDefault = $state('#9a6a30ff');
  let cpDisabled = $state('#2a6090ff');
  const tagOptions = ['important', 'work', 'personal', 'urgent', 'later'];
  const cbOptions = [
    { value: 'online', label: 'Online', status: 'active' },
    { value: 'away', label: 'Away', status: 'idle' },
    { value: 'busy', label: 'Busy', status: 'DND' },
    { value: 'offline', label: 'Offline', status: 'off', disabled: true },
  ];

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
        <div class="s-cell"><span class="s-id">textarea#processed</span><TextArea {...processor(substitutions.rules, { trusted: true })} bind:value={taProcessed} id="processed" rows={3} placeholder="type -> <- :) <3 :( -- to morph" /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">number</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">number#default</span><NumberField bind:value={numDefault} id="default" /></div>
        <div class="s-cell"><span class="s-id">number#disabled</span><NumberField bind:value={numDisabled} id="disabled" disabled /></div>
        <div class="s-cell"><span class="s-id">number#invalid</span><NumberField bind:value={numInvalid} id="invalid" min={0} max={10} /></div>
        <div class="s-cell"><span class="s-id">number#clamped</span><NumberField {...clamp({ min: 0, max: 10 })} bind:value={numClamped} id="clamped" min={0} max={10} /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">converter-field</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">converter-field#default</span><ConverterField converter={numConv} bind:value={cvDefault} id="default" placeholder="number" /></div>
        <div class="s-cell"><span class="s-id">converter-field#disabled</span><ConverterField converter={numConv} bind:value={cvDisabled} id="disabled" disabled /></div>
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

    <div class="s-row">
      <div class="s-rowname">meter</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">meter#optimum</span><Meter value={35} min={0} max={100} optimum={20} low={50} high={80} id="optimum" /></div>
        <div class="s-cell"><span class="s-id">meter#caution</span><Meter value={65} min={0} max={100} optimum={20} low={50} high={80} id="caution" /></div>
        <div class="s-cell"><span class="s-id">meter#danger</span><Meter value={94} min={0} max={100} optimum={20} low={50} high={80} id="danger" /></div>
        <div class="s-cell"><span class="s-id">meter#neutral</span><Meter value={50} min={0} max={100} id="neutral" /></div>
        <div class="s-cell"><span class="s-id">meter#fixed</span><Meter value={70} min={0} max={100} optimum={20} low={50} high={80} width="120px" id="fixed" /></div>
        <div class="s-cell"><span class="s-id">meter#disabled</span><Meter value={40} min={0} max={100} disabled id="disabled" /></div>
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

    <div class="s-section-title">Token</div>

    <div class="s-row">
      <div class="s-rowname">chip</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">chip#default</span><Chip label="rust" id="default" onRemove={() => chipRemoved++} /></div>
        <div class="s-cell"><span class="s-id">chip#static</span><Chip label="svelte" removable={false} id="static" /></div>
        <div class="s-cell"><span class="s-id">chip#long</span><Chip label="a-very-long-keyword-that-truncates" id="long" onRemove={() => chipRemoved++} /></div>
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

    <div class="s-row">
      <div class="s-rowname">password-field</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">password-field#default</span><PasswordField bind:value={pwDefault} id="default" placeholder="password" autocomplete="current-password" /></div>
        <div class="s-cell"><span class="s-id">password-field#disabled</span><PasswordField bind:value={pwDisabled} id="disabled" disabled /></div>
        <div class="s-cell"><span class="s-id">password-field#revealed</span><PasswordField bind:value={pwRevealed} id="revealed" revealedByDefault /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">star-rating</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">star-rating#default</span><StarRating bind:value={srDefault} id="default" ariaLabel="Rate" /></div>
        <div class="s-cell"><span class="s-id">star-rating#rated</span><StarRating bind:value={srRated} id="rated" ariaLabel="Rate" /></div>
        <div class="s-cell"><span class="s-id">star-rating#readonly</span><StarRating bind:value={srReadonly} id="readonly" readonly ariaLabel="Rating" /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">file-field</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">file-field#default</span><FileFieldComposite bind:files={ffDefault} id="default" /></div>
        <div class="s-cell"><span class="s-id">file-field#multiple</span><FileFieldComposite bind:files={ffMultiple} id="multiple" multiple /></div>
        <div class="s-cell"><span class="s-id">file-field#disabled</span><FileFieldComposite bind:files={ffDisabled} id="disabled" disabled /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">combobox</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">combobox#default</span><Combobox bind:value={cbDefault} id="default" options={cbOptions} placeholder="Type or pick" /></div>
        <div class="s-cell"><span class="s-id">combobox#preset</span><Combobox bind:value={cbPreset} id="preset" options={cbOptions} /></div>
        <div class="s-cell"><span class="s-id">combobox#disabled</span><Combobox bind:value={cbDisabled} id="disabled" options={cbOptions} disabled /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">tag-select</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">tag-select#default</span><TagSelect bind:value={tsDefault} id="default" options={tagOptions} width="260px" onManage={() => tsManaged++} placeholder="Add tags" /></div>
        <div class="s-cell"><span class="s-id">tag-select#max</span><TagSelect bind:value={tsMax} id="max" options={tagOptions} max={2} onManage={() => tsManaged++} placeholder="max 2" /></div>
        <div class="s-cell"><span class="s-id">tag-select#create</span><TagSelect bind:value={tsCreate} id="create" options={tagOptions} allowCreate onManage={() => tsManaged++} placeholder="type + Enter" /></div>
      </div>
    </div>

    <div class="s-row">
      <div class="s-rowname">color-picker</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">color-picker#default</span><ColorPicker bind:value={cpDefault} id="default" /></div>
        <div class="s-cell"><span class="s-id">color-picker#disabled</span><ColorPicker bind:value={cpDisabled} id="disabled" disabled /></div>
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

<!-- WIDGET (Level-2 UI-plugin, D-102) -->
<div class="sampler-panel" class:hidden={activeTab !== 'widget'}>
  <div class="sampler-body">
    <div class="s-group">
      <div class="s-rowname">substitutions-editor</div>
      <div class="s-cells">
        <div class="s-cell"><span class="s-id">substitutions-editor#demo</span><SubstitutionsEditor id="demo" /></div>
      </div>
    </div>
  </div>
</div>
