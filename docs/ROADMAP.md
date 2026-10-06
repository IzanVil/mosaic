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

## Fase 5 — Detalle de proyecto ✅ HECHA

Objetivo: vista en profundidad.

**En curso desde el 2026-10-03**, después de cerrar el rediseño. Estuvo
congelada del 2026-09-20 al 2026-10-03 mientras se usaba la aplicación; de
entonces son las dos piezas de backend que ya tenían tests.

Terminado:

- [x] `core/readme.rs`: localiza el README en la raíz y lo convierte a HTML
      seguro. Ninguna imagen se carga: las de origen `https` pasan a enlace y
      el resto a texto alternativo. 13 tests. (El HTML literal se descartaba
      entero; desde el 2026-10-06 pasa por una lista blanca, ver abajo.)
- [x] `core/git.rs`: `read_history` y `read_branches`, sin red. 7 tests.

Hecho el 2026-10-03, en la Sesión C:

- [x] Los cinco comandos: `get_project_readme`, `get_project_history` (los
      diez últimos commits), `get_project_branches`, `set_project_notes` y
      `open_external`
- [x] `open_external` con lista blanca de `http` y `https`, parseada con el
      crate `url`, que sigue el estándar de los navegadores. 4 tests
- [x] Notas en `projects.notes`: vacías se guardan como `NULL`, el mismo texto
      no se reescribe, y mueven `updated_at` solo cuando cambian. La
      documentación de `projects.rs` y de `ARCHITECTURE.md` se corrigió en el
      mismo commit. 7 tests
- [x] Plugin del portapapeles: crate en la serie 2 (2.3.3), paquete de npm
      fijado a la misma versión menor y solo permiso de escritura
- [x] `stores/navigation.ts` (unión discriminada) y `stores/projectDetail.ts`
      (las cuatro reglas de las notas). 6 tests
- [x] `ProjectView.svelte` con `ReadmeView`, `CommitList`, `BranchList`,
      `NotesEditor` y `CopyPathButton`, todo sobre los tokens
- [x] El nombre de la tarjeta abre el detalle; Escape vuelve con el foco en la
      tarjeta, salvo si se pasó por Ajustes

Verificada a mano por el usuario el 2026-10-05 con la versión de release
instalada: README, ramas, últimos commits, notas, copiar la ruta y volver con
Escape. Durante la prueba se vio que abrir el detalle no se descubría (se buscó
con doble clic), y se añadió un aviso que se cierra y no vuelve a salir.

HTML del README, cambiado el 2026-10-06 a petición del usuario: ya no se
descarta entero, sino que pasa por una lista blanca de `ammonia` (portadas
centradas, tablas, `details`, `sub`…; nada de scripts, estilos, iframes ni
manejadores de eventos). Las imágenes siguen sin cargarse y, con los vídeos e
iframes, se cuentan: un aviso encima del README dice qué falta y ofrece abrirlo
en la web del repositorio o en el editor. 6 tests más en `core/readme.rs` y 5
de `remoteWebUrl`.

Decisiones ya cerradas, para no volver a discutirlas: README renderizado en
Rust; enlaces del README abiertos en el navegador con lista blanca `http` y
`https`; imágenes no cargadas; portapapeles con el plugin oficial; notas con
autoguardado de 1,5 s más volcado inmediato al salir de la vista; navegación
con unión discriminada, sin router y sin persistir el detalle.

---

## Rediseño visual · CONGELADO

Trabajo abierto fuera del plan de fases, a partir de la auditoría visual del
2026-09-20, que listó diez problemas por gravedad.

Terminado:

- [x] `src/lib/styles/tokens.css`: color, tipografía, espaciado, radios,
      sombras, movimiento e iconografía, en oklch y con los dos temas.
- [x] `docs/DESIGN.md` con el porqué de cada decisión.
- [x] Tipografías empaquetadas: Inter Variable y JetBrains Mono, subconjunto
      latino, 96 KB, con su licencia. Antes se declaraban sin incluirlas y la
      aplicación caía a la fuente del sistema.
- [x] Rueda de etiquetas rehecha con medidas en oklab. Resuelve el problema 6
      y un fallo que no estaba en la lista: `turquesa` y `cian` eran el mismo
      color con dos nombres.
- [x] `ProjectCard` migrado, con ranura de etiquetas reservada.
- [x] `ProjectGrid` con dos densidades y `Dashboard` con el fondo migrado, que
      es lo que hace visible el contraste entre lienzo y tarjeta.
- [x] Aviso de pocos resultados bajo la barra de filtros, sin mover la rejilla.
- [x] Sesión A: `Sidebar`, `SearchBar`, `FilterBar`, `SortMenu`,
      `GitStatusBadge` y `EmptyState` migrados a los tokens.
- [x] Problema 7: el contador del botón «Filtros» cuenta solo los chips. La
      búsqueda vive en su campo y no entra en la cuenta, aunque sigue contando
      para «Limpiar filtros» y el aviso de pocos resultados.
- [x] Problema 8: una sola gramática de chip, la tintada con borde. Desaparece
      la variante rellena, que con etiquetas claras daba texto blanco a ~2:1.
- [x] Problema 10, la cabecera: Mosaic como marca, pestañas con subrayado de
      acento, metadatos en una línea terciaria y las acciones en un grupo.
- [x] Las fuentes de `tokens.css` apuntaban a Geist, que nunca se incluyó, y
      la aplicación caía a la del sistema. Ahora apuntan a Inter y JetBrains
      Mono; medido en WebKitGTK antes y después.
- [x] `TagChip` lee `--tag-chip-*` en lugar de alfas fijos en `tagColors.ts`.
- [x] Un solo «Limpiar filtros» a la vista y `GitStatusBadge`, sin uso,
      borrado.
- [x] Sesión B: `TagPicker`, `TagManager`, `Settings` y el botón de densidad
      migrados. Los desplegables de ajustes eran ilegibles en oscuro (control
      nativo de GTK) y ya no.
- [x] Cierre de la migración: `ScanSummaryBar`, el armazón de `App.svelte`,
      los avisos de `Dashboard` y `TagChip`. `app.css` ya no tiene tokens
      propios ni puente con `@theme`; Tailwind queda para la maquetación.
- [x] Conmutador de tema en la cabecera: oscuro, claro y sistema. Se guarda
      en `ui.view_state` y se aplica antes de montar la aplicación.

Deuda conocida del rediseño:

- [ ] `ProjectCard` reestiliza el disparador de `TagPicker` con
      `.anadir.fantasma :global(button)`, que alcanza también a los botones del
      panel. `TagPicker` lo neutraliza anidando sus selectores en `.panel`
      (comentado allí). Lo correcto es que la tarjeta no alcance hijos ajenos.
- Peculiaridad conocida, no es un fallo: en `pnpm tauri dev`, Vite puede
  registrar `window.__TAURI_INTERNALS__ undefined` al arrancar, antes de que
  el binario abra su ventana. Es una carrera entre Vite y el binario; no pasa
  en un build de release, que no usa Vite.

A medias:

- [ ] **Problema 1, densidad.** El modo compacto gana columnas pero no alto:
      pasa de 6,5 tarjetas visibles a 11, cuando el objetivo eran 17. Para
      llegar, la tarjeta tiene que perder filas en compacto, no solo apretarse,
      y eso obliga a que `ProjectCard` conozca la densidad. Sin decidir.

Sin empezar:


---

## Fase 6 — Pulido, atajos y ajustes avanzados

- [ ] Atajos: paleta de comandos, refrescar Git, abrir ajustes
- [x] Tema claro, oscuro y del sistema (adelantado con el rediseño)
- [ ] IDE y terminal preferidos configurables
- [ ] Escanear al arrancar
- [ ] Exportar e importar la configuración en JSON
- [ ] Modos compacto y cómodo del grid
- [ ] Barra de título propia. La del sistema es clara sobre una aplicación
      oscura (resto del problema 10 de la auditoría). No es CSS: hay que quitar
      las decoraciones y dibujarla dentro, con arrastre y redimensionado
      propios.

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
