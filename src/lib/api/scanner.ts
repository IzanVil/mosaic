import { invoke } from '@tauri-apps/api/core';

import type { ScanPath, ScanSummary } from '../types';

export function addScanPath(path: string): Promise<ScanPath> {
  return invoke<ScanPath>('add_scan_path', { path });
}

export function listScanPaths(): Promise<ScanPath[]> {
  return invoke<ScanPath[]>('list_scan_paths');
}

export function removeScanPath(id: number): Promise<boolean> {
  return invoke<boolean>('remove_scan_path', { id });
}

export function setScanPathEnabled(id: number, enabled: boolean): Promise<void> {
  return invoke<void>('set_scan_path_enabled', { id, enabled });
}

export function scanAllPaths(): Promise<ScanSummary> {
  return invoke<ScanSummary>('scan_all_paths');
}
