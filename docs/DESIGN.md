# Sistema de diseño

Decisiones del sistema visual de Mosaic y el porqué de cada una. Los valores
viven en `src/lib/styles/tokens.css`; esta página explica, no repite.

La base se escribió tras la auditoría visual del 2026-09-20, que encontró diez
problemas concretos; la escala de tamaños, el espaciado, el movimiento y las dos
densidades responden a ellos. El 2026-11-18 cambió el mundo visual entero: la
paleta, la letra y la tarjeta son ahora las de «Azulejo».

---

## Dirección

**Azulejo: blanco esmaltado y cobalto.** El tablero es literalmente un mosaico:
cada proyecto es una pieza, y su color sale de sus propias etiquetas. El nombre
de la aplicación y su icono, teselas azules, ya apuntaban ahí.

Viene de dos intentos que no se notaban. Los neutros cálidos con acento
turquesa eran «una pantalla gris con un color», que es justo el uniforme de la
categoría aunque el gris fuera cálido. La «cartela de museo» del mismo
2026-11-18 cambió la composición (tarjetas sin caja, etiquetas con punto, sin
bordes) pero conservó la paleta y la letra, y se leía como la misma aplicación:
entre el 1 y el 9 % de los píxeles cambiaban por pantalla. De la cartela se
quedan los rótulos en minúscula, los campos rellenos, la carpeta en vez de la
ruta entera y el botón principal en tinta.

**El mundo solo presta cuatro cosas**: la paleta, la letra, la densidad y una
firma, la cenefa de cada tarjeta. La estructura (lateral, cabecera, rejilla) y
los controles son los de siempre: en una herramienta, nadie tiene que aprender
dónde está nada.

El tema claro y el oscuro son igual de completos. El oscuro no invierte el
claro: conserva el lateral en cobalto y baja el fondo a un azul noche.

---

## Color

**Todos los neutros llevan el matiz 264, el del cobalto**, con poca saturación.
Así el fondo, las tarjetas y el lateral son de la misma familia, y la pantalla
no se lee como gris de sistema.

**Superficies.** `base` es el fondo del tablero y de las páginas. `raised` y
`sunken` son pasos de tono sobre él: las píldoras, los bloques de código y lo
que está bajo el ratón o seleccionado. `card` es la tarjeta, la única superficie
blanca del tablero en claro; en oscuro coincide con `raised`. `overlay` es lo
que flota: menús y diálogos.

**El lateral es de cobalto** (`--rail-*`). El armazón lleva el color de la
marca, no solo el acento: una herramienta gris con un punto de color es la
opción por defecto sin su pulido. Encima, el texto va en dos claros del mismo
matiz, las etiquetas con un filo claro (algunos colores se perdían sobre el
azul) y el foco en blanco, porque el acento no se ve sobre su propio color.

**El acento es el mismo cobalto**, en el tono que da contraste sobre el fondo.
No es el azul de categoría porque no va solo: el armazón entero es de ese
color. Y sigue siendo **solo estado**: lo fijado, la pestaña activa, el foco, la
rama actual y el contador de filtros. Ni los botones principales ni las cajas de
aviso van en cobalto.

Riesgo aceptado: la etiqueta azul de la paleta (`blue`, `#3B82F6`) es de la
misma familia que el acento. Se distinguen por luminosidad y saturación.

**Semánticos con su superficie incluida**, y sin compartir matiz con el acento:
un aviso no puede confundirse con un elemento activo. Por eso `info` pasó del
250 al 215 al llegar el cobalto.

**Cuatro niveles de texto.** Primario, secundario, terciario y deshabilitado. El
terciario es el suelo de lo legible y pasa de 4,5:1 en todas las superficies
donde se usa; el caso más justo es el claro sobre `sunken`, con 4,67.

**Las etiquetas usan los dieciséis tonos 500 de Tailwind** de `tagColors.ts`,
que son los que guarda la base de datos. Hubo una rueda medida en oklab
(`--tag-*`) que no usaba ningún componente: se retiró el 2026-10-08 para no
mantener dos paletas.

**Una etiqueta es una tesela cuadrada y su nombre en neutro.** El color va solo
en la tesela (`--tag-dot`, con el radio de `--tag-tile-radius`) y el nombre en
`--text-secondary` o `--text-primary`, así que su contraste no depende del color
elegido. En la tarjeta van sueltas (`plain`); en la barra de filtros y en la
ficha, sobre una píldora `raised` sin borde. Los chips de filtro que no son
etiquetas tienen la misma píldora, sin tesela: lo que no es una etiqueta no
lleva color. Las muestras del selector de color y del gestor también son
cuadradas.

---

## Tipografía

**Schibsted Grotesk y JetBrains Mono, empaquetadas con la aplicación.**
Schibsted Grotesk sustituyó a Inter el 2026-11-18: una grotesca de periódico,
más ancha y con más carácter, que pesa lo mismo (47 KB frente a 48) y tiene las
cifras de ancho fijo. Frente a Inter solo le faltan unos espacios especiales (el
fino, el de media eme y los de ancho cero) y un diacrítico combinable, que la
interfaz no usa. Empaquetarlas, y no
enlazarlas, es obligatorio, porque Mosaic no hace peticiones de red: la hoja de
antes de la auditoría las declaraba sin incluir ningún fichero y la aplicación
caía a la fuente del sistema. Solo va el subconjunto latino, 88 KB entre las
tres, sin dependencias en `package.json`. La licencia está junto a los ficheros.

**Siete tamaños, cada uno con su altura de línea y su peso.** El peso no se elige
aparte del tamaño: un tamaño es una decisión completa. Esto ataca el problema 5,
donde cinco datos distintos de la tarjeta pesaban prácticamente igual.

| Token | Uso |
|---|---|
| `title` 22px | Estados vacíos y títulos de ventana |
| `section` 17px | Cabeceras de sección y de modal, la marca |
| `card-title` 18px | Nombre en la tarjeta, en 700: lo único fuerte de ella |
| `card` 15px | Nombre de proyecto en listas |
| `body` 13.5px | Texto corriente |
| `meta` 12px | Rótulos, contadores, chips |
| `code` 11.5px | Rutas, sha, números alineados |

---

## Espaciado

**Escala de ocho valores, de 4 a 64.** Progresión que dobla hasta 16 y luego
crece de forma más suave. Ocho valores son suficientes para una aplicación de
esta talla y pocos para que nadie invente un 18px.

---

## Radios

**Tres: 4, 8 y 12.** El de 4 para la tarjeta, las muestras de color y los
elementos pequeños, el de 8 para controles y bloques de código, el de 12 para
superficies grandes como modales. Un azulejo tiene las esquinas casi vivas.

`pill` y `--tag-tile-radius` no cuentan como radios porque no son tamaños, son
formas: la píldora la llevan los chips y los contadores; el canto de 1,5 px, las
teselas de las etiquetas.

---

## Sombras

**Tres niveles, teñidos con el matiz de la paleta, solo para lo que flota**:
menús, diálogos y avisos emergentes. La tarjeta no lleva sombra: la separan su
borde fino y el blanco sobre el fondo. Una sombra negra pura sobre un fondo
azulado se ve sucia. En oscuro las sombras apenas separan, así que se acompañan
de un brillo interior en el borde superior.

---

## Movimiento

**Tres duraciones y dos curvas.** 120 ms para respuestas al cursor, 180 ms para
cambios de estado, 280 ms para entradas y salidas de superficies. `ease-out`
para lo que aparece, `ease-in-out` para lo que se transforma.

Con `prefers-reduced-motion` las tres duraciones pasan a cero desde el propio
token, así que ningún componente necesita acordarse.

---

## Iconografía

**Cuatro tamaños y un grosor único de 1,75.** El grosor constante es lo que hace
que un icono de 14 y otro de 24 parezcan de la misma familia. Se mantiene
`@lucide/svelte`, que ya está en el proyecto.

---

## Densidad del tablero

**Dos densidades, cómoda y compacta, definidas aquí y no en el grid.** Responde
al problema 1: con 34 proyectos solo caben seis tarjetas y media en pantalla, y
el producto promete cientos.

- **Cómoda**: columna mínima de 300px, relleno de 16 y llaga de 16 entre
  azulejos.
- **Compacta**: columna mínima de 220px, relleno de 12 y llaga de 12. Cuatro o
  cinco columnas en pantalla ancha.

La llaga es la misma en las dos direcciones, como entre azulejos de verdad.

En compacta la tarjeta pierde además la ruta y el pie (la última apertura y los
tres botones de abrir): las columnas solas no bastaban, porque ganaban ancho
pero no alto. Es la única vez que la tarjeta sabe la densidad, por su prop
`compact`. Abrir sigue a mano desde el detalle o con Ctrl+Enter en la paleta.
En compacta es donde más se nota el mosaico.

La preferencia se recuerda en `settings`, como el resto de la vista.

---

## Componentes

**Tarjeta: un azulejo con su cenefa.** Fondo `card`, borde fino y radio de 4.
Arriba, una franja de 5 px (`--tile-band`) con un tramo por etiqueta, en su
orden, del mismo ancho y separados por una llaga de 3 px (`--tile-grout`); sin
etiquetas, la franja es neutra. La calcula `tagBand` en `tagColors.ts`, con
tests. La franja se redondea a mano y no con `overflow: hidden`, que recortaría
el selector de etiquetas y la pista del nombre.

Por orden, dentro: el nombre, la carpeta que lo contiene (`~/proyectos`, no la
ruta entera, que repetía el nombre; la entera está en el tooltip), lenguaje y
Git, las etiquetas y la última apertura. Lo fijado se dice con el borde en el
acento, la chincheta y su grupo, nunca con una raya de color a un lado. Al pasar
el ratón, el borde se oscurece y asoman la chincheta, el «+» de etiquetas y las
acciones.

**Marca.** «Mosaic» en 700, precedido de cuatro teselas de 8 px con su llaga:
cobalto, ocre, verdete y terracota (`--tile-*`).

**Rótulos en minúscula y con peso**, no en versal: «Vistas», «Etiquetas» y las
secciones del panel de filtros. «Fijados» lleva además una tesela del acento,
porque es un estado.

**Campos: fondo y sin borde.** Búsqueda, números, desplegables y notas. El borde
se queda transparente para que el foco no mueva nada. Los números no llevan las
flechas nativas, que salen con el tema del sistema; las del teclado siguen
funcionando.

**Cuatro clases de botón:**

| Clase | Aspecto | Dónde |
|---|---|---|
| Principal | Tinta: fondo `--primary-bg` y texto `--primary-text` | Uno por sección como mucho: añadir carpeta, aplicar una importación, crear etiqueta, limpiar filtros |
| Secundario | Contorno, sin fondo | Formularios y la ficha: Editor, Terminal, Carpeta, Copiar ruta |
| Fantasma | Ni borde ni fondo hasta pasar el ratón | Cabecera y barra de herramientas |
| Peligro | El rojo con su superficie | Confirmar un borrado |

**Listas con reglas, sin caja**: rutas de escaneo, commits, atajos. **Los
diálogos sí llevan borde**: flotan sobre la página y sin él se confunden con lo
de debajo.

---

## Lo que este sistema todavía no resuelve

- **La barra de título del sistema** (problema 10) no es CSS: hay que quitar las
  decoraciones en `tauri.conf.json` y dibujar la barra dentro de la aplicación,
  con su zona de arrastre y sus botones. En Wayland eso incluye rehacer el
  redimensionado desde los bordes.
- **La jerarquía de la cabecera** (problema 10) es de composición, no de tokens.
- **El estado de pocos resultados** (problema 2) es de producto: depende de
  cuántos resultados hay, no del color.
- **Las capturas, el GIF y la web** enseñan todavía el diseño de la 0.3.1, que
  es el que se descarga. Se rehacen al publicar la versión que lleve Azulejo, y
  con ellas el marco de `scripts/demo/`, que tiene un fondo turquesa.
