<script lang="ts">
  import { Code, FolderOpen, Pin, PinOff, SquareTerminal, TriangleAlert } from '@lucide/svelte';

  import GitStatusBadge from './GitStatusBadge.svelte';
  import TagChip from './TagChip.svelte';
  import TagPicker from './TagPicker.svelte';
  import { openProject } from '../stores/apps';
  import { togglePinned } from '../stores/projects';
  import type { ProjectWithTags } from '../types';
  import { formatRelativeTime, shortenPath } from '../utils/format';

  interface Props {
    project: ProjectWithTags;
  }

  let { project }: Props = $props();

  const VISIBLE_TAGS = 3;

  let status = $derived(project.git_status ?? undefined);
  let shownTags = $derived(project.tags.slice(0, VISIBLE_TAGS));
  let hiddenTags = $derived(project.tags.slice(VISIBLE_TAGS));

  const ACTIONS = [
    { kind: 'ide', icon: Code, label: 'Abrir en el editor' },
    { kind: 'terminal', icon: SquareTerminal, label: 'Abrir en la terminal' },
    { kind: 'file_manager', icon: FolderOpen, label: 'Abrir la carpeta' },
  ] as const;
</script>

<article
  class="group relative flex flex-col gap-2 rounded-lg border bg-surface-1 p-4 transition
         hover:border-content-muted"
  class:border-surface-border={!project.pinned}
  class:border-accent={project.pinned}
  class:opacity-60={project.missing}
>
  <header class="flex items-start justify-between gap-2">
    <h3 class="min-w-0 truncate font-medium text-content-strong" title={project.name}>
      {project.name}
    </h3>

    <button
      type="button"
      onclick={() => togglePinned(project.id, !project.pinned)}
      title={project.pinned ? 'Dejar de destacar' : 'Destacar el proyecto'}
      aria-label={project.pinned ? 'Dejar de destacar {project.name}' : 'Destacar {project.name}'}
      aria-pressed={project.pinned}
      class="shrink-0 rounded p-1 transition hover:bg-surface-2 focus-visible:outline-2
             focus-visible:outline-offset-2 focus-visible:outline-accent"
      class:text-accent={project.pinned}
      class:text-content-muted={!project.pinned}
      class:opacity-0={!project.pinned}
      class:group-hover:opacity-100={!project.pinned}
      class:group-focus-within:opacity-100={!project.pinned}
    >
      {#if project.pinned}
        <PinOff size={14} />
      {:else}
        <Pin size={14} />
      {/if}
    </button>
  </header>

  <p class="truncate font-mono text-xs text-content-muted" title={project.path}>
    {shortenPath(project.path, 40)}
  </p>

  <div class="flex flex-wrap items-center gap-1">
    {#each shownTags as tag (tag.id)}
      <TagChip {tag} />
    {/each}

    {#if hiddenTags.length > 0}
      <span
        class="rounded-full border border-surface-border px-1.5 py-0.5 text-xs text-content-muted"
        title={hiddenTags.map((tag) => tag.name).join(', ')}
      >
        +{hiddenTags.length}
      </span>
    {/if}

    <span
      class="transition group-hover:opacity-100 group-focus-within:opacity-100"
      class:opacity-0={project.tags.length > 0}
    >
      <TagPicker projectId={project.id} assigned={project.tags} compact={project.tags.length > 0} />
    </span>
  </div>

  <div class="flex flex-wrap items-center gap-x-3 gap-y-1">
    {#if project.primary_language}
      <span class="rounded-full border border-surface-border px-2 py-0.5 text-xs text-content-muted">
        {project.primary_language}
      </span>
    {/if}

    {#if project.is_git_repo}
      <GitStatusBadge {status} />
    {/if}

    {#if project.missing}
      <span
        class="inline-flex items-center gap-1 text-xs text-content-muted"
        title="El último escaneo no encontró esta carpeta en disco"
      >
        <TriangleAlert size={12} />
        No encontrado
      </span>
    {/if}
  </div>

  <footer class="mt-auto flex items-end justify-between gap-2 pt-2">
    <span class="text-xs text-content-muted">
      {project.last_opened_at === null
        ? 'Sin abrir desde Mosaic'
        : `Abierto ${formatRelativeTime(project.last_opened_at)}`}
    </span>

    <div
      class="flex shrink-0 gap-1 opacity-0 transition group-hover:opacity-100
             group-focus-within:opacity-100"
    >
      {#each ACTIONS as action (action.kind)}
        <button
          type="button"
          onclick={() => openProject(action.kind, project.id)}
          disabled={project.missing}
          title={action.label}
          aria-label="{action.label}: {project.name}"
          class="rounded-md border border-surface-border p-1.5 text-content-muted transition
                 hover:bg-surface-2 hover:text-content-strong disabled:cursor-not-allowed
                 disabled:opacity-40 focus-visible:outline-2 focus-visible:outline-offset-2
                 focus-visible:outline-accent"
        >
          <action.icon size={14} />
        </button>
      {/each}
    </div>
  </footer>
</article>
