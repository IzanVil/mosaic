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
  <header
    class="flex shrink-0 items-center gap-4 border-b border-surface-border bg-surface-1 px-6 py-3"
  >
    <h1 class="text-sm font-semibold tracking-wide text-content-strong">Mosaic</h1>

    <nav class="flex items-center gap-1" aria-label="Secciones">
      <button
        type="button"
        onclick={() => (view = 'dashboard')}
        aria-current={view === 'dashboard' ? 'page' : undefined}
        class="inline-flex items-center gap-2 rounded-md px-3 py-1.5 text-sm transition
               hover:bg-surface-2"
        class:bg-surface-2={view === 'dashboard'}
        class:text-content-strong={view === 'dashboard'}
        class:text-content-muted={view !== 'dashboard'}
      >
        <LayoutList size={16} />
        Proyectos
      </button>

      <button
        type="button"
        onclick={() => (view = 'settings')}
        aria-current={view === 'settings' ? 'page' : undefined}
        class="inline-flex items-center gap-2 rounded-md px-3 py-1.5 text-sm transition
               hover:bg-surface-2"
        class:bg-surface-2={view === 'settings'}
        class:text-content-strong={view === 'settings'}
        class:text-content-muted={view !== 'settings'}
      >
        <SettingsIcon size={16} />
        Ajustes
      </button>
    </nav>

    <div class="ml-auto flex items-center gap-4">
      <p class="text-xs text-content-muted">
        {$projects.length}
        {$projects.length === 1 ? 'proyecto' : 'proyectos'}
      </p>

      {#if gitLabel !== ''}
        <p class="text-xs text-content-muted" aria-live="polite">{gitLabel}</p>
      {/if}

      <button
        type="button"
        onclick={refreshGit}
        disabled={$refreshingGit}
        title={GIT_BUTTON_HINT}
        class="inline-flex items-center gap-2 rounded-md border border-surface-border px-3 py-1.5
               text-sm text-content transition hover:bg-surface-2 disabled:opacity-60
               focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
      >
        <GitBranch size={16} class={$refreshingGit ? 'animate-pulse' : ''} />
        {$refreshingGit ? 'Leyendo…' : 'Git'}
      </button>

      <button
        type="button"
        onclick={scan}
        disabled={$scanning || !canScan}
        title={canScan
          ? 'Recorrer las rutas configuradas y actualizar la lista de proyectos'
          : 'Añade una carpeta en Ajustes para poder escanear'}
        class="inline-flex items-center gap-2 rounded-md border border-surface-border px-3 py-1.5
               text-sm text-content transition hover:bg-surface-2 disabled:opacity-60
               focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
      >
        <RefreshCw size={16} class={$scanning ? 'animate-spin' : ''} />
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
