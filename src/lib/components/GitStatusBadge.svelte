<script lang="ts">
  import { ArrowDown, ArrowUp, GitBranch } from '@lucide/svelte';

  import type { GitStatusEntry } from '../types';
  import { formatRelativeTime } from '../utils/format';

  interface Props {
    /** `undefined` mientras la caché no tenga entrada para este proyecto. */
    status: GitStatusEntry | undefined;
  }

  let { status }: Props = $props();

  /** Rama, o los siete primeros caracteres del sha si el HEAD está separado. */
  let label = $derived(
    status?.branch ?? (status?.last_commit_sha ? status.last_commit_sha.slice(0, 7) : '—'),
  );

  let detached = $derived(status !== undefined && status.branch === null);

  let tooltip = $derived.by(() => {
    if (!status) return 'Sin estado Git todavía';

    const lines = [
      detached ? 'HEAD separado' : `Rama ${status.branch}`,
      status.last_commit_msg ?? 'Sin commits todavía',
    ];
    if (status.last_commit_at !== null) {
      lines.push(`Último commit ${formatRelativeTime(status.last_commit_at)}`);
    }
    if (status.is_dirty) lines.push('Hay cambios sin commitear');
    lines.push(`Leído ${formatRelativeTime(status.refreshed_at)}`);
    return lines.join('\n');
  });
</script>

<span class="insignia" title={tooltip}>
  <GitBranch size={12} strokeWidth={1.75} />
  <span class="rama" class:separado={detached}>{label}</span>

  {#if status?.is_dirty}
    <span class="sucio" aria-label="Cambios sin commitear"></span>
  {/if}

  {#if status?.ahead}
    <span class="delta" aria-label="{status.ahead} commits por delante">
      <ArrowUp size={11} strokeWidth={1.75} />{status.ahead}
    </span>
  {/if}

  {#if status?.behind}
    <span class="delta" aria-label="{status.behind} commits por detrás">
      <ArrowDown size={11} strokeWidth={1.75} />{status.behind}
    </span>
  {/if}
</span>

<style>
  .insignia {
    display: inline-flex;
    flex: none;
    align-items: center;
    gap: 6px;
    font-size: var(--text-meta);
    line-height: var(--text-meta-lh);
    color: var(--text-tertiary);
  }

  .rama {
    font-family: var(--font-mono);
    font-size: var(--text-code);
    color: var(--text-secondary);
  }
  .rama.separado {
    font-style: italic;
  }

  .sucio {
    width: 6px;
    height: 6px;
    border-radius: var(--radius-pill);
    background: var(--warning);
  }

  .delta {
    display: inline-flex;
    align-items: center;
    font-family: var(--font-mono);
    font-size: var(--text-code);
  }
</style>
