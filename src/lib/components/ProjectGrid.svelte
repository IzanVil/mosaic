<script lang="ts">
  import { Pin, SearchX } from '@lucide/svelte';

  import EmptyState from './EmptyState.svelte';
  import ProjectCard from './ProjectCard.svelte';
  import { clearFilters } from '../stores/filters';
  import type { Density, ProjectWithTags } from '../types';

  interface Props {
    /** Proyectos destacados, que se pintan siempre en el bloque de arriba. */
    pinned: ProjectWithTags[];
    /** El resto, ya filtrado y ordenado. */
    rest: ProjectWithTags[];
    density: Density;
  }

  let { pinned, rest, density }: Props = $props();

  let total = $derived(pinned.length + rest.length);
</script>

{#if total === 0}
  <EmptyState
    title="Ningún proyecto coincide con los filtros"
    description="Prueba con otra búsqueda, o quita los filtros para volver a ver todo."
    actionLabel="Limpiar filtros"
    onAction={clearFilters}
  >
    {#snippet icon()}
      <SearchX size={32} strokeWidth={1.75} />
    {/snippet}
  </EmptyState>
{:else}
  <div class="tablero" class:compacto={density === 'compacto'}>
    {#if pinned.length > 0}
      <section aria-label="Proyectos fijados">
        <h2><Pin size={12} strokeWidth={1.75} /> Fijados</h2>
        <div class="rejilla">
          {#each pinned as project (project.id)}
            <ProjectCard {project} />
          {/each}
        </div>
      </section>
    {/if}

    {#if pinned.length > 0 && rest.length > 0}
      <hr />
    {/if}

    {#if rest.length > 0}
      <section aria-label="Resto de proyectos">
        <div class="rejilla">
          {#each rest as project (project.id)}
            <ProjectCard {project} />
          {/each}
        </div>
      </section>
    {/if}
  </div>
{/if}

<style>
  /*
   * Las dos densidades salen de los tokens, no de clases sueltas: la compacta
   * cambia el ancho mínimo de columna, el hueco entre tarjetas y el relleno
   * interior, y ese último lo lee `ProjectCard` de la variable que se fija
   * aquí. Con 34 proyectos, la cómoda muestra seis tarjetas y media y la
   * compacta pasa de quince.
   */
  .tablero {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
    padding: var(--space-5) var(--space-5) var(--space-7);
    --columna: var(--card-min-comodo);
    --hueco: var(--card-gap-comodo);
    --card-padding: var(--card-padding-comodo);
  }
  .tablero.compacto {
    gap: var(--space-4);
    padding: var(--space-4) var(--space-4) var(--space-6);
    --columna: var(--card-min-compacto);
    --hueco: var(--card-gap-compacto);
    --card-padding: var(--card-padding-compacto);
  }

  section {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  h2 {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    margin: 0;
    font-size: var(--text-meta);
    font-weight: var(--text-meta-weight);
    line-height: var(--text-meta-lh);
    letter-spacing: var(--tracking-wide);
    text-transform: uppercase;
    color: var(--text-tertiary);
  }

  .rejilla {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(var(--columna), 1fr));
    gap: var(--hueco);
  }

  hr {
    margin: 0;
    border: 0;
    border-top: 1px solid var(--border-subtle);
  }
</style>
