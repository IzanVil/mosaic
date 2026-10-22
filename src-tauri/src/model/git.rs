//! Estado de un repositorio Git, tal y como se guarda en la caché.

use serde::{Deserialize, Serialize};

/// Estado de un repositorio en un instante: las columnas de `git_status_cache` sin las de la caché.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitStatus {
    /// Rama actual. `None` si el HEAD está separado.
    pub branch: Option<String>,
    /// Commits que la rama local tiene y su upstream no. `None` si no hay upstream.
    pub ahead: Option<u32>,
    /// Commits que el upstream tiene y la rama local no. `None` si no hay upstream.
    pub behind: Option<u32>,
    /// Hay cambios sin commitear, ficheros sin seguir incluidos, con el criterio de `git status`.
    pub is_dirty: bool,
    pub last_commit_sha: Option<String>,
    /// Primera línea del mensaje del último commit.
    pub last_commit_msg: Option<String>,
    /// Fecha del último commit, en segundos desde el epoch Unix.
    pub last_commit_at: Option<i64>,
    /// URL del remoto `origin`, o del primer remoto que haya.
    pub remote_url: Option<String>,
}
