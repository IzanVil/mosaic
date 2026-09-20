# Tipografías empaquetadas

Mosaic no hace peticiones de red, así que las fuentes viajan dentro de la
aplicación en lugar de enlazarse desde un CDN. Solo se incluye el subconjunto
latino, que es lo que necesita la interfaz.

| Fichero | Familia | Autoría | Licencia |
|---|---|---|---|
| `inter-variable-latin.woff2` | Inter Variable | Rasmus Andersson | SIL Open Font License 1.1 |
| `jetbrains-mono-400-latin.woff2` | JetBrains Mono Regular | JetBrains s.r.o. | SIL Open Font License 1.1 |
| `jetbrains-mono-500-latin.woff2` | JetBrains Mono Medium | JetBrains s.r.o. | SIL Open Font License 1.1 |

El texto completo de la licencia está en `OFL.txt`, junto a estos ficheros, y
cubre a las dos familias.

Origen de los ficheros: paquetes `@fontsource-variable/inter` y
`@fontsource/jetbrains-mono`, de los que se copiaron los `.woff2` del subconjunto
latino. Los paquetes no quedan como dependencia: solo los ficheros.
