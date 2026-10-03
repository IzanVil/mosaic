<script lang="ts">
  import { Check, Copy } from '@lucide/svelte';
  import { onDestroy } from 'svelte';

  import { copyText } from '../api/clipboard';

  interface Props {
    /** Ruta que se copia al portapapeles. */
    path: string;
  }

  let { path }: Props = $props();

  /** Lo que se ve tras copiar dura 1,5 s: lo justo para leerlo. */
  const FEEDBACK_MS = 1500;

  let state = $state<'idle' | 'copied' | 'failed'>('idle');
  let timer: ReturnType<typeof setTimeout> | undefined;
  onDestroy(() => clearTimeout(timer));

  async function copy() {
    clearTimeout(timer);
    try {
      await copyText(path);
      state = 'copied';
    } catch {
      state = 'failed';
    }
    timer = setTimeout(() => (state = 'idle'), FEEDBACK_MS);
  }
</script>

<button type="button" class="copiar" class:hecho={state === 'copied'} onclick={copy}>
  {#if state === 'copied'}
    <Check size={14} strokeWidth={1.75} />
  {:else}
    <Copy size={14} strokeWidth={1.75} />
  {/if}
  <span aria-live="polite">
    {state === 'copied' ? 'Copiado' : state === 'failed' ? 'No se pudo copiar' : 'Copiar ruta'}
  </span>
</button>

<style>
  .copiar {
    display: inline-flex;
    flex: none;
    align-items: center;
    gap: var(--space-1);
    padding: 3px var(--space-2);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    background: transparent;
    font: inherit;
    font-size: var(--text-meta);
    line-height: var(--text-meta-lh);
    color: var(--text-tertiary);
    cursor: pointer;
    transition:
      background var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out);
  }
  .copiar:hover {
    background: var(--surface-sunken);
    color: var(--text-primary);
  }
  .copiar.hecho {
    border-color: var(--accent-border);
    color: var(--accent);
  }
  .copiar:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
</style>
