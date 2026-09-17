import { writable } from 'svelte/store';

import * as api from '../api/git';
import { mergeGitStatus, mergeOneGitStatus } from './projects';

export const refreshingGit = writable(false);
export const gitError = writable<string | null>(null);

export async function loadGitStatus(): Promise<void> {
  gitError.set(null);
  try {
    mergeGitStatus(await api.listGitStatus());
  } catch (error) {
    gitError.set(String(error));
  }
}

export async function refreshAllGitStatus(): Promise<void> {
  refreshingGit.set(true);
  gitError.set(null);
  try {
    await api.refreshAllGitStatus();
    mergeGitStatus(await api.listGitStatus());
  } catch (error) {
    gitError.set(String(error));
  } finally {
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
