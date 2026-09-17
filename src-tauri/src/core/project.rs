use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub is_git_repo: bool,
    pub primary_language: Option<String>,
    pub last_opened_at: Option<i64>,
    pub pinned: bool,
    pub notes: Option<String>,
    pub missing: bool,
    pub last_seen_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveredProject {
    pub name: String,
    pub path: PathBuf,
    pub is_git_repo: bool,
    pub primary_language: Option<String>,
    pub markers: Vec<String>,
}

impl DiscoveredProject {
    pub fn name_from_path(path: &Path) -> String {
        path.file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string_lossy().into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_from_path_uses_folder_name() {
        assert_eq!(
            DiscoveredProject::name_from_path(Path::new("/proyectos/mosaic")),
            "mosaic"
        );
    }

    #[test]
    fn name_from_path_falls_back_to_full_path() {
        assert_eq!(DiscoveredProject::name_from_path(Path::new("/")), "/");
    }
}
