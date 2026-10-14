#!/usr/bin/env bash
# Rehace docs/img/demo.gif con la interfaz actual. Necesita python3 con
# WebKit2 4.1 (gi), ffmpeg y pnpm. No abre ninguna ventana.
# Con MOSAIC_IDIOMA=en sale en inglés, con datos en inglés, en docs/img/en.
set -euo pipefail

raiz=$(cd "$(dirname "$0")/../.." && pwd)
aqui="$raiz/scripts/demo"
tmp=$(mktemp -d)
idioma=${MOSAIC_IDIOMA:-es}
gif="$raiz/docs/img/demo.gif"
[ "$idioma" = en ] && gif="$raiz/docs/img/en/demo.gif"
mkdir -p "$(dirname "$gif")"
puerto=${PUERTO:-8850}
trap 'kill "$servidor" 2>/dev/null || true; rm -rf "$tmp"' EXIT

(cd "$raiz" && pnpm build >/dev/null)
cp -r "$raiz/dist" "$tmp/sitio"
cp "$aqui/marco.html" "$tmp/sitio/"
python3 "$aqui/backend_simulado.py" "$tmp/sitio/mock.js" \
  "{\"density\":\"comodo\",\"theme\":\"dark\",\"sidebar_collapsed\":false,\"lang\":\"$idioma\"}" "$idioma"
# El backend simulado tiene que existir antes que la aplicación.
sed -i 's|<script type="module"|<script src="/mock.js"></script>\n<script type="module"|' "$tmp/sitio/index.html"

python3 -m http.server "$puerto" --directory "$tmp/sitio" >/dev/null 2>&1 &
servidor=$!
sleep 1

# Sin aceleración: WebKit sin pantalla no consigue contexto de OpenGL.
WEBKIT_DISABLE_COMPOSITING_MODE=1 WEBKIT_DISABLE_DMABUF_RENDERER=1 MOSAIC_IDIOMA="$idioma" \
  python3 "$aqui/grabar.py" "http://localhost:$puerto/marco.html" "$tmp/fotos"

python3 - "$tmp/fotos" <<'PY'
import json, os, sys
d = sys.argv[1]
f = json.load(open(os.path.join(d, 'fotogramas.json')))
with open(os.path.join(d, 'lista.txt'), 'w') as o:
    for p, s in f:
        o.write(f"file '{os.path.abspath(p)}'\nduration {s}\n")
    o.write(f"file '{os.path.abspath(f[-1][0])}'\n")
PY

ffmpeg -v error -y -f concat -safe 0 -i "$tmp/fotos/lista.txt" \
  -vf "fps=12,scale=960:-1:flags=lanczos,split[a][b];[a]palettegen=max_colors=192:stats_mode=diff[p];[b][p]paletteuse=dither=bayer:bayer_scale=4:diff_mode=rectangle" \
  -loop 0 "$gif"

echo "${gif#"$raiz/"}: $(du -h "$gif" | cut -f1)"
