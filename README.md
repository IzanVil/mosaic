# Mosaic

Organizador visual de proyectos de código para escritorio. Descubre las carpetas
de proyectos que tienes en el disco y las presenta en un tablero con su estado de
Git, etiquetas y accesos directos para abrirlas en el IDE, la terminal o el
explorador de archivos.

Local-first: todo se guarda en una base de datos SQLite en tu máquina. Sin red,
sin cuentas, sin telemetría, sin IA.

## Estado

**En construcción.** Las fases 1, 2, 3 y 4 están terminadas:

- Esquema SQLite con migraciones.
- Escaneo de rutas configurables con detección de proyectos por ficheros
  marcadores (`.git`, `package.json`, `Cargo.toml`, `pyproject.toml`, `go.mod`,
  `pom.xml`, `build.gradle`, `composer.json`, `Gemfile`).
- Detección heurística del lenguaje primario.
- Interfaz para añadir y quitar rutas de escaneo, lanzar escaneos y ver en una
  lista los proyectos encontrados.
- Estado Git de cada proyecto: rama, cambios sin commitear, commits por delante y
  por detrás del upstream y último commit, con refresco automático en segundo
  plano.
- Tablero de tarjetas con acciones rápidas: abrir el proyecto en el editor, en la
  terminal o en el explorador de archivos, y destacar los que más uses.
- Etiquetas con color, asignables desde la propia tarjeta, con gestor para
  renombrarlas, recolorearlas y borrarlas.
- Búsqueda difusa por nombre y ruta, insensible a acentos y mayúsculas.
- Filtros por etiqueta, lenguaje, estado Git, destacados y proyectos ausentes,
  con ordenación por nombre, última apertura, alta o última actualización.
- La última vista se recuerda: al volver a abrir la aplicación están la misma
  búsqueda, los mismos filtros y el mismo orden.

Pendiente: la vista de detalle de cada proyecto (Fase 5).

## Documentación

- [Arquitectura](docs/ARCHITECTURE.md) — capas, escáner, modelo de datos y
  decisiones de diseño.
- [Roadmap](docs/ROADMAP.md) — qué está hecho y qué viene después.
- [Página de presentación](web/index.html) — un único fichero HTML, sin build ni
  dependencias: se abre en el navegador tal cual.

## Stack

Rust + Tauri 2 en el backend; Svelte 5 con TypeScript, Vite y Tailwind 4 en el
frontend. SQLite vía `rusqlite`.

## Compilar

Requisitos: Rust estable, Node 20+, pnpm y las dependencias de sistema de
Tauri 2. En Fedora:

```bash
sudo dnf install webkit2gtk4.1-devel libsoup3-devel gtk3-devel \
                 openssl-devel curl wget file
```

Para otras distribuciones, macOS y Windows, ver
[los requisitos de Tauri](https://tauri.app/start/prerequisites/).

```bash
pnpm install          # dependencias del frontend
pnpm tauri dev        # app en modo desarrollo
pnpm tauri build      # binario de release
```

Tests y comprobaciones del backend:

```bash
cd src-tauri
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

El nivel de logs se controla con la variable `MOSAIC_LOG` (por ejemplo
`MOSAIC_LOG=debug pnpm tauri dev`).

## Dónde guarda los datos

- Linux: `~/.local/share/mosaic/mosaic.db`
- macOS: `~/Library/Application Support/dev.izan.mosaic/mosaic.db`
- Windows: `%APPDATA%\izan\mosaic\data\mosaic.db`

Mosaic solo lee metadatos de los repositorios; nunca modifica su contenido.

## Licencia

MIT. Ver [LICENSE](LICENSE).
