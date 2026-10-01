<script lang="ts">
  import { X } from '@lucide/svelte';

  import type { Tag } from '../types';

  interface Props {
    tag: Tag;
    removable?: boolean;
    onRemove?: () => void;
    onClick?: () => void;
    count?: number | null;
    title?: string;
    /**
     * Oculta la X hasta que **este chip** recibe el ratón o el foco.
     *
     * Es lo que usan las tarjetas del tablero: con varias etiquetas por tarjeta
     * y cientos de tarjetas, una X permanente en cada una es ruido. La X ocupa
     * su hueco igualmente, así que los chips no se mueven al pasar el cursor, y
     * sigue en el orden de tabulación: al llegar con el teclado se hace visible
     * por `:focus-within` y se activa con Enter o Espacio, como cualquier botón.
     *
     * El gestor de etiquetas no lo usa: allí la acción de quitar está siempre a
     * la vista, porque es la pantalla a la que se va justamente a eso.
     */
    revealOnHover?: boolean;
  }

  let {
    tag,
    removable = false,
    onRemove,
    onClick,
    count = null,
    title,
    revealOnHover = false,
  }: Props = $props();

  /**
   * Una sola gramática: fondo y borde tintados, texto en el color pleno.
   *
   * Antes había una variante rellena para los filtros activos, con texto
   * blanco sobre el color de la etiqueta. Con etiquetas de luminosidad alta el
   * contraste se quedaba en torno a 2:1, y además un filtro se veía distinto
   * de la etiqueta que filtra.
   *
   * Los porcentajes salen de `--tag-chip-*` en `tokens.css`, que cambian con
   * el tema. Antes eran sufijos de alfa fijos en `tagColors.ts`, iguales en
   * claro y en oscuro.
   */
  let style = $derived(
    `--chip-color: ${tag.color};` +
      ' background-color: color-mix(in oklab, var(--chip-color) var(--tag-chip-bg), transparent);' +
      ' border-color: color-mix(in oklab, var(--chip-color) var(--tag-chip-border), transparent);' +
      ' color: color-mix(in oklab, var(--chip-color) var(--tag-chip-text), transparent);',
  );
</script>

{#if onClick}
  <button type="button" class="chip pulsable" onclick={onClick} title={title ?? tag.name} {style}>
    <span class="nombre">{tag.name}</span>
    {#if count !== null}
      <span class="cuenta">{count}</span>
    {/if}
  </button>
{:else}
  <span class="chip" {style} title={title ?? tag.name}>
    <span class="nombre">{tag.name}</span>
    {#if count !== null}
      <span class="cuenta">{count}</span>
    {/if}
    {#if removable && onRemove}
      <button
        type="button"
        class="quitar"
        class:oculta={revealOnHover}
        onclick={onRemove}
        aria-label="Quitar la etiqueta {tag.name}"
        title="Quitar la etiqueta {tag.name}"
      >
        <X size={10} />
      </button>
    {/if}
  </span>
{/if}

<style>
  /* La misma forma que los chips neutros de `FilterBar`: una sola gramática. */
  .chip {
    display: inline-flex;
    max-width: 100%;
    align-items: center;
    gap: var(--space-1);
    padding: 2px var(--space-2);
    border: 1px solid;
    border-radius: var(--radius-pill);
    font: inherit;
    font-size: var(--text-meta);
    line-height: 16px;
    font-weight: var(--text-meta-weight);
  }

  .pulsable {
    cursor: pointer;
    transition: filter var(--duration-fast) var(--ease-out);
  }
  .pulsable:hover {
    filter: brightness(1.1);
  }

  .nombre {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .cuenta {
    opacity: 0.7;
  }

  .quitar {
    display: inline-flex;
    margin-right: calc(var(--space-1) * -1);
    padding: 2px;
    border: 0;
    border-radius: var(--radius-pill);
    background: transparent;
    color: inherit;
    cursor: pointer;
    transition:
      background var(--duration-fast) var(--ease-out),
      opacity var(--duration-fast) var(--ease-out);
  }
  .quitar:hover {
    background: color-mix(in oklab, var(--chip-color) var(--tag-chip-border), transparent);
  }

  /*
   * La X oculta ocupa su hueco igualmente, así que el chip no cambia de ancho
   * al pasar el cursor. Se revela con el hover o el foco sobre **este** chip,
   * no sobre la tarjeta: antes dependía del hover de la tarjeta entera y
   * aparecían tres X a la vez.
   */
  .quitar.oculta {
    opacity: 0;
  }
  .chip:hover > .quitar.oculta,
  .chip:focus-within > .quitar.oculta {
    opacity: 1;
  }

  .chip:focus-visible,
  .quitar:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }
</style>
