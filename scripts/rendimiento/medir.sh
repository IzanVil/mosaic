#!/usr/bin/env bash
# Mide el rendimiento de Mosaic con miles de proyectos. Ver docs/RENDIMIENTO.md.
#   ./scripts/rendimiento/medir.sh [CARPETA_DE_PROYECTOS]
# Sin carpeta mide solo el frontend; con ella, también el backend (los
# proyectos se generan con generar.py).
set -euo pipefail

raiz=$(cd "$(dirname "$0")/../.." && pwd)
aqui="$raiz/scripts/rendimiento"
tmp=$(mktemp -d)
puerto=${PUERTO:-8852}
trap 'kill "$servidor" 2>/dev/null || true; rm -rf "$tmp"' EXIT

if [ $# -gt 0 ]; then
  (cd "$raiz/src-tauri" && MOSAIC_RENDIMIENTO=$(cd "$1" && pwd) \
    cargo test --release --test rendimiento -- --ignored --nocapture 2>/dev/null | grep '^RAICES')
fi

(cd "$raiz" && pnpm build >/dev/null)
cp -r "$raiz/dist" "$tmp/sitio"
cp "$aqui/backend-simulado.js" "$tmp/sitio/"
sed 's|<script type="module"|<script src="/backend-simulado.js"></script>\n<script type="module"|' \
  "$tmp/sitio/index.html" > "$tmp/sitio/perf.html"

python3 -m http.server "$puerto" --directory "$tmp/sitio" >/dev/null 2>&1 &
servidor=$!
sleep 1
WEBKIT_DISABLE_COMPOSITING_MODE=1 WEBKIT_DISABLE_DMABUF_RENDERER=1 \
  python3 "$aqui/frontend.py" "http://localhost:$puerto" 24 500 2000 5000
