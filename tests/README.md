# Tests

Dónde está cada tipo de test y cómo se ejecuta.

| Tipo | Dónde | Cómo |
|---|---|---|
| Unitarios de Rust | Junto al código, en módulos `#[cfg(test)]` de `src-tauri/src/` | `cd src-tauri && cargo test` |
| Integración de Rust | `src-tauri/tests/`, que es donde Cargo los busca | `cd src-tauri && cargo test` |
| Frontend | `tests/unit/`, con vitest | `pnpm test` |
| De extremo a extremo | Todavía no hay ([#7](https://github.com/IzanVil/mosaic/issues/7)) | |

## Integración de Rust

- `scan_flow.rs`: registrar una ruta, escanear, listar, y el reescaneo que marca
  ausentes sin borrar.
- `git_flow.rs`: repositorios reales construidos en carpetas temporales,
  refrescados hasta la caché.
- `tags_flow.rs`: etiquetar proyectos descubiertos, reescanear y borrar la
  etiqueta comprobando que los proyectos siguen.

## Frontend

Los tests de `tests/unit/` prueban los stores y las funciones puras con la API
de Tauri simulada (`vi.mock`), sin abrir ninguna ventana. Cubren, entre otras
cosas, las reglas de filtrado y orden del tablero (`visibleProjects.test.ts`),
las notas del detalle, la vista guardada, los atajos y la paleta, la copia de
seguridad y el script que aplica el tema antes de pintar.

## Este directorio

`tests/integration/` y `tests/fixtures/` están reservados para los tests de
extremo a extremo del frontend y sus datos de prueba, que todavía no existen.

## En el CI

`ci.yml` ejecuta todo lo anterior en Linux, macOS y Windows en cada push a
`main`. En Windows se ejecuta un test de Rust menos:
`circular_symlink_does_not_hang_the_scan` crea enlaces simbólicos al estilo
Unix y solo corre en Linux y macOS.

`release.yml` añade una prueba de arranque: abre la app compilada en los tres
sistemas, comprueba que la ventana aparece y sigue viva, y guarda una captura
como artefacto.
