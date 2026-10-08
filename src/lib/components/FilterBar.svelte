<script lang="ts">
  import { Check, ListFilter, Pin, TriangleAlert, X } from '@lucide/svelte';

  import TagChip from './TagChip.svelte';
  import {
    clearFilters,
    filterChipCount,
    filters,
    hasActiveFilters,
    setGitState,
    setIncludeMissing,
    setPinnedOnly,
    toggleLanguageFilter,
    toggleTagFilter,
  } from '../stores/filters';
  import { availableLanguages } from '../stores/projects';
  import { tagsById } from '../stores/tags';
  import type { GitStateFilter } from '../types';
  import { dismissable } from '../utils/dismissable';

  const VISIBLE_TAG_CHIPS = 3;

  const GIT_STATES: { value: GitStateFilter; label: string }[] = [
    { value: 'all', label: 'Cualquiera' },
    { value: 'dirty', label: 'Con cambios' },
    { value: 'clean', label: 'Sin cambios' },
    { value: 'no_repo', label: 'Sin repositorio' },
  ];

  let open = $state(false);
  let expandTags = $state(false);
  let trigger = $state<HTMLButtonElement | null>(null);

  let selectedTags = $derived(
    $filters.tag_ids
      .map((id) => $tagsById.get(id))
      .filter((tag): tag is NonNullable<typeof tag> => tag !== undefined),
  );

  let shownTags = $derived(
    expandTags ? selectedTags : selectedTags.slice(0, VISIBLE_TAG_CHIPS),
  );
  let hiddenTagCount = $derived(selectedTags.length - shownTags.length);

  let gitStateLabel = $derived(
    GIT_STATES.find((state) => state.value === $filters.git_state)?.label ?? 'Cualquiera',
  );

  let count = $derived(filterChipCount($filters));

  function close() {
    open = false;
    trigger?.focus();
  }
</script>

<div class="barra">
  <div class="ancla">
    <button
      bind:this={trigger}
      type="button"
      class="disparador"
      onclick={() => (open ? close() : (open = true))}
      aria-expanded={open}
      aria-haspopup="dialog"
    >
      <ListFilter size={14} strokeWidth={1.75} />
      Filtros
      {#if count > 0}
        <span class="contador">{count}</span>
      {/if}
    </button>

    {#if open}
      <div use:dismissable={{ onDismiss: close }} role="dialog" aria-label="Filtros" class="panel">
        <p class="rotulo">Estado Git</p>
        {#each GIT_STATES as state (state.value)}
          <button
            type="button"
            class="opcion"
            role="menuitemradio"
            aria-checked={$filters.git_state === state.value}
            onclick={() => setGitState(state.value)}
          >
            {state.label}
            {#if $filters.git_state === state.value}
              <Check size={14} strokeWidth={1.75} class="marca" />
            {/if}
          </button>
        {/each}

        {#if $availableLanguages.length > 0}
          <p class="rotulo separado">Lenguaje</p>
          <div class="lenguajes">
            {#each $availableLanguages as language (language)}
              <button
                type="button"
                class="opcion"
                role="menuitemcheckbox"
                aria-checked={$filters.languages.includes(language)}
                onclick={() => toggleLanguageFilter(language)}
              >
                {language}
                {#if $filters.languages.includes(language)}
                  <Check size={14} strokeWidth={1.75} class="marca" />
                {/if}
              </button>
            {/each}
          </div>
        {/if}

        <div class="grupo separado">
          <button
            type="button"
            class="opcion"
            role="menuitemcheckbox"
            aria-checked={$filters.pinned_only}
            onclick={() => setPinnedOnly(!$filters.pinned_only)}
          >
            Solo destacados
            {#if $filters.pinned_only}
              <Check size={14} strokeWidth={1.75} class="marca" />
            {/if}
          </button>

          <button
            type="button"
            class="opcion"
            role="menuitemcheckbox"
            aria-checked={!$filters.include_missing}
            onclick={() => setIncludeMissing(!$filters.include_missing)}
          >
            Ocultar ausentes
            {#if !$filters.include_missing}
              <Check size={14} strokeWidth={1.75} class="marca" />
            {/if}
          </button>
        </div>
      </div>
    {/if}
  </div>

  {#each shownTags as tag (tag.id)}
    <TagChip {tag} removable onRemove={() => toggleTagFilter(tag.id)} />
  {/each}

  {#if hiddenTagCount > 0}
    <button
      type="button"
      class="mas"
      onclick={() => (expandTags = true)}
      title={selectedTags
        .slice(VISIBLE_TAG_CHIPS)
        .map((tag) => tag.name)
        .join(', ')}
    >
      +{hiddenTagCount} etiquetas
    </button>
  {/if}

  {#each $filters.languages as language (language)}
    <span class="chip">
      {language}
      <button
        type="button"
        class="quitar"
        onclick={() => toggleLanguageFilter(language)}
        aria-label="Quitar el filtro de lenguaje {language}"
      >
        <X size={10} />
      </button>
    </span>
  {/each}

  {#if $filters.git_state !== 'all'}
    <span class="chip">
      Git: {gitStateLabel}
      <button
        type="button"
        class="quitar"
        onclick={() => setGitState('all')}
        aria-label="Quitar el filtro de estado Git"
      >
        <X size={10} />
      </button>
    </span>
  {/if}

  {#if $filters.pinned_only}
    <span class="chip">
      <Pin size={10} />
      Solo destacados
      <button
        type="button"
        class="quitar"
        onclick={() => setPinnedOnly(false)}
        aria-label="Quitar el filtro de destacados"
      >
        <X size={10} />
      </button>
    </span>
  {/if}

  {#if !$filters.include_missing}
    <span class="chip">
      <TriangleAlert size={10} />
      Ausentes ocultos
      <button
        type="button"
        class="quitar"
        onclick={() => setIncludeMissing(true)}
        aria-label="Volver a mostrar los proyectos ausentes"
      >
        <X size={10} />
      </button>
    </span>
  {/if}

  {#if $hasActiveFilters}
    <button
      type="button"
      class="limpiar"
      onclick={() => {
        expandTags = false;
        clearFilters();
      }}
    >
      Limpiar filtros
    </button>
  {/if}
</div>

<style>
  .barra {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
  }

  .ancla {
    position: relative;
    flex: none;
  }

  .disparador {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    padding: 5px var(--space-3);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
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
  .disparador:hover,
  .disparador[aria-expanded='true'] {
    background: var(--surface-sunken);
    color: var(--text-primary);
  }

  .contador {
    min-width: 18px;
    padding-inline: 5px;
    border-radius: var(--radius-pill);
    background: var(--accent-surface);
    font-size: var(--text-meta);
    line-height: var(--text-meta-lh);
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    text-align: center;
    color: var(--accent);
  }

  .panel {
    position: absolute;
    top: 100%;
    left: 0;
    z-index: 30;
    width: 16rem;
    margin-top: var(--space-1);
    padding: var(--space-2);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    background: var(--surface-overlay);
    box-shadow: var(--shadow-lg);
  }

  .rotulo {
    margin: 0;
    padding: 0 var(--space-2) var(--space-1);
    font-size: var(--text-meta);
    line-height: var(--text-meta-lh);
    font-weight: 500;
    letter-spacing: var(--tracking-wide);
    text-transform: uppercase;
    color: var(--text-tertiary);
  }

  .separado {
    margin-top: var(--space-2);
    padding-top: var(--space-2);
    border-top: 1px solid var(--border-subtle);
  }

  .lenguajes {
    max-height: 10rem;
    overflow-y: auto;
  }

  .opcion {
    display: flex;
    width: 100%;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-1) var(--space-2);
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    font: inherit;
    font-size: var(--text-body);
    line-height: var(--text-body-lh);
    text-align: left;
    color: var(--text-secondary);
    cursor: pointer;
    transition: background var(--duration-fast) var(--ease-out);
  }
  .opcion:hover {
    background: var(--surface-sunken);
    color: var(--text-primary);
  }
  .opcion[aria-checked='true'] {
    color: var(--text-primary);
  }
  .opcion :global(.marca) {
    color: var(--accent);
  }

  /*
   * Los filtros que no son etiquetas siguen la receta del chip de etiqueta
   * (fondo, borde y texto con los porcentajes de `--tag-chip-*`) sobre el
   * neutro secundario. Es la misma gramática, solo que sin color: lo que lo
   * distingue de una etiqueta es estar en esta fila y llevar la X a la vista.
   */
  .chip {
    --chip-color: var(--text-secondary);
    display: inline-flex;
    max-width: 100%;
    align-items: center;
    gap: var(--space-1);
    padding: 2px var(--space-2);
    border: 1px solid color-mix(in oklab, var(--chip-color) var(--tag-chip-border), transparent);
    border-radius: var(--radius-pill);
    background: color-mix(in oklab, var(--chip-color) var(--tag-chip-bg), transparent);
    font-size: var(--text-meta);
    line-height: 16px;
    font-weight: var(--text-meta-weight);
    color: color-mix(in oklab, var(--chip-color) var(--tag-chip-text), var(--tag-chip-ink));
  }

  /* Mismo hueco y misma X que `TagChip`, para que las dos filas casen. */
  .quitar {
    display: inline-flex;
    margin-right: calc(var(--space-1) * -1);
    padding: 2px;
    border: 0;
    border-radius: var(--radius-pill);
    background: transparent;
    color: inherit;
    cursor: pointer;
    transition: background var(--duration-fast) var(--ease-out);
  }
  .quitar:hover {
    background: color-mix(in oklab, var(--chip-color) var(--tag-chip-border), transparent);
  }

  .mas {
    padding: 2px var(--space-2);
    border: 1px dashed var(--border-default);
    border-radius: var(--radius-pill);
    background: transparent;
    font: inherit;
    font-size: var(--text-meta);
    line-height: 16px;
    color: var(--text-tertiary);
    cursor: pointer;
  }
  .mas:hover {
    color: var(--text-secondary);
  }

  .limpiar {
    flex: none;
    margin-left: auto;
    padding: 0;
    border: 0;
    background: none;
    font: inherit;
    font-size: var(--text-meta);
    line-height: var(--text-meta-lh);
    color: var(--text-tertiary);
    cursor: pointer;
    text-underline-offset: 2px;
  }
  .limpiar:hover {
    color: var(--text-primary);
    text-decoration: underline;
  }

  button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
</style>
