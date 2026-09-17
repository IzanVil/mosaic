<script lang="ts">
  import { Check, Plus, Tag as TagIcon } from '@lucide/svelte';
  import Fuse from 'fuse.js';

  import {
    assignTagToProject,
    createTag,
    tags,
    unassignTagFromProject,
  } from '../stores/tags';
  import type { Tag, TagWithCount } from '../types';
  import { dismissable } from '../utils/dismissable';
  import { normalizeText } from '../utils/text';
  import { DEFAULT_TAG_COLOR, TAG_COLORS } from '../utils/tagColors';

  interface Props {
    projectId: number;
    assigned: Tag[];
    compact?: boolean;
  }

  let { projectId, assigned, compact = false }: Props = $props();

  let open = $state(false);
  let query = $state('');
  let newColor = $state(DEFAULT_TAG_COLOR);
  let busy = $state(false);
  let trigger = $state<HTMLButtonElement | null>(null);
  let input = $state<HTMLInputElement | null>(null);

  let assignedIds = $derived(new Set(assigned.map((tag) => tag.id)));

  let index = $derived(
    new Fuse(
      $tags.map((tag) => ({ tag, needle: normalizeText(tag.name) })),
      { keys: ['needle'], threshold: 0.3, ignoreLocation: true },
    ),
  );

  let suggestions = $derived.by(() => {
    const needle = normalizeText(query.trim());
    if (needle === '') return $tags;
    return index.search(needle).map((result) => result.item.tag);
  });

  let exactMatch = $derived.by(() => {
    const needle = normalizeText(query.trim());
    if (needle === '') return undefined;
    return $tags.find((tag) => normalizeText(tag.name) === needle);
  });

  let canCreate = $derived(query.trim() !== '' && exactMatch === undefined);

  function openPicker() {
    open = true;
    query = '';
    newColor = DEFAULT_TAG_COLOR;
  }

  function close() {
    open = false;
    trigger?.focus();
  }

  $effect(() => {
    if (open && input !== null) input.focus();
  });

  async function toggle(tag: TagWithCount | Tag) {
    busy = true;
    if (assignedIds.has(tag.id)) {
      await unassignTagFromProject(projectId, tag.id);
    } else {
      await assignTagToProject(projectId, tag.id);
    }
    busy = false;
    query = '';
    input?.focus();
  }

  async function create() {
    const name = query.trim();
    if (name === '') return;

    busy = true;
    const created = await createTag(name, newColor);
    if (created !== null) await assignTagToProject(projectId, created.id);
    busy = false;
    close();
  }

  async function onKeydown(event: KeyboardEvent) {
    if (event.key !== 'Enter') return;
    event.preventDefault();
    if (exactMatch !== undefined) {
      await toggle(exactMatch);
    } else if (canCreate) {
      await create();
    }
  }
</script>

<div class="relative">
  <button
    bind:this={trigger}
    type="button"
    onclick={() => (open ? close() : openPicker())}
    aria-expanded={open}
    aria-haspopup="dialog"
    title="Asignar etiquetas"
    class="inline-flex items-center gap-1 rounded-full border border-dashed border-surface-border
           px-2 py-0.5 text-xs text-content-muted transition hover:border-content-muted
           hover:text-content focus-visible:outline-2 focus-visible:outline-offset-1
           focus-visible:outline-accent"
  >
    <Plus size={11} />
    {#if !compact}Etiqueta{/if}
  </button>

  {#if open}
    <div
      use:dismissable={{ onDismiss: close }}
      role="dialog"
      aria-label="Asignar etiquetas"
      class="absolute left-0 top-full z-30 mt-1 w-60 rounded-lg border border-surface-border
             bg-surface-1 p-2 shadow-lg"
    >
      <input
        bind:this={input}
        bind:value={query}
        onkeydown={onKeydown}
        type="text"
        placeholder="Buscar o crear…"
        maxlength="32"
        class="w-full rounded-md border border-surface-border bg-surface-0 px-2 py-1 text-sm
               text-content placeholder:text-content-muted focus-visible:outline-2
               focus-visible:outline-offset-1 focus-visible:outline-accent"
      />

      <ul class="mt-2 max-h-48 space-y-0.5 overflow-y-auto">
        {#each suggestions as tag (tag.id)}
          <li>
            <button
              type="button"
              onclick={() => toggle(tag)}
              disabled={busy}
              class="flex w-full items-center gap-2 rounded-md px-2 py-1 text-left text-sm
                     text-content transition hover:bg-surface-2 disabled:opacity-60"
            >
              <span
                class="size-2.5 shrink-0 rounded-full"
                style="background-color: {tag.color};"
              ></span>
              <span class="min-w-0 flex-1 truncate">{tag.name}</span>
              {#if assignedIds.has(tag.id)}
                <Check size={13} class="shrink-0 text-accent" />
              {/if}
            </button>
          </li>
        {/each}

        {#if suggestions.length === 0 && !canCreate}
          <li class="px-2 py-1 text-xs text-content-muted">
            Todavía no hay etiquetas. Escribe un nombre para crear la primera.
          </li>
        {/if}
      </ul>

      {#if canCreate}
        <div class="mt-2 border-t border-surface-border pt-2">
          <button
            type="button"
            onclick={create}
            disabled={busy}
            class="flex w-full items-center gap-2 rounded-md px-2 py-1 text-left text-sm
                   text-content transition hover:bg-surface-2 disabled:opacity-60"
          >
            <TagIcon size={13} style="color: {newColor};" />
            <span class="min-w-0 flex-1 truncate">Crear «{query.trim()}»</span>
          </button>

          <div class="mt-1 flex flex-wrap gap-1 px-2 pb-1">
            {#each TAG_COLORS as color (color.hex)}
              <button
                type="button"
                onclick={() => (newColor = color.hex)}
                title={color.name}
                aria-label="Color {color.name}"
                aria-pressed={newColor === color.hex}
                class="size-4 rounded-full border transition focus-visible:outline-2
                       focus-visible:outline-offset-1 focus-visible:outline-accent"
                style="background-color: {color.hex}; border-color: {newColor === color.hex
                  ? 'currentColor'
                  : 'transparent'};"
              ></button>
            {/each}
          </div>
        </div>
      {/if}
    </div>
  {/if}
</div>
