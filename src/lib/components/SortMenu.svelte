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

<div class="relative flex shrink-0 items-center gap-1">
  <button
    bind:this={trigger}
    type="button"
    onclick={() => (open ? close() : (open = true))}
    aria-expanded={open}
    aria-haspopup="menu"
    title="Cambiar la ordenación"
    class="inline-flex items-center gap-2 rounded-md border border-surface-border px-3 py-1.5
           text-sm text-content transition hover:bg-surface-2 focus-visible:outline-2
           focus-visible:outline-offset-2 focus-visible:outline-accent"
  >
    <ArrowUpDown size={14} />
    {activeLabel}
  </button>

  <button
    type="button"
    onclick={() => setSortDirection(ascending ? 'desc' : 'asc')}
    aria-label={ascending ? 'Orden ascendente, cambiar a descendente' : 'Orden descendente, cambiar a ascendente'}
    title={ascending ? 'Ascendente' : 'Descendente'}
    class="rounded-md border border-surface-border p-1.5 text-content-muted transition
           hover:bg-surface-2 hover:text-content focus-visible:outline-2
           focus-visible:outline-offset-2 focus-visible:outline-accent"
  >
    {#if ascending}
      <ArrowUp size={14} />
    {:else}
      <ArrowDown size={14} />
    {/if}
  </button>

  {#if open}
    <div
      use:dismissable={{ onDismiss: close }}
      role="menu"
      aria-label="Ordenar por"
      class="absolute right-0 top-full z-30 mt-1 w-52 rounded-lg border border-surface-border
             bg-surface-1 p-1 shadow-lg"
    >
      {#each OPTIONS as option (option.value)}
        <button
          type="button"
          role="menuitemradio"
          aria-checked={$filters.sort === option.value}
          onclick={() => choose(option.value)}
          class="flex w-full items-center justify-between gap-2 rounded-md px-2 py-1.5 text-left
                 text-sm transition hover:bg-surface-2"
          class:text-content-strong={$filters.sort === option.value}
          class:text-content={$filters.sort !== option.value}
        >
          {option.label}
          {#if $filters.sort === option.value}
            {#if ascending}
              <ArrowUp size={13} class="text-accent" />
            {:else}
              <ArrowDown size={13} class="text-accent" />
            {/if}
          {/if}
        </button>
      {/each}
    </div>
  {/if}
</div>
