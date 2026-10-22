<script lang="ts" module>
  /** Última altura medida de una tarjeta, por densidad, para dimensionar los huecos. */
  const lastHeight: Record<'comodo' | 'compacto', number> = { comodo: 172, compacto: 104 };
</script>

<script lang="ts">
  import { tick } from 'svelte';

  import ProjectCard from './ProjectCard.svelte';
  import type { ProjectWithTags } from '../types';
  import { nearViewport } from '../utils/nearViewport';

  interface Props {
    /** El proyecto de la tarjeta. */
    project: ProjectWithTags;
    /** Densidad compacta. */
    compact: boolean;
    /** Montar la tarjeta desde el principio, sin esperar al observador. */
    eager?: boolean;
  }

  let { project, compact, eager = false }: Props = $props();

  let wrapper = $state<HTMLDivElement | null>(null);
  // `eager` solo decide el primer estado; luego manda el observador.
  // svelte-ignore state_referenced_locally
  let near = $state(eager);
  let density = $derived<'comodo' | 'compacto'>(compact ? 'compacto' : 'comodo');

  function onNearChange(isNear: boolean) {
    if (isNear || !wrapper) {
      // Si el hueco tenía el foco, se lo pasa a la tarjeta: al sustituirlo se perdería.
      if (isNear && !near) void mount(wrapper?.contains(document.activeElement) ?? false);
      return;
    }
    // No se desmonta lo que el usuario está usando: un foco o un selector abierto.
    if (wrapper.contains(document.activeElement)) return;
    lastHeight[density] = wrapper.offsetHeight;
    near = false;
  }

  /** Monta la tarjeta y, si se pide, pasa el foco a su nombre. */
  async function mount(withFocus: boolean) {
    near = true;
    if (!withFocus) return;
    await tick();
    // Se busca dentro de la tarjeta: el hueco también lleva `data-proyecto`, y
    // darle el foco a él justo antes de que desaparezca lo perdía.
    for (let intento = 0; intento < 5; intento++) {
      const nombre = wrapper?.querySelector<HTMLElement>('article [data-proyecto]');
      if (nombre) {
        nombre.focus();
        return;
      }
      await new Promise((r) => requestAnimationFrame(r));
    }
  }
</script>

<div class="celda" bind:this={wrapper} use:nearViewport={onNearChange}>
  {#if near}
    <ProjectCard {project} {compact} />
  {:else}
    <button
      type="button"
      class="hueco"
      data-proyecto={project.id}
      style:height="{lastHeight[density]}px"
      aria-label={project.name}
      onfocus={() => mount(true)}
    ></button>
  {/if}
</div>

<style>
  .celda {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  /* Solo la tarjeta se estira hasta el alto de la fila. Con `flex: 1` el hueco
     perdía su altura, y quinientos huecos cabían en una pantalla. */
  .celda > :global(article) {
    flex: 1;
  }

  /* Ocupa el sitio de la tarjeta sin pintarse: se ve solo lo que está cerca. */
  .hueco {
    display: block;
    width: 100%;
    padding: 0;
    border: 1px solid transparent;
    border-radius: var(--radius-md);
    background: transparent;
  }
</style>
