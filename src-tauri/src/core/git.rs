//! Lectura del estado de un repositorio Git.
//!
//! Mosaic **solo lee**: nunca escribe en los repositorios ni habla con la red.
//! Por eso `git2` se compila sin las features `ssh` ni `https`, y `ahead`/
//! `behind` se calculan comparando referencias locales, sin hacer `fetch`. Los
//! números pueden estar desactualizados respecto al remoto real, que es
//! exactamente lo que muestra `git status` sin conexión.

use std::path::Path;

use git2::{BranchType, Repository, StatusOptions};
use serde::{Deserialize, Serialize};

use crate::errors::Result;

/// Estado de un repositorio Git en un instante dado.
///
/// Espejo de las columnas de `git_status_cache`, sin los campos de la caché.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitStatus {
    /// Rama actual. `None` si el HEAD está separado.
    pub branch: Option<String>,
    /// Commits que la rama local tiene y su upstream no. `None` si no hay upstream.
    pub ahead: Option<u32>,
    /// Commits que el upstream tiene y la rama local no. `None` si no hay upstream.
    pub behind: Option<u32>,
    /// Hay cambios sin commitear: ficheros modificados o sin seguir, respetando
    /// `.gitignore`. Es el mismo criterio que `git status`.
    pub is_dirty: bool,
    pub last_commit_sha: Option<String>,
    /// Primera línea del mensaje del último commit.
    pub last_commit_msg: Option<String>,
    /// Fecha del último commit, en segundos desde el epoch Unix.
    pub last_commit_at: Option<i64>,
    /// URL del remoto `origin`, o del primer remoto que haya.
    pub remote_url: Option<String>,
}

/// Lee el estado del repositorio que hay en `path`.
///
/// `path` debe ser la raíz del repositorio: no se busca hacia arriba, de modo
/// que un repositorio anidado devuelve su propio estado y no el de su contenedor.
pub fn read_git_status(path: &Path) -> Result<GitStatus> {
    let repo = Repository::open(path)?;

    let (branch, commit) = read_head(&repo);

    let (ahead, behind) = branch
        .as_deref()
        .and_then(|name| ahead_behind(&repo, name))
        .map_or((None, None), |(ahead, behind)| (Some(ahead), Some(behind)));

    Ok(GitStatus {
        branch,
        ahead,
        behind,
        is_dirty: is_dirty(&repo)?,
        last_commit_sha: commit.as_ref().map(|commit| commit.id().to_string()),
        last_commit_msg: commit
            .as_ref()
            .and_then(|commit| commit.summary().ok().flatten())
            .map(str::to_string),
        last_commit_at: commit.as_ref().map(|commit| commit.time().seconds()),
        remote_url: remote_url(&repo),
    })
}

/// Devuelve la rama actual y el commit al que apunta HEAD.
///
/// Contempla los dos casos en los que no hay commit: un repositorio recién
/// inicializado (rama sin nacer, de la que sí sabemos el nombre) y un HEAD
/// separado (hay commit pero no rama).
fn read_head(repo: &Repository) -> (Option<String>, Option<git2::Commit<'_>>) {
    let Ok(head) = repo.head() else {
        return (unborn_branch_name(repo), None);
    };

    let branch = if head.is_branch() {
        head.shorthand().ok().map(str::to_string)
    } else {
        None
    };

    (branch, head.peel_to_commit().ok())
}

/// Nombre de la rama de un repositorio sin ningún commit todavía.
fn unborn_branch_name(repo: &Repository) -> Option<String> {
    let head = repo.find_reference("HEAD").ok()?;
    let target = head.symbolic_target().ok().flatten()?;
    target.strip_prefix("refs/heads/").map(str::to_string)
}

/// Commits de diferencia entre la rama local y su upstream, si lo tiene.
fn ahead_behind(repo: &Repository, branch_name: &str) -> Option<(u32, u32)> {
    let branch = repo.find_branch(branch_name, BranchType::Local).ok()?;
    let upstream = branch.upstream().ok()?;

    let local = branch.get().target()?;
    let remote = upstream.get().target()?;

    let (ahead, behind) = repo.graph_ahead_behind(local, remote).ok()?;
    Some((ahead as u32, behind as u32))
}

/// Indica si hay cambios sin commitear, incluidos ficheros sin seguir.
///
/// Los repositorios bare no tienen árbol de trabajo, así que nunca están sucios.
fn is_dirty(repo: &Repository) -> Result<bool> {
    if repo.is_bare() {
        return Ok(false);
    }

    let mut options = StatusOptions::new();
    options
        .include_untracked(true)
        // Basta con saber que un directorio sin seguir existe: entrar a contar
        // sus ficheros puede ser carísimo y no cambia la respuesta.
        .recurse_untracked_dirs(false)
        .include_ignored(false);

    Ok(!repo.statuses(Some(&mut options))?.is_empty())
}

/// URL del remoto `origin`; si no existe, la del primer remoto configurado.
fn remote_url(repo: &Repository) -> Option<String> {
    if let Ok(origin) = repo.find_remote("origin") {
        if let Ok(url) = origin.url() {
            return Some(url.to_string());
        }
    }

    let names = repo.remotes().ok()?;
    // `StringArray::iter` entrega `Result<Option<&str>, Error>` por nombre.
    let first = names.iter().find_map(|name| name.ok().flatten())?;
    let remote = repo.find_remote(first).ok()?;
    remote.url().ok().map(str::to_string)
}

/// Resumen de un refresco masivo del estado Git.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitRefreshSummary {
    /// Repositorios leídos y cacheados con éxito.
    pub refreshed: usize,
    /// Repositorios que no se pudieron leer; su entrada en la caché se borra.
    pub failed: usize,
    pub elapsed_ms: u64,
}

/// Relee el estado de todos los proyectos que son repositorios Git y lo cachea.
///
/// Un repositorio ilegible no aborta el refresco: se registra el fallo, se borra
/// su entrada de la caché para no mostrar datos rancios y se sigue con el resto.
pub fn refresh_all(db: &crate::db::Db) -> Result<GitRefreshSummary> {
    use crate::db::repositories::{git_status as cache, projects as projects_repo};

    let started = std::time::Instant::now();
    let now = crate::core::now_ts();

    let projects = db.with_conn(projects_repo::list_all)?;
    let mut refreshed = 0;
    let mut failed = 0;

    for project in projects
        .iter()
        .filter(|project| project.is_git_repo && !project.missing)
    {
        match read_git_status(Path::new(&project.path)) {
            Ok(status) => {
                db.with_conn(|conn| cache::upsert(conn, project.id, &status, now))?;
                refreshed += 1;
            }
            Err(err) => {
                tracing::warn!(path = %project.path, %err, "no se pudo leer el repositorio");
                db.with_conn(|conn| cache::delete(conn, project.id))?;
                failed += 1;
            }
        }
    }

    let summary = GitRefreshSummary {
        refreshed,
        failed,
        elapsed_ms: started.elapsed().as_millis() as u64,
    };
    tracing::info!(?summary, "estado Git refrescado");
    Ok(summary)
}

/// Relee el estado de un único proyecto.
///
/// Devuelve `None` si el proyecto no es un repositorio Git. Si lo es pero no se
/// puede leer, borra su entrada de la caché y propaga el error.
pub fn refresh_one(
    db: &crate::db::Db,
    project_id: i64,
) -> Result<Option<crate::db::repositories::git_status::GitStatusEntry>> {
    use crate::db::repositories::{git_status as cache, projects as projects_repo};

    let project = db.with_conn(|conn| projects_repo::get_by_id(conn, project_id))?;
    if !project.is_git_repo {
        return Ok(None);
    }

    let now = crate::core::now_ts();
    match read_git_status(Path::new(&project.path)) {
        Ok(status) => {
            db.with_conn(|conn| cache::upsert(conn, project_id, &status, now))?;
            db.with_conn(|conn| cache::get(conn, project_id))
        }
        Err(err) => {
            db.with_conn(|conn| cache::delete(conn, project_id))?;
            Err(err)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use git2::{IndexAddOption, Oid, RepositoryInitOptions, Signature};

    use super::*;

    /// Inicializa un repositorio con `main` como rama inicial, para que los
    /// tests no dependan de `init.defaultBranch` de la máquina.
    fn init_repo(path: &Path) -> Repository {
        let mut options = RepositoryInitOptions::new();
        options.initial_head("main");
        Repository::init_opts(path, &options).unwrap()
    }

    /// Añade todo lo que no esté ignorado y crea un commit sobre HEAD.
    fn commit_all(repo: &Repository, message: &str) -> Oid {
        let mut index = repo.index().unwrap();
        index
            .add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
            .unwrap();
        index.write().unwrap();

        let tree = repo.find_tree(index.write_tree().unwrap()).unwrap();
        let signature = Signature::now("Test", "test@example.invalid").unwrap();

        let parent = repo.head().ok().and_then(|head| head.peel_to_commit().ok());
        let parents: Vec<&git2::Commit<'_>> = parent.iter().collect();

        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &parents,
        )
        .unwrap()
    }

    fn write(path: &Path, contents: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }

    #[test]
    fn reads_branch_and_last_commit() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        write(&tmp.path().join("README.md"), "hola");
        let oid = commit_all(&repo, "feat: primer commit\n\ncuerpo que no se muestra");

        let status = read_git_status(tmp.path()).unwrap();

        assert_eq!(status.branch.as_deref(), Some("main"));
        assert_eq!(
            status.last_commit_sha.as_deref(),
            Some(&oid.to_string()[..])
        );
        assert_eq!(
            status.last_commit_msg.as_deref(),
            Some("feat: primer commit"),
            "solo la primera línea del mensaje"
        );
        assert!(status.last_commit_at.is_some());
    }

    #[test]
    fn clean_repository_is_not_dirty() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        write(&tmp.path().join("README.md"), "hola");
        commit_all(&repo, "inicial");

        assert!(!read_git_status(tmp.path()).unwrap().is_dirty);
    }

    #[test]
    fn modified_tracked_file_makes_it_dirty() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        write(&tmp.path().join("README.md"), "hola");
        commit_all(&repo, "inicial");

        write(&tmp.path().join("README.md"), "hola, otra vez");

        assert!(read_git_status(tmp.path()).unwrap().is_dirty);
    }

    #[test]
    fn untracked_file_makes_it_dirty() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        write(&tmp.path().join("README.md"), "hola");
        commit_all(&repo, "inicial");

        write(&tmp.path().join("notas.txt"), "sin añadir");

        assert!(
            read_git_status(tmp.path()).unwrap().is_dirty,
            "el criterio es el mismo que el de `git status`"
        );
    }

    #[test]
    fn ignored_file_does_not_make_it_dirty() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        write(&tmp.path().join(".gitignore"), "target/\n");
        write(&tmp.path().join("README.md"), "hola");
        commit_all(&repo, "inicial");

        write(&tmp.path().join("target/artefacto.bin"), "basura");

        assert!(!read_git_status(tmp.path()).unwrap().is_dirty);
    }

    #[test]
    fn unborn_repository_reports_its_branch_without_commit() {
        let tmp = tempfile::tempdir().unwrap();
        init_repo(tmp.path());

        let status = read_git_status(tmp.path()).unwrap();

        assert_eq!(status.branch.as_deref(), Some("main"));
        assert_eq!(status.last_commit_sha, None);
        assert_eq!(status.last_commit_msg, None);
        assert!(!status.is_dirty);
    }

    #[test]
    fn detached_head_has_no_branch_but_keeps_the_commit() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        write(&tmp.path().join("README.md"), "hola");
        let oid = commit_all(&repo, "inicial");

        repo.set_head_detached(oid).unwrap();

        let status = read_git_status(tmp.path()).unwrap();
        assert_eq!(status.branch, None);
        assert_eq!(
            status.last_commit_sha.as_deref(),
            Some(&oid.to_string()[..])
        );
    }

    #[test]
    fn ahead_and_behind_are_none_without_upstream() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        write(&tmp.path().join("README.md"), "hola");
        commit_all(&repo, "inicial");

        let status = read_git_status(tmp.path()).unwrap();
        assert_eq!(status.ahead, None);
        assert_eq!(status.behind, None);
    }

    /// Apunta `refs/remotes/origin/main` a `oid` y lo declara upstream de `main`.
    fn set_upstream(repo: &Repository, oid: Oid) {
        repo.remote("origin", "https://example.invalid/repo.git")
            .ok();
        repo.reference("refs/remotes/origin/main", oid, true, "test")
            .unwrap();
        repo.find_branch("main", BranchType::Local)
            .unwrap()
            .set_upstream(Some("origin/main"))
            .unwrap();
    }

    #[test]
    fn counts_commits_ahead_of_the_upstream() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        write(&tmp.path().join("README.md"), "hola");
        let base = commit_all(&repo, "inicial");
        set_upstream(&repo, base);

        write(&tmp.path().join("README.md"), "un cambio");
        commit_all(&repo, "segundo");

        let status = read_git_status(tmp.path()).unwrap();
        assert_eq!(status.ahead, Some(1));
        assert_eq!(status.behind, Some(0));
    }

    #[test]
    fn counts_commits_behind_the_upstream() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        write(&tmp.path().join("README.md"), "hola");
        let base = commit_all(&repo, "inicial");
        write(&tmp.path().join("README.md"), "un cambio");
        let adelantado = commit_all(&repo, "segundo");

        // El remoto se queda con el commit nuevo y la rama local retrocede.
        set_upstream(&repo, adelantado);
        repo.reference("refs/heads/main", base, true, "test")
            .unwrap();

        let status = read_git_status(tmp.path()).unwrap();
        assert_eq!(status.ahead, Some(0));
        assert_eq!(status.behind, Some(1));
    }

    #[test]
    fn reads_the_origin_remote_url() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        repo.remote("upstream", "https://example.invalid/otro.git")
            .unwrap();
        repo.remote("origin", "https://example.invalid/repo.git")
            .unwrap();

        let status = read_git_status(tmp.path()).unwrap();
        assert_eq!(
            status.remote_url.as_deref(),
            Some("https://example.invalid/repo.git"),
            "origin tiene prioridad"
        );
    }

    #[test]
    fn falls_back_to_the_first_remote_when_there_is_no_origin() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        repo.remote("upstream", "https://example.invalid/otro.git")
            .unwrap();

        let status = read_git_status(tmp.path()).unwrap();
        assert_eq!(
            status.remote_url.as_deref(),
            Some("https://example.invalid/otro.git")
        );
    }

    #[test]
    fn repository_without_remotes_has_no_url() {
        let tmp = tempfile::tempdir().unwrap();
        init_repo(tmp.path());

        assert_eq!(read_git_status(tmp.path()).unwrap().remote_url, None);
    }

    #[test]
    fn bare_repository_is_never_dirty() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = Repository::init_bare(tmp.path()).unwrap();
        assert!(repo.is_bare());

        let status = read_git_status(tmp.path()).unwrap();

        assert!(
            !status.is_dirty,
            "un repositorio bare no tiene árbol de trabajo que ensuciar"
        );
    }

    #[test]
    fn a_plain_directory_is_not_a_repository() {
        let tmp = tempfile::tempdir().unwrap();

        let err = read_git_status(tmp.path()).unwrap_err();
        assert!(matches!(err, crate::errors::AppError::Git(_)));
    }

    #[test]
    fn nested_repository_reports_its_own_state_not_its_containers() {
        let tmp = tempfile::tempdir().unwrap();
        let outer = init_repo(tmp.path());
        write(&tmp.path().join("README.md"), "fuera");
        commit_all(&outer, "commit de fuera");

        let inner_path = tmp.path().join("anidado");
        fs::create_dir_all(&inner_path).unwrap();
        let inner = init_repo(&inner_path);
        write(&inner_path.join("dentro.txt"), "dentro");
        commit_all(&inner, "commit de dentro");

        let status = read_git_status(&inner_path).unwrap();
        assert_eq!(status.last_commit_msg.as_deref(), Some("commit de dentro"));
    }
}
