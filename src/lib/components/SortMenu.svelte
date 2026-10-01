<script lang="ts">
  import { ArrowDown, ArrowUp, ArrowUpDown } from '@lucide/svelte';

  import { filters, setSort, setSortDirection } from '../stores/filters';
  import type { SortOption } from '../types';
  import { dismissable } from '../utils/dismissable';

  const OPTIONS: { value: SortOption; label: string }[] = [
    { value: 'name', label: 'Nombre' },
    { value: 'last_opened', label: 'Última apertura' },
    { value: 'created', label: 'Fecha de alta' },
    { value: 'updated', label: 'Última actualización' },
  ];

  let open = $state(false);
  let trigger = $state<HTMLButtonElement | null>(null);

  let activeLabel = $derived(
    OPTIONS.find((option) => option.value === $filters.sort)?.label ?? 'Nombre',
  );
  let ascending = $derived($filters.sort_dir === 'asc');

  function close() {
    open = false;
    trigger?.focus();
  }

  function choose(option: SortOption) {
    setSort(option);
    close();
  }
</script>

<div class="orden">
  <button
    bind:this={trigger}
    type="button"
    class="boton"
    onclick={() => (open ? close() : (open = true))}
    aria-expanded={open}
    aria-haspopup="menu"
    title="Cambiar la ordenación"
  >
    <ArrowUpDown size={14} strokeWidth={1.75} />
    {activeLabel}
  </button>

  <button
    type="button"
    class="boton icono"
    onclick={() => setSortDirection(ascending ? 'desc' : 'asc')}
    aria-label={ascending ? 'Orden ascendente, cambiar a descendente' : 'Orden descendente, cambiar a ascendente'}
    title={ascending ? 'Ascendente' : 'Descendente'}
  >
    {#if ascending}
      <ArrowUp size={14} strokeWidth={1.75} />
    {:else}
      <ArrowDown size={14} strokeWidth={1.75} />
    {/if}
  </button>

  {#if open}
    <div use:dismissable={{ onDismiss: close }} role="menu" aria-label="Ordenar por" class="menu">
      {#each OPTIONS as option (option.value)}
        <button
          type="button"
          class="opcion"
          role="menuitemradio"
          aria-checked={$filters.sort === option.value}
          onclick={() => choose(option.value)}
        >
          {option.label}
          {#if $filters.sort === option.value}
            {#if ascending}
              <ArrowUp size={14} strokeWidth={1.75} class="marca" />
            {:else}
              <ArrowDown size={14} strokeWidth={1.75} class="marca" />
            {/if}
          {/if}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .orden {
    position: relative;
    display: flex;
    flex: none;
    align-items: center;
    gap: var(--space-1);
  }

  .boton {
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
  .boton:hover,
  .boton[aria-expanded='true'] {
    background: var(--surface-sunken);
    color: var(--text-primary);
  }
  .boton.icono {
    padding: 6px;
    color: var(--text-tertiary);
  }

  .menu {
    position: absolute;
    top: 100%;
    right: 0;
    z-index: 30;
    width: 13rem;
    margin-top: var(--space-1);
    padding: var(--space-1);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    background: var(--surface-overlay);
    box-shadow: var(--shadow-lg);
  }

  .opcion {
    display: flex;
    width: 100%;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
    padding: 6px var(--space-2);
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

  button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
</style>
