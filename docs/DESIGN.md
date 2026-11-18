# Sistema de diseño

Decisiones del sistema visual de Mosaic y el porqué de cada una. Los valores
viven en `src/lib/styles/tokens.css`; esta página explica, no repite.

Escrito tras la auditoría visual del 2026-09-20, que encontró diez problemas
concretos. Cada decisión de aquí responde a alguno. El 2026-11-18 se rehízo la
composición («cartela de museo», abajo); la paleta, la tipografía y el
movimiento siguen siendo los de entonces.

---

## Dirección

**Oscuro cálido con acento turquesa.** Herramienta seria, con carácter propio,
que no se confunde con la categoría.

El gris azulado y el acento azul son la elección por defecto de casi todas las
herramientas para desarrolladores. Mosaic se mueve al lado cálido del neutro y
deja el azul para las etiquetas, que es donde el usuario pone su propio
significado.

El tema claro se define desde el primer día aunque la aplicación arranque en
oscuro: así la Fase 6 solo añade el conmutador, en vez de inventar una paleta
con prisa.

**Cartela de museo.** Cada proyecto se presenta como la cartela de una obra: el
nombre manda y los datos van debajo, como un pie. Sin cajas: separan el aire, una
regla fina y el tono de la superficie, no los bordes. La crítica del 2026-11-17
encontró que las etiquetas pesaban más que los nombres, que cada tarjeta repetía
el nombre en la ruta y marcaba lo fijado dos veces, y que el tablero se leía como
una cuadrícula de recuadros. Esta dirección responde a las tres cosas sin tocar
la paleta.

---

## Color

**Tres superficies separadas por cinco puntos de luminosidad.** La paleta
anterior las separaba cuatro y el resultado era el problema 3 de la auditoría:
con treinta tarjetas, la rejilla era una masa uniforme porque el fondo y la
tarjeta se distinguían apenas. Desde la cartela, la tarjeta no lleva superficie:
`base` es el tablero y las páginas, `raised` la cabecera, el panel lateral, los
campos y las píldoras, y `sunken` lo que está bajo el ratón o seleccionado. La
cabecera y el panel lateral no tienen borde; el tono basta.

**Matiz 75 en todos los neutros.** Un cálido muy bajo de saturación. Es
suficiente para que la pantalla no se lea como gris de sistema y no tanto como
para parecer sepia.

**Acento turquesa, no azul.** Evita el choque con las etiquetas azules, que son
las más usadas, y distingue el producto de sus referencias.

**El acento es solo estado.** Lo fijado, la pestaña activa, el foco y la rama
actual. Ni los botones principales ni las cajas de aviso van en turquesa: si todo
lo importante es turquesa, el turquesa deja de decir nada.

**Las etiquetas usan los dieciséis tonos 500 de Tailwind** de `tagColors.ts`,
que son los que guarda la base de datos. Hubo una rueda medida en oklab
(`--tag-*` en `tokens.css`) que corregía el parecido entre `turquesa` y `cian` y
su cercanía al acento, pero no la usaba ningún componente: se retiró el
2026-10-08 para no mantener dos paletas.

**Cuatro niveles de texto.** Primario, secundario, terciario y deshabilitado. El
terciario es el suelo de lo legible: todo lo que sea contenido está por encima.

**Semánticos con su superficie incluida.** Cada color de estado trae su fondo
translúcido ya calculado, para que ningún componente improvise una opacidad.

**Una etiqueta es un punto de color y su nombre en neutro.** Hasta el
2026-11-18 el chip entero se teñía (fondo, borde y un texto mezclado con una
tinta para pasar de 4,5:1), y tres etiquetas de colores pesaban más que el
nombre del proyecto. Ahora el color va solo en el punto (`--tag-dot`) y el
nombre en `--text-secondary` o `--text-primary`, así que su contraste ya no
depende del color elegido. En la tarjeta van sueltas (`plain`); en la barra de
filtros y en la ficha, sobre una píldora `raised` sin borde. Los chips de filtro
que no son etiquetas tienen la misma píldora, sin punto: lo que no es una
etiqueta no lleva color. Los tokens `--tag-chip-*` se retiraron.

---

## Tipografía

**Inter Variable y JetBrains Mono, empaquetadas con la aplicación.** La hoja anterior las declaraba sin incluir
ni un fichero de fuente, y ninguna de las dos está instalada en la máquina de
desarrollo: la aplicación llevaba toda su vida cayendo a la fuente del sistema.
De ahí buena parte del «sabe a plantilla». Empaquetarlas, y no enlazarlas, es
obligatorio, porque Mosaic no hace peticiones de red. Solo va el subconjunto
latino: 96 KB entre las tres variantes, sin dejar dependencias en
`package.json`.

**Siete tamaños, cada uno con su altura de línea y su peso.** El peso no se elige
aparte del tamaño: un tamaño es una decisión completa. Esto ataca el problema 5,
donde cinco datos distintos de la tarjeta pesaban prácticamente igual.

| Token | Uso |
|---|---|
| `title` 22px | Estados vacíos y títulos de ventana |
| `section` 17px | Cabeceras de sección y de modal |
| `card-title` 17px | Nombre en la tarjeta, lo único fuerte de ella |
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

**Tres: 4, 8 y 12.** El de 4 para elementos pequeños, el de 8 para controles y
bloques de código, el de 12 para superficies grandes como modales. La tarjeta ya
no tiene radio porque no tiene caja.

`pill` no cuenta como cuarto radio porque no es un tamaño, es una forma: solo la
llevan las etiquetas y los contadores.

---

## Sombras

**Tres niveles, teñidos con el matiz de la paleta, solo para lo que flota**:
menús, diálogos y avisos emergentes. Lo que está en la página no lleva sombra.
Una sombra negra pura sobre un fondo cálido se ve sucia. En oscuro las sombras apenas separan, así que se
acompañan de un brillo interior en el borde superior, que es lo que de verdad
levanta una superficie sobre un fondo oscuro.

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

- **Cómoda**: columna mínima de 300px, relleno vertical de 16, hueco entre
  columnas de 32.
- **Compacta**: columna mínima de 220px, relleno vertical de 12, hueco entre
  columnas de 24. Cuatro o cinco columnas en pantalla ancha.

No hay hueco entre filas: separa la regla de cada tarjeta y su relleno. La
tarjeta tampoco tiene relleno lateral, para que su regla y su texto empiecen en
la misma vertical.

En compacta la tarjeta pierde además la ruta y el pie (la última apertura y los
tres botones de abrir): las columnas solas no bastaban, porque ganaban ancho
pero no alto. Es la única vez que la tarjeta sabe la densidad, por su prop
`compact`. Abrir sigue a mano desde el detalle o con Ctrl+Enter en la paleta.

La preferencia se recuerda en `settings`, como el resto de la vista.

---

## Componentes

**Tarjeta.** Una regla fina encima y nada más: sin fondo, borde, radio ni
sombra. Por orden: el nombre, la carpeta que lo contiene (`~/proyectos`, no la
ruta entera, que repetía el nombre; la entera está en el tooltip), lenguaje y
Git, las etiquetas y la última apertura. Lo fijado se dice con la regla en
turquesa, la chincheta y su grupo; ya no lleva borde de color ni raya a la
izquierda. Al pasar el ratón, la regla se oscurece y asoman la chincheta, el «+»
de etiquetas y las acciones.

**Rótulos en minúscula y con peso**, no en versal: «Fijados», «Vistas»,
«Etiquetas» y las secciones del panel de filtros. La versal era la única voz que gritaba en una interfaz que habla en
minúscula.

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
