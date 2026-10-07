<script lang="ts">
  import { ArrowRight, FolderGit2, Search } from '@lucide/svelte';
  import { tick } from 'svelte';

  import { openProject } from '../stores/apps';
  import { openProjectDetail } from '../stores/navigation';
  import { projects, rankProjects } from '../stores/projects';
  import { dismissable } from '../utils/dismissable';
  import { paletteItems, type PaletteAction, type PaletteItem } from '../utils/palette';

  interface Props {
    /** Acciones que ofrece la paleta, además de los proyectos. */
    actions: PaletteAction[];
    /** Ejecuta una acción por su id. */
    onAction: (id: string) => void;
    onClose: () => void;
    /** Tecla de mando de este sistema, para la pista de abajo. */
    modKey: string;
  }

  let { actions, onAction, onClose, modKey }: Props = $props();

  let query = $state('');
  let selected = $state(0);
  let input = $state<HTMLInputElement | null>(null);
  let list = $state<HTMLUListElement | null>(null);

  let items = $derived(paletteItems(query, actions, $projects, rankProjects));

  $effect(() => {
    input?.focus();
  });

  /** Al cambiar el texto se vuelve al primer resultado. */
  function onInput() {
    selected = 0;
  }

  function move(delta: number) {
    if (items.length === 0) return;
    selected = (selected + delta + items.length) % items.length;
    void tick().then(() =>
      list?.querySelector<HTMLElement>(`[data-indice="${selected}"]`)?.scrollIntoView({ block: 'nearest' }),
    );
  }

  /**
   * Ejecuta un resultado. Con la tecla de mando, un proyecto se abre en el
   * editor en lugar de en su detalle.
   */
  function run(item: PaletteItem | undefined, inEditor = false) {
    if (!item) return;
    onClose();
    if (item.kind === 'action') {
      onAction(item.action.id);
    } else if (inEditor) {
      void openProject('ide', item.project.id);
    } else {
      openProjectDetail(item.project.id);
    }
  }

  function onKeydown(event: KeyboardEvent) {
    if (event.key === 'ArrowDown') {
      event.preventDefault();
      move(1);
    } else if (event.key === 'ArrowUp') {
      event.preventDefault();
      move(-1);
    } else if (event.key === 'Enter') {
      event.preventDefault();
      run(items[selected], event.ctrlKey || event.metaKey);
    }
  }

  let firstProject = $derived(items.findIndex((item) => item.kind === 'project'));
</script>

<div class="fondo" role="presentation">
  <div
    class="paleta"
    role="dialog"
    aria-modal="true"
    aria-label="Paleta de comandos"
    use:dismissable={{ onDismiss: onClose }}
  >
    <div class="buscar">
      <Search size={16} strokeWidth={1.75} />
      <input
        bind:this={input}
        bind:value={query}
        oninput={onInput}
        onkeydown={onKeydown}
        type="text"
        placeholder="Busca un proyecto o una acción…"
        spellcheck="false"
        autocomplete="off"
        role="combobox"
        aria-expanded="true"
        aria-controls="paleta-resultados"
        aria-activedescendant={items.length > 0 ? `paleta-${selected}` : undefined}
        aria-label="Buscar en la paleta"
      />
    </div>

    {#if items.length === 0}
      <p class="vacio">Nada coincide con «{query}».</p>
    {:else}
      <ul bind:this={list} id="paleta-resultados" role="listbox" aria-label="Resultados">
        {#each items as item, i (item.kind === 'action' ? `a-${item.action.id}` : `p-${item.project.id}`)}
          {#if i === 0 && item.kind === 'action'}
            <li class="grupo" role="presentation">Acciones</li>
          {/if}
          {#if i === firstProject}
            <li class="grupo" role="presentation">{query.trim() === '' ? 'Abiertos hace poco' : 'Proyectos'}</li>
          {/if}
          <!-- El teclado lo lleva el campo de búsqueda (combobox con
               aria-activedescendant); las opciones solo atienden al ratón. -->
          <li
            id="paleta-{i}"
            data-indice={i}
            role="option"
            aria-selected={i === selected}
            class="item"
            class:activo={i === selected}
            onmousemove={() => (selected = i)}
            onclick={(event) => run(item, event.ctrlKey || event.metaKey)}
            onkeydown={() => {}}
          >
            {#if item.kind === 'action'}
              <ArrowRight size={14} strokeWidth={1.75} class="icono" />
              <span class="texto">{item.action.label}</span>
              {#if item.action.shortcut}<kbd>{item.action.shortcut}</kbd>{/if}
            {:else}
              <FolderGit2 size={14} strokeWidth={1.75} class="icono" />
              <span class="texto">{item.project.name}</span>
              <span class="ruta">{item.project.path}</span>
            {/if}
          </li>
        {/each}
      </ul>
    {/if}

    <p class="pistas">
      <span><kbd>↑</kbd><kbd>↓</kbd> moverse</span>
      <span><kbd>↵</kbd> abrir</span>
      <span><kbd>{modKey}↵</kbd> proyecto en el editor</span>
      <span><kbd>Esc</kbd> cerrar</span>
    </p>
  </div>
</div>

<style>
  .fondo {
    position: fixed;
    inset: 0;
    z-index: 60;
    display: flex;
    justify-content: center;
    align-items: flex-start;
    padding: 12vh var(--space-5) var(--space-5);
    background: color-mix(in oklab, var(--surface-base) 60%, transparent);
  }

  .paleta {
    display: flex;
    flex-direction: column;
    width: 100%;
    max-width: 38rem;
    max-height: 70vh;
    overflow: hidden;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-lg);
    background: var(--surface-overlay);
    box-shadow: var(--shadow-lg);
  }

  .buscar {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-3) var(--space-4);
    border-bottom: 1px solid var(--border-subtle);
    color: var(--text-tertiary);
  }
  .buscar input {
    flex: 1;
    min-width: 0;
    border: 0;
    background: transparent;
    font: inherit;
    font-size: var(--text-card);
    color: var(--text-primary);
    outline: none;
  }
  .buscar input::placeholder {
    color: var(--text-tertiary);
  }

  ul {
    margin: 0;
    padding: var(--space-2);
    overflow-y: auto;
    list-style: none;
  }

  .grupo {
    padding: var(--space-2) var(--space-2) var(--space-1);
    font-size: var(--text-meta);
    line-height: var(--text-meta-lh);
    font-weight: 600;
    color: var(--text-tertiary);
  }

  .item {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: 7px var(--space-2);
    border-radius: var(--radius-sm);
    font-size: var(--text-body);
    line-height: var(--text-body-lh);
    color: var(--text-secondary);
    cursor: pointer;
  }
  .item.activo {
    background: var(--accent-surface);
    color: var(--text-primary);
  }
  .item :global(.icono) {
    flex: none;
    color: var(--text-tertiary);
  }
  .item.activo :global(.icono) {
    color: var(--accent);
  }

  .texto {
    flex: none;
  }
  .ruta {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-family: var(--font-mono);
    font-size: var(--text-code);
    color: var(--text-tertiary);
  }

  kbd {
    margin-left: auto;
    padding: 0 5px;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    font-family: var(--font-mono);
    font-size: var(--text-code);
    color: var(--text-tertiary);
  }

  .vacio {
    margin: 0;
    padding: var(--space-5) var(--space-4);
    font-size: var(--text-body);
    color: var(--text-tertiary);
  }

  .pistas {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-1) var(--space-4);
    margin: 0;
    padding: var(--space-2) var(--space-4);
    border-top: 1px solid var(--border-subtle);
    font-size: var(--text-meta);
    color: var(--text-tertiary);
  }
  .pistas span {
    display: inline-flex;
    align-items: center;
    gap: 3px;
  }
  .pistas kbd {
    margin-left: 0;
  }
</style>
