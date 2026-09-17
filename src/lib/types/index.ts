export interface Project {
  id: number;
  name: string;
  path: string;
  is_git_repo: boolean;
  primary_language: string | null;
  last_opened_at: number | null;
  pinned: boolean;
  notes: string | null;
  missing: boolean;
  last_seen_at: number | null;
  created_at: number;
  updated_at: number;
}
export interface Tag {
  id: number;
  name: string;
  color: string;
  created_at: number;
}
export interface TagWithCount extends Tag {
  project_count: number;
}

export interface ProjectWithTags extends Project {
  tags: Tag[];
  git_status: GitStatusEntry | null;
}
export interface ScanPath {
  id: number;
  path: string;
  enabled: boolean;
  created_at: number;
  last_scan_at: number | null;
}
export interface ScanSummary {
  roots_scanned: number;
  roots_unavailable: number;
  entries_visited: number;
  projects_found: number;
  projects_new: number;
  projects_updated: number;
  projects_missing: number;
  truncated: boolean;
  elapsed_ms: number;
}
export interface GitStatusEntry {
  project_id: number;
  refreshed_at: number;
  branch: string | null;
  ahead: number | null;
  behind: number | null;
  is_dirty: boolean;
  last_commit_sha: string | null;
  last_commit_msg: string | null;
  last_commit_at: number | null;
  remote_url: string | null;
}
export interface GitRefreshSummary {
  refreshed: number;
  failed: number;
  elapsed_ms: number;
}
export type AppKind = 'ide' | 'terminal' | 'file_manager';
export interface DetectedApp {
  id: string;
  name: string;
  kind: AppKind;
}
export interface PreferredApps {
  ide: string;
  terminal: string;
}
export type GitStateFilter = 'all' | 'dirty' | 'clean' | 'no_repo';
export type SortOption = 'name' | 'last_opened' | 'created' | 'updated';
export type SortDirection = 'asc' | 'desc';
export interface FilterState {
  query: string;
  tag_ids: number[];
  languages: string[];
  git_state: GitStateFilter;
  pinned_only: boolean;
  include_missing: boolean;
  sort: SortOption;
  sort_dir: SortDirection;
}
export interface ViewState {
  filters: FilterState;
  sidebar_collapsed: boolean;
}
