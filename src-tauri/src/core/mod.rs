pub mod git;
pub mod launcher;
pub mod project;
pub mod scanner;
pub mod tag;

pub use project::{DiscoveredProject, Project};
pub use tag::{Tag, TagWithCount};

pub fn strip_verbatim_prefix(path: std::path::PathBuf) -> std::path::PathBuf {
    let Some(texto) = path.to_str() else {
        return path;
    };

    if let Some(resto) = texto.strip_prefix(r"\\?\UNC\") {
        return std::path::PathBuf::from(format!(r"\\{resto}"));
    }
    if let Some(resto) = texto.strip_prefix(r"\\?\") {
        return std::path::PathBuf::from(resto);
    }
    path
}

pub fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn strips_the_windows_verbatim_prefix() {
        assert_eq!(
            strip_verbatim_prefix(PathBuf::from(r"\\?\C:\Users\izan\Proyectos")),
            PathBuf::from(r"C:\Users\izan\Proyectos")
        );
    }

    #[test]
    fn rewrites_a_verbatim_unc_path_as_a_plain_unc_path() {
        assert_eq!(
            strip_verbatim_prefix(PathBuf::from(r"\\?\UNC\servidor\recurso\codigo")),
            PathBuf::from(r"\\servidor\recurso\codigo")
        );
    }

    #[test]
    fn leaves_every_other_path_untouched() {
        for ruta in ["/home/izan/proyectos", r"C:\Proyectos", "relativa/sin/raiz"] {
            assert_eq!(
                strip_verbatim_prefix(PathBuf::from(ruta)),
                PathBuf::from(ruta),
                "no debería tocar «{ruta}»"
            );
        }
    }
}
