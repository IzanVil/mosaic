<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import {
    GitBranch,
    LayoutList,
    Monitor,
    Moon,
    RefreshCw,
    Settings as SettingsIcon,
    Sun,
  } from '@lucide/svelte';

  import CommandPalette from './lib/components/CommandPalette.svelte';
  import ShortcutsHelp from './lib/components/ShortcutsHelp.svelte';
  import Dashboard from './lib/views/Dashboard.svelte';
  import ProjectView from './lib/views/ProjectView.svelte';
  import Settings from './lib/views/Settings.svelte';
  import { loadApps } from './lib/stores/apps';
  import { cycleTheme, startPersistingViewState, theme } from './lib/stores/filters';
  import {
    gitError,
    lastGitRefresh,
    loadGitStatus,
    refreshAllGitStatus,
    refreshingGit,
  } from './lib/stores/gitStatus';
  import { goToDashboard, goToSettings, openTagManager, route } from './lib/stores/navigation';
  import { loadProjects, projects, runScan, scanning } from './lib/stores/projects';
  import { loadScanPaths, scanPaths } from './lib/stores/scanPaths';
  import { loadTags } from './lib/stores/tags';
  import { formatRelativeTime, now_ts } from './lib/utils/format';
  import type { PaletteAction } from './lib/utils/palette';
  import { isMac, isTypingTarget, matchShortcut, shortcutList } from './lib/utils/shortcuts';
  import { tick as nextRender } from 'svelte';

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

  /**
   * Un solo botón para tres estados: cada clic pasa al siguiente. El icono es
   * el del tema actual y el `title` dice cuál viene, para que el clic no sea
   * una sorpresa.
   */
  const THEME_INFO = {
    dark: { icon: Moon, label: 'Tema oscuro', next: 'claro' },
    light: { icon: Sun, label: 'Tema claro', next: 'el del sistema' },
    system: { icon: Monitor, label: 'Tema del sistema', next: 'oscuro' },
  } as const;

  let themeInfo = $derived(THEME_INFO[$theme]);

  const mac = isMac();
  const modKey = mac ? '⌘' : 'Ctrl+';
  let paletteOpen = $state(false);
  let helpOpen = $state(false);

  /** Lo que se puede hacer desde la paleta, además de abrir proyectos. */
  const PALETTE_ACTIONS: PaletteAction[] = [
    { id: 'git', label: 'Releer el estado de Git', keywords: ['refrescar', 'actualizar'], shortcut: `${modKey}R` },
    { id: 'scan', label: 'Escanear las rutas', keywords: ['buscar proyectos', 'actualizar'] },
    { id: 'dashboard', label: 'Ir a Proyectos', keywords: ['tablero', 'inicio'] },
    { id: 'settings', label: 'Ir a Ajustes', keywords: ['preferencias', 'configuración'], shortcut: `${modKey},` },
    { id: 'tags', label: 'Gestionar etiquetas', keywords: ['tags', 'colores'] },
    { id: 'theme', label: 'Cambiar el tema', keywords: ['claro', 'oscuro', 'sistema'] },
    { id: 'help', label: 'Ver los atajos de teclado', keywords: ['ayuda', 'teclas'], shortcut: '?' },
  ];

  function runAction(id: string) {
    if (id === 'git') void refreshGit();
    else if (id === 'scan') void scan();
    else if (id === 'dashboard') goToDashboard();
    else if (id === 'settings') goToSettings();
    else if (id === 'tags') openTagManager();
    else if (id === 'theme') cycleTheme();
    else if (id === 'help') helpOpen = true;
  }

  /** Va al tablero si hace falta y pone el cursor en la búsqueda. */
  async function focusSearch() {
    if ($route.view !== 'dashboard') {
      goToDashboard();
      await nextRender();
    }
    document.getElementById('busqueda')?.focus();
  }

  /**
   * Atajos globales. Se atienden en `window`, pero los desplegables y modales
   * que usan `dismissable` se quedan antes con su Escape.
   */
  function onGlobalKeydown(event: KeyboardEvent) {
    const action = matchShortcut(
      {
        key: event.key,
        ctrlKey: event.ctrlKey,
        metaKey: event.metaKey,
        altKey: event.altKey,
        shiftKey: event.shiftKey,
        typing: isTypingTarget(event.target),
      },
      mac,
    );
    if (action === null) return;

    // Ctrl+R recargaría la ventana como en un navegador: aquí relee Git.
    event.preventDefault();
    if (action === 'palette') {
      helpOpen = false;
      paletteOpen = !paletteOpen;
    } else if (action === 'help') {
      paletteOpen = false;
      helpOpen = true;
    } else if (action === 'refresh-git') {
      void refreshGit();
    } else if (action === 'settings') {
      goToSettings();
    } else if (action === 'focus-search') {
      void focusSearch();
    }
  }

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
    const clock = setInterval(() => (tick = now_ts()), CLOCK_TICK_MS);

    // `main.ts` ya ha leído la vista guardada antes de montar.
    const stopPersisting = startPersistingViewState();

    void (async () => {
      await Promise.all([loadProjects(), loadScanPaths(), loadTags(), loadApps()]);
      await loadGitStatus();
    })();

    return () => {
      clearInterval(clock);
      stopPersisting();
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
    goToDashboard();
    await refreshAllGitStatus();
  }
</script>

<svelte:window onkeydown={onGlobalKeydown} />

<div class="armazon">
  <!-- En macOS la cabecera ocupa el sitio de la barra de título: tiene que arrastrar la ventana. -->
  <header class="cabecera" class:mac data-tauri-drag-region={mac || undefined}>
    <h1 class="marca" data-tauri-drag-region={mac || undefined}>Mosaic</h1>

    <nav class="pestanas" aria-label="Secciones">
      <button
        type="button"
        class="pestana"
        onclick={goToDashboard}
        aria-current={$route.view !== 'settings' ? 'page' : undefined}
      >
        <LayoutList size={16} strokeWidth={1.75} />
        Proyectos
      </button>

      <button
        type="button"
        class="pestana"
        onclick={goToSettings}
        aria-current={$route.view === 'settings' ? 'page' : undefined}
      >
        <SettingsIcon size={16} strokeWidth={1.75} />
        Ajustes
      </button>
    </nav>

    <p class="metadatos" data-tauri-drag-region={mac || undefined}>
      <span>
        {$projects.length}
        {$projects.length === 1 ? 'proyecto' : 'proyectos'}
      </span>
      {#if gitLabel !== ''}
        <span class="sep" aria-hidden="true">·</span>
        <span aria-live="polite">{gitLabel}</span>
      {/if}
    </p>

    <button
      type="button"
      class="tema"
      onclick={cycleTheme}
      title="{themeInfo.label}. Pulsa para pasar a {themeInfo.next}."
      aria-label="{themeInfo.label}. Cambiar a {themeInfo.next}"
    >
      <themeInfo.icon size={16} strokeWidth={1.75} />
    </button>

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
    <p class="alerta" role="alert">{$gitError}</p>
  {/if}

  <main class="principal">
    {#if $route.view === 'dashboard'}
      <Dashboard onGoToSettings={goToSettings} onScan={scan} />
    {:else if $route.view === 'project'}
      <!-- El key recrea la vista entera si se abre otro proyecto. -->
      {#key $route.projectId}
        <ProjectView projectId={$route.projectId} />
      {/key}
    {:else}
      <div class="desplazable"><Settings /></div>
    {/if}
  </main>
</div>

{#if paletteOpen}
  <CommandPalette
    actions={PALETTE_ACTIONS}
    onAction={runAction}
    onClose={() => (paletteOpen = false)}
    {modKey}
  />
{/if}

{#if helpOpen}
  <ShortcutsHelp shortcuts={shortcutList(mac)} onClose={() => (helpOpen = false)} />
{/if}

<style>
  .armazon {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--surface-base);
    color: var(--text-secondary);
  }

  .principal {
    display: flex;
    flex: 1;
    flex-direction: column;
    min-height: 0;
  }

  .desplazable {
    flex: 1;
    overflow-y: auto;
  }

  .alerta {
    margin: var(--space-4) var(--space-5) 0;
    padding: var(--space-3) var(--space-4);
    border: 1px solid color-mix(in oklab, var(--danger) 40%, transparent);
    border-radius: var(--radius-md);
    background: var(--danger-surface);
    font-size: var(--text-body);
    line-height: var(--text-body-lh);
    color: var(--danger);
  }

  /*
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
  .cabecera.mac {
    padding-left: var(--titlebar-inset-mac);
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
  .tema {
    display: inline-flex;
    flex: none;
    padding: 7px;
    border: 0;
    border-radius: var(--radius-md);
    background: transparent;
    color: var(--text-tertiary);
    cursor: pointer;
    transition:
      background var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out);
  }
  .tema:hover {
    background: var(--surface-sunken);
    color: var(--text-primary);
  }

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
