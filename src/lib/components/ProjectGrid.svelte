<script lang="ts">
  import { Pin, SearchX } from '@lucide/svelte';

  import EmptyState from './EmptyState.svelte';
  import LazyCard from './LazyCard.svelte';
  import { t } from '../i18n';
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

  // Las que caben en la primera pantalla se montan sin esperar al observador, para
  // que el tablero no aparezca a saltos al abrirlo.
  const EAGER = 24;
</script>

{#if total === 0}
  <EmptyState
    title={$t('tablero.ningunoCoincideTitulo')}
    description={$t('tablero.ningunoCoincideTexto')}
    actionLabel={$t('filtros.limpiar')}
    onAction={clearFilters}
  >
    {#snippet icon()}
      <SearchX size={32} strokeWidth={1.75} />
    {/snippet}
  </EmptyState>
{:else}
  <div class="tablero" class:compacto={density === 'compacto'}>
    {#if pinned.length > 0}
      <section aria-label={$t('tablero.fijadosSeccion')}>
        <h2><Pin size={12} strokeWidth={1.75} /> {$t('tablero.fijados')}</h2>
        <div class="rejilla">
          {#each pinned as project, i (project.id)}
            <LazyCard {project} compact={density === 'compacto'} eager={i < EAGER} />
          {/each}
        </div>
      </section>
    {/if}

    {#if pinned.length > 0 && rest.length > 0}
      <hr />
    {/if}

    {#if rest.length > 0}
      <section aria-label={$t('tablero.restoSeccion')}>
        <div class="rejilla">
          {#each rest as project, i (project.id)}
            <LazyCard {project} compact={density === 'compacto'} eager={i < EAGER} />
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
