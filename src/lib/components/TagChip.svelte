<script lang="ts">
  import { X } from '@lucide/svelte';

  import { t } from '../i18n';
  import type { Tag } from '../types';

  interface Props {
    /** La etiqueta que se pinta. */
    tag: Tag;
    /** Lleva una X para quitarla. */
    removable?: boolean;
    /** Se ha pulsado la X. */
    onRemove?: () => void;
    /** Si se da, el chip entero es un botón. */
    onClick?: () => void;
    /** Número que se enseña al lado del nombre; `null` para ninguno. */
    count?: number | null;
    /** Tooltip; por defecto, el nombre. */
    title?: string;
    /** Oculta la X hasta tener el ratón o el foco: en las tarjetas, una X fija es ruido. */
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

  // Una sola gramática: fondo y borde tintados y texto mezclado con la tinta, con los
  // porcentajes de `--tag-chip-*`, que cambian con el tema. Sin variante rellena: con
  // etiquetas claras, el texto blanco se quedaba en 2:1.
  let style = $derived(
    `--chip-color: ${tag.color};` +
      ' background-color: color-mix(in oklab, var(--chip-color) var(--tag-chip-bg), transparent);' +
      ' border-color: color-mix(in oklab, var(--chip-color) var(--tag-chip-border), transparent);' +
      ' color: color-mix(in oklab, var(--chip-color) var(--tag-chip-text), var(--tag-chip-ink));',
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
        aria-label={$t('chip.quitar', { nombre: tag.name })}
        title={$t('chip.quitar', { nombre: tag.name })}
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

  /* Oculta, la X sigue ocupando su hueco: el chip no cambia de ancho. Se revela sobre este
     chip y no sobre la tarjeta, que sacaba todas las X a la vez. */
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
