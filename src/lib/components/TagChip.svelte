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
    /** Sin fondo, solo el punto y el nombre: en la tarjeta, una píldora pesa más que el proyecto. */
    plain?: boolean;
  }

  let {
    tag,
    removable = false,
    onRemove,
    onClick,
    count = null,
    title,
    revealOnHover = false,
    plain = false,
  }: Props = $props();
</script>

{#if onClick}
  <button
    type="button"
    class="chip pulsable"
    class:plain
    onclick={onClick}
    title={title ?? tag.name}
    style:--chip-color={tag.color}
  >
    <span class="punto" aria-hidden="true"></span>
    <span class="nombre">{tag.name}</span>
    {#if count !== null}
      <span class="cuenta">{count}</span>
    {/if}
  </button>
{:else}
  <span class="chip" class:plain title={title ?? tag.name} style:--chip-color={tag.color}>
    <span class="punto" aria-hidden="true"></span>
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
  /* El color va solo en el punto y el nombre en neutro: con el chip entero teñido, las
     etiquetas pesaban más que el nombre del proyecto. Misma píldora que los chips neutros
     de `FilterBar`. */
  .chip {
    display: inline-flex;
    max-width: 100%;
    align-items: center;
    gap: var(--tag-dot-gap);
    padding: 2px var(--space-2);
    border: 0;
    border-radius: var(--radius-pill);
    background: var(--surface-raised);
    font: inherit;
    font-size: var(--text-meta);
    line-height: 16px;
    font-weight: var(--text-meta-weight);
    color: var(--text-primary);
  }
  .chip.plain {
    padding: 0;
    background: none;
    color: var(--text-secondary);
  }

  .punto {
    flex: none;
    width: var(--tag-dot);
    height: var(--tag-dot);
    border-radius: var(--tag-tile-radius);
    background: var(--chip-color);
  }

  .pulsable {
    cursor: pointer;
    transition: background var(--duration-fast) var(--ease-out);
  }
  .pulsable:hover {
    background: var(--surface-sunken);
  }

  .nombre {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .cuenta {
    color: var(--text-tertiary);
  }

  .quitar {
    display: inline-flex;
    margin-left: calc(var(--space-1) - var(--tag-dot-gap));
    margin-right: calc(var(--space-1) * -1);
    padding: 2px;
    border: 0;
    border-radius: var(--radius-pill);
    background: transparent;
    color: var(--text-tertiary);
    cursor: pointer;
    transition:
      background var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out),
      opacity var(--duration-fast) var(--ease-out);
  }
  .quitar:hover {
    background: var(--surface-sunken);
    color: var(--text-primary);
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
