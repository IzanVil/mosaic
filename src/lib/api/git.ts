import { invoke } from '@tauri-apps/api/core';

import type { GitRefreshSummary, GitStatusEntry } from '../types';

export function listGitStatus(): Promise<GitStatusEntry[]> {
  return invoke<GitStatusEntry[]>('list_git_status');
}

export function refreshGitStatus(projectId: number): Promise<GitStatusEntry | null> {
  return invoke<GitStatusEntry | null>('refresh_git_status', { projectId });
}
export function refreshAllGitStatus(): Promise<GitRefreshSummary> {
  return invoke<GitRefreshSummary>('refresh_all_git_status');
}
