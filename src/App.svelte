<script lang="ts">
  import { onMount } from 'svelte';
  import { RefreshCw, Settings as SettingsIcon, LayoutList } from '@lucide/svelte';

  import Dashboard from './lib/views/Dashboard.svelte';
  import Settings from './lib/views/Settings.svelte';
  import { lastScan, loadProjects, projects, runScan, scanning } from './lib/stores/projects';
  import { loadScanPaths } from './lib/stores/scanPaths';
  import { formatDuration } from './lib/utils/format';

  type View = 'dashboard' | 'settings';

  let view = $state<View>('dashboard');

  // Carga inicial: proyectos ya conocidos y rutas configuradas.
  onMount(() => {
    void loadProjects();
    void loadScanPaths();
  });

  async function scan() {
    await runScan();
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
      {#if $lastScan}
        <p class="text-xs text-content-muted">
          {$lastScan.projects_found}
          {$lastScan.projects_found === 1 ? 'proyecto' : 'proyectos'} en
          {formatDuration($lastScan.elapsed_ms)}
          {#if $lastScan.truncated}
            · escaneo truncado
          {/if}
        </p>
      {:else}
        <p class="text-xs text-content-muted">
          {$projects.length}
          {$projects.length === 1 ? 'proyecto' : 'proyectos'}
        </p>
      {/if}

      <button
        type="button"
        onclick={scan}
        disabled={$scanning}
        class="inline-flex items-center gap-2 rounded-md border border-surface-border px-3 py-1.5
               text-sm text-content transition hover:bg-surface-2 disabled:opacity-60
               focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
      >
        <RefreshCw size={16} class={$scanning ? 'animate-spin' : ''} />
        {$scanning ? 'Escaneando…' : 'Escanear'}
      </button>
    </div>
  </header>

  <main class="flex flex-1 flex-col overflow-y-auto">
    {#if view === 'dashboard'}
      <Dashboard onGoToSettings={() => (view = 'settings')} onScan={scan} />
    {:else}
      <Settings />
    {/if}
  </main>
</div>
