//! Lógica de negocio de Mosaic, independiente de Tauri.

pub mod backup;
pub mod git;
pub mod launcher;
pub mod readme;
pub mod scanner;

/// Quita el prefijo `\\?\` de las rutas canónicas de Windows, que si no llegaría a cada tarjeta.
pub fn strip_verbatim_prefix(path: std::path::PathBuf) -> std::path::PathBuf {
    // Sin `#[cfg(windows)]` a propósito: trabajando sobre el texto, se prueba en cualquier sistema.
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
pub fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        // Un reloj anterior a 1970 no lo puede arreglar el usuario: mejor 0 que un error.
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
