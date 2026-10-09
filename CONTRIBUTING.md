# Contribuir a Mosaic

<sub>[English](CONTRIBUTING.en.md)</sub>

Gracias por querer echar una mano. Mosaic es pequeño y quiere seguir siéndolo,
así que esta guía es corta.

## Antes de empezar

Mosaic tiene unos principios que no se negocian. Una propuesta que rompa alguno
no entrará, por buena que sea:

- **Local.** Cero peticiones de red. Ni telemetría, ni analíticas, ni
  comprobación de versiones, ni fuentes enlazadas.
- **Sin IA.** Ninguna API de modelos.
- **Solo lectura sobre tus repositorios.** Mosaic lee metadatos y nunca escribe
  en una carpeta de proyecto ni ejecuta Git con efectos.
- **Nada se borra.** Un proyecto que desaparece se marca como ausente, no se
  elimina, para no perder sus etiquetas y notas.
- **Rápido con cientos de proyectos.**

Para algo más grande que un arreglo, abre antes un issue y lo hablamos. Ahorra
trabajo a los dos.

## Preparar el entorno

Necesitas Rust estable, Node 20 o superior, [pnpm](https://pnpm.io) y [las dependencias de
Tauri 2](https://tauri.app/start/prerequisites/) de tu sistema. El README tiene
la lista para Fedora.

```bash
pnpm install
MOSAIC_LOG=debug pnpm tauri dev
```

La base de datos de desarrollo es la misma que la de la app instalada. Para
probar sin tocar la tuya, en Linux puedes apuntar a otra carpeta:

```bash
XDG_DATA_HOME=/tmp/mosaic-pruebas pnpm tauri dev
```

## Dónde va cada cosa

```
views/ → stores/ → api/ → invoke()  ⇄  commands/ → core/ → db/
```

- `src-tauri/src/core/`: la lógica, sin Tauri. Aquí van los tests unitarios.
- `src-tauri/src/db/repositories/`: todo el SQL, y solo aquí.
- `src-tauri/src/commands/`: valida la entrada y delega. Sin lógica.
- `src/lib/api/`: el único sitio del frontend que llama a `invoke()`.
- `src/lib/types/index.ts`: copia literal de los structs de Rust. Si cambias
  uno, cambia el otro.
- `src/lib/styles/tokens.css`: todos los colores, tamaños y espacios. Los
  componentes no escriben valores sueltos.

Más detalle en [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) y el porqué del
diseño en [`docs/DESIGN.md`](docs/DESIGN.md).

## Antes de abrir un pull request

Lo mismo que comprueba el CI, en Linux, macOS y Windows:

```bash
pnpm check            # tipos y plantillas, falla también con avisos
pnpm test             # tests del frontend
pnpm build

cd src-tauri
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Y además:

- **Un cambio de comportamiento lleva su test.** Las reglas que no son obvias,
  como las fechas de un proyecto o qué cuenta como proyecto, son las que más
  tests tienen.
- **Sin dependencias nuevas sin hablarlo antes.** Cada una es código que no
  controlamos.
- **Nunca `unwrap()` fuera de los tests**, y los errores van por `AppError`.
- **Comentarios solo para el porqué.** Una línea de documentación en lo público
  y, dentro del código, solo lo que explique una decisión que no se ve leyendo.

## Commits

[Conventional Commits](https://www.conventionalcommits.org/es/), en español y
pequeños: un tema por commit.

```
feat(ui): tooltip en el nombre de la tarjeta
fix(core): no marcar ausentes los proyectos de una ruta deshabilitada
```

## Informar de un fallo

Abre un issue con tu sistema operativo, la versión de Mosaic, qué hiciste, qué
esperabas y qué pasó. Si puedes, añade lo que salga al lanzarlo con
`MOSAIC_LOG=debug`.

## Licencia

Al contribuir aceptas que tu aportación se publique con la
[licencia Apache 2.0](LICENSE) del proyecto.
