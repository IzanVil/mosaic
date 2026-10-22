# Tests

Dónde está cada tipo de test y cómo se ejecuta.

| Tipo | Dónde | Cómo |
|---|---|---|
| Unitarios de Rust | Junto al código, en módulos `#[cfg(test)]` de `src-tauri/src/` | `cd src-tauri && cargo test` |
| Integración de Rust | `src-tauri/tests/`, que es donde Cargo los busca | `cd src-tauri && cargo test` |
| Frontend | `tests/unit/`, con vitest | `pnpm test` |
| De extremo a extremo | `tests/integration/`, con Playwright | `pnpm test:e2e` |

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

## De extremo a extremo

`tests/integration/recorridos.spec.ts` recorre la interfaz compilada como lo
haría alguien que la usa: añadir una carpeta y escanear, crear una etiqueta
desde una tarjeta y filtrar por ella, escribir notas y salir del detalle (con
el botón y con Escape) y abrir un proyecto desde la paleta.

No abren la app de Tauri: `vite preview` sirve la interfaz y
`tests/fixtures/backend.ts` hace de backend, con un estado en memoria que
sobrevive a recargar la página. Así corren en cualquier sistema y en segundos;
la parte de Rust ya la cubren sus propios tests.

Corren en WebKit, el motor de Tauri en Linux y macOS, y en Chromium, el de
WebView2 en Windows. La primera vez hay que descargar los navegadores:

```bash
pnpm exec playwright install webkit chromium
pnpm test:e2e
```

Playwright no da soporte a todas las distribuciones: en Fedora, por ejemplo,
Chromium funciona pero WebKit pide dos bibliotecas de Ubuntu. En el CI corren
los dos.

## En el CI

`ci.yml` ejecuta los tests de Rust y de vitest en Linux, macOS y Windows en
cada push a `main`, y los de extremo a extremo en Linux. En Windows se ejecuta
un test de Rust menos: `circular_symlink_does_not_hang_the_scan` crea enlaces
simbólicos al estilo Unix y solo corre en Linux y macOS.

`release.yml` añade una prueba de arranque: abre la app compilada en los tres
sistemas, comprueba que la ventana aparece y sigue viva, y guarda una captura
como artefacto.
