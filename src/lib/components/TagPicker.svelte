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
  import { t } from '../i18n';
  import { dismissable } from '../utils/dismissable';
  import { normalizeText } from '../utils/text';
  import { opensUpward } from '../utils/placement';
  import { DEFAULT_TAG_COLOR, TAG_COLORS } from '../utils/tagColors';

  interface Props {
    /** El proyecto al que se asignan. */
    projectId: number;
    /** Las etiquetas que ya tiene. */
    assigned: Tag[];
    /** Solo el «+», sin el texto. */
    compact?: boolean;
    /** Disparador sin borde, como texto terciario: el de la tarjeta sin etiquetas. */
    plain?: boolean;
  }

  let { projectId, assigned, compact = false, plain = false }: Props = $props();

  let open = $state(false);
  let query = $state('');
  let newColor = $state(DEFAULT_TAG_COLOR);
  let busy = $state(false);
  let trigger = $state<HTMLButtonElement | null>(null);
  let input = $state<HTMLInputElement | null>(null);
  let panel = $state<HTMLDivElement | null>(null);
  let upward = $state(false);

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
    upward = false;
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

  $effect(() => {
    if (panel === null || trigger === null) return;
    upward = opensUpward(
      trigger.getBoundingClientRect(),
      panel.offsetHeight,
      window.innerHeight,
    );
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

<div class="selector">
  <button
    bind:this={trigger}
    type="button"
    class="disparador"
    class:texto={plain}
    onclick={() => (open ? close() : openPicker())}
    aria-expanded={open}
    aria-haspopup="dialog"
    title={$t('selector.asignar')}
  >
    <Plus size={11} strokeWidth={1.75} />
    {#if !compact}{$t('selector.etiqueta')}{/if}
  </button>

  {#if open}
    <div
      use:dismissable={{ onDismiss: close }}
      bind:this={panel}
      role="dialog"
      aria-label={$t('selector.asignar')}
      class="panel"
      class:arriba={upward}
    >
      <input
        bind:this={input}
        bind:value={query}
        onkeydown={onKeydown}
        type="text"
        placeholder={$t('selector.buscar')}
        maxlength="32"
      />

      <ul>
        {#each suggestions as tag (tag.id)}
          <li>
            <button type="button" class="opcion" onclick={() => toggle(tag)} disabled={busy}>
              <span class="color" style="background-color: {tag.color};"></span>
              <span class="nombre">{tag.name}</span>
              {#if assignedIds.has(tag.id)}
                <Check size={14} strokeWidth={1.75} class="marca" />
              {/if}
            </button>
          </li>
        {/each}

        {#if suggestions.length === 0 && !canCreate}
          <li class="vacio">{$t('selector.vacio')}</li>
        {/if}
      </ul>

      {#if canCreate}
        <div class="crear">
          <button type="button" class="opcion" onclick={create} disabled={busy}>
            <TagIcon size={14} strokeWidth={1.75} style="color: {newColor};" />
            <span class="nombre">{$t('selector.crear', { nombre: query.trim() })}</span>
          </button>

          <div class="paleta">
            {#each TAG_COLORS as color (color.hex)}
              <button
                type="button"
                class="muestra"
                class:elegida={newColor === color.hex}
                onclick={() => (newColor = color.hex)}
                title={$t(`color.${color.id}`)}
                aria-label={$t('selector.color', { nombre: $t(`color.${color.id}`) })}
                aria-pressed={newColor === color.hex}
                style="background-color: {color.hex};"
              ></button>
            {/each}
          </div>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .selector {
    position: relative;
  }

  .disparador {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    padding: 2px var(--space-2);
    border: 1px dashed var(--border-default);
    border-radius: var(--radius-pill);
    background: transparent;
    font: inherit;
    font-size: var(--text-meta);
    line-height: 16px;
    color: var(--text-tertiary);
    cursor: pointer;
    transition:
      border-color var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out);
  }
  .disparador:hover {
    border-color: var(--border-strong);
    color: var(--text-secondary);
  }
  .disparador.texto {
    border: 0;
    padding-inline: 0;
  }

  .panel {
    position: absolute;
    top: 100%;
    left: 0;
    z-index: 30;
    width: 15rem;
    margin-top: var(--space-1);
    padding: var(--space-2);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    background: var(--surface-overlay);
    box-shadow: var(--shadow-lg);
  }
  .panel.arriba {
    top: auto;
    bottom: 100%;
    margin-top: 0;
    margin-bottom: var(--space-1);
  }

  input {
    width: 100%;
    box-sizing: border-box;
    padding: 5px var(--space-2);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    background: var(--surface-base);
    font: inherit;
    font-size: var(--text-body);
    line-height: var(--text-body-lh);
    color: var(--text-primary);
  }
  input::placeholder {
    color: var(--text-tertiary);
  }
  input:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }

  ul {
    display: flex;
    flex-direction: column;
    gap: 2px;
    max-height: 12rem;
    margin: var(--space-2) 0 0;
    padding: 0;
    overflow-y: auto;
    list-style: none;
  }

  .opcion {
    display: flex;
    width: 100%;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-1) var(--space-2);
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    font: inherit;
    font-size: var(--text-body);
    line-height: var(--text-body-lh);
    text-align: left;
    color: var(--text-secondary);
    cursor: pointer;
    transition: background var(--duration-fast) var(--ease-out);
  }
  .opcion:hover:not(:disabled) {
    background: var(--surface-sunken);
    color: var(--text-primary);
  }
  .opcion:disabled {
    opacity: 0.6;
    cursor: progress;
  }
  .opcion :global(.marca) {
    flex: none;
    color: var(--accent);
  }

  .color {
    flex: none;
    width: 10px;
    height: 10px;
    border-radius: var(--radius-pill);
  }

  .nombre {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .vacio {
    padding: var(--space-1) var(--space-2);
    font-size: var(--text-meta);
    line-height: var(--text-meta-lh);
    color: var(--text-tertiary);
  }

  .crear {
    margin-top: var(--space-2);
    padding-top: var(--space-2);
    border-top: 1px solid var(--border-subtle);
  }

  .paleta {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-1);
    padding: var(--space-1) var(--space-2);
  }

  .muestra {
    width: 16px;
    height: 16px;
    padding: 0;
    border: 2px solid transparent;
    border-radius: var(--radius-pill);
    cursor: pointer;
  }
  .muestra.elegida {
    border-color: var(--text-primary);
  }

  button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
</style>
