import { invoke } from '@tauri-apps/api/core';

export function getViewState(): Promise<string | null> {
  return invoke<string | null>('get_view_state');
}
export function setViewState(json: string): Promise<void> {
  return invoke<void>('set_view_state', { json });
}
