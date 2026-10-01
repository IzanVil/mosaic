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

  /**
   * Clases que revelan la X con el ratón o el foco sobre el propio chip.
   *
   * Antes dependía de `group-hover`, es decir del hover de la tarjeta entera,
   * que hacía aparecer tres X a la vez al pasar por encima.
   */
  const REVEAL =
    'opacity-0 [:hover>&]:opacity-100 [:focus-within>&]:opacity-100 focus-visible:opacity-100';

  const SHAPE =
    'inline-flex max-w-full items-center gap-1 rounded-full border px-2 py-0.5 text-xs' +
    ' leading-4 transition focus-visible:outline-2 focus-visible:outline-offset-1' +
    ' focus-visible:outline-accent';
</script>

{#if onClick}
  <button
    type="button"
    onclick={onClick}
    title={title ?? tag.name}
    class="{SHAPE} cursor-pointer hover:brightness-110"
    {style}
  >
    <span class="truncate">{tag.name}</span>
    {#if count !== null}
      <span class="opacity-70">{count}</span>
    {/if}
  </button>
{:else}
  <span class={SHAPE} {style} title={title ?? tag.name}>
    <span class="truncate">{tag.name}</span>
    {#if count !== null}
      <span class="opacity-70">{count}</span>
    {/if}
    {#if removable && onRemove}
      <button
        type="button"
        onclick={onRemove}
        aria-label="Quitar la etiqueta {tag.name}"
        title="Quitar la etiqueta {tag.name}"
        class="-mr-1 rounded-full p-0.5 transition hover:bg-black/10 focus-visible:outline-2
               focus-visible:outline-offset-1 focus-visible:outline-accent dark:hover:bg-white/20
               {removable && revealOnHover ? REVEAL : ''}"
      >
        <X size={10} />
      </button>
    {/if}
  </span>
{/if}
