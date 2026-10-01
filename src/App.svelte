<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { GitBranch, RefreshCw, Settings as SettingsIcon, LayoutList } from '@lucide/svelte';

  import Dashboard from './lib/views/Dashboard.svelte';
  import Settings from './lib/views/Settings.svelte';
  import { loadApps } from './lib/stores/apps';
  import { loadViewState, startPersistingViewState } from './lib/stores/filters';
  import {
    gitError,
    lastGitRefresh,
    loadGitStatus,
    refreshAllGitStatus,
    refreshingGit,
  } from './lib/stores/gitStatus';
  import { loadProjects, projects, runScan, scanning } from './lib/stores/projects';
  import { loadScanPaths, scanPaths } from './lib/stores/scanPaths';
  import { loadTags } from './lib/stores/tags';
  import { formatRelativeTime, now_ts } from './lib/utils/format';

  /** Lo emite el backend al terminar un refresco automático de estado Git. */
  const GIT_REFRESHED = 'git-status-refreshed';
  /** Lo emite el backend cuando el escaneo al arrancar encontró proyectos. */
  const PROJECTS_RESCANNED = 'projects-rescanned';
  const GIT_BUTTON_HINT =
    'Relee del disco la rama, los cambios sin commitear y el último commit de cada ' +
    'repositorio. No hace fetch: no toca la red. También se refresca solo cada 5 minutos.';
  /** Segundos que el resultado de un refresco manual desplaza al «hace X». */
  const OUTCOME_VISIBLE_SECONDS = 8;
  /**
   * Cada cuánto se recalcula el «hace X».
   *
   * Treinta segundos deja el texto con hasta medio minuto de retraso, que es
   * preferible a despertar el render cada segundo para contar algo que al
   * usuario le da igual con esa precisión.
   */
  const CLOCK_TICK_MS = 30_000;

  type View = 'dashboard' | 'settings';

  let view = $state<View>('dashboard');
  let ready = $state(false);
  let tick = $state(now_ts());

  let canScan = $derived($scanPaths.length > 0);

  let gitLabel = $derived.by(() => {
    if ($refreshingGit) return 'Leyendo los repositorios…';

    const last = $lastGitRefresh;
    if (last === null) return '';

    const age = tick - last.at;
    if (last.manual && age <= OUTCOME_VISIBLE_SECONDS) {
      const read = `${last.read} ${last.read === 1 ? 'repositorio leído' : 'repositorios leídos'}`;
      const parts = [read];
      if (last.changed === 0) {
        parts.push('sin cambios');
      } else {
        parts.push(`${last.changed} ${last.changed === 1 ? 'actualizado' : 'actualizados'}`);
      }
      if (last.failed > 0) {
        parts.push(`${last.failed} ${last.failed === 1 ? 'ilegible' : 'ilegibles'}`);
      }
      return parts.join(' · ');
    }

    return `Git leído ${formatRelativeTime(last.at)}`;
  });

  onMount(() => {
    const stopListening = Promise.all([
      listen(GIT_REFRESHED, () => void loadGitStatus()),
      // El escaneo al arrancar puede haber descubierto proyectos nuevos, así que
      // la lista se relee entera en lugar de mezclar solo el estado de Git.
      listen(PROJECTS_RESCANNED, () => void loadProjects()),
    ]);
    let stopPersisting: (() => void) | null = null;

    const clock = setInterval(() => (tick = now_ts()), CLOCK_TICK_MS);

    void (async () => {
      await loadViewState();
      ready = true;
      stopPersisting = startPersistingViewState();

      await Promise.all([loadProjects(), loadScanPaths(), loadTags(), loadApps()]);
      await loadGitStatus();
    })();

    return () => {
      clearInterval(clock);
      stopPersisting?.();
      void stopListening.then((stops) => stops.forEach((stop) => stop()));
    };
  });

  async function refreshGit() {
    await refreshAllGitStatus();
    tick = now_ts();
  }

  async function scan() {
    const summary = await runScan();
    if (summary === null) return;
    view = 'dashboard';
    await refreshAllGitStatus();
  }
</script>

<div class="flex h-full flex-col">
  <header class="cabecera">
    <h1 class="marca">Mosaic</h1>

    <nav class="pestanas" aria-label="Secciones">
      <button
        type="button"
        class="pestana"
        onclick={() => (view = 'dashboard')}
        aria-current={view === 'dashboard' ? 'page' : undefined}
      >
        <LayoutList size={16} strokeWidth={1.75} />
        Proyectos
      </button>

      <button
        type="button"
        class="pestana"
        onclick={() => (view = 'settings')}
        aria-current={view === 'settings' ? 'page' : undefined}
      >
        <SettingsIcon size={16} strokeWidth={1.75} />
        Ajustes
      </button>
    </nav>

    <p class="metadatos">
      <span>
        {$projects.length}
        {$projects.length === 1 ? 'proyecto' : 'proyectos'}
      </span>
      {#if gitLabel !== ''}
        <span class="sep" aria-hidden="true">·</span>
        <span aria-live="polite">{gitLabel}</span>
      {/if}
    </p>

    <div class="acciones" role="group" aria-label="Acciones">
      <button
        type="button"
        class="accion"
        onclick={refreshGit}
        disabled={$refreshingGit}
        title={GIT_BUTTON_HINT}
      >
        <GitBranch
          size={14}
          strokeWidth={1.75}
          class={$refreshingGit ? 'animate-pulse' : ''}
        />
        {$refreshingGit ? 'Leyendo…' : 'Git'}
      </button>

      <button
        type="button"
        class="accion"
        onclick={scan}
        disabled={$scanning || !canScan}
        title={canScan
          ? 'Recorrer las rutas configuradas y actualizar la lista de proyectos'
          : 'Añade una carpeta en Ajustes para poder escanear'}
      >
        <RefreshCw size={14} strokeWidth={1.75} class={$scanning ? 'animate-spin' : ''} />
        {$scanning ? 'Escaneando…' : 'Escanear'}
      </button>
    </div>
  </header>

  {#if $gitError}
    <p
      class="mx-6 mt-4 rounded-md border border-surface-border bg-surface-1 px-4 py-3 text-sm
             text-content"
      role="alert"
    >
      {$gitError}
    </p>
  {/if}

  <main class="flex min-h-0 flex-1 flex-col">
    {#if !ready}
      <p class="px-6 py-16 text-center text-sm text-content-muted">Cargando…</p>
    {:else if view === 'dashboard'}
      <Dashboard onGoToSettings={() => (view = 'settings')} onScan={scan} />
    {:else}
      <div class="flex-1 overflow-y-auto"><Settings /></div>
    {/if}
  </main>
</div>

<style>
  /*
   * Solo la cabecera. El resto de este fichero es el armazón de la aplicación
   * y sigue con utilidades de Tailwind hasta que le toque migrar.
   *
   * Cuatro pesos, de más a menos: la marca, la navegación, las acciones y los
   * metadatos. Antes los seis elementos tenían el mismo tamaño de letra y el
   * ojo no sabía por dónde empezar.
   */
  .cabecera {
    display: flex;
    flex: none;
    align-items: center;
    gap: var(--space-5);
    min-height: 48px;
    padding: 0 var(--space-5);
    border-bottom: 1px solid var(--border-subtle);
    background: var(--surface-raised);
  }

  .marca {
    margin: 0;
    font-size: var(--text-section);
    line-height: var(--text-section-lh);
    font-weight: var(--text-section-weight);
    letter-spacing: var(--tracking-tight);
    color: var(--text-primary);
  }

  /* Las pestañas ocupan todo el alto para que el subrayado pise el borde. */
  .pestanas {
    display: flex;
    align-self: stretch;
    gap: var(--space-1);
  }

  .pestana {
    position: relative;
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    padding: 0 var(--space-3);
    border: 0;
    background: transparent;
    font: inherit;
    font-size: var(--text-body);
    line-height: var(--text-body-lh);
    font-weight: 500;
    color: var(--text-tertiary);
    cursor: pointer;
    transition: color var(--duration-fast) var(--ease-out);
  }
  .pestana:hover {
    color: var(--text-secondary);
  }
  .pestana[aria-current='page'] {
    color: var(--text-primary);
  }
  .pestana::after {
    content: '';
    position: absolute;
    right: var(--space-3);
    bottom: -1px;
    left: var(--space-3);
    height: 2px;
    border-radius: var(--radius-pill);
    background: transparent;
    transition: background var(--duration-fast) var(--ease-out);
  }
  .pestana[aria-current='page']::after {
    background: var(--accent);
  }

  .metadatos {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    min-width: 0;
    margin: 0 0 0 auto;
    overflow: hidden;
    font-size: var(--text-meta);
    line-height: var(--text-meta-lh);
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
    color: var(--text-tertiary);
  }
  .sep {
    color: var(--text-disabled);
  }

  /* Un solo bloque con borde: dos acciones hermanas, no dos botones sueltos. */
  .acciones {
    display: flex;
    flex: none;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  .accion {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    padding: 5px var(--space-3);
    border: 0;
    background: transparent;
    font: inherit;
    font-size: var(--text-body);
    line-height: var(--text-body-lh);
    color: var(--text-secondary);
    cursor: pointer;
    transition:
      background var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out);
  }
  .accion + .accion {
    border-left: 1px solid var(--border-default);
  }
  .accion:hover:not(:disabled) {
    background: var(--surface-sunken);
    color: var(--text-primary);
  }
  .accion:disabled {
    color: var(--text-disabled);
    cursor: not-allowed;
  }

  button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }
</style>
