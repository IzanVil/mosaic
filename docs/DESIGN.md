# Sistema de diseño

Decisiones del sistema visual de Mosaic y el porqué de cada una. Los valores
viven en `src/lib/styles/tokens.css`; esta página explica, no repite.

Escrito tras la auditoría visual del 2026-09-20, que encontró diez problemas
concretos. Cada decisión de aquí responde a alguno.

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

---

## Color

**Tres superficies separadas por cinco puntos de luminosidad.** La paleta
anterior las separaba cuatro y el resultado era el problema 3 de la auditoría:
con treinta tarjetas, la rejilla era una masa uniforme porque el fondo y la
tarjeta se distinguían apenas.

**Matiz 75 en todos los neutros.** Un cálido muy bajo de saturación. Es
suficiente para que la pantalla no se lea como gris de sistema y no tanto como
para parecer sepia.

**Acento turquesa, no azul.** Evita el choque con las etiquetas azules, que son
las más usadas, y distingue el producto de sus referencias.

**La rueda de etiquetas se redistribuyó para dejarle sitio.** Medido en oklab,
el acento turquesa original quedaba a Δ=0.035 de las etiquetas `turquesa` y
`cian`, que es indistinguible. Y esas dos estaban a Δ=0.038 **entre sí**: eran el
mismo color con dos nombres. Ahora los quince matices cromáticos se reparten por
los 310 grados que quedan fuera de la franja del acento, con la luminosidad
alternando entre vecinos. Ningún par de etiquetas baja de Δ=0.091 y el acento
queda a Δ=0.116 de la más parecida.

**Cuatro niveles de texto.** Primario, secundario, terciario y deshabilitado. El
terciario es el suelo de lo legible: todo lo que sea contenido está por encima.

**Semánticos con su superficie incluida.** Cada color de estado trae su fondo
translúcido ya calculado, para que ningún componente improvise una opacidad.

**Las dieciséis etiquetas comparten luminosidad y croma.** Solo cambia el matiz.
Eso arregla el problema 6: `archivado` era un gris pizarra oscuro entre colores
vivos y desaparecía sobre el fondo. Ahora el neutro tiene la misma presencia,
solo que sin color.

**El chip de etiqueta se construye con tres porcentajes**, no con hexadecimales
sueltos en el componente. Cambiar el contraste de todos los chips es cambiar una
variable.

---

## Tipografía

**Inter Variable y JetBrains Mono, empaquetadas con la aplicación.** La hoja anterior las declaraba sin incluir
ni un fichero de fuente, y ninguna de las dos está instalada en la máquina de
desarrollo: la aplicación llevaba toda su vida cayendo a la fuente del sistema.
De ahí buena parte del «sabe a plantilla». Empaquetarlas, y no enlazarlas, es
obligatorio, porque Mosaic no hace peticiones de red. Solo va el subconjunto
latino: 96 KB entre las tres variantes, sin dejar dependencias en
`package.json`.

**Seis tamaños, cada uno con su altura de línea y su peso.** El peso no se elige
aparte del tamaño: un tamaño es una decisión completa. Esto ataca el problema 5,
donde cinco datos distintos de la tarjeta pesaban prácticamente igual.

| Token | Uso |
|---|---|
| `title` 22px | Estados vacíos y títulos de ventana |
| `section` 17px | Cabeceras de sección y de modal |
| `card` 15px | Nombre de proyecto |
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

**Tres: 4, 8 y 12.** El de 4 para elementos pequeños, el de 8 para tarjetas y
controles, el de 12 para superficies grandes como modales.

`pill` no cuenta como cuarto radio porque no es un tamaño, es una forma: solo la
llevan las etiquetas y los contadores.

---

## Sombras

**Tres niveles, teñidos con el matiz de la paleta.** Una sombra negra pura sobre
un fondo cálido se ve sucia. En oscuro las sombras apenas separan, así que se
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

- **Cómoda**: columna mínima de 300px, relleno de 16, hueco de 12. Es la de hoy.
- **Compacta**: columna mínima de 220px, relleno de 12, hueco de 8. Cuatro o
  cinco columnas en pantalla ancha.

La preferencia se recuerda en `settings`, como el resto de la vista.

---

## Lo que este sistema todavía no resuelve

- **La barra de título del sistema** (problema 10) no es CSS: hay que quitar las
  decoraciones en `tauri.conf.json` y dibujar la barra dentro de la aplicación,
  con su zona de arrastre y sus botones. En Wayland eso incluye rehacer el
  redimensionado desde los bordes.
- **La jerarquía de la cabecera** (problema 10) es de composición, no de tokens.
- **El estado de pocos resultados** (problema 2) es de producto: depende de
  cuántos resultados hay, no del color.
