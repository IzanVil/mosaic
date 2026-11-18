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
  import { parentFolder } from '../utils/path';
  import { tagBand } from '../utils/tagColors';

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
  // Solo la carpeta que lo contiene: el nombre ya está encima y repetirlo era ruido.
  let folder = $derived(parentFolder(project.path));
  let band = $derived(tagBand(project.tags.map((tag) => tag.color)));

  let branchLabel = $derived(
    status?.branch ?? (status?.last_commit_sha ? status.last_commit_sha.slice(0, 7) : null),
  );

  const ACTIONS = [
    { kind: 'ide', icon: Code, label: 'tarjeta.abrirEditor' },
    { kind: 'terminal', icon: SquareTerminal, label: 'tarjeta.abrirTerminal' },
    { kind: 'file_manager', icon: FolderOpen, label: 'tarjeta.abrirCarpeta' },
  ] as const;
</script>

<article
  class="tarjeta"
  class:fijada={project.pinned}
  class:ausente={project.missing}
  style:--cenefa={band}
>
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
    <p class="ruta" title={project.path}>{folder}</p>
  {/if}

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

  <div class="etiquetas">
    {#each shownTags as tag (tag.id)}
      <TagChip
        {tag}
        plain
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
  /* Un azulejo: la única superficie blanca del tablero, con su cenefa encima. */
  .tarjeta {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: var(--card-stack);
    /* La rejilla fija el relleno según la densidad; el 16 es el de reserva. Arriba se suma
       la cenefa, que va encima del relleno y no dentro. */
    --relleno: var(--card-padding, var(--space-4));
    padding: calc(var(--relleno) + var(--tile-band) - var(--space-1)) var(--relleno) var(--space-3);
    min-width: 0;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    background: var(--surface-card);
    transition: border-color var(--duration-fast) var(--ease-out);
  }
  /* La cenefa: los colores de sus etiquetas, o la neutra si no tiene. Redondeada a mano y
     no con `overflow: hidden`, que recortaría el selector de etiquetas y la pista. */
  .tarjeta::before {
    content: '';
    position: absolute;
    inset: 0 0 auto;
    height: var(--tile-band);
    border-radius: calc(var(--radius-sm) - 1px) calc(var(--radius-sm) - 1px) 0 0;
    background: var(--cenefa, var(--border-default));
  }
  .tarjeta:hover,
  .tarjeta:focus-within {
    border-color: var(--border-strong);
  }
  /* El estado lleva el acento; el grupo «Fijados» y la chincheta dicen ya el resto. */
  .tarjeta.fijada {
    border-color: var(--accent-border);
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
    font-size: var(--text-card-title);
    line-height: var(--text-card-title-lh);
    font-weight: var(--text-card-title-weight);
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
    gap: var(--space-1) var(--space-2);
    /* El alto del «+» que asoma al pasar: sin esto, la fila crecía bajo el ratón. */
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
    min-height: 24px;
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
    background: var(--surface-sunken);
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
