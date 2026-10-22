<script lang="ts">
  import { onMount } from 'svelte';
  import { ChevronDown, FolderPlus, Trash2 } from '@lucide/svelte';

  import { chooseFolder } from '../api/dialog';
  import BackupSection from '../components/BackupSection.svelte';
  import EmptyState from '../components/EmptyState.svelte';
  import ExcludedDirsEditor from '../components/ExcludedDirsEditor.svelte';
  import ScanSummaryBar from '../components/ScanSummaryBar.svelte';
  import SettingRow from '../components/SettingRow.svelte';
  import {
    advancedSettings,
    advancedSettingsError,
    loadAdvancedSettings,
    saveAdvancedField,
  } from '../stores/advancedSettings';
  import {
    appsError,
    ides,
    preferredIde,
    preferredTerminal,
    setPreferredApp,
    terminals,
  } from '../stores/apps';
  import { lastScan } from '../stores/projects';
  import {
    addScanPath,
    loadingScanPaths,
    removeScanPath,
    scanPaths,
    scanPathsError,
    setScanPathEnabled,
  } from '../stores/scanPaths';
  import { lang, LANG_PREFERENCES, langPreference, systemLang, t, type Key } from '../i18n';
  import { formatRelativeTime } from '../utils/format';

  let picking = $state(false);

  const APP_SECTIONS = [
    { kind: 'ide', title: 'comun.editor', empty: 'ajustes.sinEditor' },
    { kind: 'terminal', title: 'comun.terminal', empty: 'ajustes.sinTerminal' },
  ] as const satisfies readonly { kind: string; title: Key; empty: Key }[];

  /** Abre el selector nativo de carpetas y registra la ruta elegida. */
  async function pickFolder() {
    picking = true;
    try {
      const selected = await chooseFolder();
      if (selected !== null) await addScanPath(selected);
    } finally {
      picking = false;
    }
  }

  /** Opciones del refresco de Git, en minutos. `0` lo desactiva. */
  const GIT_INTERVALS = [0, 1, 5, 15, 30, 60] as const;

  function intervalLabel(minutes: number): string {
    if (minutes === 0) return $t('ajustes.gitDesactivado');
    if (minutes === 1) return $t('ajustes.gitCadaMinuto');
    if (minutes === 60) return $t('ajustes.gitCadaHora');
    return $t('ajustes.gitCadaMinutos', { n: minutes });
  }

  onMount(() => {
    void loadAdvancedSettings();
  });

  /** Guarda un número al confirmarlo; si el backend lo rechaza, el campo vuelve a lo guardado. */
  async function saveNumber(
    field: 'max_depth' | 'max_entries_per_scan',
    input: HTMLInputElement,
  ) {
    const previous = $advancedSettings?.current[field];
    const value = Math.round(input.valueAsNumber);
    if (Number.isNaN(value) || value === previous) {
      if (previous !== undefined) input.value = String(previous);
      return;
    }
    const ok = await saveAdvancedField(field, value);
    if (!ok && previous !== undefined) input.value = String(previous);
  }
</script>

<section class="ajustes">
  {#if $lastScan}
    <ScanSummaryBar summary={$lastScan} />
  {/if}

  <div class="contenido">
    <label class="app idioma">
      <span class="etiqueta">{$t('idioma.titulo')}</span>
      <span class="desplegable">
        <select bind:value={$langPreference}>
          {#each LANG_PREFERENCES as preference (preference)}
            <option value={preference}>
              {preference === 'system' ? `${$t('idioma.sistema')} (${$t(`idioma.${systemLang()}`)})` : $t(`idioma.${preference}`)}
            </option>
          {/each}
        </select>
        <ChevronDown size={14} strokeWidth={1.75} class="flecha" />
      </span>
    </label>

    <header class="cabecera">
      <div>
        <h2>{$t('ajustes.rutasTitulo')}</h2>
        <p class="explica">
          {$advancedSettings
            ? $t('ajustes.rutasTexto', { n: $advancedSettings.current.max_depth })
            : $t('ajustes.rutasTextoCorto')}
        </p>
      </div>

      <button type="button" class="primario" onclick={pickFolder} disabled={picking}>
        <FolderPlus size={16} strokeWidth={1.75} />
        {$t('ajustes.anadirCarpeta')}
      </button>
    </header>

    {#if $scanPathsError}
      <p class="error" role="alert">{$scanPathsError}</p>
    {/if}

    {#if $loadingScanPaths && $scanPaths.length === 0}
      <p class="cargando">{$t('ajustes.cargandoRutas')}</p>
    {:else if $scanPaths.length === 0}
      <EmptyState
        title={$t('ajustes.sinRutasTitulo')}
        description={$t('ajustes.sinRutasTexto')}
        actionLabel={$t('ajustes.anadirCarpeta')}
        onAction={pickFolder}
      />
    {:else}
      <ul class="rutas">
        {#each $scanPaths as scanPath (scanPath.id)}
          <li>
            <label class="ruta">
              <input
                type="checkbox"
                checked={scanPath.enabled}
                onchange={(event) =>
                  setScanPathEnabled(scanPath.id, event.currentTarget.checked)}
              />
              <span class="textos">
                <span class="camino" class:apagada={!scanPath.enabled} title={scanPath.path}>
                  {scanPath.path}
                </span>
                <span class="cuando">
                  {$t('ajustes.ultimoEscaneo', { cuando: formatRelativeTime(scanPath.last_scan_at, $lang) })}
                </span>
              </span>
            </label>

            <button
              type="button"
              class="icono"
              onclick={() => removeScanPath(scanPath.id)}
              title={$t('ajustes.eliminarRuta')}
              aria-label={$t('ajustes.eliminarRutaNombre', { ruta: scanPath.path })}
            >
              <Trash2 size={16} strokeWidth={1.75} />
            </button>
          </li>
        {/each}
      </ul>

      <p class="nota">
        {$t('ajustes.rutasNota')}
      </p>
    {/if}

    {#if $advancedSettingsError}
      <p class="error" role="alert">{$t('ajustes.errorLeer', { error: $advancedSettingsError })}</p>
    {:else if $advancedSettings}
      {@const actual = $advancedSettings.current}
      {@const limites = $advancedSettings.limits}

      <section class="bloque" aria-labelledby="titulo-escaneo">
        <div>
          <h2 id="titulo-escaneo">{$t('ajustes.escaneoTitulo')}</h2>
          <p class="explica">{$t('ajustes.escaneoTexto')}</p>
        </div>

        <div class="ajustes-lista">
          <SettingRow
            label={$t('ajustes.profundidad')}
            controlId="ajuste-profundidad"
            field="max_depth"
            effect={$t('ajustes.profundidadEfecto', { min: limites.max_depth_min, max: limites.max_depth_max })}
          >
            <input
              id="ajuste-profundidad"
              class="numero"
              type="number"
              min={limites.max_depth_min}
              max={limites.max_depth_max}
              value={actual.max_depth}
              onchange={(event) => saveNumber('max_depth', event.currentTarget)}
            />
          </SettingRow>

          <SettingRow
            label={$t('ajustes.tope')}
            controlId="ajuste-tope"
            field="max_entries_per_scan"
            effect={$t('ajustes.topeEfecto')}
          >
            <input
              id="ajuste-tope"
              class="numero ancho"
              type="number"
              min={limites.max_entries_min}
              max={limites.max_entries_max}
              step="1000"
              value={actual.max_entries_per_scan}
              onchange={(event) => saveNumber('max_entries_per_scan', event.currentTarget)}
            />
          </SettingRow>

          <SettingRow
            label={$t('ajustes.arranque')}
            controlId="ajuste-arranque"
            field="scan_on_startup"
            effect={$t('ajustes.arranqueEfecto')}
          >
            <input
              id="ajuste-arranque"
              class="interruptor"
              type="checkbox"
              checked={actual.scan_on_startup}
              onchange={(event) => void saveAdvancedField('scan_on_startup', event.currentTarget.checked)}
            />
          </SettingRow>

          <SettingRow
            label={$t('ajustes.excluidas')}
            field="excluded_dirs"
            effect={$t('ajustes.excluidasEfecto')}
          >
            <ExcludedDirsEditor
              dirs={actual.excluded_dirs}
              defaults={$advancedSettings.defaults.excluded_dirs}
              max={limites.excluded_dirs_max}
              onChange={(dirs) => saveAdvancedField('excluded_dirs', dirs)}
            />
          </SettingRow>
        </div>
      </section>

      <section class="bloque" aria-labelledby="titulo-git">
        <div>
          <h2 id="titulo-git">{$t('app.git')}</h2>
          <p class="explica">{$t('ajustes.gitTexto')}</p>
        </div>

        <div class="ajustes-lista">
          <SettingRow
            label={$t('ajustes.refresco')}
            controlId="ajuste-git"
            field="git_refresh_interval_minutes"
            effect={$t('ajustes.refrescoEfecto')}
          >
            <span class="desplegable">
              <select
                id="ajuste-git"
                value={actual.git_refresh_interval_minutes}
                onchange={(event) =>
                  void saveAdvancedField('git_refresh_interval_minutes', Number(event.currentTarget.value))}
              >
                {#each GIT_INTERVALS as minutes (minutes)}
                  <option value={minutes}>{intervalLabel(minutes)}</option>
                {/each}
                {#if !GIT_INTERVALS.some((minutes) => minutes === actual.git_refresh_interval_minutes)}
                  <!-- Un valor guardado a mano que no está en la lista se enseña tal cual. -->
                  <option value={actual.git_refresh_interval_minutes}>
                    {intervalLabel(actual.git_refresh_interval_minutes)}
                  </option>
                {/if}
              </select>
              <ChevronDown size={14} strokeWidth={1.75} class="flecha" />
            </span>
          </SettingRow>
        </div>
      </section>
    {/if}

    <section class="bloque">
      <div>
        <h2>{$t('ajustes.appsTitulo')}</h2>
        <p class="explica">
          {$t('ajustes.appsTexto')}
        </p>
      </div>

      {#if $appsError}
        <p class="error" role="alert">{$appsError}</p>
      {/if}

      {#each APP_SECTIONS as section (section.kind)}
        {@const available = section.kind === 'ide' ? $ides : $terminals}
        {@const selected = section.kind === 'ide' ? $preferredIde : $preferredTerminal}

        <label class="app">
          <span class="etiqueta">{$t(section.title)}</span>

          {#if available.length === 0}
            <span class="explica">{$t(section.empty)}</span>
          {:else}
            <span class="desplegable">
              <select
                value={selected}
                onchange={(event) => setPreferredApp(section.kind, event.currentTarget.value)}
              >
                <option value="">{$t('ajustes.automatico', { nombre: available[0]?.name ?? '' })}</option>
                {#each available as app (app.id)}
                  <option value={app.id}>{app.name}</option>
                {/each}
              </select>
              <ChevronDown size={14} strokeWidth={1.75} class="flecha" />
            </span>
          {/if}
        </label>
      {/each}

      <p class="nota">
        {$t('ajustes.explorador')}
      </p>
    </section>

    <BackupSection />
  </div>
</section>

<style>
  .ajustes {
    display: flex;
    flex: 1;
    flex-direction: column;
    min-height: 100%;
    background: var(--surface-base);
  }

  .contenido {
    display: flex;
    flex: 1;
    flex-direction: column;
    gap: var(--space-4);
    width: 100%;
    max-width: 48rem;
    box-sizing: border-box;
    padding: var(--space-5);
  }

  .cabecera {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
  }

  h2 {
    margin: 0;
    font-size: var(--text-section);
    line-height: var(--text-section-lh);
    font-weight: var(--text-section-weight);
    letter-spacing: var(--tracking-tight);
    color: var(--text-primary);
  }

  .explica {
    margin: var(--space-1) 0 0;
    font-size: var(--text-body);
    line-height: var(--text-body-lh);
    color: var(--text-secondary);
  }

  .nota {
    margin: 0;
    font-size: var(--text-meta);
    line-height: var(--text-meta-lh);
    color: var(--text-tertiary);
  }

  .cargando {
    margin: 0;
    padding: var(--space-6) 0;
    font-size: var(--text-body);
    text-align: center;
    color: var(--text-tertiary);
  }

  .error {
    margin: 0;
    padding: var(--space-3) var(--space-4);
    border: 1px solid color-mix(in oklab, var(--danger) 40%, transparent);
    border-radius: var(--radius-md);
    background: var(--danger-surface);
    font-size: var(--text-body);
    line-height: var(--text-body-lh);
    color: var(--danger);
  }

  .primario {
    display: inline-flex;
    flex: none;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    border: 0;
    border-radius: var(--radius-md);
    background: var(--accent);
    font: inherit;
    font-size: var(--text-body);
    line-height: var(--text-body-lh);
    font-weight: 500;
    color: var(--text-on-accent);
    cursor: pointer;
    transition: background var(--duration-fast) var(--ease-out);
  }
  .primario:hover:not(:disabled) {
    background: var(--accent-hover);
  }
  .primario:disabled {
    opacity: 0.6;
    cursor: progress;
  }

  .rutas {
    margin: 0;
    padding: 0;
    list-style: none;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    background: var(--surface-raised);
  }
  .rutas li {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    padding: var(--space-3) var(--space-4);
  }
  .rutas li + li {
    border-top: 1px solid var(--border-subtle);
  }

  .ruta {
    display: flex;
    flex: 1;
    min-width: 0;
    align-items: center;
    gap: var(--space-3);
    cursor: pointer;
  }
  .ruta input {
    flex: none;
    width: 16px;
    height: 16px;
    margin: 0;
    accent-color: var(--accent);
  }

  .textos {
    min-width: 0;
  }

  .camino {
    display: block;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-family: var(--font-mono);
    font-size: var(--text-code);
    line-height: var(--text-code-lh);
    color: var(--text-primary);
  }
  .camino.apagada {
    color: var(--text-tertiary);
  }

  .cuando {
    display: block;
    font-size: var(--text-meta);
    line-height: var(--text-meta-lh);
    color: var(--text-tertiary);
  }

  .icono {
    display: inline-flex;
    flex: none;
    padding: var(--space-2);
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-tertiary);
    cursor: pointer;
    transition:
      background var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out);
  }
  .icono:hover {
    background: var(--surface-sunken);
    color: var(--text-primary);
  }

  .ajustes-lista {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .numero {
    width: 6rem;
    padding: 6px var(--space-3);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    background: var(--surface-raised);
    font: inherit;
    font-size: var(--text-body);
    font-variant-numeric: tabular-nums;
    color: var(--text-primary);
  }
  .numero.ancho {
    width: 9rem;
  }

  .interruptor {
    width: 18px;
    height: 18px;
    margin: 0;
    accent-color: var(--accent);
    cursor: pointer;
  }

  .bloque {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    margin-top: var(--space-2);
    padding-top: var(--space-5);
    border-top: 1px solid var(--border-subtle);
  }

  .app {
    display: flex;
    align-items: center;
    gap: var(--space-4);
  }
  /* El idioma va primero y aparte: es lo que busca quien no entiende el resto. */
  .app.idioma {
    padding-bottom: var(--space-4);
    border-bottom: 1px solid var(--border-subtle);
  }
  .app.idioma .etiqueta {
    width: auto;
    white-space: nowrap;
  }

  .etiqueta {
    flex: none;
    width: 6rem;
    font-size: var(--text-body);
    line-height: var(--text-body-lh);
    color: var(--text-secondary);
  }

  /* Sin `appearance: none`, WebKitGTK pinta el control del tema GTK: fondo blanco con el texto
     claro de la app, ilegible. La flecha es un icono porque la nativa se va con él. */
  .desplegable {
    position: relative;
    display: inline-flex;
    align-items: center;
  }
  .desplegable :global(.flecha) {
    position: absolute;
    right: var(--space-2);
    color: var(--text-tertiary);
    pointer-events: none;
  }

  select {
    appearance: none;
    padding: 6px calc(var(--space-3) + 18px) 6px var(--space-3);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    background: var(--surface-raised);
    font: inherit;
    font-size: var(--text-body);
    line-height: var(--text-body-lh);
    color: var(--text-primary);
    cursor: pointer;
  }

  button:focus-visible,
  select:focus-visible,
  input:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
</style>
