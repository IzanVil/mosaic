<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    icon?: Snippet;
    title: string;
    description: string;
    actionLabel?: string;
    onAction?: () => void;
  }

  let { icon, title, description, actionLabel, onAction }: Props = $props();
</script>

<div class="flex flex-col items-center justify-center gap-3 px-6 py-16 text-center">
  {#if icon}
    <div class="text-content-muted">{@render icon()}</div>
  {/if}

  <h2 class="text-lg font-medium text-content-strong">{title}</h2>
  <p class="max-w-sm text-sm text-content-muted">{description}</p>

  {#if actionLabel && onAction}
    <button
      type="button"
      onclick={onAction}
      class="mt-2 rounded-md bg-accent px-4 py-2 text-sm font-medium text-surface-0
             transition hover:bg-accent-strong focus-visible:outline-2
             focus-visible:outline-offset-2 focus-visible:outline-accent"
    >
      {actionLabel}
    </button>
  {/if}
</div>
