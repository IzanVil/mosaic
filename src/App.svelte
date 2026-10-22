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
    applyGitRefresh,
    lastGitRefresh,
    loadGitStatus,
    refreshAllGitStatus,
    refreshingGit,
  } from './lib/stores/gitStatus';
  import { goToDashboard, goToSettings, openTagManager, route } from './lib/stores/navigation';
  import { loadProjects, projects, runScan, scanning } from './lib/stores/projects';
  import { loadScanPaths, scanPaths } from './lib/stores/scanPaths';
  import { loadTags } from './lib/stores/tags';
  import { lang, t, tNow, translate } from './lib/i18n';
  import type { GitRefreshSummary } from './lib/types';
  import { formatRelativeTime, now_ts } from './lib/utils/format';
  import type { PaletteAction } from './lib/utils/palette';
  import { isMac, isTypingTarget, matchShortcut, shortcutList } from './lib/utils/shortcuts';
  import { tick as nextRender } from 'svelte';

  /** Lo emite el backend al terminar un refresco automático de estado Git. */
  const GIT_REFRESHED = 'git-status-refreshed';
  /** Lo emite el backend cuando el escaneo al arrancar encontró proyectos. */
  const PROJECTS_RESCANNED = 'projects-rescanned';
  /** Segundos que el resultado de un refresco manual desplaza al «hace X». */
  const OUTCOME_VISIBLE_SECONDS = 8;
  /** Cada cuánto se recalcula el «hace X»: mejor 30 s de retraso que un render por segundo. */
  const CLOCK_TICK_MS = 30_000;

  /** Un botón para tres temas: el icono es el actual y el `title` dice cuál viene. */
  const THEME_INFO = {
    dark: { icon: Moon, label: 'app.temaOscuro', next: 'app.temaSiguienteClaro' },
    light: { icon: Sun, label: 'app.temaClaro', next: 'app.temaSiguienteSistema' },
    system: { icon: Monitor, label: 'app.temaSistema', next: 'app.temaSiguienteOscuro' },
  } as const;

  let themeInfo = $derived(THEME_INFO[$theme]);
  let themeText = $derived({ actual: $t(themeInfo.label), siguiente: $t(themeInfo.next) });

  const mac = isMac();
  const modKey = mac ? '⌘' : 'Ctrl+';
  let paletteOpen = $state(false);
  let helpOpen = $state(false);

  /** Lo que se puede hacer desde la paleta, además de abrir proyectos. */
  const ACCIONES = [
    { id: 'git', label: 'paleta.git', keywords: 'paleta.gitClaves', shortcut: `${modKey}R` },
    { id: 'scan', label: 'paleta.escanear', keywords: 'paleta.escanearClaves' },
    { id: 'dashboard', label: 'paleta.proyectos', keywords: 'paleta.proyectosClaves' },
    { id: 'settings', label: 'paleta.ajustes', keywords: 'paleta.ajustesClaves', shortcut: `${modKey},` },
    { id: 'tags', label: 'paleta.etiquetas', keywords: 'paleta.etiquetasClaves' },
    { id: 'theme', label: 'paleta.tema', keywords: 'paleta.temaClaves' },
    { id: 'help', label: 'paleta.ayuda', keywords: 'paleta.ayudaClaves', shortcut: '?' },
  ] as const;

  // Las palabras clave van en los dos idiomas: quien busca «settings» con la
  // interfaz en español también tiene que encontrar Ajustes.
  let paletteActions = $derived<PaletteAction[]>(
    ACCIONES.map(({ id, label, keywords, ...rest }) => ({
      id,
      label: $t(label),
      keywords: [...$t(keywords).split(' '), ...tOtro(keywords).split(' ')],
      ...rest,
    })),
  );

  function tOtro(clave: (typeof ACCIONES)[number]['keywords']): string {
    return tNow(clave) === translate('es', clave) ? translate('en', clave) : translate('es', clave);
  }

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

  /** Atajos globales; los desplegables con `dismissable` se quedan antes su Escape. */
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
    if ($refreshingGit) return $t('app.gitLeyendoRepos');

    const last = $lastGitRefresh;
    if (last === null) return '';

    const age = tick - last.at;
    if (last.manual && age <= OUTCOME_VISIBLE_SECONDS) {
      const parts = [$t('app.gitLeidos', { n: last.read })];
      if (last.changed === 0) {
        parts.push($t('app.gitSinCambios'));
      } else {
        parts.push($t('app.gitActualizados', { n: last.changed }));
      }
      if (last.failed > 0) {
        parts.push($t('app.gitIlegibles', { n: last.failed }));
      }
      return parts.join(' · ');
    }

    return $t('app.gitLeidoHace', { cuando: formatRelativeTime(last.at, $lang) });
  });

  onMount(() => {
    const stopListening = Promise.all([
      listen<GitRefreshSummary>(GIT_REFRESHED, (event) => applyGitRefresh(event.payload)),
      // Puede haber proyectos nuevos: la lista se relee entera.
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

    <nav class="pestanas" aria-label={$t('app.secciones')}>
      <button
        type="button"
        class="pestana"
        onclick={goToDashboard}
        aria-current={$route.view !== 'settings' ? 'page' : undefined}
      >
        <LayoutList size={16} strokeWidth={1.75} />
        {$t('app.proyectos')}
      </button>

      <button
        type="button"
        class="pestana"
        onclick={goToSettings}
        aria-current={$route.view === 'settings' ? 'page' : undefined}
      >
        <SettingsIcon size={16} strokeWidth={1.75} />
        {$t('app.ajustes')}
      </button>
    </nav>

    <p class="metadatos" data-tauri-drag-region={mac || undefined}>
      <span>{$t('app.numProyectos', { n: $projects.length })}</span>
      {#if gitLabel !== ''}
        <span class="sep" aria-hidden="true">·</span>
        <span aria-live="polite">{gitLabel}</span>
      {/if}
    </p>

    <button
      type="button"
      class="tema"
      onclick={cycleTheme}
      title={$t('app.temaTitulo', themeText)}
      aria-label={$t('app.temaEtiqueta', themeText)}
    >
      <themeInfo.icon size={16} strokeWidth={1.75} />
    </button>

    <div class="acciones" role="group" aria-label={$t('app.acciones')}>
      <button
        type="button"
        class="accion"
        onclick={refreshGit}
        disabled={$refreshingGit}
        title={$t('app.gitAyuda')}
      >
        <GitBranch
          size={14}
          strokeWidth={1.75}
          class={$refreshingGit ? 'animate-pulse' : ''}
        />
        {$refreshingGit ? $t('app.gitLeyendo') : $t('app.git')}
      </button>

      <button
        type="button"
        class="accion"
        onclick={scan}
        disabled={$scanning || !canScan}
        title={canScan ? $t('app.escanearAyuda') : $t('app.escanearSinRutas')}
      >
        <RefreshCw size={14} strokeWidth={1.75} class={$scanning ? 'animate-spin' : ''} />
        {$scanning ? $t('app.escaneando') : $t('app.escanear')}
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
    actions={paletteActions}
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

  /* Cuatro pesos, de más a menos: marca, navegación, acciones y metadatos. Con uno solo, el
     ojo no sabía por dónde empezar. */
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
