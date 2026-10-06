<script lang="ts">
  import {
    ArrowLeft,
    Code,
    FolderOpen,
    SquareTerminal,
    TriangleAlert,
  } from '@lucide/svelte';
  import { onDestroy, tick } from 'svelte';

  import BranchList from '../components/BranchList.svelte';
  import CommitList from '../components/CommitList.svelte';
  import CopyPathButton from '../components/CopyPathButton.svelte';
  import NotesEditor from '../components/NotesEditor.svelte';
  import ReadmeView from '../components/ReadmeView.svelte';
  import TagChip from '../components/TagChip.svelte';
  import TagPicker from '../components/TagPicker.svelte';
  import { openProject } from '../stores/apps';
  import { leaveProjectDetail } from '../stores/navigation';
  import { branches, closeDetail, history, openDetail, readme } from '../stores/projectDetail';
  import { loadingProjects, projects } from '../stores/projects';
  import { unassignTagFromProject } from '../stores/tags';
  import { formatRelativeTime } from '../utils/format';

  interface Props {
    /** Proyecto que se muestra. La vista se recrea entera si cambia. */
    projectId: number;
  }

  let { projectId }: Props = $props();

  /**
   * El proyecto sale de `projects`, la única fuente del tablero: así una
   * etiqueta asignada aquí, o el refresco de Git de fondo, se ven al momento en
   * las dos vistas sin copiar nada.
   */
  let project = $derived($projects.find((candidate) => candidate.id === projectId));
  let status = $derived(project?.git_status ?? null);
  let branchLabel = $derived(
    status?.branch ?? (status?.last_commit_sha ? status.last_commit_sha.slice(0, 7) : null),
  );

  const ACTIONS = [
    { kind: 'ide', icon: Code, label: 'Editor' },
    { kind: 'terminal', icon: SquareTerminal, label: 'Terminal' },
    { kind: 'file_manager', icon: FolderOpen, label: 'Carpeta' },
  ] as const;

  /**
   * El detalle se abre cuando el proyecto ya está en `projects`, no al montar.
   * Si la lista todavía no ha llegado del backend (por ejemplo, justo al
   * arrancar), abrirlo al montar cargaría unas notas vacías en el editor
   * aunque las tuviera guardadas.
   */
  let opened = false;
  $effect(() => {
    if (opened || !project) return;
    opened = true;
    openDetail(projectId, project.notes);
  });

  // Las notas pendientes se guardan al salir por cualquier camino: Escape, el
  // botón de volver o una pestaña de la cabecera.
  onDestroy(() => {
    void closeDetail();
  });

  /** Vuelve al tablero y devuelve el foco a la tarjeta de la que se vino. */
  async function back() {
    const focusId = leaveProjectDetail();
    if (focusId === null) return;
    await tick();
    document.querySelector<HTMLElement>(`[data-proyecto="${focusId}"]`)?.focus();
  }

  function onKeydown(event: KeyboardEvent) {
    // Los desplegables cierran con Escape antes que la vista: `dismissable` lo
    // atrapa en fase de captura y no deja que llegue hasta aquí.
    if (event.key === 'Escape' && !event.defaultPrevented) {
      event.preventDefault();
      void back();
    }
  }
</script>

<svelte:window onkeydown={onKeydown} />

{#if !project && ($loadingProjects || $projects.length === 0)}
  <!-- Al arrancar la lista aún no ha llegado: decir «no está» sería falso. -->
  <section class="perdido"><p>Cargando el proyecto…</p></section>
{:else if !project}
  <section class="perdido">
    <p>Este proyecto ya no está en la lista.</p>
    <button type="button" class="volver" onclick={back}>
      <ArrowLeft size={16} strokeWidth={1.75} /> Volver a proyectos
    </button>
  </section>
{:else}
  <section class="detalle" aria-labelledby="titulo-proyecto">
    <header class="cabecera">
      <div class="envoltura">
        <button type="button" class="volver" onclick={back} title="Volver a proyectos (Esc)">
          <ArrowLeft size={16} strokeWidth={1.75} />
          Proyectos
          <kbd>Esc</kbd>
        </button>

        <div class="titulo">
          <div class="nombre">
            <h1 id="titulo-proyecto">{project.name}</h1>
            <p class="ruta">
              <code title={project.path}>{project.path}</code>
              <CopyPathButton path={project.path} />
            </p>
            <p class="estado">
              {#if project.primary_language}<span>{project.primary_language}</span>{/if}
              {#if branchLabel}
                <span class="rama" title={status?.branch === null ? 'HEAD separado' : undefined}>
                  {branchLabel}
                </span>
                {#if status?.is_dirty}<span class="sucio">Cambios sin commitear</span>{/if}
                {#if status?.ahead}<span class="delta" title="Commits por delante">↑{status.ahead}</span>{/if}
                {#if status?.behind}<span class="delta" title="Commits por detrás">↓{status.behind}</span>{/if}
              {/if}
              <span>
                {project.last_opened_at === null
                  ? 'Sin abrir desde Mosaic'
                  : `Abierto ${formatRelativeTime(project.last_opened_at)}`}
              </span>
            </p>
          </div>

          <div class="acciones" role="group" aria-label="Abrir el proyecto">
            {#each ACTIONS as action (action.kind)}
              <button
                type="button"
                class="accion"
                onclick={() => openProject(action.kind, project.id)}
                disabled={project.missing}
                title={project.missing ? 'La carpeta ya no está en disco' : undefined}
              >
                <action.icon size={16} strokeWidth={1.75} />
                {action.label}
              </button>
            {/each}
          </div>
        </div>

        <div class="etiquetas">
          {#each project.tags as tag (tag.id)}
            <TagChip {tag} removable onRemove={() => unassignTagFromProject(project.id, tag.id)} />
          {/each}
          <TagPicker projectId={project.id} assigned={project.tags} compact={project.tags.length > 0} />
        </div>

        {#if project.missing}
          <p class="ausente" role="status">
            <TriangleAlert size={16} strokeWidth={1.75} />
            El último escaneo no encontró esta carpeta. Sus etiquetas y notas se conservan por si vuelve.
          </p>
        {/if}
      </div>
    </header>

    <div class="envoltura cuerpo">
      <div class="principal">
        <ReadmeView
          section={$readme}
          missing={project.missing}
          projectId={project.id}
          remoteUrl={status?.remote_url ?? null}
        />
      </div>

      <aside class="lateral">
        <section aria-labelledby="titulo-notas">
          <h2 id="titulo-notas">Notas</h2>
          <NotesEditor />
        </section>

        <!-- Las ramas son pocas y cortas; van antes que la lista larga de commits. -->
        <section aria-labelledby="titulo-ramas">
          <h2 id="titulo-ramas">Ramas</h2>
          <BranchList section={$branches} isRepo={project.is_git_repo} />
        </section>

        <section aria-labelledby="titulo-historial">
          <h2 id="titulo-historial">Últimos commits</h2>
          <CommitList section={$history} isRepo={project.is_git_repo} />
        </section>
      </aside>
    </div>
  </section>
{/if}

<style>
  .detalle {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    background: var(--surface-base);
  }

  .envoltura {
    width: 100%;
    max-width: 76rem;
    box-sizing: border-box;
    margin-inline: auto;
    padding-inline: var(--space-5);
  }

  .cabecera {
    padding-block: var(--space-4) var(--space-5);
    border-bottom: 1px solid var(--border-subtle);
    background: var(--surface-raised);
  }

  .volver {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    margin-left: calc(var(--space-2) * -1);
    padding: var(--space-1) var(--space-2);
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    font: inherit;
    font-size: var(--text-body);
    line-height: var(--text-body-lh);
    color: var(--text-tertiary);
    cursor: pointer;
    transition:
      background var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out);
  }
  .volver:hover {
    background: var(--surface-sunken);
    color: var(--text-primary);
  }

  kbd {
    padding: 0 5px;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    font-family: var(--font-mono);
    font-size: var(--text-code);
    color: var(--text-tertiary);
  }

  .titulo {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-4);
    margin-top: var(--space-3);
  }

  .nombre {
    display: flex;
    flex: 1;
    flex-direction: column;
    gap: var(--space-2);
    min-width: 0;
  }

  h1 {
    margin: 0;
    overflow-wrap: anywhere;
    font-size: var(--text-title);
    line-height: var(--text-title-lh);
    font-weight: var(--text-title-weight);
    letter-spacing: var(--tracking-tight);
    color: var(--text-primary);
  }

  .ruta {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    min-width: 0;
    margin: 0;
  }
  .ruta code {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-family: var(--font-mono);
    font-size: var(--text-code);
    line-height: var(--text-code-lh);
    color: var(--text-tertiary);
  }

  .estado {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-1) var(--space-3);
    margin: 0;
    font-size: var(--text-meta);
    line-height: var(--text-meta-lh);
    color: var(--text-secondary);
  }
  .rama,
  .delta {
    font-family: var(--font-mono);
    font-size: var(--text-code);
  }
  .sucio {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    color: var(--warning);
  }
  .sucio::before {
    content: '';
    width: 6px;
    height: 6px;
    border-radius: var(--radius-pill);
    background: currentColor;
  }
  .delta {
    color: var(--text-tertiary);
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
    padding: 6px var(--space-3);
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

  .etiquetas {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-1);
    margin-top: var(--space-4);
  }

  .ausente {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin: var(--space-4) 0 0;
    padding: var(--space-2) var(--space-3);
    border: 1px solid color-mix(in oklab, var(--warning) 40%, transparent);
    border-radius: var(--radius-md);
    background: var(--warning-surface);
    font-size: var(--text-body);
    color: var(--text-primary);
  }

  .cuerpo {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(16rem, 21rem);
    gap: var(--space-6);
    align-items: start;
    padding-block: var(--space-5) var(--space-7);
  }

  .principal {
    min-width: 0;
    padding: var(--space-5);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-lg);
    background: var(--surface-raised);
  }

  .lateral {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
    min-width: 0;
  }

  h2 {
    margin: 0 0 var(--space-3);
    font-size: var(--text-meta);
    line-height: var(--text-meta-lh);
    font-weight: 600;
    color: var(--text-secondary);
  }

  .perdido {
    display: flex;
    flex: 1;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-3);
    color: var(--text-secondary);
  }

  button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  @media (max-width: 860px) {
    .cuerpo {
      grid-template-columns: minmax(0, 1fr);
    }
  }
</style>
