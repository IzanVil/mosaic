# GIF de demostración

`docs/img/demo.gif`, el recorrido animado de los README, se rehace con:

```bash
./scripts/demo/hacer-demo.sh
```

Compila la interfaz, la abre en WebKitGTK sin pantalla con un backend simulado
y proyectos inventados, sigue el recorrido de `grabar.py` y junta los
fotogramas en un GIF con ffmpeg. Tarda medio minuto y no abre ninguna ventana.

Necesita `pnpm`, `ffmpeg` y Python con WebKit2 4.1 (en Fedora,
`python3-gobject` y `webkit2gtk4.1`).

- `backend_simulado.py`: los datos que ve la interfaz.
- `marco.html`: la ventana, el puntero y el rótulo de las teclas.
- `grabar.py`: los pasos del recorrido y lo que dura cada fotograma.

GitHub no reproduce vídeos guardados en el repositorio, solo imágenes
animadas: por eso es un GIF.
