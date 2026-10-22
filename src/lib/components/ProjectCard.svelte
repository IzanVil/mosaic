<script lang="ts">
  import { Code, FolderOpen, Pin, PinOff, SquareTerminal, TriangleAlert } from '@lucide/svelte';

  import TagChip from './TagChip.svelte';
  import TagPicker from './TagPicker.svelte';
  import { lang, t } from '../i18n';
  import { openProject } from '../stores/apps';
  import { openProjectDetail } from '../stores/navigation';
  import { togglePinned } from '../stores/projects';
  import { unassignTagFromProject } from '../stores/tags';
  import type { ProjectWithTags } from '../types';
  import { formatRelativeTime } from '../utils/format';

  interface Props {
    /** El proyecto de la tarjeta. */
    project: ProjectWithTags;
    /** Sin ruta ni pie, para que quepan más filas en el modo compacto. */
    compact?: boolean;
  }

  let { project, compact = false }: Props = $props();

  const VISIBLE_TAGS = 3;

  let status = $derived(project.git_status ?? undefined);
  let shownTags = $derived(project.tags.slice(0, VISIBLE_TAGS));
  let hiddenTags = $derived(project.tags.slice(VISIBLE_TAGS));

  /** Las dos últimas partes de la ruta: el prefijo es el de todas; la entera, en el tooltip. */
  let shortPath = $derived(project.path.split('/').filter(Boolean).slice(-2).join('/'));

  let branchLabel = $derived(
    status?.branch ?? (status?.last_commit_sha ? status.last_commit_sha.slice(0, 7) : null),
  );

  const ACTIONS = [
    { kind: 'ide', icon: Code, label: 'tarjeta.abrirEditor' },
    { kind: 'terminal', icon: SquareTerminal, label: 'tarjeta.abrirTerminal' },
    { kind: 'file_manager', icon: FolderOpen, label: 'tarjeta.abrirCarpeta' },
  ] as const;
</script>

<article class="tarjeta" class:fijada={project.pinned} class:ausente={project.missing}>
  <header>
    <div class="titulo">
      <h3>
        <button
          type="button"
          class="nombre"
          data-proyecto={project.id}
          onclick={() => openProjectDetail(project.id)}
          aria-describedby="pista-{project.id}"
        >
          {project.name}
        </button>
      </h3>
      <span class="pista" id="pista-{project.id}" role="tooltip">{$t('tarjeta.pista')}</span>
    </div>

    <button
      type="button"
      class="pin"
      class:activo={project.pinned}
      onclick={() => togglePinned(project.id, !project.pinned)}
      title={project.pinned ? $t('tarjeta.dejarDestacar') : $t('tarjeta.destacar')}
      aria-label={project.pinned
        ? $t('tarjeta.dejarDestacarNombre', { nombre: project.name })
        : $t('tarjeta.destacarNombre', { nombre: project.name })}
      aria-pressed={project.pinned}
    >
      {#if project.pinned}
        <PinOff size={14} strokeWidth={1.75} />
      {:else}
        <Pin size={14} strokeWidth={1.75} />
      {/if}
    </button>
  </header>

  {#if !compact}
    <p class="ruta" title={project.path}>{shortPath}</p>
  {/if}

  <div class="etiquetas">
    {#each shownTags as tag (tag.id)}
      <TagChip
        {tag}
        removable
        revealOnHover
        onRemove={() => unassignTagFromProject(project.id, tag.id)}
      />
    {/each}
    {#if hiddenTags.length > 0}
      <span class="mas" title={hiddenTags.map((tag) => tag.name).join(', ')}>
        +{hiddenTags.length}
      </span>
    {/if}

    <span class="anadir">
      <TagPicker
        projectId={project.id}
        assigned={project.tags}
        compact={project.tags.length > 0}
        plain={project.tags.length === 0}
      />
    </span>
  </div>

  <p class="estado">
    {#if project.primary_language}<span class="lenguaje">{project.primary_language}</span>{/if}
    {#if branchLabel}
      {#if project.primary_language}<span class="sep">·</span>{/if}
      <span class="rama" title={status?.branch === null ? $t('tarjeta.headSeparado') : undefined}>
        {branchLabel}
      </span>
      {#if status?.is_dirty}
        <span class="sucio" title={$t('tarjeta.sucio')}>●</span>
      {/if}
      {#if status?.ahead}<span class="delta" title={$t('tarjeta.porDelante')}>↑{status.ahead}</span>{/if}
      {#if status?.behind}<span class="delta" title={$t('tarjeta.porDetras')}>↓{status.behind}</span>{/if}
    {/if}
    {#if project.missing}
      <span class="ausente-aviso" title={$t('tarjeta.ausente')}>
        <TriangleAlert size={12} strokeWidth={1.75} />
        {$t('tarjeta.noEncontrado')}
      </span>
    {/if}
  </p>

  {#if !compact}
    <footer>
      <span class="apertura">
        {project.last_opened_at === null
          ? $t('tarjeta.sinAbrir')
          : $t('tarjeta.abierto', { cuando: formatRelativeTime(project.last_opened_at, $lang) })}
      </span>

      <div class="acciones">
        {#each ACTIONS as action (action.kind)}
          <button
            type="button"
            onclick={() => openProject(action.kind, project.id)}
            disabled={project.missing}
            title={$t(action.label)}
            aria-label="{$t(action.label)}: {project.name}"
          >
            <action.icon size={14} strokeWidth={1.75} />
          </button>
        {/each}
      </div>
    </footer>
  {/if}
</article>

<style>
  .tarjeta {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    /* La rejilla fija el relleno según la densidad; el 16 es el de reserva. */
    padding: var(--card-padding, var(--space-4));
    min-width: 0;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    background: var(--surface-raised);
    box-shadow: var(--shadow-sm);
    transition:
      border-color var(--duration-fast) var(--ease-out),
      background var(--duration-fast) var(--ease-out);
  }
  .tarjeta:hover {
    border-color: var(--border-strong);
    background: var(--surface-sunken);
  }
  .tarjeta.fijada {
    border-color: var(--accent-border);
    box-shadow: var(--shadow-sm), inset 2px 0 0 var(--accent);
  }
  .tarjeta.ausente {
    opacity: 0.55;
  }

  header {
    display: flex;
    align-items: start;
    justify-content: space-between;
    gap: var(--space-2);
  }

  h3 {
    margin: 0;
    min-width: 0;
    font-size: var(--text-card);
    line-height: var(--text-card-lh);
    font-weight: var(--text-card-weight);
    letter-spacing: var(--tracking-tight);
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  /* Un botón, para que llegue el teclado; `data-proyecto` es donde vuelve el foco del detalle. */
  .nombre {
    display: block;
    max-width: 100%;
    padding: 0;
    border: 0;
    border-radius: var(--radius-sm);
    background: none;
    font: inherit;
    letter-spacing: inherit;
    color: inherit;
    text-align: left;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    cursor: pointer;
    transition: color var(--duration-fast) var(--ease-out);
  }
  /* Subrayado además del color: que se lea como algo que se pulsa. */
  .nombre:hover {
    color: var(--accent);
    text-decoration: underline;
    text-underline-offset: 3px;
  }

  /* Fuera del `h3`, que recorta con `overflow: hidden`. */
  .titulo {
    position: relative;
    min-width: 0;
  }
  .pista {
    position: absolute;
    top: 100%;
    left: 0;
    z-index: 5;
    margin-top: var(--space-1);
    padding: 2px var(--space-2);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    background: var(--surface-overlay);
    box-shadow: var(--shadow-sm);
    font-size: var(--text-meta);
    line-height: var(--text-meta-lh);
    color: var(--text-secondary);
    white-space: nowrap;
    pointer-events: none;
    opacity: 0;
    transition: opacity var(--duration-fast) var(--ease-out);
  }
  .titulo:has(.nombre:hover) .pista,
  .titulo:has(.nombre:focus-visible) .pista {
    opacity: 1;
  }

  .ruta {
    margin: 0;
    font-family: var(--font-mono);
    font-size: var(--text-code);
    line-height: var(--text-code-lh);
    color: var(--text-tertiary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  /* Siempre, tenga etiquetas o no: si no, la rejilla estiraba la tarjeta y el hueco quedaba
     abajo, donde no significa nada. */
  .etiquetas {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-1);
    min-height: 22px;
  }

  /* Solo asoma al pasar o con el teclado. `display: none` y no opacidad: invisible seguía
     ocupando sitio y, en compacto, reservaba una línea vacía. */
  .anadir {
    display: none;
  }
  .tarjeta:hover .anadir,
  .tarjeta:focus-within .anadir {
    display: inline-flex;
  }

  .mas {
    font-size: var(--text-meta);
    color: var(--text-tertiary);
    padding-inline: var(--space-1);
  }

  .estado {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-1);
    margin: 0;
    font-size: var(--text-meta);
    line-height: var(--text-meta-lh);
    color: var(--text-secondary);
  }
  .lenguaje {
    color: var(--text-secondary);
  }
  .sep {
    color: var(--text-disabled);
  }
  .rama {
    font-family: var(--font-mono);
    font-size: var(--text-code);
    color: var(--text-secondary);
  }
  .sucio {
    color: var(--warning);
    font-size: 9px;
    line-height: 1;
  }
  .delta {
    font-family: var(--font-mono);
    font-size: var(--text-code);
    color: var(--text-tertiary);
  }
  .ausente-aviso {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    color: var(--text-tertiary);
  }

  footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
    margin-top: var(--space-1);
    min-height: 26px;
  }

  .apertura {
    font-size: var(--text-meta);
    color: var(--text-tertiary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .acciones {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    flex: none;
    opacity: 0;
    transition: opacity var(--duration-fast) var(--ease-out);
  }
  .tarjeta:hover .acciones,
  .tarjeta:focus-within .acciones {
    opacity: 1;
  }

  .acciones button {
    display: inline-flex;
    padding: 5px;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-tertiary);
    cursor: pointer;
    transition:
      background var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out);
  }
  .acciones button:hover:not(:disabled) {
    background: var(--surface-base);
    color: var(--text-primary);
  }
  .acciones button:disabled {
    color: var(--text-disabled);
    cursor: not-allowed;
  }

  .pin {
    flex: none;
    display: inline-flex;
    padding: 4px;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-disabled);
    cursor: pointer;
    opacity: 0;
    transition: opacity var(--duration-fast) var(--ease-out);
  }
  .pin.activo {
    opacity: 1;
    color: var(--accent);
  }
  .tarjeta:hover .pin,
  .tarjeta:focus-within .pin {
    opacity: 1;
  }
  .pin:hover {
    color: var(--text-primary);
  }

  button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
</style>
