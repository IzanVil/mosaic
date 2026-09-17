import { invoke } from '@tauri-apps/api/core';

import type { AppKind, DetectedApp, PreferredApps } from '../types';

export function listDetectedApps(): Promise<DetectedApp[]> {
  return invoke<DetectedApp[]>('list_detected_apps');
}

export function openIn(kind: AppKind, projectId: number): Promise<void> {
  return invoke<void>('open_in', { kind, projectId });
}

export function getPreferredApps(): Promise<PreferredApps> {
  return invoke<PreferredApps>('get_preferred_apps');
}

export function setPreferredApp(kind: AppKind, appId: string): Promise<void> {
  return invoke<void>('set_preferred_app', { kind, appId });
}
