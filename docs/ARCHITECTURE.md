# Arquitectura

Mosaic es una aplicación de escritorio Tauri 2: un backend en Rust que hace todo
el trabajo con el sistema de ficheros y la base de datos, y un frontend Svelte 5
que solo pinta y recoge interacciones. No hay servidor, ni red, ni estado
compartido fuera de la máquina del usuario.

## Capas

```
┌──────────────────────────────────────────────┐
│  Frontend (Svelte 5 + TypeScript)            │
│                                              │
│  views/ ──► stores/ ──► api/ ──► invoke()    │
│  components/                                 │
└──────────────────────────────┬───────────────┘
                               │  IPC de Tauri
┌──────────────────────────────▼───────────────┐
│  commands/   validación y conversión de error│
├──────────────────────────────────────────────┤
│  core/       lógica de dominio, sin Tauri    │
├──────────────────────────────────────────────┤
│  db/         SQLite: conexión, migraciones,  │
│              repositorios (todo el SQL)      │
└──────────────────────────────────────────────┘
```

Las reglas de dependencia:

- `commands/` no contiene lógica de negocio. Valida la entrada, mueve el trabajo
  bloqueante fuera del hilo de la interfaz y convierte `AppError` en `String`,
  que es lo que entiende el puente de Tauri.
- `core/` no conoce Tauri. Es donde vive el escáner y el modelo de dominio, y es
  lo que cubren los tests unitarios.
- `db/` concentra todo el SQL. Ningún otro módulo escribe consultas.
- En el frontend, **ningún componente llama a `invoke()`**. Solo lo hace
  `lib/api/`; los componentes hablan con los stores y los stores con la API.

## Backend

### `errors.rs`

`AppError` con `thiserror`: `Db`, `Migration`, `Io`, `InvalidPath`, `NotFound`,
`Internal`. Implementa `From<AppError> for String` para la frontera con el
frontend. No se usa `unwrap()` fuera de los tests.

### `db/`

- **`connection.rs`** — `Db`, que envuelve una `Connection` en un `Mutex`. Mosaic
  es un proceso con un único escritor, así que un pool sería complejidad sin
  beneficio; se reevaluará si las fases 4-5 introducen lecturas concurrentes
  pesadas. Al abrir se aplican `foreign_keys = ON`, `journal_mode = WAL` y
  `busy_timeout = 5 s`. La ruta por defecto la decide `directories`.
- **`migrations.rs`** — migraciones embebidas con `include_str!` y aplicadas con
  `rusqlite_migration` al abrir la base de datos. Para añadir una, se crea
  `migrations/NNN_nombre.sql` y se añade al final del vector; el orden existente
  nunca se reordena.
- **`repositories/`** — `projects` y `scan_paths`. Devuelven structs de dominio,
  no filas.

### `core/scanner.rs`

El escáner recorre cada raíz con `walkdir` y `follow_links(false)`, lo que lo
hace inmune a los ciclos de enlaces simbólicos. La profundidad cuenta la raíz
como nivel 0. Si el recorrido supera `max_entries_per_scan` entradas se aborta y
el resumen lo marca como truncado.

Una carpeta es **candidata** si contiene alguno de los marcadores: `.git`,
`package.json`, `Cargo.toml`, `pyproject.toml`, `go.mod`, `pom.xml`,
`build.gradle`, `composer.json`, `Gemfile`.

Una candidata se **registra** como proyecto cuando:

1. contiene su propio `.git`, aunque un ancestro ya esté registrado; **o**
2. ningún ancestro suyo está registrado ya como proyecto.

De ahí que un monorepo (`.git` en la raíz, `package.json` en `packages/*`) dé una
sola tarjeta, mientras que dos repositorios anidados den dos.

### Detección del lenguaje primario

Heurística, en orden:

1. **Manifiesto inequívoco.** `Cargo.toml` → Rust, `go.mod` → Go,
   `pyproject.toml` → Python, `build.gradle.kts` → Kotlin,
   `build.gradle`/`pom.xml` → Java, `composer.json` → PHP, `Gemfile` → Ruby.
2. **`package.json`**, que por sí solo no distingue TypeScript de JavaScript, de
   Svelte ni de Vue: se resuelve contando extensiones y quedándose con el más
   frecuente de esa familia; si no hay ninguno, JavaScript.
3. **Conteo de extensiones** hasta dos niveles, ignorando los directorios
   excluidos. Se descartan Markdown, JSON, YAML y TOML: aparecen en casi todos
   los repositorios y no dicen nada del lenguaje.
4. Si nada decide, `NULL`.

Los empates se rompen alfabéticamente para que el resultado sea determinista.

### `core/git.rs`

Mosaic **solo lee** de los repositorios y nunca habla con la red: `git2` se
compila sin las features `ssh` ni `https`, y con `vendored-libgit2`, de modo que
no depende de que haya una libgit2 en el sistema.

`ahead` y `behind` se calculan con `graph_ahead_behind` entre la rama local y su
upstream, **sin hacer `fetch`**. Son `NULL` si la rama no tiene upstream. Los
números pueden estar desfasados respecto al remoto real, exactamente igual que
`git status` sin conexión.

`is_dirty` usa el mismo criterio que `git status`: ficheros modificados más
ficheros sin seguir, respetando `.gitignore`. No se entra a recorrer los
directorios sin seguir, porque saber que existen basta y contarlos puede ser
carísimo.

Casos que el lector contempla: rama sin nacer (repositorio recién inicializado,
del que sí se sabe el nombre de la rama), HEAD separado (hay commit pero no
rama) y repositorios bare (nunca sucios). `Repository::open` no busca hacia
arriba, así que un repositorio anidado devuelve su propio estado y no el de su
contenedor.

El refresco masivo es tolerante a fallos: un repositorio ilegible no aborta el
proceso, se registra el aviso y **se borra su entrada de la caché** para no
mostrar datos rancios.

### Refresco en segundo plano

Una tarea lanzada en el `setup` de Tauri refresca el estado Git cada N minutos
(5 por defecto, configurable; `0` lo desactiva). El intervalo se relee en cada
vuelta, así que un cambio en los ajustes surte efecto sin reiniciar. Hace un
primer refresco a los 5 segundos del arranque porque la caché que quedó de la
sesión anterior puede estar muy desactualizada.

Al terminar emite el evento `git-status-refreshed`, que el frontend escucha para
recargar la caché. La caché es reconstruible: si se borra, el siguiente refresco
la repuebla.

## Modelo de datos

Seis tablas: `scan_paths`, `projects`, `tags`, `project_tags`,
`git_status_cache` y `settings` (clave-valor). Todas las marcas de tiempo son
enteros en segundos desde el epoch Unix; todos los booleanos son enteros 0/1.

### Semántica de las marcas de tiempo de `projects`

Es la parte con más reglas y la que más tests tiene:

| Campo | Cuándo cambia |
|---|---|
| `created_at` | Solo en la inserción. Nunca se modifica. |
| `updated_at` | Solo si cambian metadatos propios: nombre, lenguaje primario, condición de repositorio Git o notas. |
| `last_seen_at` | Cada vez que el escáner ve el proyecto en disco. |
| `missing` | `0` cuando el escáner lo ve, `1` cuando deja de verlo. |

Un proyecto que desaparece del disco **se marca, no se borra**: borrarlo haría
perder las etiquetas y las notas que el usuario le haya puesto.

Para decidir qué marcar como ausente se comparan los identificadores vistos en el
escaneo, no las marcas de tiempo: `last_seen_at` tiene resolución de un segundo y
dos escaneos consecutivos pueden compartir instante. Además solo se consideran
los proyectos que cuelgan de alguna raíz recién escaneada, de forma que los que
viven bajo una ruta deshabilitada o eliminada conservan su estado.

## Frontend

Svelte 5 con runas dentro de los componentes (`$state`, `$derived`, `$props`,
snippets) y stores clásicos de `svelte/store` para el estado global. Los tipos de
`lib/types/index.ts` son un espejo literal de los structs de Rust: los campos van
en `snake_case` porque serde los serializa con sus nombres tal cual, que a su vez
coinciden con las columnas SQL. Cualquier cambio en un struct exige el cambio
equivalente aquí.

Tailwind 4 con configuración CSS-first: no hay `tailwind.config.js`. Los colores
se definen como tokens en `@theme` dentro de `src/app.css` y se redefinen bajo
`.dark`, de modo que las utilidades de Tailwind cambian de tema sin recompilar.

## Seguridad

- Mosaic **solo lee** de los repositorios. Nunca escribe en ellos.
- Las capacidades de Tauri están al mínimo: `core:default` y `dialog:allow-open`.
- La CSP de producción es `default-src 'self'` con las excepciones justas para
  imágenes y para los estilos que Svelte inyecta en runtime. En desarrollo la
  página la sirve Vite desde un origen externo, sobre el que Tauri no inyecta la
  cabecera, así que la política solo se ejerce de verdad en los builds de
  release.
- Cero red, cero telemetría, cero cuentas, cero IA.

## Tests

- **Unitarios**: en módulos `#[cfg(test)]` junto al código, con árboles de
  ficheros temporales (`tempfile`). Cubren el escáner (marcadores, profundidad,
  exclusiones, symlinks circulares, anidamiento), la detección de lenguaje y la
  semántica de los repositorios.
- **Integración**: `src-tauri/tests/`, que es donde Cargo los busca.
  `scan_flow.rs` ejercita registrar ruta → escanear → listar, y el reescaneo que
  marca ausentes sin borrar.
- El `tests/` de la raíz queda reservado para los end-to-end del frontend y para
  los repositorios Git de prueba que necesitará la Fase 2.
