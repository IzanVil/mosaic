//! Lógica de negocio de Mosaic, independiente de Tauri.

pub mod git;
pub mod launcher;
pub mod project;
pub mod scanner;
pub mod tag;

pub use project::{DiscoveredProject, Project};
pub use tag::{Tag, TagWithCount};

/// Quita el prefijo verbatim que Windows pone en las rutas canónicas.
///
/// `std::fs::canonicalize` devuelve en Windows rutas como `\\?\C:\proyectos`,
/// que funcionan para el sistema de ficheros pero se guardarían tal cual en
/// `projects.path` y se verían en cada tarjeta. El caso UNC
/// (`\\?\UNC\servidor\recurso`) se reescribe a su forma normal.
///
/// No va tras `#[cfg(windows)]` a propósito: trabajando sobre el texto, la
/// función es comprobable desde cualquier plataforma y en Linux no encuentra
/// nunca esos prefijos.
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

/// Marca de tiempo actual en segundos desde el epoch Unix.
///
/// Si el reloj del sistema estuviese antes del epoch se devuelve `0`, que es
/// preferible a propagar un error por algo que no puede corregir el usuario.
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
