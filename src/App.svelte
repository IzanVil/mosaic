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
    loadGitStatus,
    refreshAllGitStatus,
    refreshingGit,
  } from './lib/stores/gitStatus';
  import { loadProjects, projects, runScan, scanning } from './lib/stores/projects';
  import { loadScanPaths, scanPaths } from './lib/stores/scanPaths';
  import { loadTags } from './lib/stores/tags';

  const GIT_REFRESHED = 'git-status-refreshed';

  type View = 'dashboard' | 'settings';

  let view = $state<View>('dashboard');
  let ready = $state(false);

  let canScan = $derived($scanPaths.length > 0);

  onMount(() => {
    const stopListening = listen(GIT_REFRESHED, () => void loadGitStatus());
    let stopPersisting: (() => void) | null = null;

    void (async () => {
      await loadViewState();
      ready = true;
      stopPersisting = startPersistingViewState();

      await Promise.all([loadProjects(), loadScanPaths(), loadTags(), loadApps()]);
      await loadGitStatus();
    })();

    return () => {
      stopPersisting?.();
      void stopListening.then((stop) => stop());
    };
  });

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

      <button
        type="button"
        onclick={refreshAllGitStatus}
        disabled={$refreshingGit}
        title="Releer el estado Git de todos los repositorios"
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
