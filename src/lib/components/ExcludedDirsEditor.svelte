<script lang="ts">
  import { Plus, RotateCcw, X } from '@lucide/svelte';
  import { onDestroy } from 'svelte';

  interface Props {
    /** Carpetas excluidas guardadas. */
    dirs: string[];
    /** Las de fábrica, para «Restaurar». */
    defaults: string[];
    /** Cuántas puede haber como mucho. */
    max: number;
    /** Guarda la lista entera; devuelve si se guardó. */
    onChange: (dirs: string[]) => Promise<boolean>;
  }

  let { dirs, defaults, max, onChange }: Props = $props();

  let draft = $state('');
  let localError = $state<string | null>(null);
  let confirmingRestore = $state(false);
  let confirmTimer: ReturnType<typeof setTimeout> | undefined;
  onDestroy(() => clearTimeout(confirmTimer));

  let isDefault = $derived(
    dirs.length === defaults.length && dirs.every((dir, i) => dir === defaults[i]),
  );

  /**
   * Lo que el backend rechazaría, comprobado antes para decirlo junto al campo
   * sin esperar al error. El backend vuelve a comprobarlo de todos modos.
   */
  function problem(name: string): string | null {
    if (name === '') return 'Escribe el nombre de una carpeta.';
    if (name.includes('/') || name.includes('\\')) return 'Es un nombre de carpeta, sin barras.';
    if (dirs.includes(name)) return `«${name}» ya está en la lista.`;
    if (dirs.length >= max) return `No caben más de ${max}.`;
    return null;
  }

  async function add() {
    const name = draft.trim();
    localError = problem(name);
    if (localError !== null) return;
    if (await onChange([...dirs, name])) draft = '';
  }

  function remove(name: string) {
    void onChange(dirs.filter((dir) => dir !== name));
  }

  /** Dos clics: el primero avisa de lo que se pierde, el segundo restaura. */
  function restore() {
    if (!confirmingRestore) {
      confirmingRestore = true;
      confirmTimer = setTimeout(() => (confirmingRestore = false), 4000);
      return;
    }
    clearTimeout(confirmTimer);
    confirmingRestore = false;
    void onChange([...defaults]);
  }
</script>

<div class="editor">
  {#if dirs.length === 0}
    <p class="vacio">Ninguna: el escáner lo recorre todo, incluido <code>node_modules</code>.</p>
  {:else}
    <ul class="chips" aria-label="Carpetas excluidas">
      {#each dirs as dir (dir)}
        <li>
          <code>{dir}</code>
          <button type="button" onclick={() => remove(dir)} aria-label="Dejar de excluir {dir}" title="Dejar de excluir {dir}">
            <X size={12} strokeWidth={2} />
          </button>
        </li>
      {/each}
    </ul>
  {/if}

  <div class="anadir">
    <input
      type="text"
      bind:value={draft}
      oninput={() => (localError = null)}
      onkeydown={(event) => {
        if (event.key === 'Enter') {
          event.preventDefault();
          void add();
        }
      }}
      placeholder="Nombre de carpeta, por ejemplo vendor"
      aria-label="Carpeta que excluir"
      spellcheck="false"
      autocomplete="off"
    />
    <button type="button" class="boton" onclick={add}>
      <Plus size={14} strokeWidth={1.75} /> Añadir
    </button>
    <button type="button" class="boton" class:aviso={confirmingRestore} onclick={restore} disabled={isDefault}>
      <RotateCcw size={14} strokeWidth={1.75} />
      {confirmingRestore ? 'Pulsa otra vez: se quitan tus cambios' : 'Restaurar las predeterminadas'}
    </button>
  </div>
  {#if localError}
    <p class="error" role="alert">{localError}</p>
  {/if}
</div>

<style>
  .editor {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    width: 100%;
  }

  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-1);
    margin: 0;
    padding: 0;
    list-style: none;
  }
  .chips li {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    padding: 1px 2px 1px var(--space-2);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-pill);
    background: var(--surface-raised);
  }
  .chips code {
    font-family: var(--font-mono);
    font-size: var(--text-code);
    line-height: var(--text-code-lh);
    color: var(--text-secondary);
  }
  .chips button {
    display: inline-flex;
    padding: 3px;
    border: 0;
    border-radius: var(--radius-pill);
    background: transparent;
    color: var(--text-tertiary);
    cursor: pointer;
  }
  .chips button:hover {
    background: var(--surface-sunken);
    color: var(--text-primary);
  }

  .vacio {
    margin: 0;
    font-size: var(--text-meta);
    color: var(--warning);
  }
  .vacio code {
    font-family: var(--font-mono);
  }

  .anadir {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }
  .anadir input {
    flex: 1;
    min-width: 12rem;
    padding: 6px var(--space-3);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    background: var(--surface-raised);
    font-family: var(--font-mono);
    font-size: var(--text-code);
    color: var(--text-primary);
  }
  .anadir input::placeholder {
    font-family: var(--font-sans);
    color: var(--text-tertiary);
  }

  .boton {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    padding: 6px var(--space-3);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    background: transparent;
    font: inherit;
    font-size: var(--text-body);
    color: var(--text-secondary);
    cursor: pointer;
  }
  .boton:hover:not(:disabled) {
    background: var(--surface-sunken);
    color: var(--text-primary);
  }
  .boton:disabled {
    color: var(--text-disabled);
    cursor: default;
  }
  .boton.aviso {
    border-color: color-mix(in oklab, var(--warning) 50%, transparent);
    color: var(--warning);
  }

  .error {
    margin: 0;
    font-size: var(--text-meta);
    color: var(--danger);
  }

  button:focus-visible,
  input:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
</style>
