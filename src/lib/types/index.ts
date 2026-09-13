/**
 * Tipos compartidos con el backend.
 *
 * Son un espejo literal de los structs de Rust: los campos van en `snake_case`
 * porque serde los serializa con sus nombres tal cual, que a su vez coinciden
 * con las columnas SQL. Cualquier cambio aquí debe ir acompañado del cambio
 * equivalente en `src-tauri/src/`.
 */

/** Espejo de `core::project::Project`. */
export interface Project {
  id: number;
  name: string;
  /** Ruta absoluta y canónica. Es la clave natural del proyecto. */
  path: string;
  is_git_repo: boolean;
  primary_language: string | null;
  /** Segundos desde el epoch Unix. */
  last_opened_at: number | null;
  pinned: boolean;
  notes: string | null;
  /** El último escaneo de su raíz no encontró la carpeta en disco. */
  missing: boolean;
  last_seen_at: number | null;
  created_at: number;
  updated_at: number;
}

/** Espejo de `db::repositories::scan_paths::ScanPath`. */
export interface ScanPath {
  id: number;
  path: string;
  enabled: boolean;
  created_at: number;
  last_scan_at: number | null;
}

/** Espejo de `core::scanner::ScanSummary`. */
export interface ScanSummary {
  roots_scanned: number;
  roots_unavailable: number;
  entries_visited: number;
  projects_found: number;
  projects_new: number;
  projects_updated: number;
  projects_missing: number;
  /** Alguna raíz alcanzó el tope de entradas y quedó incompleta. */
  truncated: boolean;
  elapsed_ms: number;
}
