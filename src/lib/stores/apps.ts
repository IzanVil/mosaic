import { derived, writable } from 'svelte/store';

import * as api from '../api/system';
import type { AppKind, DetectedApp } from '../types';
import { markProjectOpened } from './projects';

export const detectedApps = writable<DetectedApp[]>([]);
export const preferredIde = writable('');
export const preferredTerminal = writable('');
export const appsError = writable<string | null>(null);

export const ides = derived(detectedApps, (apps) => apps.filter((app) => app.kind === 'ide'));
export const terminals = derived(detectedApps, (apps) =>
  apps.filter((app) => app.kind === 'terminal'),
);

export async function loadApps(): Promise<void> {
  appsError.set(null);
  try {
    detectedApps.set(await api.listDetectedApps());
    const preferred = await api.getPreferredApps();
    preferredIde.set(preferred.ide);
    preferredTerminal.set(preferred.terminal);
  } catch (error) {
    appsError.set(String(error));
  }
}

export async function openProject(kind: AppKind, projectId: number): Promise<boolean> {
  appsError.set(null);
  try {
    await api.openIn(kind, projectId);
    markProjectOpened(projectId, Math.floor(Date.now() / 1000));
    return true;
  } catch (error) {
    appsError.set(String(error));
    return false;
  }
}

export async function setPreferredApp(kind: AppKind, appId: string): Promise<void> {
  appsError.set(null);
  try {
    await api.setPreferredApp(kind, appId);
    if (kind === 'ide') preferredIde.set(appId);
    if (kind === 'terminal') preferredTerminal.set(appId);
  } catch (error) {
    appsError.set(String(error));
  }
}
