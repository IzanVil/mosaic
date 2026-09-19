//! Lectura y renderizado del README de un proyecto.
//!
//! El README es contenido de terceros: viene de un repositorio que Mosaic no
//! controla. Por eso el HTML crudo del Markdown **se descarta antes de
//! renderizar** y lo que queda pasa además por un saneador, en vez de confiar
//! en uno solo de los dos filtros.
//!
//! El renderizado vive en el backend y no en el webview a propósito: así el
//! Markdown de origen no se procesa nunca dentro de la ventana, y las reglas de
//! saneado quedan cubiertas por `cargo test`.

use std::path::{Path, PathBuf};

use pulldown_cmark::{html, Event, Options, Parser, Tag, TagEnd};
use serde::{Deserialize, Serialize};

use crate::errors::Result;

/// Tope de tamaño del fichero. Por encima se recorta y se avisa.
pub const MAX_BYTES: usize = 512 * 1024;

/// Nombres de fichero aceptados, en orden de preferencia.
///
/// Se comparan sin distinguir mayúsculas y **solo en la raíz** del proyecto:
/// un `docs/README.md` no es el README del proyecto.
const CANDIDATES: &[&str] = &[
    "readme.md",
    "readme.markdown",
    "readme.mdown",
    "readme.txt",
    "readme",
];

/// README de un proyecto, ya convertido a HTML seguro.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadmePreview {
    /// Nombre del fichero encontrado, tal y como está en disco.
    pub file_name: String,
    /// HTML saneado, listo para insertar.
    pub html: String,
    /// El fichero pasaba de [`MAX_BYTES`] y se ha recortado.
    pub truncated: bool,
}

/// Busca el README en la raíz de `dir`.
///
/// Devuelve `None` si no hay ninguno, que es un caso normal y no un error: la
/// interfaz lo dice con un mensaje en lugar de tratarlo como fallo.
pub fn find(dir: &Path) -> Option<PathBuf> {
    let entries: Vec<(String, PathBuf)> = std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .filter(|entry| entry.file_type().map(|t| t.is_file()).unwrap_or(false))
        .map(|entry| {
            (
                entry.file_name().to_string_lossy().to_lowercase(),
                entry.path(),
            )
        })
        .collect();

    CANDIDATES.iter().find_map(|candidate| {
        entries
            .iter()
            .find(|(name, _)| name == candidate)
            .map(|(_, path)| path.clone())
    })
}

/// Convierte Markdown en HTML seguro.
///
/// Las imágenes no se dejan pasar: la CSP de la aplicación bloquea los
/// orígenes remotos y abrir el protocolo de ficheros para las rutas relativas
/// daría al webview acceso a cualquier ruta del disco. Una imagen con origen
/// `https` se convierte en un enlace, que el usuario puede abrir en su
/// navegador; cualquier otra se queda en su texto alternativo.
pub fn render(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_FOOTNOTES);

    let mut eventos = Vec::new();
    let mut en_imagen: Option<String> = None;

    for evento in Parser::new_ext(markdown, options) {
        match evento {
            Event::Start(Tag::Image { dest_url, .. }) => {
                en_imagen = Some(dest_url.to_string());
                eventos.push(Event::Html("".into()));
            }
            Event::End(TagEnd::Image) => {
                if let Some(origen) = en_imagen.take() {
                    eventos.push(Event::Html(imagen_como_texto(&origen).into()));
                }
            }
            // El texto dentro de una imagen es su alternativo, y se emite junto
            // al resto del reemplazo al cerrar la etiqueta.
            Event::Text(texto) if en_imagen.is_some() => {
                if let Some(origen) = en_imagen.take() {
                    en_imagen = Some(format!("{origen}\n{texto}"));
                }
            }
            // CommonMark deja pasar HTML literal, y aquí no interesa ni una
            // etiqueta: el saneador posterior admitiría varias, entre ellas
            // `<img>`, que es justo lo que no queremos cargar.
            Event::Html(_) | Event::InlineHtml(_) => {}
            otro => eventos.push(otro),
        }
    }

    let mut bruto = String::new();
    html::push_html(&mut bruto, eventos.into_iter());

    ammonia::Builder::default()
        .rm_tags(["img"])
        .url_schemes(["http", "https"].into_iter().collect())
        .link_rel(Some("noopener noreferrer"))
        .add_generic_attributes(["class"])
        .clean(&bruto)
        .to_string()
}

/// Reemplazo de una imagen: enlace si es remota por `https`, texto si no.
fn imagen_como_texto(origen_y_alt: &str) -> String {
    let (origen, alt) = origen_y_alt
        .split_once('\n')
        .unwrap_or((origen_y_alt, "imagen"));
    let alt = if alt.trim().is_empty() { "imagen" } else { alt };
    let alt = escapar(alt);

    if origen.starts_with("https://") {
        format!(
            "<a class=\"readme-imagen\" href=\"{}\">{alt}</a>",
            escapar(origen)
        )
    } else {
        format!("<span class=\"readme-imagen\">{alt}</span>")
    }
}

fn escapar(texto: &str) -> String {
    texto
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Lee y renderiza el README de un proyecto, si tiene alguno.
pub fn read(dir: &Path) -> Result<Option<ReadmePreview>> {
    let Some(path) = find(dir) else {
        return Ok(None);
    };

    let bytes = std::fs::read(&path)?;
    let truncated = bytes.len() > MAX_BYTES;
    // `from_utf8_lossy` sobre un corte a mitad de carácter deja un reemplazo
    // visible, que es preferible a rechazar el fichero entero.
    let markdown = String::from_utf8_lossy(&bytes[..bytes.len().min(MAX_BYTES)]);

    Ok(Some(ReadmePreview {
        file_name: path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default(),
        html: render(&markdown),
        truncated,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn escribir(dir: &Path, nombre: &str, contenido: &str) {
        std::fs::write(dir.join(nombre), contenido).unwrap();
    }

    #[test]
    fn render_drops_script_tags() {
        let html = render("Hola\n\n<script>alert(1)</script>\n");
        assert!(
            !html.contains("script"),
            "el HTML crudo no debe sobrevivir: {html}"
        );
        assert!(html.contains("Hola"));
    }

    #[test]
    fn render_drops_javascript_links() {
        let html = render("[pulsa](javascript:alert(1))");
        assert!(
            !html.contains("javascript:"),
            "esquema no permitido: {html}"
        );
        assert!(
            html.contains("pulsa"),
            "el texto del enlace se conserva: {html}"
        );
    }

    #[test]
    fn render_drops_event_handlers() {
        let html = render("<img src=x onerror=alert(1)>");
        assert!(!html.contains("onerror"));
        assert!(!html.contains("<img"));
    }

    #[test]
    fn render_keeps_http_and_https_links_with_safe_rel() {
        let html = render("[web](https://example.invalid/guia)");
        assert!(html.contains("https://example.invalid/guia"));
        assert!(
            html.contains("noopener"),
            "los enlaces salen con rel seguro: {html}"
        );
    }

    #[test]
    fn render_turns_a_remote_image_into_a_link() {
        let html = render("![captura](https://example.invalid/foto.png)");
        assert!(!html.contains("<img"), "ninguna imagen se carga: {html}");
        assert!(html.contains("https://example.invalid/foto.png"));
        assert!(
            html.contains("captura"),
            "se conserva el texto alternativo: {html}"
        );
    }

    #[test]
    fn render_turns_a_relative_image_into_plain_text() {
        let html = render("![diagrama](./docs/esquema.png)");
        assert!(!html.contains("<img"));
        assert!(
            !html.contains("esquema.png"),
            "una ruta local no se enlaza: {html}"
        );
        assert!(html.contains("diagrama"));
    }

    #[test]
    fn render_supports_tables_and_task_lists() {
        let html = render("| a | b |\n|---|---|\n| 1 | 2 |\n\n- [x] hecho\n");
        assert!(html.contains("<table>"));
        assert!(html.contains("checkbox") || html.contains("hecho"));
    }

    #[test]
    fn find_prefers_markdown_over_plain_text() {
        let tmp = tempfile::tempdir().unwrap();
        escribir(tmp.path(), "README.txt", "texto");
        escribir(tmp.path(), "README.md", "# markdown");

        let encontrado = find(tmp.path()).unwrap();
        assert_eq!(encontrado.file_name().unwrap(), "README.md");
    }

    #[test]
    fn find_ignores_the_casing_of_the_file_name() {
        let tmp = tempfile::tempdir().unwrap();
        escribir(tmp.path(), "readme.MD", "# grita");

        assert!(find(tmp.path()).is_some());
    }

    #[test]
    fn find_only_looks_at_the_root() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir(tmp.path().join("docs")).unwrap();
        escribir(&tmp.path().join("docs"), "README.md", "# profundo");

        assert!(find(tmp.path()).is_none(), "no se baja a subdirectorios");
    }

    #[test]
    fn read_returns_none_without_a_readme() {
        let tmp = tempfile::tempdir().unwrap();
        escribir(tmp.path(), "Cargo.toml", "[package]");

        assert!(read(tmp.path()).unwrap().is_none());
    }

    #[test]
    fn read_renders_the_file_it_finds() {
        let tmp = tempfile::tempdir().unwrap();
        escribir(tmp.path(), "README.md", "# Título\n\nUn párrafo.\n");

        let preview = read(tmp.path()).unwrap().unwrap();
        assert_eq!(preview.file_name, "README.md");
        assert!(preview.html.contains("<h1>"));
        assert!(preview.html.contains("Un párrafo."));
        assert!(!preview.truncated);
    }

    #[test]
    fn read_truncates_a_file_over_the_limit_and_says_so() {
        let tmp = tempfile::tempdir().unwrap();
        let enorme = "a".repeat(MAX_BYTES + 1_000);
        escribir(tmp.path(), "README.md", &enorme);

        let preview = read(tmp.path()).unwrap().unwrap();
        assert!(preview.truncated, "debe avisar de que se recortó");
        assert!(preview.html.len() <= MAX_BYTES + 256);
    }
}
