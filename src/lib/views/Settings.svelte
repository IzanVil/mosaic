<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import { onMount } from 'svelte';
  import { ChevronDown, FolderPlus, Trash2 } from '@lucide/svelte';

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
  import { formatRelativeTime } from '../utils/format';

  let picking = $state(false);

  const APP_SECTIONS = [
    {
      kind: 'ide',
      title: 'Editor',
      empty: 'No se ha encontrado ningún editor reconocido en el PATH.',
    },
    {
      kind: 'terminal',
      title: 'Terminal',
      empty: 'No se ha encontrado ningún emulador de terminal reconocido en el PATH.',
    },
  ] as const;

  /** Abre el selector nativo de carpetas y registra la ruta elegida. */
  async function pickFolder() {
    picking = true;
    try {
      const selected = await open({ directory: true, multiple: false });
      if (typeof selected === 'string') {
        await addScanPath(selected);
      }
    } finally {
      picking = false;
    }
  }

  /** Opciones del refresco de Git. `0` lo desactiva. */
  const GIT_INTERVALS = [
    { value: 0, label: 'Desactivado' },
    { value: 1, label: 'Cada minuto' },
    { value: 5, label: 'Cada 5 minutos' },
    { value: 15, label: 'Cada 15 minutos' },
    { value: 30, label: 'Cada 30 minutos' },
    { value: 60, label: 'Cada hora' },
  ] as const;

  onMount(() => {
    void loadAdvancedSettings();
  });

  /**
   * Guarda un número al confirmarlo (Enter o salir del campo). Si el backend
   * lo rechaza, el campo vuelve a enseñar el valor que sigue guardado.
   */
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
    <header class="cabecera">
      <div>
        <h2>Rutas de escaneo</h2>
        <p class="explica">
          Carpetas donde Mosaic busca proyectos, hasta cuatro niveles de profundidad.
        </p>
      </div>

      <button type="button" class="primario" onclick={pickFolder} disabled={picking}>
        <FolderPlus size={16} strokeWidth={1.75} />
        Añadir carpeta
      </button>
    </header>

    {#if $scanPathsError}
      <p class="error" role="alert">{$scanPathsError}</p>
    {/if}

    {#if $loadingScanPaths && $scanPaths.length === 0}
      <p class="cargando">Cargando rutas…</p>
    {:else if $scanPaths.length === 0}
      <EmptyState
        title="Ninguna ruta configurada"
        description="Elige la carpeta donde guardas tus proyectos para empezar."
        actionLabel="Añadir carpeta"
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
                  Último escaneo: {formatRelativeTime(scanPath.last_scan_at)}
                </span>
              </span>
            </label>

            <button
              type="button"
              class="icono"
              onclick={() => removeScanPath(scanPath.id)}
              title="Eliminar la ruta (los proyectos ya descubiertos se conservan)"
              aria-label="Eliminar la ruta {scanPath.path}"
            >
              <Trash2 size={16} strokeWidth={1.75} />
            </button>
          </li>
        {/each}
      </ul>

      <p class="nota">
        Deshabilitar una ruta la excluye de los escaneos sin borrarla. Eliminarla no borra los
        proyectos ya descubiertos: conservan sus etiquetas y notas.
      </p>
    {/if}

    {#if $advancedSettingsError}
      <p class="error" role="alert">No se pudieron leer los ajustes: {$advancedSettingsError}</p>
    {:else if $advancedSettings}
      {@const actual = $advancedSettings.current}
      {@const limites = $advancedSettings.limits}

      <section class="bloque" aria-labelledby="titulo-escaneo">
        <div>
          <h2 id="titulo-escaneo">Escaneo</h2>
          <p class="explica">Cómo recorre Mosaic tus carpetas. Cada cambio se guarda solo.</p>
        </div>

        <div class="ajustes-lista">
          <SettingRow
            label="Profundidad"
            controlId="ajuste-profundidad"
            field="max_depth"
            effect={`Niveles por debajo de cada ruta, de ${limites.max_depth_min} a ${limites.max_depth_max}. Se aplica en el próximo escaneo.`}
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
            label="Tope de entradas"
            controlId="ajuste-tope"
            field="max_entries_per_scan"
            effect="Si una ruta tiene más, el escaneo se detiene y avisa. Se aplica en el próximo escaneo."
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
            label="Escanear al arrancar"
            controlId="ajuste-arranque"
            field="scan_on_startup"
            effect="Recorre las rutas ocho segundos después de abrir Mosaic. Se aplica la próxima vez que lo abras."
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
            label="Carpetas excluidas"
            field="excluded_dirs"
            effect="Nombres de carpeta que el escáner salta allí donde aparezcan. Se aplica en el próximo escaneo."
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
          <h2 id="titulo-git">Git</h2>
          <p class="explica">Mosaic relee el estado de tus repositorios en segundo plano, sin red.</p>
        </div>

        <div class="ajustes-lista">
          <SettingRow
            label="Refresco automático"
            controlId="ajuste-git"
            field="git_refresh_interval_minutes"
            effect="Se aplica en el acto: al cambiarlo, Mosaic relee Git y empieza a contar de nuevo."
          >
            <span class="desplegable">
              <select
                id="ajuste-git"
                value={actual.git_refresh_interval_minutes}
                onchange={(event) =>
                  void saveAdvancedField('git_refresh_interval_minutes', Number(event.currentTarget.value))}
              >
                {#each GIT_INTERVALS as option (option.value)}
                  <option value={option.value}>{option.label}</option>
                {/each}
                {#if !GIT_INTERVALS.some((option) => option.value === actual.git_refresh_interval_minutes)}
                  <!-- Un valor guardado a mano que no está en la lista se enseña tal cual. -->
                  <option value={actual.git_refresh_interval_minutes}>
                    Cada {actual.git_refresh_interval_minutes} minutos
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
        <h2>Aplicaciones</h2>
        <p class="explica">
          Con qué se abren los proyectos. Mosaic detecta lo que hay instalado; si no eliges nada,
          usa el primero de la lista.
        </p>
      </div>

      {#if $appsError}
        <p class="error" role="alert">{$appsError}</p>
      {/if}

      {#each APP_SECTIONS as section (section.kind)}
        {@const available = section.kind === 'ide' ? $ides : $terminals}
        {@const selected = section.kind === 'ide' ? $preferredIde : $preferredTerminal}

        <label class="app">
          <span class="etiqueta">{section.title}</span>

          {#if available.length === 0}
            <span class="explica">{section.empty}</span>
          {:else}
            <span class="desplegable">
              <select
                value={selected}
                onchange={(event) => setPreferredApp(section.kind, event.currentTarget.value)}
              >
                <option value="">Automático ({available[0]?.name})</option>
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
        El explorador de archivos no se configura aquí: se usa el que tenga asociado el sistema.
      </p>
    </section>
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

  .etiqueta {
    flex: none;
    width: 6rem;
    font-size: var(--text-body);
    line-height: var(--text-body-lh);
    color: var(--text-secondary);
  }

  /*
   * Sin `appearance: none`, WebKitGTK pinta el control nativo del tema GTK:
   * fondo blanco con el texto claro de la aplicación encima, ilegible. La
   * flecha la pone el icono, porque con la apariencia nativa se va también la
   * suya.
   */
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
