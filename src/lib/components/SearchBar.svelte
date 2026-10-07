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

<div class="busqueda">
  <div class="campo">
    <Search size={14} strokeWidth={1.75} class="lupa" />
    <input
      id="busqueda"
      bind:value
      oninput={onInput}
      type="search"
      placeholder="Buscar por nombre o ruta…"
      aria-label="Buscar proyectos"
    />
    {#if value !== ''}
      <button
        type="button"
        class="borrar"
        onclick={clear}
        aria-label="Limpiar la búsqueda"
        title="Limpiar la búsqueda"
      >
        <X size={13} strokeWidth={1.75} />
      </button>
    {/if}
  </div>

  {#if showCount}
    <p class="cuenta" aria-live="polite">
      {$visibleProjects.length} de {$projects.length}
    </p>
  {/if}
</div>

<style>
  .busqueda {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  .campo {
    position: relative;
    flex: 1;
    min-width: 0;
  }

  .campo :global(.lupa) {
    position: absolute;
    top: 50%;
    left: var(--space-3);
    transform: translateY(-50%);
    color: var(--text-tertiary);
    pointer-events: none;
  }

  input {
    width: 100%;
    box-sizing: border-box;
    padding: 5px 32px 5px 34px;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    background: var(--surface-raised);
    font: inherit;
    font-size: var(--text-body);
    line-height: var(--text-body-lh);
    color: var(--text-primary);
    transition: border-color var(--duration-fast) var(--ease-out);
  }
  input::placeholder {
    color: var(--text-tertiary);
  }
  input:hover {
    border-color: var(--border-strong);
  }
  input:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
    border-color: var(--accent-border);
  }
  /* La X nativa duplicaría la nuestra, que además limpia sin esperar al debounce. */
  input::-webkit-search-cancel-button {
    display: none;
  }

  .borrar {
    position: absolute;
    top: 50%;
    right: var(--space-2);
    transform: translateY(-50%);
    display: inline-flex;
    padding: var(--space-1);
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-tertiary);
    cursor: pointer;
    transition:
      background var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out);
  }
  .borrar:hover {
    background: var(--surface-sunken);
    color: var(--text-primary);
  }
  .borrar:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }

  .cuenta {
    flex: none;
    margin: 0;
    font-size: var(--text-meta);
    line-height: var(--text-meta-lh);
    font-variant-numeric: tabular-nums;
    color: var(--text-tertiary);
  }
</style>
