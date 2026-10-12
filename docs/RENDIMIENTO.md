# Rendimiento con miles de proyectos

Medido el 2026-10-12 para la issue [#5](https://github.com/IzanVil/mosaic/issues/5).
Uno de los principios de Mosaic es ir fluido con más de 500 proyectos; hasta
ahora solo se había probado con unos 35.

## Cómo se mide

```bash
python3 scripts/rendimiento/generar.py /tmp/proyectos   # 5.000 proyectos, ~1 min
./scripts/rendimiento/medir.sh /tmp/proyectos           # backend y frontend
./scripts/rendimiento/medir.sh                          # solo el frontend
```

- **Proyectos de prueba** (`generar.py`): tres carpetas con 500, 1.500 y 3.000
  proyectos, para medir con 500, 2.000 y 5.000. Cada uno tiene su manifiesto,
  un README y 12 ficheros de código; el 70 % es un repositorio Git con un
  commit, y uno de cada cinco de esos tiene cambios sin commitear. En total,
  173.500 ficheros y 70.456 entradas que recorrer.
- **Backend** (`src-tauri/tests/rendimiento.rs`, ignorado en los tests normales):
  escaneo, reescaneo sin cambios, refresco de Git y la lista que pide el
  tablero, convertida a JSON. En modo release.
- **Frontend** (`scripts/rendimiento/frontend.py`): la interfaz compilada en
  WebKitGTK sin pantalla, con un backend simulado de N proyectos. Mide cuánto
  tarda en estar pintada cada tarjeta, lo que tarda la búsqueda en responder
  (sin los 150 ms de espera a que se deje de teclear) y la memoria del proceso
  que pinta la página.

Límites de la medición: los proyectos viven en `/tmp`, que es memoria y no
disco, así que el primer escaneo en un disco de verdad tardará más. Y WebKit
sin pantalla pinta sin aceleración gráfica; la app real puede pintar algo más
rápido, pero el coste medido abajo está sobre todo en crear la página, no en
pintarla.

## Backend

| Proyectos | Escaneo | Reescaneo | Git (repos) | Lista del tablero |
|---|---|---|---|---|
| 500 | 0,15 s | 0,12 s | 0,37 s (350) | 2 ms, 271 KB |
| 2.000 | 0,56 s | 0,50 s | 1,45 s (1.400) | 10 ms, 1,1 MB |
| 5.000 | 1,41 s | 1,24 s | 3,6 s (3.500) | 24 ms, 2,7 MB |

Crece en proporción al número de proyectos. **Antes del arreglo del escáner**
el escaneo de 5.000 tardaba 4,5 s y no crecía en proporción (de 500 a 5.000,
21 veces más): para aplicar la regla de anidamiento comparaba cada candidato
con todos los proyectos ya registrados. Ahora mira solo sus carpetas padre.

El refresco de Git lee los repositorios uno detrás de otro, a 1 ms cada uno.
Va en segundo plano, así que 3,6 s con 3.500 repositorios no bloquean nada,
aunque el botón «Git» tarda eso en responder.

Con el tope por defecto de 50.000 entradas por ruta no se recortó ningún
escaneo: la carpeta más grande, con 3.000 proyectos, tiene unas 42.000.

## Frontend

Antes del arreglo de la #13:

| Proyectos | Tablero pintado | Búsqueda | Borrar la búsqueda | Memoria |
|---|---|---|---|---|
| 24 | 0,6 s | 29 ms | 0,2 s | 176 MB |
| 500 | **3,5 s** | 0,2 s | **1,9 s** | 474 MB |
| 2.000 | 12,3 s | 0,6 s | 7,6 s | 1,7 GB |
| 5.000 | 27 s | 1,5 s | 18 s | 2,9 GB |

**Con 500 proyectos no se cumplía el principio.** Cada tarjeta costaba unos
6 ms y 0,6 MB, y el coste se pagaba cada vez que la lista volvía a crecer:
borrar una búsqueda volvía a montar todas las tarjetas.

### Dónde está el coste

Experimentos con 500 proyectos, quitando una pieza cada vez de `ProjectCard`
(3,6 s con la tarjeta completa):

| Se quita | Tablero pintado | Ahorro |
|---|---|---|
| Los chips de etiqueta | 2,1 s | 1,5 s |
| Los iconos (fijar, alerta y abrir) | 2,8 s | 0,8 s |
| El pie con sus tres botones | 2,8 s | 0,8 s |
| El selector de etiquetas | 3,4 s | 0,2 s |
| Pintar las tarjetas fuera de pantalla (`content-visibility`) | 3,1 s | 0,5 s |
| **Todo: la tarjeta solo con su nombre** | **0,46 s, 167 MB** | 3,1 s |

No hay una pieza culpable: cada tarjeta monta muchos componentes pequeños
(unos seis iconos, contando la X de cada chip, y dos o tres chips), y con 500
tarjetas son miles.

### Solo se montan las tarjetas que se ven

Arreglado para la issue [#13](https://github.com/IzanVil/mosaic/issues/13).
Cada hueco de la rejilla es ahora un `LazyCard`: un botón vacío del alto de una
tarjeta, que se convierte en la tarjeta de verdad cuando está a menos de 800 px
de lo visible y vuelve a ser hueco cuando se aleja. Un solo
`IntersectionObserver` por contenedor con scroll (`utils/nearViewport.ts`)
vigila todos los huecos. Las 24 primeras de cada grupo se montan al abrir, para
que la primera pantalla no aparezca a saltos.

La rejilla sigue siendo CSS: no hay cálculos de filas ni de columnas. Y el
hueco lleva el mismo `data-proyecto` que el nombre de la tarjeta, de modo que
volver del detalle con Escape o recorrer con Tab llega también a una tarjeta
que está lejos: al recibir el foco se monta y le pasa el foco a su nombre.

| Proyectos | Tablero pintado | Borrar la búsqueda | Memoria |
|---|---|---|---|
| 24 | 0,9 s → 0,9 s | 0,2 s → 0,2 s | 182 → 184 MB |
| 500 | 3,6 s → **1,0 s** | 1,9 s → **0,32 s** | 474 → **206 MB** |
| 2.000 | 12,3 s → **1,0 s** | 7,6 s → **0,46 s** | 1,7 GB → **254 MB** |
| 5.000 | 27 s → **1,4 s** | 18 s → **0,97 s** | 2,9 GB → **372 MB** |

Los «ahora» y los «antes» de 24 y 500 se midieron en la misma tanda; los
«antes» de 2.000 y 5.000 son de la medición anterior. Los tiempos absolutos
varían con la carga de la máquina: con el mismo código, 24 proyectos tardaron
0,6 s en una tanda y 0,9 s en otra. Lo que importa es la diferencia con la fila de 24: con 500 y con
2.000 proyectos, abrir el tablero cuesta casi lo mismo que con 24.

Comprobado en WebKit con 500 proyectos: al abrir hay 24 tarjetas montadas; tras
bajar hasta la mitad, 42, todas alrededor de lo visible, y las de arriba se han
desmontado; y dar el foco al hueco de la tarjeta 480 la monta y deja el foco en
su nombre. Las capturas del README salen iguales que antes, salvo la fecha
relativa de «Abierto hace N días».
