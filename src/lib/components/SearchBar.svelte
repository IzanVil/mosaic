<script lang="ts">
  import { onDestroy, untrack } from 'svelte';
  import { Search, X } from '@lucide/svelte';
  import { get } from 'svelte/store';

  import { filters, setQuery } from '../stores/filters';
  import { projects, visibleProjects } from '../stores/projects';
  import { debounce } from '../utils/debounce';

  const DEBOUNCE_MS = 150;

  let value = $state(get(filters).query);

  const push = debounce(setQuery, DEBOUNCE_MS);
  onDestroy(push.cancel);

  $effect(() => {
    const incoming = $filters.query;
    untrack(() => {
      if (incoming === value) return;
      push.cancel();
      value = incoming;
    });
  });

  function onInput() {
    push(value);
  }

  function clear() {
    push.cancel();
    value = '';
    setQuery('');
  }

  let showCount = $derived($projects.length > 0);
</script>

<div class="flex items-center gap-3">
  <div class="relative min-w-0 flex-1">
    <Search
      size={15}
      class="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-content-muted"
    />
    <input
      bind:value
      oninput={onInput}
      type="search"
      placeholder="Buscar por nombre o ruta…"
      aria-label="Buscar proyectos"
      class="w-full rounded-md border border-surface-border bg-surface-1 py-1.5 pl-9 pr-8 text-sm
             text-content placeholder:text-content-muted focus-visible:outline-2
             focus-visible:outline-offset-1 focus-visible:outline-accent
             [&::-webkit-search-cancel-button]:hidden"
    />
    {#if value !== ''}
      <button
        type="button"
        onclick={clear}
        aria-label="Limpiar la búsqueda"
        title="Limpiar la búsqueda"
        class="absolute right-2 top-1/2 -translate-y-1/2 rounded p-1 text-content-muted transition
               hover:bg-surface-2 hover:text-content focus-visible:outline-2
               focus-visible:outline-offset-1 focus-visible:outline-accent"
      >
        <X size={13} />
      </button>
    {/if}
  </div>

  {#if showCount}
    <p class="shrink-0 text-xs text-content-muted" aria-live="polite">
      {$visibleProjects.length} de {$projects.length}
    </p>
  {/if}
</div>
