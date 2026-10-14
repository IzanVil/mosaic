#!/usr/bin/env bash
# Rehace las capturas fijas del README (docs/img) y, si se le pasa la carpeta de
# imágenes de la web, también las de la web. Necesita lo mismo que hacer-demo.sh.
#   ./scripts/demo/hacer-capturas.sh [../mosaic-web/site/assets/img]
# Con MOSAIC_IDIOMA=en salen en inglés, con datos en inglés, en docs/img/en.
set -euo pipefail

raiz=$(cd "$(dirname "$0")/../.." && pwd)
aqui="$raiz/scripts/demo"
web=${1:+$(cd "$1" && pwd)}
idioma=${MOSAIC_IDIOMA:-es}
img="$raiz/docs/img"
[ "$idioma" = en ] && img="$img/en"
mkdir -p "$img"
tmp=$(mktemp -d)
puerto=${PUERTO:-8851}
trap 'kill "$servidor" 2>/dev/null || true; rm -rf "$tmp" "$img/.capturas"' EXIT

(cd "$raiz" && pnpm build >/dev/null)
cp -r "$raiz/dist" "$tmp/sitio"
cp "$aqui/ventana.html" "$tmp/sitio/"
python3 "$aqui/backend_simulado.py" "$tmp/sitio/mock.js" '{}' "$idioma"
sed -i 's|<script type="module"|<script src="/mock.js"></script>\n<script type="module"|' "$tmp/sitio/index.html"

python3 -m http.server "$puerto" --directory "$tmp/sitio" >/dev/null 2>&1 &
servidor=$!
sleep 1

WEBKIT_DISABLE_COMPOSITING_MODE=1 WEBKIT_DISABLE_DMABUF_RENDERER=1 \
  MOSAIC_IDIOMA="$idioma" python3 "$aqui/capturas.py" "http://localhost:$puerto" "$img" ${web:+"$web"}
