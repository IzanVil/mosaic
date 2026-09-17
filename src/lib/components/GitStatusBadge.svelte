<script lang="ts">
  import { ArrowDown, ArrowUp, GitBranch } from '@lucide/svelte';

  import type { GitStatusEntry } from '../types';
  import { formatRelativeTime } from '../utils/format';

  interface Props {
    status: GitStatusEntry | undefined;
  }

  let { status }: Props = $props();

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

<span
  class="inline-flex shrink-0 items-center gap-1.5 text-xs text-content-muted"
  title={tooltip}
>
  <GitBranch size={12} />
  <span class="font-mono" class:italic={detached}>{label}</span>

  {#if status?.is_dirty}
    <span
      class="size-1.5 rounded-full bg-amber-400"
      aria-label="Cambios sin commitear"
    ></span>
  {/if}

  {#if status?.ahead}
    <span class="inline-flex items-center" aria-label="{status.ahead} commits por delante">
      <ArrowUp size={11} />{status.ahead}
    </span>
  {/if}

  {#if status?.behind}
    <span class="inline-flex items-center" aria-label="{status.behind} commits por detrás">
      <ArrowDown size={11} />{status.behind}
    </span>
  {/if}
</span>
