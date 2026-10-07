<script lang="ts">
  import { Keyboard, X } from '@lucide/svelte';

  import { dismissable } from '../utils/dismissable';
  import type { ShortcutInfo } from '../utils/shortcuts';

  interface Props {
    /** Los atajos, ya con la tecla de mando de este sistema. */
    shortcuts: ShortcutInfo[];
    onClose: () => void;
  }

  let { shortcuts, onClose }: Props = $props();
</script>

<div class="fondo" role="presentation">
  <div
    class="ayuda"
    role="dialog"
    aria-modal="true"
    aria-labelledby="titulo-atajos"
    use:dismissable={{ onDismiss: onClose }}
  >
    <header>
      <Keyboard size={18} strokeWidth={1.75} />
      <h2 id="titulo-atajos">Atajos de teclado</h2>
      <button type="button" class="cerrar" onclick={onClose} aria-label="Cerrar">
        <X size={16} strokeWidth={1.75} />
      </button>
    </header>
    <dl>
      {#each shortcuts as shortcut (shortcut.keys)}
        <div>
          <dt><kbd>{shortcut.keys}</kbd></dt>
          <dd>{shortcut.description}</dd>
        </div>
      {/each}
    </dl>
  </div>
</div>

<style>
  .fondo {
    position: fixed;
    inset: 0;
    z-index: 60;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-5);
    background: color-mix(in oklab, var(--surface-base) 60%, transparent);
  }

  .ayuda {
    width: 100%;
    max-width: 28rem;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-lg);
    background: var(--surface-overlay);
    box-shadow: var(--shadow-lg);
  }

  header {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-4);
    border-bottom: 1px solid var(--border-subtle);
    color: var(--text-tertiary);
  }
  h2 {
    flex: 1;
    margin: 0;
    font-size: var(--text-section);
    font-weight: var(--text-section-weight);
    color: var(--text-primary);
  }

  .cerrar {
    display: inline-flex;
    padding: var(--space-1);
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-tertiary);
    cursor: pointer;
  }
  .cerrar:hover {
    background: var(--surface-sunken);
    color: var(--text-primary);
  }
  .cerrar:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  dl {
    display: flex;
    flex-direction: column;
    margin: 0;
    padding: var(--space-2) var(--space-4) var(--space-4);
  }
  dl div {
    display: grid;
    grid-template-columns: 6rem minmax(0, 1fr);
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-2) 0;
    border-bottom: 1px solid var(--border-subtle);
  }
  dl div:last-child {
    border-bottom: 0;
  }
  dt {
    margin: 0;
  }
  dd {
    margin: 0;
    font-size: var(--text-body);
    color: var(--text-secondary);
  }

  kbd {
    padding: 1px 6px;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    background: var(--surface-raised);
    font-family: var(--font-mono);
    font-size: var(--text-code);
    color: var(--text-primary);
  }
</style>
