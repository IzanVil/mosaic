//! Lectura y renderizado del README de un proyecto.
//!
//! El README es de terceros. Su HTML pasa, porque muchos montan así la portada
//! o las tablas, pero solo por la lista blanca de `ammonia`: sin scripts,
//! estilos, iframes ni manejadores. Ninguna imagen se carga, y lo que se quita
//! se cuenta en [`ReadmePreview`] para que la interfaz lo diga.
//!
//! Se renderiza aquí y no en el webview para que el Markdown nunca se procese
//! dentro de la ventana y el saneado lo cubra `cargo test`.

use std::path::{Path, PathBuf};

use pulldown_cmark::{html, Event, Options, Parser, Tag, TagEnd};
use serde::{Deserialize, Serialize};

use crate::errors::Result;

/// Tope de tamaño del fichero. Por encima se recorta y se avisa.
pub const MAX_BYTES: usize = 512 * 1024;

/// Nombres de README aceptados, por preferencia, sin distinguir mayúsculas y solo en la raíz.
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
    /// Imágenes que no se cargan (del Markdown o del HTML), sin las insignias.
    pub images_omitted: u32,
    /// Insignias (licencia, versión, CI…) que no se cargan.
    pub badges_omitted: u32,
    /// Vídeos, audios, iframes, SVG y otros elementos que no se muestran.
    pub media_omitted: u32,
}

/// HTML ya saneado y lo que se quedó fuera por el camino.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rendered {
    /// HTML saneado, listo para insertar.
    pub html: String,
    /// Imágenes que no se cargan, sin las insignias.
    pub images_omitted: u32,
    /// Insignias que no se cargan.
    pub badges_omitted: u32,
    /// Otros elementos que no se muestran.
    pub media_omitted: u32,
}

/// Servicios cuyas imágenes son siempre insignias.
const BADGE_HOSTS: &[&str] = &[
    "img.shields.io",
    "shields.io",
    "badgen.net",
    "badge.fury.io",
    "badges.gitter.im",
    "ci.appveyor.com",
    "api.netlify.com",
    "travis-ci.org",
    "travis-ci.com",
];

/// Si una imagen es una insignia, que se cuenta aparte: doce insignias no son doce capturas.
pub fn is_badge(src: &str) -> bool {
    let src = src.trim().to_ascii_lowercase();
    let Some((_, resto)) = src.split_once("://") else {
        return false;
    };
    let resto = resto.split(['?', '#']).next().unwrap_or("");
    let (host, ruta) = resto.split_once('/').unwrap_or((resto, ""));
    if BADGE_HOSTS.contains(&host) || host.starts_with("badge") {
        return true;
    }
    // Así las sirven GitHub Actions, GitLab, Codecov o Coveralls.
    ruta.split('/')
        .any(|tramo| tramo == "badge" || tramo == "badges" || tramo.starts_with("badge."))
}

/// Etiquetas que no se muestran y se cuentan como omitidas; `script` y `style` no se cuentan.
const MEDIA_TAGS: &[&str] = &[
    "video", "audio", "iframe", "object", "embed", "svg", "canvas",
];

/// Busca el README en la raíz de `dir`; no tenerlo es normal, no un error.
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
pub fn render(markdown: &str) -> String {
    render_with_report(markdown).html
}

/// Como [`render`], contando además las imágenes y los elementos que se quedan fuera.
pub fn render_with_report(markdown: &str) -> Rendered {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_FOOTNOTES);

    let mut eventos = Vec::new();
    let mut en_imagen: Option<String> = None;
    let mut imagenes = 0u32;
    let mut insignias = 0u32;
    let mut medios = 0u32;

    for evento in Parser::new_ext(markdown, options) {
        match evento {
            // Las imágenes no se cargan: la CSP bloquea lo remoto, y abrir el protocolo de
            // ficheros para las relativas daría al webview acceso a todo el disco.
            Event::Start(Tag::Image { dest_url, .. }) => {
                if is_badge(&dest_url) {
                    insignias += 1;
                } else {
                    imagenes += 1;
                }
                en_imagen = Some(dest_url.to_string());
                eventos.push(Event::Html("".into()));
            }
            Event::End(TagEnd::Image) => {
                if let Some(origen) = en_imagen.take() {
                    eventos.push(Event::Html(imagen_como_texto(&origen).into()));
                }
            }
            // Dentro de una imagen, el texto es su alternativo: va en el reemplazo al cerrarla.
            Event::Text(texto) if en_imagen.is_some() => {
                if let Some(origen) = en_imagen.take() {
                    en_imagen = Some(format!("{origen}\n{texto}"));
                }
            }
            // El saneador no sabe cambiar un `<img>` por su alternativo: se hace antes, y
            // aquí se cuenta lo que no se verá. Lo demás lo filtra la lista blanca.
            Event::Html(fragmento) | Event::InlineHtml(fragmento) => {
                medios += contar_medios(&fragmento);
                let limpio = sustituir_imagenes_html(&fragmento, &mut imagenes, &mut insignias);
                eventos.push(Event::Html(limpio.into()));
            }
            otro => eventos.push(otro),
        }
    }

    let mut bruto = String::new();
    html::push_html(&mut bruto, eventos.into_iter());

    let html = ammonia::Builder::default()
        .rm_tags(["img"])
        .add_tags(["details", "summary"])
        .add_tag_attributes("details", ["open"])
        .add_tag_attributes("table", ["align"])
        .add_tag_attributes("td", ["align"])
        .add_tag_attributes("th", ["align"])
        .add_tag_attributes("div", ["align"])
        .add_tag_attributes("p", ["align"])
        .add_tag_attributes("h1", ["align"])
        .add_tag_attributes("h2", ["align"])
        .add_tag_attributes("h3", ["align"])
        .url_schemes(["http", "https"].into_iter().collect())
        .link_rel(Some("noopener noreferrer"))
        .add_generic_attributes(["class"])
        .clean(&bruto)
        .to_string();

    Rendered {
        html,
        images_omitted: imagenes,
        badges_omitted: insignias,
        media_omitted: medios,
    }
}

/// Cuenta las etiquetas de [`MEDIA_TAGS`] que abren en un fragmento de HTML.
fn contar_medios(fragmento: &str) -> u32 {
    let minusculas = fragmento.to_ascii_lowercase();
    MEDIA_TAGS
        .iter()
        .map(|etiqueta| {
            let apertura = format!("<{etiqueta}");
            minusculas
                .match_indices(&apertura)
                .filter(|(i, _)| {
                    // `<svg` sí, `<svgfoo` no: tras el nombre viene un espacio,
                    // una barra o el cierre.
                    minusculas[i + apertura.len()..]
                        .chars()
                        .next()
                        .is_none_or(|c| c.is_ascii_whitespace() || c == '>' || c == '/')
                })
                .count() as u32
        })
        .sum()
}

/// Cambia cada `<img>` de un fragmento por su alternativo y cuenta imágenes e insignias.
fn sustituir_imagenes_html(fragmento: &str, imagenes: &mut u32, insignias: &mut u32) -> String {
    // Es presentación, no seguridad: lo que se escape de aquí lo quita `ammonia` después.
    let minusculas = fragmento.to_ascii_lowercase();
    let mut salida = String::with_capacity(fragmento.len());
    let mut resto = 0;

    while let Some(rel) = minusculas[resto..].find("<img") {
        let inicio = resto + rel;
        let tras_nombre = minusculas[inicio + 4..].chars().next();
        if !tras_nombre.is_none_or(|c| c.is_ascii_whitespace() || c == '>' || c == '/') {
            salida.push_str(&fragmento[resto..inicio + 4]);
            resto = inicio + 4;
            continue;
        }
        let Some(fin) = fin_de_etiqueta(&fragmento[inicio..]) else {
            break;
        };
        let etiqueta = &fragmento[inicio..inicio + fin];
        let alt = atributo(etiqueta, "alt").filter(|a| !a.trim().is_empty());
        salida.push_str(&fragmento[resto..inicio]);
        salida.push_str(&format!(
            "<span class=\"readme-imagen\">{}</span>",
            escapar(alt.as_deref().unwrap_or("imagen"))
        ));
        if atributo(etiqueta, "src").is_some_and(|src| is_badge(&src)) {
            *insignias += 1;
        } else {
            *imagenes += 1;
        }
        resto = inicio + fin;
    }
    salida.push_str(&fragmento[resto..]);
    salida
}

/// Longitud de una etiqueta hasta su `>`, saltando los que van entre comillas.
fn fin_de_etiqueta(texto: &str) -> Option<usize> {
    let mut comillas: Option<char> = None;
    for (i, c) in texto.char_indices() {
        match (comillas, c) {
            (Some(q), _) if c == q => comillas = None,
            (Some(_), _) => {}
            (None, '"' | '\'') => comillas = Some(c),
            (None, '>') => return Some(i + 1),
            _ => {}
        }
    }
    None
}

/// Valor de un atributo dentro de una etiqueta, entre comillas o sin ellas.
fn atributo(etiqueta: &str, nombre: &str) -> Option<String> {
    let minusculas = etiqueta.to_ascii_lowercase();
    let mut desde = 0;
    while let Some(rel) = minusculas[desde..].find(nombre) {
        let i = desde + rel;
        let antes = minusculas[..i].chars().last();
        let despues = &etiqueta[i + nombre.len()..];
        let tras_espacios = despues.trim_start();
        if antes.is_some_and(|c| c.is_ascii_whitespace()) && tras_espacios.starts_with('=') {
            let valor = tras_espacios[1..].trim_start();
            return Some(match valor.chars().next() {
                Some(q @ ('"' | '\'')) => valor[1..].split(q).next().unwrap_or("").to_string(),
                _ => valor
                    .split(|c: char| c.is_ascii_whitespace() || c == '>' || c == '/')
                    .next()
                    .unwrap_or("")
                    .to_string(),
            });
        }
        desde = i + nombre.len();
    }
    None
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
    // Cortar a mitad de un carácter deja un reemplazo visible: mejor que rechazar el fichero.
    let markdown = String::from_utf8_lossy(&bytes[..bytes.len().min(MAX_BYTES)]);

    let rendered = render_with_report(&markdown);
    Ok(Some(ReadmePreview {
        file_name: path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default(),
        html: rendered.html,
        truncated,
        images_omitted: rendered.images_omitted,
        badges_omitted: rendered.badges_omitted,
        media_omitted: rendered.media_omitted,
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
            "un script nunca sobrevive: {html}"
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
    fn render_keeps_whitelisted_html_like_a_centered_header() {
        let html = render(
            "<div align=\"center\">\n<h1>Mosaic</h1>\n<p><b>Todo en un tablero.</b></p>\n</div>\n",
        );
        assert!(html.contains("<div align=\"center\">"), "{html}");
        assert!(html.contains("<h1>Mosaic</h1>"), "{html}");
        assert!(html.contains("<b>Todo en un tablero.</b>"), "{html}");
    }

    #[test]
    fn render_keeps_html_tables_and_collapsible_blocks() {
        let html = render(
            "<table><tr><td align=\"center\">uno<sub>2</sub></td></tr></table>\n\n<details><summary>Más</summary>\n\ndentro\n\n</details>\n",
        );
        assert!(html.contains("<td align=\"center\">"), "{html}");
        assert!(html.contains("<sub>2</sub>"), "{html}");
        assert!(
            html.contains("<details>") && html.contains("<summary>Más</summary>"),
            "{html}"
        );
    }

    #[test]
    fn render_strips_dangerous_parts_of_whitelisted_html() {
        let html = render(
            "<div align=\"center\" style=\"position:fixed\" onclick=\"alert(1)\"><a href=\"javascript:alert(1)\">x</a></div>\n\n<iframe src=\"https://example.invalid\"></iframe>\n",
        );
        assert!(!html.contains("style="), "sin estilos en línea: {html}");
        assert!(!html.contains("onclick"), "sin manejadores: {html}");
        assert!(!html.contains("javascript:"), "sin javascript: {html}");
        assert!(!html.contains("<iframe"), "sin iframes: {html}");
    }

    #[test]
    fn render_turns_html_images_into_their_alt_text() {
        let html = render(
            "<p><a href=\"https://example.invalid/licencia\"><img src=\"https://img.example.invalid/insignia.svg\" alt=\"Licencia Apache 2.0\"></a></p>\n",
        );
        assert!(!html.contains("<img"), "ninguna imagen se carga: {html}");
        assert!(
            !html.contains("insignia.svg"),
            "la imagen no se enlaza: {html}"
        );
        assert!(
            html.contains("Licencia Apache 2.0")
                && html.contains("https://example.invalid/licencia"),
            "el enlace queda con el texto alternativo: {html}"
        );
    }

    #[test]
    fn render_counts_every_image_and_media_element_it_drops() {
        let markdown = "![uno](https://example.invalid/a.png)\n\n<img src=\"b.png\" alt=\"dos\">\n\n<picture><source srcset=\"c.webp\"><img src=\"c.png\"></picture>\n\n<video src=\"d.mp4\"></video>\n\n<iframe src=\"https://example.invalid\"></iframe>\n\n<svgfoo>no cuenta</svgfoo>\n";
        let rendered = render_with_report(markdown);
        assert_eq!(rendered.images_omitted, 3, "{rendered:?}");
        assert_eq!(rendered.media_omitted, 2, "{rendered:?}");
    }

    #[test]
    fn render_reports_nothing_omitted_for_plain_markdown() {
        let rendered = render_with_report("# Hola\n\nSin imágenes ni vídeos.\n");
        assert_eq!(
            (
                rendered.images_omitted,
                rendered.badges_omitted,
                rendered.media_omitted
            ),
            (0, 0, 0)
        );
    }

    #[test]
    fn is_badge_recognises_the_usual_badge_services() {
        for src in [
            "https://img.shields.io/badge/licencia-Apache%202.0-2c8c8c",
            "https://badgen.net/npm/v/mosaic",
            "https://github.com/IzanVil/mosaic/actions/workflows/ci.yml/badge.svg",
            "https://github.com/IzanVil/mosaic/actions/workflows/ci.yml/badge.svg?branch=main",
            "https://codecov.io/gh/IzanVil/mosaic/graph/badge.svg?token=x",
            "https://gitlab.com/grupo/proyecto/badges/main/pipeline.svg",
            "HTTPS://IMG.SHIELDS.IO/github/stars/IzanVil/mosaic",
        ] {
            assert!(is_badge(src), "debería ser insignia: {src}");
        }
    }

    #[test]
    fn is_badge_leaves_screenshots_and_icons_alone() {
        for src in [
            "docs/img/portada-oscura.png",
            "src-tauri/icons/128x128@2x.png",
            "https://example.invalid/captura.png",
            "https://raw.githubusercontent.com/IzanVil/mosaic/main/docs/img/badgeless.png",
            "https://example.invalid/badger/foto.png",
            "",
        ] {
            assert!(!is_badge(src), "no debería ser insignia: {src}");
        }
    }

    #[test]
    fn render_counts_badges_apart_from_images() {
        // Lo que tiene el README de Mosaic: el icono, la portada y capturas,
        // más las insignias de licencia y de descarga.
        let markdown = "<p><img src=\"src-tauri/icons/128x128@2x.png\" alt=\"Icono\"></p>\n\n<p><img src=\"https://img.shields.io/badge/licencia-Apache-2c8c8c\" alt=\"Licencia\">\n<a href=\"https://example.invalid\"><img src=\"https://img.shields.io/badge/Descargar-0.2.0-2c8c8c\" alt=\"Descargar\"></a></p>\n\n![CI](https://github.com/IzanVil/mosaic/actions/workflows/ci.yml/badge.svg)\n\n![Captura](docs/img/paleta.png)\n";
        let rendered = render_with_report(markdown);
        assert_eq!(rendered.images_omitted, 2, "{rendered:?}");
        assert_eq!(rendered.badges_omitted, 3, "{rendered:?}");
        assert!(
            rendered.html.contains("Descargar"),
            "la insignia deja su texto: {}",
            rendered.html
        );
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
