<script lang="ts">
  import { Check, ListFilter, Pin, TriangleAlert, X } from '@lucide/svelte';

  import TagChip from './TagChip.svelte';
  import {
    activeFilterCount,
    clearFilters,
    filters,
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

  let count = $derived(activeFilterCount($filters));

  function close() {
    open = false;
    trigger?.focus();
  }
</script>

<div class="flex flex-wrap items-center gap-2">
  <div class="relative shrink-0">
    <button
      bind:this={trigger}
      type="button"
      onclick={() => (open ? close() : (open = true))}
      aria-expanded={open}
      aria-haspopup="dialog"
      class="inline-flex items-center gap-2 rounded-md border border-surface-border px-3 py-1.5
             text-sm text-content transition hover:bg-surface-2 focus-visible:outline-2
             focus-visible:outline-offset-2 focus-visible:outline-accent"
    >
      <ListFilter size={14} />
      Filtros
      {#if count > 0}
        <span class="rounded-full bg-accent px-1.5 text-xs font-medium text-surface-0">
          {count}
        </span>
      {/if}
    </button>

    {#if open}
      <div
        use:dismissable={{ onDismiss: close }}
        role="dialog"
        aria-label="Filtros"
        class="absolute left-0 top-full z-30 mt-1 w-64 rounded-lg border border-surface-border
               bg-surface-1 p-2 shadow-lg"
      >
        <p class="px-2 pb-1 text-xs font-medium uppercase tracking-wide text-content-muted">
          Estado Git
        </p>
        {#each GIT_STATES as state (state.value)}
          <button
            type="button"
            role="menuitemradio"
            aria-checked={$filters.git_state === state.value}
            onclick={() => setGitState(state.value)}
            class="flex w-full items-center justify-between rounded-md px-2 py-1 text-left text-sm
                   text-content transition hover:bg-surface-2"
          >
            {state.label}
            {#if $filters.git_state === state.value}
              <Check size={13} class="text-accent" />
            {/if}
          </button>
        {/each}

        {#if $availableLanguages.length > 0}
          <p
            class="mt-2 border-t border-surface-border px-2 pb-1 pt-2 text-xs font-medium uppercase
                   tracking-wide text-content-muted"
          >
            Lenguaje
          </p>
          <div class="max-h-40 overflow-y-auto">
            {#each $availableLanguages as language (language)}
              <button
                type="button"
                role="menuitemcheckbox"
                aria-checked={$filters.languages.includes(language)}
                onclick={() => toggleLanguageFilter(language)}
                class="flex w-full items-center justify-between rounded-md px-2 py-1 text-left
                       text-sm text-content transition hover:bg-surface-2"
              >
                {language}
                {#if $filters.languages.includes(language)}
                  <Check size={13} class="text-accent" />
                {/if}
              </button>
            {/each}
          </div>
        {/if}

        <div class="mt-2 border-t border-surface-border pt-2">
          <button
            type="button"
            role="menuitemcheckbox"
            aria-checked={$filters.pinned_only}
            onclick={() => setPinnedOnly(!$filters.pinned_only)}
            class="flex w-full items-center justify-between rounded-md px-2 py-1 text-left text-sm
                   text-content transition hover:bg-surface-2"
          >
            Solo destacados
            {#if $filters.pinned_only}
              <Check size={13} class="text-accent" />
            {/if}
          </button>

          <button
            type="button"
            role="menuitemcheckbox"
            aria-checked={!$filters.include_missing}
            onclick={() => setIncludeMissing(!$filters.include_missing)}
            class="flex w-full items-center justify-between rounded-md px-2 py-1 text-left text-sm
                   text-content transition hover:bg-surface-2"
          >
            Ocultar ausentes
            {#if !$filters.include_missing}
              <Check size={13} class="text-accent" />
            {/if}
          </button>
        </div>
      </div>
    {/if}
  </div>

  {#each shownTags as tag (tag.id)}
    <TagChip {tag} selected removable onRemove={() => toggleTagFilter(tag.id)} />
  {/each}

  {#if hiddenTagCount > 0}
    <button
      type="button"
      onclick={() => (expandTags = true)}
      title={selectedTags
        .slice(VISIBLE_TAG_CHIPS)
        .map((tag) => tag.name)
        .join(', ')}
      class="rounded-full border border-surface-border px-2 py-0.5 text-xs text-content-muted
             transition hover:bg-surface-2 focus-visible:outline-2 focus-visible:outline-offset-1
             focus-visible:outline-accent"
    >
      +{hiddenTagCount} etiquetas
    </button>
  {/if}

  {#each $filters.languages as language (language)}
    <span
      class="inline-flex items-center gap-1 rounded-full border border-surface-border bg-surface-1
             px-2 py-0.5 text-xs text-content"
    >
      {language}
      <button
        type="button"
        onclick={() => toggleLanguageFilter(language)}
        aria-label="Quitar el filtro de lenguaje {language}"
        class="-mr-1 rounded-full p-0.5 transition hover:bg-surface-2 focus-visible:outline-2
               focus-visible:outline-offset-1 focus-visible:outline-accent"
      >
        <X size={10} />
      </button>
    </span>
  {/each}

  {#if $filters.git_state !== 'all'}
    <span
      class="inline-flex items-center gap-1 rounded-full border border-surface-border bg-surface-1
             px-2 py-0.5 text-xs text-content"
    >
      Git: {gitStateLabel}
      <button
        type="button"
        onclick={() => setGitState('all')}
        aria-label="Quitar el filtro de estado Git"
        class="-mr-1 rounded-full p-0.5 transition hover:bg-surface-2 focus-visible:outline-2
               focus-visible:outline-offset-1 focus-visible:outline-accent"
      >
        <X size={10} />
      </button>
    </span>
  {/if}

  {#if $filters.pinned_only}
    <span
      class="inline-flex items-center gap-1 rounded-full border border-surface-border bg-surface-1
             px-2 py-0.5 text-xs text-content"
    >
      <Pin size={10} />
      Solo destacados
      <button
        type="button"
        onclick={() => setPinnedOnly(false)}
        aria-label="Quitar el filtro de destacados"
        class="-mr-1 rounded-full p-0.5 transition hover:bg-surface-2 focus-visible:outline-2
               focus-visible:outline-offset-1 focus-visible:outline-accent"
      >
        <X size={10} />
      </button>
    </span>
  {/if}

  {#if !$filters.include_missing}
    <span
      class="inline-flex items-center gap-1 rounded-full border border-surface-border bg-surface-1
             px-2 py-0.5 text-xs text-content"
    >
      <TriangleAlert size={10} />
      Ausentes ocultos
      <button
        type="button"
        onclick={() => setIncludeMissing(true)}
        aria-label="Volver a mostrar los proyectos ausentes"
        class="-mr-1 rounded-full p-0.5 transition hover:bg-surface-2 focus-visible:outline-2
               focus-visible:outline-offset-1 focus-visible:outline-accent"
      >
        <X size={10} />
      </button>
    </span>
  {/if}

  {#if count > 0}
    <button
      type="button"
      onclick={() => {
        expandTags = false;
        clearFilters();
      }}
      class="ml-auto shrink-0 text-xs text-content-muted underline-offset-2 transition
             hover:text-content hover:underline focus-visible:outline-2
             focus-visible:outline-offset-2 focus-visible:outline-accent"
    >
      Limpiar filtros
    </button>
  {/if}
</div>
