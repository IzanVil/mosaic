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

/** Espejo de `core::tag::Tag`. */
export interface Tag {
  id: number;
  /** Nombre tal y como lo escribió el usuario. Único ignorando mayúsculas. */
  name: string;
  /** Color en formato `#RRGGBB`, siempre en mayúsculas. */
  color: string;
  created_at: number;
}

/**
 * Espejo de `core::tag::TagWithCount`.
 *
 * El backend serializa el `Tag` aplanado, así que `project_count` llega al
 * mismo nivel que sus campos.
 */
export interface TagWithCount extends Tag {
  /** Proyectos que tienen la etiqueta asignada. */
  project_count: number;
}

/**
 * Espejo de `db::repositories::projects::ProjectWithTags`.
 *
 * El `Project` llega aplanado. Es la única lectura que necesita el tablero.
 */
export interface ProjectWithTags extends Project {
  /** Etiquetas asignadas, ordenadas por nombre. Vacío si no tiene ninguna. */
  tags: Tag[];
  /** `null` si el proyecto no es repositorio o si nunca se refrescó. */
  git_status: GitStatusEntry | null;
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

/**
 * Espejo de `db::repositories::git_status::GitStatusEntry`.
 *
 * El backend serializa el `GitStatus` aplanado, así que sus campos llegan al
 * mismo nivel que `project_id` y `refreshed_at`.
 */
export interface GitStatusEntry {
  project_id: number;
  /** Cuándo se leyó el repositorio, en segundos desde el epoch Unix. */
  refreshed_at: number;
  /** Rama actual. `null` si el HEAD está separado. */
  branch: string | null;
  /** Commits por delante del upstream. `null` si la rama no tiene upstream. */
  ahead: number | null;
  /** Commits por detrás del upstream. `null` si la rama no tiene upstream. */
  behind: number | null;
  /** Hay cambios sin commitear, con el mismo criterio que `git status`. */
  is_dirty: boolean;
  last_commit_sha: string | null;
  /** Primera línea del mensaje del último commit. */
  last_commit_msg: string | null;
  last_commit_at: number | null;
  remote_url: string | null;
}

/** Espejo de `core::git::GitRefreshSummary`. */
export interface GitRefreshSummary {
  refreshed: number;
  failed: number;
  elapsed_ms: number;
}

/** Espejo de `core::git::CommitInfo`. */
export interface CommitInfo {
  sha: string;
  /** Los siete primeros caracteres del sha, que es como se citan a mano. */
  short_sha: string;
  /** Primera línea del mensaje. */
  summary: string;
  author_name: string;
  /** Segundos desde el epoch Unix. */
  committed_at: number;
}

/** Espejo de `core::git::BranchInfo`. */
export interface BranchInfo {
  /** Nombre corto: `main`, o `origin/main` para las remotas. */
  name: string;
  /** Es la rama a la que apunta HEAD. Siempre `false` en las remotas. */
  is_head: boolean;
}

/** Espejo de `core::git::Branches`. */
export interface Branches {
  local: BranchInfo[];
  /** Remotas conocidas localmente: las que dejó el último `fetch` del usuario. */
  remote: BranchInfo[];
}

/** Espejo de `core::readme::ReadmePreview`. */
export interface ReadmePreview {
  /** Nombre del fichero encontrado, tal y como está en disco. */
  file_name: string;
  /** HTML ya saneado en el backend, listo para insertar. */
  html: string;
  /** El fichero pasaba de 512 KiB y se ha recortado. */
  truncated: boolean;
  /** Imágenes que no se cargan (del Markdown o del HTML). */
  images_omitted: number;
  /** Vídeos, iframes, SVG y otros elementos que no se muestran. */
  media_omitted: number;
}

/** Espejo de `core::launcher::AppKind`. */
export type AppKind = 'ide' | 'terminal' | 'file_manager';

/** Espejo de `core::launcher::DetectedApp`. */
export interface DetectedApp {
  /** Nombre del ejecutable; es el identificador que se guarda en los ajustes. */
  id: string;
  name: string;
  kind: AppKind;
}

/** Espejo de `commands::system::PreferredApps`. Cadena vacía = el primero disponible. */
export interface PreferredApps {
  ide: string;
  terminal: string;
}

/*
 * Los tipos de aquí abajo son estado de la interfaz: no tienen struct
 * equivalente en Rust. El backend guarda [`ViewState`] como un JSON opaco en
 * la clave `ui.view_state` y solo comprueba que sea JSON válido.
 */

/** Estado Git por el que se puede filtrar el tablero. */
export type GitStateFilter = 'all' | 'dirty' | 'clean' | 'no_repo';

/** Criterio de ordenación del tablero. */
export type SortOption = 'name' | 'last_opened' | 'created' | 'updated';

export type SortDirection = 'asc' | 'desc';

/**
 * Búsqueda, filtros y ordenación activos.
 *
 * Las etiquetas de `tag_ids` se combinan en OR entre sí (un proyecto vale si
 * tiene cualquiera de ellas), igual que los lenguajes de `languages`. Todo lo
 * demás se combina en AND.
 */
export interface FilterState {
  query: string;
  tag_ids: number[];
  languages: string[];
  git_state: GitStateFilter;
  pinned_only: boolean;
  /** `false` oculta los proyectos que el escáner ya no encuentra en disco. */
  include_missing: boolean;
  sort: SortOption;
  sort_dir: SortDirection;
}

/** Densidad del tablero. `comodo` es la de siempre. */
export type Density = 'comodo' | 'compacto';

/** Tema elegido. `system` sigue la preferencia del sistema operativo. */
export type ThemeMode = 'dark' | 'light' | 'system';

/** Lo que se persiste entre sesiones para recuperar la última vista. */
export interface ViewState {
  filters: FilterState;
  sidebar_collapsed: boolean;
  density: Density;
  theme: ThemeMode;
  /** Ya se cerró (o ya no hace falta) el aviso de que el nombre abre el detalle. */
  detail_tip_dismissed: boolean;
}
