<script lang="ts">
  import type { Section } from '../stores/projectDetail';
  import type { CommitInfo } from '../types';
  import { formatRelativeTime } from '../utils/format';

  interface Props {
    /** Historial tal y como lo carga `stores/projectDetail.ts`. */
    section: Section<CommitInfo[]>;
    /** Si es `false`, la lista vacía se explica como «no es un repositorio». */
    isRepo: boolean;
  }

  let { section, isRepo }: Props = $props();
</script>

{#if section.status === 'loading'}
  <p class="nota">Leyendo el historial…</p>
{:else if section.status === 'error'}
  <p class="nota error">No se pudo leer el historial: {section.message}</p>
{:else if section.data.length === 0}
  <p class="nota">{isRepo ? 'Esta rama todavía no tiene commits.' : 'No es un repositorio Git.'}</p>
{:else}
  <ol class="commits">
    {#each section.data as commit (commit.sha)}
      <li>
        <span class="resumen" title={commit.summary}>{commit.summary}</span>
        <span class="meta">
          <code title={commit.sha}>{commit.short_sha}</code>
          <span class="autor">{commit.author_name}</span>
          <span class="cuando">{formatRelativeTime(commit.committed_at)}</span>
        </span>
      </li>
    {/each}
  </ol>
{/if}

<style>
  .commits {
    display: flex;
    flex-direction: column;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  li {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: var(--space-2) 0;
    border-top: 1px solid var(--border-subtle);
  }
  li:first-child {
    border-top: 0;
    padding-top: 0;
  }

  .resumen {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: var(--text-body);
    line-height: var(--text-body-lh);
    color: var(--text-primary);
  }

  .meta {
    display: flex;
    align-items: baseline;
    gap: var(--space-2);
    min-width: 0;
    font-size: var(--text-meta);
    line-height: var(--text-meta-lh);
    color: var(--text-tertiary);
  }

  code {
    flex: none;
    font-family: var(--font-mono);
    font-size: var(--text-code);
    color: var(--text-secondary);
  }

  .autor {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .cuando {
    flex: none;
    margin-left: auto;
  }

  .nota {
    margin: 0;
    font-size: var(--text-meta);
    line-height: var(--text-meta-lh);
    color: var(--text-tertiary);
  }
  .nota.error {
    color: var(--danger);
  }
</style>
