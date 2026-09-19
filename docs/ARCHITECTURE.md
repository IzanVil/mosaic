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

### `core/launcher.rs`

Abrir un proyecto significa lanzar un proceso externo, así que **nunca se
construye una línea de shell**: los procesos se lanzan con `Command` pasando la
ruta como un argumento suelto. Una carpeta llamada `raro; rm -rf ~/ #` llega
entera como un único argumento, y hay un test que lo fija.

Los IDEs y los terminales se detectan buscando ejecutables en el `PATH`, contra
una tabla ordenada por preferencia. El explorador de archivos no sale de ninguna
tabla: lo resuelve el sistema a través del crate `open`.

Cada terminal recibe el directorio con su propia bandera (`--directory` en kitty,
`--workdir` en Konsole, `--cwd` en WezTerm…), además de lanzarse con
`current_dir`. Lo segundo solo no basta: varios terminales hablan con un servidor
que ya existe y no heredarían el directorio de trabajo.

La elección de aplicación es "la preferida si sigue instalada, si no la primera
disponible". Si el usuario eligió un editor y luego lo desinstaló, se avisa por el
log y se usa otro en vez de fallar.

`last_opened_at` solo se registra si el lanzamiento tuvo éxito. Ni eso ni la marca
de favorito tocan `updated_at`: son preferencias de presentación, no metadatos que
describan la carpeta.

## Modelo de datos

Seis tablas: `scan_paths`, `projects`, `tags`, `project_tags`,
`git_status_cache` y `settings` (clave-valor). Todas las marcas de tiempo son
enteros en segundos desde el epoch Unix; todos los booleanos son enteros 0/1.

### Semántica de las marcas de tiempo de `projects`

Es la parte con más reglas y la que más tests tiene:

| Campo | Cuándo cambia |
|---|---|
| `created_at` | Solo en la inserción. Nunca se modifica. |
| `updated_at` | Hoy, solo si cambia el nombre, el lenguaje primario o la condición de repositorio Git. Nada más. |
| `last_seen_at` | Cada vez que el escáner ve el proyecto en disco. |
| `missing` | `0` cuando el escáner lo ve, `1` cuando deja de verlo. |

`notes` queda fuera de esa comparación porque ningún comando las escribe
todavía. La Fase 5, que introduce la edición, tiene que añadirlas a la
comparación y actualizar esta tabla **en el mismo commit**: esta página describe
lo que el código hace, no lo que se pretende que haga.

Un proyecto que desaparece del disco **se marca, no se borra**: borrarlo haría
perder las etiquetas y las notas que el usuario le haya puesto.

Para decidir qué marcar como ausente se comparan los identificadores vistos en el
escaneo, no las marcas de tiempo: `last_seen_at` tiene resolución de un segundo y
dos escaneos consecutivos pueden compartir instante. Además solo se consideran
los proyectos que cuelgan de alguna raíz recién escaneada, de forma que los que
viven bajo una ruta deshabilitada o eliminada conservan su estado.

### Etiquetas

`tags` y `project_tags` existen desde la migración `001`. La `002` añade un
índice único sobre `lower(name)`: el `UNIQUE` de la `001` usa la colación BINARY
de SQLite, así que «Cliente» y «cliente» habrían sido dos etiquetas distintas. La
validación en `core/tag.rs` repite la comprobación antes de insertar, solo para
poder devolver un mensaje legible en vez del error crudo de la base de datos.

El nombre se guarda tal y como lo escribió el usuario, recortado, de 1 a 32
caracteres y sin caracteres de control. El color se normaliza siempre a
`#RRGGBB` en mayúsculas, de modo que la base de datos no acumula dos formas del
mismo color.

Asignar o quitar una etiqueta **no toca la tabla `projects`**: la relación es
externa a ella, así que `updated_at`, `last_seen_at` y `missing` se quedan
quietos. Lo fija el test `assigning_tags_never_touches_the_projects_table`.

Borrar una etiqueta retira sus asignaciones por `ON DELETE CASCADE` y no toca los
proyectos. La interfaz confirma antes, diciendo a cuántos proyectos afecta.

`list_projects_with_tags` alimenta el tablero entero con **dos** consultas: los
proyectos con su caché Git por `LEFT JOIN`, y todas las asignaciones de etiquetas
en bloque. Con 500 proyectos, una consulta por tarjeta serían 501 viajes a
SQLite.

### Estado de la vista

La última búsqueda, los filtros, la ordenación y si el panel lateral está plegado
se guardan como un único JSON en la clave `ui.view_state` de `settings`, con 500
ms de debounce. El backend no interpreta ese JSON: solo comprueba que lo sea y
que no pase de 64 KiB, y si lo guardado está corrupto devuelve `null` para que la
aplicación arranque con los filtros por defecto. No se exponen comandos
genéricos de ajustes, que permitirían al frontend escribir claves del escáner o
de Git sin pasar por su validación.

## Frontend

Svelte 5 con runas dentro de los componentes (`$state`, `$derived`, `$props`,
snippets) y stores clásicos de `svelte/store` para el estado global. Los tipos de
`lib/types/index.ts` son un espejo literal de los structs de Rust: los campos van
en `snake_case` porque serde los serializa con sus nombres tal cual, que a su vez
coinciden con las columnas SQL. Cualquier cambio en un struct exige el cambio
equivalente aquí.

El estado del tablero está partido en dos: `stores/filters.ts` guarda la
intención del usuario y `stores/projects.ts` la aplica en un único store
derivado, `visibleProjects`. Ningún componente filtra por su cuenta; si lo
hiciera, acabaría discrepando del contador de resultados de la barra de
búsqueda. La separación también evita el bucle de escribir filtros desde un
efecto que lee la lista ya filtrada: los filtros solo se cambian desde
manejadores de eventos.

La búsqueda difusa es `fuse.js` sobre el nombre y la ruta, con el texto
normalizado —sin acentos y en minúsculas— en los dos lados de la comparación. El
índice se reconstruye solo cuando cambia la lista de proyectos, no en cada
pulsación. El filtrado es del frontend a propósito: la lista completa ya está en
memoria y con cientos de proyectos la respuesta es instantánea. Se reevaluará en
la Fase 7 si con miles se nota.

Las etiquetas de un filtro múltiple se combinan en OR entre sí, igual que los
lenguajes; el resto de filtros se combinan en AND. Borrar una etiqueta la saca
también de los filtros activos: si no, el tablero aparecería vacío sin nada que
lo explicara en el siguiente arranque. El conteo por etiqueta del panel lateral
se ajusta al asignar y al desasignar, sin recalcularlo entero.

El estado Git que llega con el refresco automático se **mezcla** por
`project_id` en los proyectos que ya están en memoria en lugar de reemplazar la
lista: con cientos de proyectos, reescribir el array completo cada cinco minutos
haría parpadear el tablero.

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
  marca ausentes sin borrar. `git_flow.rs` construye repositorios reales y los
  refresca hasta la caché. `tags_flow.rs` etiqueta proyectos descubiertos,
  reescanea y borra la etiqueta comprobando que los proyectos siguen.
- El `tests/` de la raíz queda reservado para los end-to-end del frontend.
