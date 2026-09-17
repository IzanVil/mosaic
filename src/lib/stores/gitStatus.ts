import { writable } from 'svelte/store';

import * as api from '../api/git';
import { now_ts } from '../utils/format';
import { mergeGitStatus, mergeOneGitStatus } from './projects';

export interface GitRefreshOutcome {
  at: number;
  read: number;
  failed: number;
  changed: number;
  manual: boolean;
}

const MIN_SPINNER_MS = 450;

export const refreshingGit = writable(false);
export const gitError = writable<string | null>(null);
export const lastGitRefresh = writable<GitRefreshOutcome | null>(null);

function delay(milliseconds: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}

export async function loadGitStatus(manual = false): Promise<void> {
  gitError.set(null);
  try {
    const entries = await api.listGitStatus();
    const changed = mergeGitStatus(entries);
    lastGitRefresh.set({
      at: now_ts(),
      read: entries.length,
      failed: 0,
      changed,
      manual,
    });
  } catch (error) {
    gitError.set(String(error));
  }
}

export async function refreshAllGitStatus(): Promise<void> {
  refreshingGit.set(true);
  gitError.set(null);
  const started = Date.now();
  try {
    const summary = await api.refreshAllGitStatus();
    const changed = mergeGitStatus(await api.listGitStatus());
    lastGitRefresh.set({
      at: now_ts(),
      read: summary.refreshed,
      failed: summary.failed,
      changed,
      manual: true,
    });
  } catch (error) {
    gitError.set(String(error));
  } finally {
    const elapsed = Date.now() - started;
    if (elapsed < MIN_SPINNER_MS) await delay(MIN_SPINNER_MS - elapsed);
    refreshingGit.set(false);
  }
}

export async function refreshOneGitStatus(projectId: number): Promise<void> {
  gitError.set(null);
  try {
    mergeOneGitStatus(projectId, await api.refreshGitStatus(projectId));
  } catch (error) {
    gitError.set(String(error));
  }
}
