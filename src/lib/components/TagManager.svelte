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

<div
  class="fixed inset-0 z-40 flex items-start justify-center bg-black/40 p-6 pt-20"
  role="presentation"
>
  <div
    use:dismissable={{ onDismiss: onClose, enabled: canDismiss }}
    role="dialog"
    aria-modal="true"
    aria-label="Gestionar etiquetas"
    class="flex max-h-[70vh] w-full max-w-xl flex-col rounded-lg border border-surface-border
           bg-surface-1 shadow-xl"
  >
    <header class="flex items-center justify-between gap-4 border-b border-surface-border px-5 py-3">
      <h2 class="text-sm font-semibold text-content-strong">Etiquetas</h2>
      <button
        type="button"
        onclick={onClose}
        aria-label="Cerrar"
        class="rounded p-1 text-content-muted transition hover:bg-surface-2 hover:text-content
               focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
      >
        <X size={16} />
      </button>
    </header>

    <div class="flex items-center gap-2 border-b border-surface-border px-5 py-3">
      <div class="relative">
        <button
          type="button"
          onclick={() => (colorEditing = colorEditing === 0 ? null : 0)}
          aria-label="Elegir el color de la etiqueta nueva"
          title="Color"
          class="flex size-8 items-center justify-center rounded-md border border-surface-border
                 transition hover:bg-surface-2 focus-visible:outline-2
                 focus-visible:outline-offset-2 focus-visible:outline-accent"
        >
          <span class="size-4 rounded-full" style="background-color: {newColor};"></span>
        </button>

        {#if colorEditing === 0}
          <div
            use:dismissable={{ onDismiss: () => (colorEditing = null) }}
            class="absolute left-0 top-full z-50 mt-1 w-48 rounded-lg border border-surface-border
                   bg-surface-1 p-2 shadow-lg"
          >
            <div class="flex flex-wrap gap-1">
              {#each TAG_COLORS as color (color.hex)}
                <button
                  type="button"
                  onclick={() => {
                    newColor = color.hex;
                    colorEditing = null;
                  }}
                  title={color.name}
                  aria-label="Color {color.name}"
                  class="size-5 rounded-full transition hover:scale-110 focus-visible:outline-2
                         focus-visible:outline-offset-1 focus-visible:outline-accent"
                  style="background-color: {color.hex};"
                ></button>
              {/each}
            </div>
            <label
              class="mt-2 flex items-center gap-2 border-t border-surface-border pt-2 text-xs
                     text-content-muted"
            >
              <Palette size={13} />
              Personalizado
              <input
                type="color"
                value={colorInputValue(newColor)}
                oninput={(event) => (newColor = event.currentTarget.value.toUpperCase())}
                class="ml-auto h-6 w-10 cursor-pointer rounded border border-surface-border
                       bg-transparent"
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
        maxlength="32"
        placeholder="Nueva etiqueta…"
        class="min-w-0 flex-1 rounded-md border border-surface-border bg-surface-0 px-2 py-1.5
               text-sm text-content placeholder:text-content-muted focus-visible:outline-2
               focus-visible:outline-offset-1 focus-visible:outline-accent"
      />

      <button
        type="button"
        onclick={create}
        disabled={creating || newName.trim() === ''}
        class="inline-flex shrink-0 items-center gap-1 rounded-md bg-accent px-3 py-1.5 text-sm
               font-medium text-surface-0 transition hover:bg-accent-strong disabled:opacity-50
               focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
      >
        <Plus size={14} />
        Crear
      </button>
    </div>

    {#if $tagsError}
      <p class="border-b border-surface-border px-5 py-2 text-xs text-content" role="alert">
        {$tagsError}
      </p>
    {/if}

    <div class="min-h-0 flex-1 overflow-y-auto">
      {#if $tags.length === 0}
        <p class="px-5 py-10 text-center text-sm text-content-muted">
          Todavía no hay etiquetas. Crea la primera arriba.
        </p>
      {:else}
        <ul>
          {#each $tags as tag (tag.id)}
            <li class="border-b border-surface-border/60 px-5 py-2 last:border-b-0">
              <div class="flex items-center gap-2">
                <div class="relative shrink-0">
                  <button
                    type="button"
                    onclick={() => (colorEditing = colorEditing === tag.id ? null : tag.id)}
                    aria-label="Cambiar el color de {tag.name}"
                    title="Cambiar el color"
                    class="flex size-7 items-center justify-center rounded-md border
                           border-surface-border transition hover:bg-surface-2
                           focus-visible:outline-2 focus-visible:outline-offset-2
                           focus-visible:outline-accent"
                  >
                    <span class="size-3.5 rounded-full" style="background-color: {tag.color};"></span>
                  </button>

                  {#if colorEditing === tag.id}
                    <div
                      use:dismissable={{ onDismiss: () => (colorEditing = null) }}
                      class="absolute left-0 top-full z-50 mt-1 w-48 rounded-lg border
                             border-surface-border bg-surface-1 p-2 shadow-lg"
                    >
                      <div class="flex flex-wrap gap-1">
                        {#each TAG_COLORS as color (color.hex)}
                          <button
                            type="button"
                            onclick={() => recolor(tag.id, tag.name, color.hex)}
                            title={color.name}
                            aria-label="Color {color.name}"
                            class="size-5 rounded-full transition hover:scale-110
                                   focus-visible:outline-2 focus-visible:outline-offset-1
                                   focus-visible:outline-accent"
                            style="background-color: {color.hex};"
                          ></button>
                        {/each}
                      </div>
                      <label
                        class="mt-2 flex items-center gap-2 border-t border-surface-border pt-2
                               text-xs text-content-muted"
                      >
                        <Palette size={13} />
                        Personalizado
                        <input
                          type="color"
                          value={colorInputValue(tag.color)}
                          onchange={(event) =>
                            recolor(tag.id, tag.name, event.currentTarget.value.toUpperCase())}
                          class="ml-auto h-6 w-10 cursor-pointer rounded border
                                 border-surface-border bg-transparent"
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
                  maxlength="32"
                  aria-label="Nombre de la etiqueta {tag.name}"
                  class="min-w-0 flex-1 rounded-md border border-transparent bg-transparent px-2
                         py-1 text-sm text-content transition hover:border-surface-border
                         focus-visible:border-surface-border focus-visible:bg-surface-0
                         focus-visible:outline-2 focus-visible:outline-offset-1
                         focus-visible:outline-accent"
                />

                <span class="shrink-0 text-xs text-content-muted">
                  {tag.project_count}
                  {tag.project_count === 1 ? 'proyecto' : 'proyectos'}
                </span>

                <button
                  type="button"
                  onclick={() => (confirmingId = tag.id)}
                  aria-label="Borrar la etiqueta {tag.name}"
                  title="Borrar la etiqueta"
                  class="shrink-0 rounded-md p-1.5 text-content-muted transition
                         hover:bg-surface-2 hover:text-content focus-visible:outline-2
                         focus-visible:outline-offset-2 focus-visible:outline-accent"
                >
                  <Trash2 size={14} />
                </button>
              </div>

              {#if confirmingId === tag.id}
                <div class="mt-2 rounded-md border border-surface-border bg-surface-2 px-3 py-2">
                  <p class="text-xs text-content">
                    {tag.project_count === 0
                      ? 'Esta etiqueta no está asignada a ningún proyecto.'
                      : `Esto quitará la etiqueta de ${tag.project_count} ${
                          tag.project_count === 1 ? 'proyecto' : 'proyectos'
                        }. No se borran los proyectos.`}
                  </p>
                  <div class="mt-2 flex justify-end gap-2">
                    <button
                      type="button"
                      onclick={() => (confirmingId = null)}
                      class="rounded-md border border-surface-border px-2.5 py-1 text-xs
                             text-content transition hover:bg-surface-1 focus-visible:outline-2
                             focus-visible:outline-offset-2 focus-visible:outline-accent"
                    >
                      Cancelar
                    </button>
                    <button
                      type="button"
                      onclick={() => confirmDelete(tag.id)}
                      class="rounded-md bg-accent px-2.5 py-1 text-xs font-medium text-surface-0
                             transition hover:bg-accent-strong focus-visible:outline-2
                             focus-visible:outline-offset-2 focus-visible:outline-accent"
                    >
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
