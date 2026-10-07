<div align="center">

<img src="src-tauri/icons/128x128@2x.png" width="88" height="88" alt="Icono de Mosaic">

<h1>Mosaic</h1>

<p><b>Español</b> · <a href="README.en.md">English</a></p>

<p><b>Todos tus proyectos de código, en un solo tablero.</b></p>

<p>Estado de Git en vivo, etiquetas y un clic para abrir cada proyecto.<br>
Local, privado y sin una sola petición de red.</p>

<p>
<img src="https://img.shields.io/badge/licencia-Apache%202.0-2c8c8c?style=flat-square" alt="Licencia Apache 2.0">
<img src="https://img.shields.io/badge/Tauri-2-24c8db?style=flat-square&logo=tauri&logoColor=white" alt="Tauri 2">
<img src="https://img.shields.io/badge/Svelte-5-ff3e00?style=flat-square&logo=svelte&logoColor=white" alt="Svelte 5">
<img src="https://img.shields.io/badge/Rust-b7410e?style=flat-square&logo=rust&logoColor=white" alt="Rust">
<img src="https://img.shields.io/badge/estado-alfa-d4a017?style=flat-square" alt="Estado: alfa">
</p>

<p>
<a href="https://github.com/IzanVil/mosaic/releases/latest"><img src="https://img.shields.io/badge/Descargar-Mosaic%200.1.0-2c8c8c?style=for-the-badge" alt="Descargar Mosaic 0.1.0" height="36"></a>
</p>

<p>
<a href="https://mosaic-app.i-vilches.workers.dev"><b>Web</b></a> ·
<a href="#-descargas"><b>Descargar</b></a> ·
<a href="#-lo-que-hace"><b>Lo que hace</b></a> ·
<a href="#-capturas"><b>Capturas</b></a> ·
<a href="#-empezar"><b>Empezar</b></a> ·
<a href="#-estado"><b>Estado</b></a> ·
<a href="#-compilar"><b>Compilar</b></a>
</p>

<br>

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/img/portada-oscura.png">
  <source media="(prefers-color-scheme: light)" srcset="docs/img/portada-clara.png">
  <img src="docs/img/portada-oscura.png" alt="El tablero de Mosaic: tres proyectos fijados arriba y el resto en rejilla, cada tarjeta con su ruta, sus etiquetas de color, su lenguaje y su rama de Git, y a la izquierda el panel de vistas y etiquetas">
</picture>

</div>

<br>

## 💡 Por qué

Cuarenta carpetas en `~/proyectos`, la mitad con cambios sin commitear que ya
no recuerdas, y para abrir la que buscas toca `cd`, `ls` y adivinar.
**Mosaic te da la vista de pájaro.**

<table>
  <tr>
    <td width="25%" valign="top">
      <h3>🔒 Local</h3>
      Todo vive en un SQLite en tu máquina. Cero peticiones de red, ni siquiera <code>git fetch</code>.
    </td>
    <td width="25%" valign="top">
      <h3>🕶️ Privado</h3>
      Sin cuentas, sin telemetría y sin analítica.
    </td>
    <td width="25%" valign="top">
      <h3>📖 Solo lectura</h3>
      Lee tus repositorios y nunca escribe en ellos.
    </td>
    <td width="25%" valign="top">
      <h3>🧘 Sin IA</h3>
      Ninguna API de modelos. Lee metadatos y te los enseña.
    </td>
  </tr>
</table>

## ✨ Lo que hace

<table>
  <tr>
    <td width="33%" valign="top">
      <h3>🔍 Los encuentra solo</h3>
      Le das una carpeta y detecta cada proyecto por su <code>.git</code>, <code>package.json</code>, <code>Cargo.toml</code>, <code>go.mod</code>… Deduce también su lenguaje principal.
    </td>
    <td width="33%" valign="top">
      <h3>🌿 Git de un vistazo</h3>
      Rama, cambios sin commitear, commits por delante y por detrás del upstream y último commit. Se refresca solo en segundo plano.
    </td>
    <td width="33%" valign="top">
      <h3>⚡ Un clic para abrir</h3>
      Abre cada proyecto en tu editor, tu terminal o tu explorador. Mosaic detecta lo que tienes instalado.
    </td>
  </tr>
  <tr>
    <td valign="top">
      <h3>🏷️ Etiquetas con color</h3>
      Se asignan desde la propia tarjeta y se crean al vuelo con solo escribir un nombre nuevo.
    </td>
    <td valign="top">
      <h3>🎯 Búsqueda y filtros</h3>
      Búsqueda difusa que ignora acentos y mayúsculas. Filtra por etiqueta, lenguaje, estado de Git o destacados.
    </td>
    <td valign="top">
      <h3>🌗 Claro u oscuro</h3>
      O el del sistema. Mosaic recuerda el tema, los filtros y el orden entre sesiones.
    </td>
  </tr>
  <tr>
    <td colspan="3" valign="top">
      <h3>📂 Y una ficha por proyecto</h3>
      Pulsa el nombre de una tarjeta y ves su README, sus ramas, sus últimos commits y tus notas, que se guardan solas mientras escribes. Escape te devuelve al tablero.
    </td>
  </tr>
</table>

## 📸 Capturas

<p align="center">
  <img src="docs/img/filtros.png" alt="Búsqueda «api» con cinco filtros activos en chips (dos etiquetas, dos lenguajes y «Ausentes ocultos»), el contador «Filtros 5» y el aviso de que dos proyectos coinciden">
  <br><sub><b>Busca y filtra.</b> Cada filtro es un chip con su X, y el contador dice cuántos hay puestos.</sub>
</p>

<br>

<table>
  <tr>
    <td width="58%" align="center">
      <img src="docs/img/gestor-etiquetas.png" alt="El gestor de etiquetas: un campo para crear una nueva con su color y la lista de etiquetas con cuántos proyectos tiene cada una">
      <br><sub><b>Gestiona las etiquetas</b> en un solo panel.</sub>
    </td>
    <td width="42%" align="center">
      <img src="docs/img/selector-etiquetas.png" alt="El selector de etiquetas abierto desde una tarjeta, con un buscador y las etiquetas asignadas marcadas">
      <br><sub><b>Etiqueta desde la tarjeta</b>, sin salir del tablero.</sub>
    </td>
  </tr>
</table>

<br>

<p align="center">
  <img src="docs/img/temas.png" alt="El tablero partido en diagonal: la mitad izquierda en tema oscuro y la derecha en tema claro">
  <br><sub><b>Dos temas</b> con la misma paleta cálida, y un botón en la cabecera para cambiar de uno a otro.</sub>
</p>

## 🚀 Empezar

```
1. Ajustes  →  Añadir carpeta       la que contiene tus proyectos
2. Escanear                         Mosaic recorre el disco y llena el tablero
3. Etiqueta, destaca y filtra       la próxima vez lo encontrarás igual
```

El estado de Git se lee solo a los cinco segundos de arrancar y después cada
cinco minutos. El botón **Git** de la cabecera fuerza una relectura.

**Atajos** (desde la próxima versión): <kbd>Ctrl</kbd>+<kbd>K</kbd> abre una
paleta para saltar a cualquier proyecto o acción, <kbd>Ctrl</kbd>+<kbd>R</kbd>
relee Git, <kbd>/</kbd> busca y <kbd>?</kbd> enseña todos los atajos. En macOS,
<kbd>⌘</kbd> en lugar de <kbd>Ctrl</kbd>.

<details>
<summary><b>¿Cómo decide qué es un proyecto?</b></summary>

<br>

Una carpeta entra en el tablero si tiene un fichero marcador (`.git`,
`package.json`, `Cargo.toml`, `pyproject.toml`, `go.mod`, `pom.xml`,
`build.gradle`, `composer.json` o `Gemfile`) y, además, **o tiene su propio
`.git`, o ninguna carpeta por encima está ya registrada**. Así, un monorepo es
una tarjeta, y dos repositorios anidados son dos.

Baja cuatro niveles, no sigue enlaces simbólicos y se salta `node_modules`,
`target`, `.venv` y similares. Si una raíz pasa de 50.000 entradas, se detiene
y te avisa de que el escaneo quedó incompleto.

</details>

<details>
<summary><b>¿Y si borro o muevo una carpeta?</b></summary>

<br>

El proyecto se marca como **ausente, no se borra**: conserva sus etiquetas por
si vuelve. Puedes ocultar los ausentes con un filtro.

</details>

<details>
<summary><b>¿Puede escanear solo al arrancar?</b></summary>

<br>

Sí: en **Ajustes → Escaneo**, «Escanear al arrancar». Mosaic recorrerá tus
rutas ocho segundos después de abrirse. Ahí mismo se cambian también la
profundidad, las carpetas que se saltan y la frecuencia con la que relee Git.

Esa sección llega en la próxima versión. En la 0.1.0 se activa así:

```bash
sqlite3 ~/.local/share/mosaic/mosaic.db \
  "INSERT INTO settings (key, value) VALUES ('scan.on_startup', 'true')
   ON CONFLICT(key) DO UPDATE SET value = 'true';"
```

</details>

<details>
<summary><b>¿Cómo paso mis etiquetas y notas a otro equipo?</b></summary>

<br>

En **Ajustes → Copia de seguridad**, «Exportar…» guarda un fichero JSON con
tus ajustes y rutas y, si dejas marcada la casilla, tus etiquetas, notas y
fijados. En el otro equipo, «Importar…» te enseña antes qué va a cambiar.
Importar solo añade: no borra nada ni pisa notas que ya tengas. Llega en la
próxima versión.

</details>

<details>
<summary><b>¿Dónde guarda los datos?</b></summary>

<br>

En un único fichero SQLite. Para empezar de cero, cierra Mosaic y bórralo.

| Sistema | Ruta |
|---|---|
| Linux | `~/.local/share/mosaic/mosaic.db` |
| macOS | `~/Library/Application Support/dev.izan.mosaic/mosaic.db` |
| Windows | `%APPDATA%\izan\mosaic\data\mosaic.db` |

</details>

## 🧭 Estado

**Alfa:** se usa a diario, pero le faltan piezas. Lo detallado está en el
[roadmap](docs/ROADMAP.md).

| | | |
|:-:|---|---|
| ✅ | **Escaneo** | Raíces configurables, detección de proyectos y lenguaje |
| ✅ | **Git** | Estado en vivo con refresco en segundo plano |
| ✅ | **Tablero** | Tarjetas, destacados y apertura en editor y terminal |
| ✅ | **Etiquetas y filtros** | Búsqueda difusa y vista guardada entre sesiones |
| ✅ | **Diseño** | Sistema propio, tema claro y oscuro |
| ✅ | **Detalle de proyecto** | README, historial, ramas y notas |
| 🟡 | **Pulido** | Ajustes avanzados y atajos hechos; copia de seguridad hechos; falta el modo compacto |
| 🟡 | **Distribución** | 0.1.0 publicada para las tres plataformas; faltan las firmas |

## 📦 Descargas

**Mosaic 0.1.0** ya está publicado para las tres plataformas. Elige tu sistema:

<table>
  <tr>
    <th width="33%">🍎 macOS</th>
    <th width="33%">🪟 Windows</th>
    <th width="33%">🐧 Linux</th>
  </tr>
  <tr>
    <td align="center" valign="top">
      <a href="https://github.com/IzanVil/mosaic/releases/download/v0.1.0/Mosaic_0.1.0_universal.dmg"><img src="https://img.shields.io/badge/.dmg-universal-2c8c8c?style=flat-square" alt="Descargar el .dmg universal"></a>
      <br><sub>Intel y Apple Silicon · 8,3 MB</sub>
    </td>
    <td align="center" valign="top">
      <a href="https://github.com/IzanVil/mosaic/releases/download/v0.1.0/Mosaic_0.1.0_x64-setup.exe"><img src="https://img.shields.io/badge/.exe-instalador-2c8c8c?style=flat-square" alt="Descargar el instalador .exe"></a>
      <br><sub>Recomendado · 3,4 MB</sub>
      <br><br>
      <a href="https://github.com/IzanVil/mosaic/releases/download/v0.1.0/Mosaic_0.1.0_x64_en-US.msi"><img src="https://img.shields.io/badge/.msi-paquete-555?style=flat-square" alt="Descargar el paquete .msi"></a>
      <br><sub>Para despliegues · 4,6 MB</sub>
    </td>
    <td align="center" valign="top">
      <a href="https://github.com/IzanVil/mosaic/releases/download/v0.1.0/Mosaic_0.1.0_amd64.AppImage"><img src="https://img.shields.io/badge/.AppImage-cualquier%20distro-2c8c8c?style=flat-square" alt="Descargar el .AppImage"></a>
      <br><sub>Sin instalar · 79 MB</sub>
      <br><br>
      <a href="https://github.com/IzanVil/mosaic/releases/download/v0.1.0/Mosaic_0.1.0_amd64.deb"><img src="https://img.shields.io/badge/.deb-Debian%20·%20Ubuntu-555?style=flat-square" alt="Descargar el .deb"></a>
      <a href="https://github.com/IzanVil/mosaic/releases/download/v0.1.0/Mosaic-0.1.0-1.x86_64.rpm"><img src="https://img.shields.io/badge/.rpm-Fedora%20·%20openSUSE-555?style=flat-square" alt="Descargar el .rpm"></a>
      <br><sub>4,6 MB cada uno</sub>
    </td>
  </tr>
</table>

Todas las versiones, con sus notas, están en
[releases](https://github.com/IzanVil/mosaic/releases).

<details>
<summary><b>Cómo instalarlo en cada sistema</b></summary>

<br>

**macOS.** Abre el `.dmg` y arrastra Mosaic a Aplicaciones. Como el binario no
va firmado, la primera vez macOS lo bloquea: ve a *Ajustes del Sistema →
Privacidad y seguridad* y pulsa «Abrir igualmente».

**Windows.** Ejecuta el `.exe`. SmartScreen avisará de que el editor es
desconocido, porque el instalador no va firmado: pulsa «Más información» y
después «Ejecutar de todas formas».

**Linux.**

```bash
# Fedora, openSUSE y derivadas
sudo dnf install ./Mosaic-0.1.0-1.x86_64.rpm

# Debian, Ubuntu y derivadas
sudo apt install ./Mosaic_0.1.0_amd64.deb

# Cualquier distribución, sin instalar nada
chmod +x Mosaic_0.1.0_amd64.AppImage && ./Mosaic_0.1.0_amd64.AppImage
```

</details>

Los binarios no van firmados: firmar cuesta dinero en macOS y en Windows. El
código es abierto, así que siempre puedes [compilarlo tú](#-compilar).

## 🔧 Compilar

Necesitas Rust estable, Node 20 o superior, [pnpm](https://pnpm.io) y
[las dependencias de Tauri 2](https://tauri.app/start/prerequisites/) para tu
sistema.

```bash
pnpm install          # dependencias
pnpm tauri dev        # la app en desarrollo
pnpm tauri build      # binario de release
```

<details>
<summary><b>Dependencias en Fedora</b></summary>

<br>

```bash
sudo dnf install webkit2gtk4.1-devel libsoup3-devel gtk3-devel \
                 openssl-devel curl wget file
```

</details>

<details>
<summary><b>Tests y comprobaciones</b></summary>

<br>

```bash
pnpm check            # svelte-check, falla también con avisos
pnpm test             # tests de vitest

cd src-tauri
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

La lógica de dominio vive en `src-tauri/src/core/` y la cubren los tests
unitarios; los de integración están en `src-tauri/tests/`. Los logs se
controlan con `MOSAIC_LOG` (por ejemplo, `MOSAIC_LOG=debug pnpm tauri dev`).

</details>

## 🧱 Hecho con

<table>
  <tr>
    <td width="33%" valign="top"><b>Backend</b><br><sub>Rust y <a href="https://tauri.app">Tauri 2</a>. SQLite con <code>rusqlite</code>. Repositorios leídos con <code>git2</code>, compilado sin soporte de red.</sub></td>
    <td width="33%" valign="top"><b>Frontend</b><br><sub><a href="https://svelte.dev">Svelte 5</a> con TypeScript estricto y Vite. Búsqueda difusa con <code>fuse.js</code>.</sub></td>
    <td width="33%" valign="top"><b>Diseño</b><br><sub>Tokens propios en oklch, neutros cálidos y acento turquesa. Inter y JetBrains Mono van empaquetadas.</sub></td>
  </tr>
</table>

Para entrar en detalle: [arquitectura](docs/ARCHITECTURE.md) ·
[sistema de diseño](docs/DESIGN.md) · [roadmap](docs/ROADMAP.md).

## 📄 Licencia

[Apache 2.0](LICENSE). Las tipografías empaquetadas mantienen su propia
licencia, la [SIL OFL 1.1](src/lib/assets/fonts/OFL.txt).

<br>

<div align="center">
<sub>Hecho con calma, para quien tiene demasiados proyectos.</sub>
</div>
