//! Lectura del estado de un repositorio Git.
//!
//! Mosaic solo lee: nunca escribe en los repositorios ni habla con la red, así
//! que `ahead` y `behind` comparan referencias locales, sin `fetch`, como
//! `git status` sin conexión.

use std::collections::HashMap;
use std::path::Path;

use git2::{BranchType, Repository, StatusOptions};
use serde::{Deserialize, Serialize};

use crate::db::repositories::git_status::GitStatusEntry;
use crate::errors::Result;
use crate::model::git::GitStatus;

/// Un commit del historial, con lo justo para pintar una fila.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitInfo {
    pub sha: String,
    /// Los siete primeros caracteres del sha, que es como se citan a mano.
    pub short_sha: String,
    /// Primera línea del mensaje. El cuerpo no se lee: no cabe en una fila.
    pub summary: String,
    pub author_name: String,
    /// Fecha del commit, en segundos desde el epoch Unix.
    pub committed_at: i64,
}

/// Una rama del repositorio.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BranchInfo {
    /// Nombre corto: `main`, o `origin/main` para las remotas.
    pub name: String,
    /// Es la rama a la que apunta HEAD. Siempre `false` en las remotas.
    pub is_head: bool,
}

/// Ramas de un repositorio, separadas por tipo.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Branches {
    pub local: Vec<BranchInfo>,
    /// Ramas remotas que dejó el último `fetch` o `clone`: Mosaic no habla con la red.
    pub remote: Vec<BranchInfo>,
}

/// Commits que enseña la vista de detalle de un proyecto.
pub const HISTORY_LIMIT: usize = 10;

/// Los últimos `limit` commits desde HEAD, del más reciente al más antiguo.
pub fn read_history(path: &Path, limit: usize) -> Result<Vec<CommitInfo>> {
    let repo = Repository::open(path)?;

    // Una rama sin commits es un repositorio recién creado, no un error.
    if repo.head().is_err() {
        return Ok(Vec::new());
    }

    let mut walk = repo.revwalk()?;
    walk.push_head()?;
    // TIME a secas desordena los commits del mismo segundo, que es la resolución
    // de Git; TOPOLOGICAL deja siempre cada commit antes que sus padres.
    walk.set_sorting(git2::Sort::TIME | git2::Sort::TOPOLOGICAL)?;

    let mut commits = Vec::with_capacity(limit.min(64));
    for oid in walk.take(limit) {
        let commit = repo.find_commit(oid?)?;
        let sha = commit.id().to_string();
        commits.push(CommitInfo {
            short_sha: sha.chars().take(7).collect(),
            sha,
            summary: commit.summary().ok().flatten().unwrap_or("").to_string(),
            author_name: commit.author().name().unwrap_or("").to_string(),
            committed_at: commit.time().seconds(),
        });
    }
    Ok(commits)
}

/// Lista las ramas locales y las remotas que hay en disco.
pub fn read_branches(path: &Path) -> Result<Branches> {
    let repo = Repository::open(path)?;
    let mut branches = Branches::default();

    for tipo in [BranchType::Local, BranchType::Remote] {
        for entrada in repo.branches(Some(tipo))? {
            let (branch, _) = entrada?;
            let Some(name) = branch.name()?.map(str::to_string) else {
                // Un nombre que no es UTF-8 no se puede mostrar: mejor saltarlo que abortar.
                continue;
            };

            // `origin/HEAD` apunta a la rama por defecto del remoto: no es una rama más.
            if name.ends_with("/HEAD") {
                continue;
            }

            let info = BranchInfo {
                name,
                is_head: tipo == BranchType::Local && branch.is_head(),
            };
            match tipo {
                BranchType::Local => branches.local.push(info),
                BranchType::Remote => branches.remote.push(info),
            }
        }
    }

    Ok(branches)
}

/// Lee el estado del repositorio cuya raíz es `path`.
pub fn read_git_status(path: &Path) -> Result<GitStatus> {
    // `open` no busca hacia arriba: un repositorio anidado da su propio estado.
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

/// Rama actual y commit de HEAD, contando la rama sin nacer y el HEAD separado.
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
fn is_dirty(repo: &Repository) -> Result<bool> {
    if repo.is_bare() {
        return Ok(false);
    }

    let mut options = StatusOptions::new();
    options
        .include_untracked(true)
        // Basta con saber que existe un directorio sin seguir; recorrerlo puede ser carísimo.
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
    let first = names.iter().find_map(|name| name.ok().flatten())?;
    let remote = repo.find_remote(first).ok()?;
    remote.url().ok().map(str::to_string)
}

/// Resumen de un refresco de todos los repositorios, con el parche que aplica la interfaz.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitRefreshSummary {
    /// Repositorios leídos y cacheados con éxito.
    pub refreshed: usize,
    /// Repositorios que no se pudieron leer; su entrada en la caché se borra.
    pub failed: usize,
    pub elapsed_ms: u64,
    /// Entradas con un estado distinto del de la caché; la hora de lectura no cuenta.
    pub changed: Vec<GitStatusEntry>,
    /// Proyectos que se han quedado sin estado por no poderse leer.
    pub removed: Vec<i64>,
}

/// Relee todos los repositorios, guarda su estado en la caché y dice qué ha cambiado.
pub fn refresh_all(db: &crate::db::Db) -> Result<GitRefreshSummary> {
    use crate::db::repositories::{git_status as cache, projects as projects_repo};

    let started = std::time::Instant::now();
    let now = crate::core::now_ts();

    let projects = db.with_conn(projects_repo::list_all)?;
    let before: HashMap<i64, GitStatus> = db
        .with_conn(cache::list_all)?
        .into_iter()
        .map(|entry| (entry.project_id, entry.status))
        .collect();
    let mut refreshed = 0;
    let mut failed = 0;
    let mut changed = Vec::new();
    let mut removed = Vec::new();

    for project in projects
        .iter()
        .filter(|project| project.is_git_repo && !project.missing)
    {
        match read_git_status(Path::new(&project.path)) {
            Ok(status) => {
                db.with_conn(|conn| cache::upsert(conn, project.id, &status, now))?;
                refreshed += 1;
                if before.get(&project.id) != Some(&status) {
                    changed.push(GitStatusEntry {
                        project_id: project.id,
                        refreshed_at: now,
                        status,
                    });
                }
            }
            // Un repositorio ilegible no aborta el refresco, y su entrada se borra
            // para no enseñar datos rancios.
            Err(err) => {
                tracing::warn!(path = %project.path, %err, "no se pudo leer el repositorio");
                db.with_conn(|conn| cache::delete(conn, project.id))?;
                failed += 1;
                if before.contains_key(&project.id) {
                    removed.push(project.id);
                }
            }
        }
    }

    let summary = GitRefreshSummary {
        refreshed,
        failed,
        elapsed_ms: started.elapsed().as_millis() as u64,
        changed,
        removed,
    };
    tracing::info!(
        refreshed,
        failed,
        changed = summary.changed.len(),
        removed = summary.removed.len(),
        elapsed_ms = summary.elapsed_ms,
        "estado Git refrescado"
    );
    Ok(summary)
}

/// Relee un proyecto: `None` si no es un repositorio; si no se puede leer, borra su caché.
pub fn refresh_one(db: &crate::db::Db, project_id: i64) -> Result<Option<GitStatusEntry>> {
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

    #[test]
    fn read_history_returns_commits_from_newest_to_oldest() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        write(&tmp.path().join("a.txt"), "uno");
        commit_all(&repo, "feat: primero");
        write(&tmp.path().join("b.txt"), "dos");
        commit_all(&repo, "feat: segundo");
        write(&tmp.path().join("c.txt"), "tres");
        commit_all(&repo, "feat: tercero");

        let historial = read_history(tmp.path(), 10).unwrap();
        let mensajes: Vec<&str> = historial.iter().map(|c| c.summary.as_str()).collect();
        assert_eq!(
            mensajes,
            vec!["feat: tercero", "feat: segundo", "feat: primero"]
        );

        let primero = &historial[0];
        assert_eq!(primero.short_sha.len(), 7);
        assert!(primero.sha.starts_with(&primero.short_sha));
        assert_eq!(primero.author_name, "Test");
        assert!(primero.committed_at > 0);
    }

    #[test]
    fn read_history_respects_the_limit() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        for n in 0..5 {
            write(&tmp.path().join(format!("{n}.txt")), "x");
            commit_all(&repo, &format!("commit {n}"));
        }

        assert_eq!(read_history(tmp.path(), 2).unwrap().len(), 2);
        assert_eq!(read_history(tmp.path(), 99).unwrap().len(), 5);
    }

    #[test]
    fn read_history_is_empty_on_an_unborn_branch() {
        let tmp = tempfile::tempdir().unwrap();
        init_repo(tmp.path());

        assert!(read_history(tmp.path(), 10).unwrap().is_empty());
    }

    #[test]
    fn read_history_works_with_a_detached_head() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        write(&tmp.path().join("a.txt"), "uno");
        let primero = commit_all(&repo, "feat: primero");
        write(&tmp.path().join("b.txt"), "dos");
        commit_all(&repo, "feat: segundo");
        repo.set_head_detached(primero).unwrap();

        let historial = read_history(tmp.path(), 10).unwrap();
        assert_eq!(
            historial.len(),
            1,
            "solo lo accesible desde el HEAD separado"
        );
        assert_eq!(historial[0].summary, "feat: primero");
    }

    #[test]
    fn read_branches_marks_the_current_one() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = init_repo(tmp.path());
        write(&tmp.path().join("a.txt"), "uno");
        commit_all(&repo, "feat: primero");
        let commit = repo.head().unwrap().peel_to_commit().unwrap();
        repo.branch("feat/etiquetas", &commit, false).unwrap();

        let ramas = read_branches(tmp.path()).unwrap();
        let nombres: Vec<&str> = ramas.local.iter().map(|b| b.name.as_str()).collect();
        assert!(nombres.contains(&"main"));
        assert!(nombres.contains(&"feat/etiquetas"));

        let actual: Vec<&str> = ramas
            .local
            .iter()
            .filter(|b| b.is_head)
            .map(|b| b.name.as_str())
            .collect();
        assert_eq!(actual, vec!["main"], "solo una rama es HEAD");
        assert!(ramas.remote.is_empty(), "sin remoto no hay ramas remotas");
    }

    #[test]
    fn read_branches_lists_remote_branches_without_touching_the_network() {
        let origen = tempfile::tempdir().unwrap();
        let repo_origen = init_repo(origen.path());
        write(&origen.path().join("a.txt"), "uno");
        commit_all(&repo_origen, "feat: primero");

        let clon = tempfile::tempdir().unwrap();
        let destino = clon.path().join("copia");
        let repo = Repository::clone(origen.path().to_str().unwrap(), &destino).unwrap();
        drop(repo);

        let ramas = read_branches(&destino).unwrap();
        let remotas: Vec<&str> = ramas.remote.iter().map(|b| b.name.as_str()).collect();
        assert!(
            remotas.contains(&"origin/main"),
            "las ramas remotas conocidas en disco se listan: {remotas:?}"
        );
        assert!(
            !remotas.iter().any(|n| n.ends_with("/HEAD")),
            "origin/HEAD es un puntero simbólico, no una rama: {remotas:?}"
        );
        assert!(ramas.remote.iter().all(|b| !b.is_head));
    }

    #[test]
    fn read_branches_is_empty_on_an_unborn_branch() {
        let tmp = tempfile::tempdir().unwrap();
        init_repo(tmp.path());

        let ramas = read_branches(tmp.path()).unwrap();
        assert!(ramas.local.is_empty(), "sin commits no hay ramas todavía");
        assert!(ramas.remote.is_empty());
    }
}
