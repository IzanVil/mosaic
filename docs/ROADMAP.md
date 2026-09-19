# Roadmap

Estado de las fases de construcción de Mosaic. Se actualiza al cerrar cada fase.

---

## Fase 1 — Esqueleto y escaneo básico ✅

Objetivo: configurar rutas y ver una lista de proyectos detectados.

- [x] Proyecto Tauri 2 + Svelte 5 + TypeScript + Tailwind 4
- [x] SQLite con migración inicial (`001_initial.sql`)
- [x] `add_scan_path(path)`
- [x] `list_scan_paths()`
- [x] `scan_all_paths()`: recorrido con `walkdir` hasta profundidad 4, detección
      por ficheros marcadores, upsert por ruta y detección de lenguaje primario
- [x] `list_projects()`
- [x] Vista Dashboard con la lista de proyectos
- [x] Vista Ajustes para añadir y quitar rutas de escaneo
- [x] Logs con `tracing` a stdout, nivel configurable con `MOSAIC_LOG`
- [x] Tests de `core/` y de los repositorios, más un test de integración del
      flujo completo
- [x] CI en Linux: `fmt --check`, `clippy -D warnings`, `cargo test`

**Extras sobre lo planeado:** `remove_scan_path` y `set_scan_path_enabled`;
columnas `missing` y `last_seen_at` en `projects` para no borrar los proyectos
que desaparecen del disco; tope de entradas por escaneo con aviso.

---

## Fase 2 — Integración Git ✅

Objetivo: mostrar el estado Git en vivo de cada proyecto.

- [x] `core/git.rs` con `read_git_status(path) -> GitStatus` usando `git2`
- [x] Leer rama actual, ahead/behind, dirty y último commit
- [x] `refresh_git_status(project_id)` y `refresh_all_git_status()`
- [x] Persistir en `git_status_cache`
- [x] Refresco automático en segundo plano cada N minutos (por defecto 5)
- [x] `GitStatusBadge.svelte`: rama, indicador dirty, flechas ↑↓ y tooltip con el
      último mensaje de commit
- [x] Repositorios Git de prueba, creados con `git2` dentro de los propios tests
      en lugar de commitearlos en `tests/fixtures/`

**Extras sobre lo planeado:** `list_git_status()` y un store propio, para no
cambiar el contrato de `list_projects`; botón de refresco manual en la cabecera;
evento `git-status-refreshed` para que la interfaz se entere del refresco de
fondo; primer refresco a los 5 s del arranque, porque la caché de la sesión
anterior puede estar muy desactualizada.

---

## Fase 3 — Tarjetas visuales y grid ✅

Objetivo: la experiencia visual principal.

- [x] `ProjectCard.svelte` con acciones rápidas al pasar el ratón
- [x] `ProjectGrid.svelte`: grid responsivo con `auto-fill` y mínimo de 280 px
- [x] Abrir en IDE, en terminal y en el explorador de archivos
- [x] `open_in(kind, project_id)` y `commands/system.rs`
- [x] Detección automática de los IDEs y terminales disponibles
- [x] Marcar como favorito

Las etiquetas en la tarjeta quedan para la Fase 4, que es cuando existen.

**Extras sobre lo planeado:** selector de editor y terminal preferidos en los
ajustes, porque detectar varios y no poder elegir no sirve de nada; registro de
`last_opened_at` al abrir un proyecto, visible en la propia tarjeta; las acciones
se deshabilitan en los proyectos que ya no están en el disco.

---

## Fase 4 — Etiquetas, búsqueda y filtros ✅

Objetivo: organizar cientos de proyectos.

- [x] CRUD de etiquetas con paleta de 16 colores y hexadecimal libre
- [x] Asignar y quitar etiquetas a proyectos (N:M)
- [x] `TagPicker.svelte` con autocompletado y creación en línea
- [x] Búsqueda difusa en el frontend con `fuse.js`, sobre el nombre y la ruta
- [x] Filtros por etiqueta, lenguaje, estado Git, destacados y ausentes
- [x] Ordenación por nombre, última apertura, fecha de alta y última
      actualización, con dirección invertible
- [x] `Sidebar.svelte` con las vistas y las etiquetas, plegable
- [x] `TagManager.svelte`: renombrar, recolorear y borrar con confirmación
- [x] La última vista (búsqueda, filtros, orden, sidebar) se recuerda entre
      sesiones

**Decisiones:** las etiquetas de un filtro múltiple se combinan en OR y el resto
de filtros en AND; el filtrado vive entero en el frontend, sobre la lista que ya
está en memoria, y se revisará en la Fase 7 si con miles de proyectos duele.

**Extras sobre lo planeado:** migración `002` con un índice único sobre
`lower(name)`, porque el `UNIQUE` de la `001` es sensible a mayúsculas y
«Cliente» y «cliente» habrían convivido; `list_projects_with_tags`, que trae
proyectos, etiquetas y estado Git en dos consultas para que el tablero se pinte
con una sola llamada; `get_view_state` / `set_view_state` para persistir la
vista; y el estado Git del tablero, que ahora se mezcla por proyecto en lugar de
reemplazar la lista, para que el refresco automático no haga parpadear las
tarjetas.

---

## Fase 5 — Detalle de proyecto

Objetivo: vista en profundidad.

- [ ] `ProjectView.svelte` con cabecera y botón de copiar ruta
- [ ] Preview del README renderizado
- [ ] Historial de los últimos 10 commits
- [ ] Ramas locales y remotas
- [ ] Notas personales guardadas en `projects.notes`
- [ ] Etiquetas asignadas y acciones rápidas ampliadas

---

## Fase 6 — Pulido, atajos y ajustes avanzados

- [ ] Atajos: paleta de comandos, refrescar Git, abrir ajustes
- [ ] Tema claro, oscuro y del sistema
- [ ] IDE y terminal preferidos configurables
- [ ] Escanear al arrancar
- [ ] Exportar e importar la configuración en JSON
- [ ] Modos compacto y cómodo del grid

---

## Fase 7 — Empaquetado y distribución

- [x] Metadatos completos en `tauri.conf.json`
- [x] Iconos propios para todas las plataformas, generados con `tauri icon`
      a partir de `icons/source.png`
- [x] `release.yml`: compila en macOS, Windows y Linux, publica un borrador de
      release al etiquetar y deja artefactos descargables en las ejecuciones
      manuales
- [x] README con captura e instrucciones de instalación
- [x] Rutas verbatim de Windows (`\\?\C:\...`) normalizadas antes de guardarlas
- [ ] **Verificar los tres binarios en su sistema**: nadie ha ejecutado todavía
      el `.dmg` ni el `.msi`
- [ ] Firma de código en macOS y Windows, que requiere certificados de pago
- [ ] CI multiplataforma para los tests, no solo para los builds
- [ ] Validar la CSP en un build de release de cada plataforma
- [ ] `CONTRIBUTING.md`
