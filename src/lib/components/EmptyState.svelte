<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    /** Icono opcional, renderizado sobre el título. */
    icon?: Snippet;
    title: string;
    description: string;
    /** Si se indican ambos, se muestra un botón de acción. */
    actionLabel?: string;
    onAction?: () => void;
  }

  let { icon, title, description, actionLabel, onAction }: Props = $props();
</script>

<div class="vacio">
  {#if icon}
    <div class="icono">{@render icon()}</div>
  {/if}

  <h2>{title}</h2>
  <p>{description}</p>

  {#if actionLabel && onAction}
    <button type="button" onclick={onAction}>{actionLabel}</button>
  {/if}
</div>

<style>
  .vacio {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-3);
    padding: var(--space-8) var(--space-5);
    text-align: center;
  }

  .icono {
    display: inline-flex;
    margin-bottom: var(--space-1);
    color: var(--text-tertiary);
  }

  h2 {
    margin: 0;
    font-size: var(--text-title);
    line-height: var(--text-title-lh);
    font-weight: var(--text-title-weight);
    letter-spacing: var(--tracking-tight);
    color: var(--text-primary);
  }

  p {
    max-width: 24rem;
    margin: 0;
    font-size: var(--text-body);
    line-height: var(--text-body-lh);
    color: var(--text-secondary);
  }

  button {
    margin-top: var(--space-2);
    padding: var(--space-2) var(--space-4);
    border: 0;
    border-radius: var(--radius-md);
    background: var(--accent);
    font: inherit;
    font-size: var(--text-body);
    line-height: var(--text-body-lh);
    font-weight: 500;
    color: var(--text-on-accent);
    cursor: pointer;
    transition: background var(--duration-fast) var(--ease-out);
  }
  button:hover {
    background: var(--accent-hover);
  }
  button:active {
    background: var(--accent-active);
  }
  button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
</style>
