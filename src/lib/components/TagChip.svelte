<script lang="ts">
  import { X } from '@lucide/svelte';

  import type { Tag } from '../types';
  import { chipBackground, chipBorder } from '../utils/tagColors';

  interface Props {
    tag: Tag;
    selected?: boolean;
    removable?: boolean;
    onRemove?: () => void;
    onClick?: () => void;
    count?: number | null;
    title?: string;
    revealOnHover?: boolean;
  }

  let {
    tag,
    selected = false,
    removable = false,
    onRemove,
    onClick,
    count = null,
    title,
    revealOnHover = false,
  }: Props = $props();

  let style = $derived(
    selected
      ? `background-color: ${tag.color}; border-color: ${tag.color}; color: white;`
      : `background-color: ${chipBackground(tag.color)}; border-color: ${chipBorder(tag.color)}; color: ${tag.color};`,
  );

  const SHAPE =
    'inline-flex max-w-full items-center gap-1 rounded-full border px-2 py-0.5 text-xs' +
    ' leading-4 transition focus-visible:outline-2 focus-visible:outline-offset-1' +
    ' focus-visible:outline-accent';
</script>

{#if onClick}
  <button
    type="button"
    onclick={onClick}
    aria-pressed={selected}
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
        class="-mr-1 rounded-full p-0.5 transition hover:bg-black/10 focus-visible:opacity-100
               focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-accent
               dark:hover:bg-white/20"
        class:opacity-0={revealOnHover}
        class:group-hover:opacity-100={revealOnHover}
        class:group-focus-within:opacity-100={revealOnHover}
      >
        <X size={10} />
      </button>
    {/if}
  </span>
{/if}
