<script lang="ts">
  import { Palette, Plus, Trash2, X } from '@lucide/svelte';

  import { createTag, deleteTag, tags, tagsError, updateTag } from '../stores/tags';
  import { dismissable } from '../utils/dismissable';
  import { colorInputValue, DEFAULT_TAG_COLOR, TAG_COLORS } from '../utils/tagColors';

  interface Props {
    onClose: () => void;
  }

  let { onClose }: Props = $props();

  let newName = $state('');
  let newColor = $state(DEFAULT_TAG_COLOR);
  let creating = $state(false);
  let colorEditing = $state<number | null>(null);
  let confirmingId = $state<number | null>(null);
  let newInput = $state<HTMLInputElement | null>(null);

  let canDismiss = $derived(colorEditing === null && confirmingId === null);

  $effect(() => {
    newInput?.focus();
  });

  async function create() {
    if (newName.trim() === '') return;
    creating = true;
    const created = await createTag(newName, newColor);
    creating = false;
    if (created !== null) {
      newName = '';
      newColor = DEFAULT_TAG_COLOR;
      newInput?.focus();
    }
  }

  async function rename(id: number, name: string, color: string, previous: string) {
    if (name.trim() === previous) return;
    if (name.trim() === '') {
      tagsError.set('El nombre de la etiqueta no puede estar vacío');
      return;
    }
    await updateTag(id, name, color);
  }

  async function recolor(id: number, name: string, color: string) {
    colorEditing = null;
    await updateTag(id, name, color);
  }

  async function confirmDelete(id: number) {
    const done = await deleteTag(id);
    if (done) confirmingId = null;
  }
</script>

<div class="fondo" role="presentation">
  <div
    use:dismissable={{ onDismiss: onClose, enabled: canDismiss }}
    role="dialog"
    aria-modal="true"
    aria-label="Gestionar etiquetas"
    class="modal"
  >
    <header>
      <h2>Etiquetas</h2>
      <button type="button" class="icono" onclick={onClose} aria-label="Cerrar">
        <X size={16} strokeWidth={1.75} />
      </button>
    </header>

    <div class="nueva">
      <div class="ancla">
        <button
          type="button"
          class="caja-color"
          onclick={() => (colorEditing = colorEditing === 0 ? null : 0)}
          aria-label="Elegir el color de la etiqueta nueva"
          title="Color"
        >
          <span class="punto" style="background-color: {newColor};"></span>
        </button>

        {#if colorEditing === 0}
          <div use:dismissable={{ onDismiss: () => (colorEditing = null) }} class="paleta">
            <div class="muestras">
              {#each TAG_COLORS as color (color.hex)}
                <button
                  type="button"
                  class="muestra"
                  onclick={() => {
                    newColor = color.hex;
                    colorEditing = null;
                  }}
                  title={color.name}
                  aria-label="Color {color.name}"
                  style="background-color: {color.hex};"
                ></button>
              {/each}
            </div>
            <label class="personalizado">
              <Palette size={14} strokeWidth={1.75} />
              Personalizado
              <input
                type="color"
                value={colorInputValue(newColor)}
                oninput={(event) => (newColor = event.currentTarget.value.toUpperCase())}
              />
            </label>
          </div>
        {/if}
      </div>

      <input
        bind:this={newInput}
        bind:value={newName}
        onkeydown={(event) => {
          if (event.key === 'Enter') {
            event.preventDefault();
            void create();
          }
        }}
        type="text"
        class="campo"
        maxlength="32"
        placeholder="Nueva etiqueta…"
      />

      <button
        type="button"
        class="primario"
        onclick={create}
        disabled={creating || newName.trim() === ''}
      >
        <Plus size={14} strokeWidth={1.75} />
        Crear
      </button>
    </div>

    {#if $tagsError}
      <p class="error" role="alert">{$tagsError}</p>
    {/if}

    <div class="lista">
      {#if $tags.length === 0}
        <p class="vacio">Todavía no hay etiquetas. Crea la primera arriba.</p>
      {:else}
        <ul>
          {#each $tags as tag (tag.id)}
            <li>
              <div class="fila">
                <div class="ancla">
                  <button
                    type="button"
                    class="caja-color pequena"
                    onclick={() => (colorEditing = colorEditing === tag.id ? null : tag.id)}
                    aria-label="Cambiar el color de {tag.name}"
                    title="Cambiar el color"
                  >
                    <span class="punto" style="background-color: {tag.color};"></span>
                  </button>

                  {#if colorEditing === tag.id}
                    <div use:dismissable={{ onDismiss: () => (colorEditing = null) }} class="paleta">
                      <div class="muestras">
                        {#each TAG_COLORS as color (color.hex)}
                          <button
                            type="button"
                            class="muestra"
                            onclick={() => recolor(tag.id, tag.name, color.hex)}
                            title={color.name}
                            aria-label="Color {color.name}"
                            style="background-color: {color.hex};"
                          ></button>
                        {/each}
                      </div>
                      <label class="personalizado">
                        <Palette size={14} strokeWidth={1.75} />
                        Personalizado
                        <input
                          type="color"
                          value={colorInputValue(tag.color)}
                          onchange={(event) =>
                            recolor(tag.id, tag.name, event.currentTarget.value.toUpperCase())}
                        />
                      </label>
                    </div>
                  {/if}
                </div>

                <input
                  value={tag.name}
                  onblur={(event) => rename(tag.id, event.currentTarget.value, tag.color, tag.name)}
                  onkeydown={(event) => {
                    if (event.key === 'Enter') event.currentTarget.blur();
                    if (event.key === 'Escape') event.currentTarget.value = tag.name;
                  }}
                  type="text"
                  class="nombre"
                  maxlength="32"
                  aria-label="Nombre de la etiqueta {tag.name}"
                />

                <span class="cuenta">
                  {tag.project_count}
                  {tag.project_count === 1 ? 'proyecto' : 'proyectos'}
                </span>

                <button
                  type="button"
                  class="icono"
                  onclick={() => (confirmingId = tag.id)}
                  aria-label="Borrar la etiqueta {tag.name}"
                  title="Borrar la etiqueta"
                >
                  <Trash2 size={14} strokeWidth={1.75} />
                </button>
              </div>

              {#if confirmingId === tag.id}
                <div class="confirmar">
                  <p>
                    {tag.project_count === 0
                      ? 'Esta etiqueta no está asignada a ningún proyecto.'
                      : `Esto quitará la etiqueta de ${tag.project_count} ${
                          tag.project_count === 1 ? 'proyecto' : 'proyectos'
                        }. No se borran los proyectos.`}
                  </p>
                  <div class="botones">
                    <button type="button" class="secundario" onclick={() => (confirmingId = null)}>
                      Cancelar
                    </button>
                    <button type="button" class="peligro" onclick={() => confirmDelete(tag.id)}>
                      Borrar
                    </button>
                  </div>
                </div>
              {/if}
            </li>
          {/each}
        </ul>
      {/if}
    </div>
  </div>
</div>

<style>
  /*
   * El velo es el propio fondo de la aplicación medio transparente: oscurece
   * en el tema oscuro y aclara en el claro, sin un negro fijo que en claro
   * ensuciaría.
   */
  .fondo {
    position: fixed;
    inset: 0;
    z-index: 40;
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding: var(--space-8) var(--space-5) var(--space-5);
    background: color-mix(in oklab, var(--surface-base) 72%, transparent);
  }

  .modal {
    display: flex;
    flex-direction: column;
    width: 100%;
    max-width: 36rem;
    max-height: 70vh;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-lg);
    background: var(--surface-overlay);
    box-shadow: var(--shadow-lg);
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
    padding: var(--space-3) var(--space-5);
    border-bottom: 1px solid var(--border-subtle);
  }

  h2 {
    margin: 0;
    font-size: var(--text-section);
    line-height: var(--text-section-lh);
    font-weight: var(--text-section-weight);
    letter-spacing: var(--tracking-tight);
    color: var(--text-primary);
  }

  .nueva {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-5);
    border-bottom: 1px solid var(--border-subtle);
  }

  .ancla {
    position: relative;
    flex: none;
  }

  .caja-color {
    display: flex;
    width: 32px;
    height: 32px;
    align-items: center;
    justify-content: center;
    padding: 0;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    background: transparent;
    cursor: pointer;
    transition: background var(--duration-fast) var(--ease-out);
  }
  .caja-color.pequena {
    width: 28px;
    height: 28px;
  }
  .caja-color:hover {
    background: var(--surface-sunken);
  }

  .punto {
    width: 14px;
    height: 14px;
    border-radius: var(--radius-pill);
  }

  .paleta {
    position: absolute;
    top: 100%;
    left: 0;
    z-index: 50;
    width: 12rem;
    margin-top: var(--space-1);
    padding: var(--space-2);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    background: var(--surface-overlay);
    box-shadow: var(--shadow-lg);
  }

  .muestras {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-1);
  }

  .muestra {
    width: 20px;
    height: 20px;
    padding: 0;
    border: 0;
    border-radius: var(--radius-pill);
    cursor: pointer;
    transition: transform var(--duration-fast) var(--ease-out);
  }
  .muestra:hover {
    transform: scale(1.1);
  }

  .personalizado {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-top: var(--space-2);
    padding-top: var(--space-2);
    border-top: 1px solid var(--border-subtle);
    font-size: var(--text-meta);
    line-height: var(--text-meta-lh);
    color: var(--text-tertiary);
  }
  .personalizado input {
    width: 40px;
    height: 24px;
    margin-left: auto;
    padding: 0;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    background: transparent;
    cursor: pointer;
  }

  .campo {
    flex: 1;
    min-width: 0;
    padding: 6px var(--space-2);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    background: var(--surface-base);
    font: inherit;
    font-size: var(--text-body);
    line-height: var(--text-body-lh);
    color: var(--text-primary);
  }
  .campo::placeholder {
    color: var(--text-tertiary);
  }

  .primario {
    display: inline-flex;
    flex: none;
    align-items: center;
    gap: var(--space-1);
    padding: 6px var(--space-3);
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
  .primario:hover:not(:disabled) {
    background: var(--accent-hover);
  }
  .primario:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .error {
    margin: 0;
    padding: var(--space-2) var(--space-5);
    border-bottom: 1px solid var(--border-subtle);
    background: var(--danger-surface);
    font-size: var(--text-meta);
    line-height: var(--text-meta-lh);
    color: var(--danger);
  }

  .lista {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
  }

  .vacio {
    margin: 0;
    padding: var(--space-6) var(--space-5);
    font-size: var(--text-body);
    line-height: var(--text-body-lh);
    text-align: center;
    color: var(--text-tertiary);
  }

  ul {
    margin: 0;
    padding: 0;
    list-style: none;
  }

  li {
    padding: var(--space-2) var(--space-5);
    border-bottom: 1px solid var(--border-subtle);
  }
  li:last-child {
    border-bottom: 0;
  }

  .fila {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .nombre {
    flex: 1;
    min-width: 0;
    padding: var(--space-1) var(--space-2);
    border: 1px solid transparent;
    border-radius: var(--radius-sm);
    background: transparent;
    font: inherit;
    font-size: var(--text-body);
    line-height: var(--text-body-lh);
    color: var(--text-primary);
    transition: border-color var(--duration-fast) var(--ease-out);
  }
  .nombre:hover {
    border-color: var(--border-default);
  }
  .nombre:focus-visible {
    border-color: var(--border-default);
    background: var(--surface-base);
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }

  .cuenta {
    flex: none;
    font-size: var(--text-meta);
    line-height: var(--text-meta-lh);
    font-variant-numeric: tabular-nums;
    color: var(--text-tertiary);
  }

  .icono {
    display: inline-flex;
    flex: none;
    padding: 6px;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-tertiary);
    cursor: pointer;
    transition:
      background var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out);
  }
  .icono:hover {
    background: var(--surface-sunken);
    color: var(--text-primary);
  }

  .confirmar {
    margin-top: var(--space-2);
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    background: var(--surface-sunken);
  }
  .confirmar p {
    margin: 0;
    font-size: var(--text-meta);
    line-height: var(--text-meta-lh);
    color: var(--text-secondary);
  }

  .botones {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-2);
    margin-top: var(--space-2);
  }

  .secundario,
  .peligro {
    padding: var(--space-1) 10px;
    border-radius: var(--radius-md);
    font: inherit;
    font-size: var(--text-meta);
    line-height: var(--text-meta-lh);
    font-weight: 500;
    cursor: pointer;
    transition: background var(--duration-fast) var(--ease-out);
  }
  .secundario {
    border: 1px solid var(--border-default);
    background: transparent;
    color: var(--text-secondary);
  }
  .secundario:hover {
    background: var(--surface-raised);
    color: var(--text-primary);
  }
  /* Borrar es la única acción irreversible del modal: no lleva el acento. */
  .peligro {
    border: 1px solid color-mix(in oklab, var(--danger) 40%, transparent);
    background: var(--danger-surface);
    color: var(--danger);
  }
  .peligro:hover {
    background: color-mix(in oklab, var(--danger) 24%, transparent);
  }

  button:focus-visible,
  input:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
</style>
