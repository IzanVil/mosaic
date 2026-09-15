<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { GitBranch, RefreshCw, Settings as SettingsIcon, LayoutList } from '@lucide/svelte';

  import Dashboard from './lib/views/Dashboard.svelte';
  import Settings from './lib/views/Settings.svelte';
  import { loadApps } from './lib/stores/apps';
  import {
    gitError,
    loadGitStatus,
    refreshAllGitStatus,
    refreshingGit,
  } from './lib/stores/gitStatus';
  import { loadProjects, projects, runScan, scanning } from './lib/stores/projects';
  import { loadScanPaths, scanPaths } from './lib/stores/scanPaths';

  /** Lo emite el backend al terminar un refresco automático de estado Git. */
  const GIT_REFRESHED = 'git-status-refreshed';

  type View = 'dashboard' | 'settings';

  let view = $state<View>('dashboard');

  /** Sin rutas configuradas no hay nada que escanear. */
  let canScan = $derived($scanPaths.length > 0);

  // Carga inicial: lo que ya está en la base de datos, sin tocar el disco.
  onMount(() => {
    void loadProjects();
    void loadScanPaths();
    void loadGitStatus();
    void loadApps();

    // El refresco automático corre en el backend; aquí solo recogemos el aviso.
    const unlisten = listen(GIT_REFRESHED, () => void loadGitStatus());
    return () => void unlisten.then((stop) => stop());
  });

  async function scan() {
    const summary = await runScan();
    // El resultado de escanear son los proyectos: si el usuario lo lanzó desde
    // Ajustes, quedarse ahí hace que la acción parezca no haber hecho nada.
    if (summary !== null) view = 'dashboard';
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

  <main class="flex flex-1 flex-col overflow-y-auto">
    {#if view === 'dashboard'}
      <Dashboard onGoToSettings={() => (view = 'settings')} onScan={scan} />
    {:else}
      <Settings />
    {/if}
  </main>
</div>
