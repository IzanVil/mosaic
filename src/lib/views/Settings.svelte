<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import { FolderPlus, Trash2 } from '@lucide/svelte';

  import EmptyState from '../components/EmptyState.svelte';
  import ScanSummaryBar from '../components/ScanSummaryBar.svelte';
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
</script>

<section class="flex flex-1 flex-col">
  {#if $lastScan}
    <ScanSummaryBar summary={$lastScan} />
  {/if}

  <div class="flex flex-1 flex-col gap-4 p-6">
    <header class="flex items-center justify-between gap-4">
      <div>
        <h2 class="text-base font-medium text-content-strong">Rutas de escaneo</h2>
        <p class="text-sm text-content-muted">
          Carpetas donde Mosaic busca proyectos, hasta cuatro niveles de profundidad.
        </p>
      </div>

      <button
        type="button"
        onclick={pickFolder}
        disabled={picking}
        class="inline-flex shrink-0 items-center gap-2 rounded-md bg-accent px-3 py-2 text-sm
               font-medium text-surface-0 transition hover:bg-accent-strong
               disabled:opacity-60 focus-visible:outline-2 focus-visible:outline-offset-2
               focus-visible:outline-accent"
      >
        <FolderPlus size={16} />
        Añadir carpeta
      </button>
    </header>

    {#if $scanPathsError}
      <p
        class="rounded-md border border-surface-border bg-surface-1 px-4 py-3 text-sm text-content"
        role="alert"
      >
        {$scanPathsError}
      </p>
    {/if}

    {#if $loadingScanPaths && $scanPaths.length === 0}
      <p class="py-8 text-center text-sm text-content-muted">Cargando rutas…</p>
    {:else if $scanPaths.length === 0}
      <EmptyState
        title="Ninguna ruta configurada"
        description="Elige la carpeta donde guardas tus proyectos para empezar."
        actionLabel="Añadir carpeta"
        onAction={pickFolder}
      />
    {:else}
      <ul class="divide-y divide-surface-border rounded-md border border-surface-border">
        {#each $scanPaths as scanPath (scanPath.id)}
          <li class="flex items-center gap-4 px-4 py-3">
            <label class="flex min-w-0 flex-1 items-center gap-3">
              <input
                type="checkbox"
                checked={scanPath.enabled}
                onchange={(event) =>
                  setScanPathEnabled(scanPath.id, event.currentTarget.checked)}
                class="size-4 shrink-0 accent-accent"
              />
              <span class="min-w-0">
                <span
                  class="block truncate font-mono text-sm"
                  class:text-content-strong={scanPath.enabled}
                  class:text-content-muted={!scanPath.enabled}
                  title={scanPath.path}
                >
                  {scanPath.path}
                </span>
                <span class="block text-xs text-content-muted">
                  Último escaneo: {formatRelativeTime(scanPath.last_scan_at)}
                </span>
              </span>
            </label>

            <button
              type="button"
              onclick={() => removeScanPath(scanPath.id)}
              title="Eliminar la ruta (los proyectos ya descubiertos se conservan)"
              aria-label="Eliminar la ruta {scanPath.path}"
              class="shrink-0 rounded-md p-2 text-content-muted transition hover:bg-surface-2
                     hover:text-content-strong focus-visible:outline-2
                     focus-visible:outline-offset-2 focus-visible:outline-accent"
            >
              <Trash2 size={16} />
            </button>
          </li>
        {/each}
      </ul>

      <p class="text-xs text-content-muted">
        Deshabilitar una ruta la excluye de los escaneos sin borrarla. Eliminarla no borra los
        proyectos ya descubiertos: conservan sus etiquetas y notas.
      </p>
    {/if}

    <section class="flex flex-col gap-3 border-t border-surface-border pt-6">
      <div>
        <h2 class="text-base font-medium text-content-strong">Aplicaciones</h2>
        <p class="text-sm text-content-muted">
          Con qué se abren los proyectos. Mosaic detecta lo que hay instalado; si no eliges nada,
          usa el primero de la lista.
        </p>
      </div>

      {#if $appsError}
        <p
          class="rounded-md border border-surface-border bg-surface-1 px-4 py-3 text-sm text-content"
          role="alert"
        >
          {$appsError}
        </p>
      {/if}

      {#each APP_SECTIONS as section (section.kind)}
        {@const available = section.kind === 'ide' ? $ides : $terminals}
        {@const selected = section.kind === 'ide' ? $preferredIde : $preferredTerminal}

        <label class="flex items-center gap-4">
          <span class="w-24 shrink-0 text-sm text-content">{section.title}</span>

          {#if available.length === 0}
            <span class="text-sm text-content-muted">{section.empty}</span>
          {:else}
            <select
              value={selected}
              onchange={(event) => setPreferredApp(section.kind, event.currentTarget.value)}
              class="rounded-md border border-surface-border bg-surface-0 px-3 py-1.5 text-sm
                     text-content focus-visible:outline-2 focus-visible:outline-offset-2
                     focus-visible:outline-accent"
            >
              <option value="">Automático ({available[0]?.name})</option>
              {#each available as app (app.id)}
                <option value={app.id}>{app.name}</option>
              {/each}
            </select>
          {/if}
        </label>
      {/each}

      <p class="text-xs text-content-muted">
        El explorador de archivos no se configura aquí: se usa el que tenga asociado el sistema.
      </p>
    </section>
  </div>
</section>
