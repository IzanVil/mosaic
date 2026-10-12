"""Mide el tablero con muchos proyectos en WebKitGTK, sin pantalla.

Uso: frontend.py URL N [N ...]

URL sirve el frontend compilado con `perf.html` al lado (lo prepara
`medir.sh`). Para cada N carga el tablero con N proyectos inventados y mide:
cuánto tarda en pintar todas las tarjetas, cuánto tarda la búsqueda en
responder (sin contar los 150 ms de espera a que dejes de teclear) y la
memoria del proceso que pinta la página.
"""

import json
import os
import sys

import gi

gi.require_version("Gtk", "3.0")
gi.require_version("WebKit2", "4.1")
from gi.repository import GLib, Gtk, WebKit2  # noqa: E402

url, tamanos = sys.argv[1], [int(n) for n in sys.argv[2:]]
resultados = []

# Escribe en la búsqueda y mide hasta que el número de tarjetas cambia y el
# navegador vuelve a pintar. Devuelve el resultado en el título de la página.
MEDIR = """
(async () => {
  const doc = document;
  // Cada proyecto es una celda con `data-proyecto`: la tarjeta montada o su hueco.
  const tarjetas = () => doc.querySelectorAll('[data-proyecto]').length;
  const montadas = () => doc.querySelectorAll('article.tarjeta').length;
  const total = window.__N;
  const t0 = performance.now();
  while (tarjetas() < total && performance.now() - t0 < 60000) await new Promise(r => setTimeout(r, 20));
  const pintado = performance.now() - window.__inicio;
  await new Promise(r => setTimeout(r, 500));
  const buscar = async (texto) => {
    const antes = tarjetas();
    const input = doc.getElementById('busqueda');
    const set = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set;
    const t = performance.now();
    set.call(input, texto);
    input.dispatchEvent(new Event('input', { bubbles: true }));
    while (tarjetas() === antes && performance.now() - t < 20000) await new Promise(r => requestAnimationFrame(r));
    await new Promise(r => requestAnimationFrame(r));
    return { ms: performance.now() - t - 150, quedan: tarjetas() };
  };
  const b1 = await buscar('facturas');
  const b2 = await buscar('');
  document.title = 'R' + JSON.stringify({ pintado, busqueda: b1, vuelta: b2, nodos: doc.getElementsByTagName('*').length, montadas: montadas() });
})();
"""


def memoria_kb():
    total = 0
    for pid in os.listdir("/proc"):
        if not pid.isdigit():
            continue
        try:
            nombre = open(f"/proc/{pid}/comm").read().strip()
            padre = int(open(f"/proc/{pid}/stat").read().split()[3])
            if nombre.startswith("WebKitWebProces") and padre == os.getpid():
                for linea in open(f"/proc/{pid}/status"):
                    if linea.startswith("VmRSS"):
                        total += int(linea.split()[1])
        except (OSError, ValueError):
            pass
    return total


def medir(n):
    win = Gtk.OffscreenWindow()
    view = WebKit2.WebView()
    win.add(view)
    win.set_default_size(1370, 890)
    win.show_all()

    def titulo(v, _):
        t = v.get_title() or ""
        if t.startswith("R"):
            r = json.loads(t[1:])
            r["n"] = n
            r["memoria_mb"] = round(memoria_kb() / 1024)
            resultados.append(r)
            win.destroy()
            GLib.idle_add(siguiente)

    view.connect("notify::title", titulo)
    view.connect(
        "load-changed",
        lambda v, ev: ev == WebKit2.LoadEvent.FINISHED
        and v.evaluate_javascript(MEDIR, -1, None, None, None, None, None),
    )
    view.load_uri(f"{url}/perf.html?n={n}")


def siguiente():
    if not tamanos:
        Gtk.main_quit()
        return False
    medir(tamanos.pop(0))
    return False


GLib.timeout_add_seconds(600, lambda: (print("TIEMPO AGOTADO"), Gtk.main_quit()))
GLib.idle_add(siguiente)
Gtk.main()
for r in resultados:
    print(
        f"{r['n']:>5} proyectos: tablero pintado en {r['pintado']:.0f} ms, {r['nodos']} nodos, "
        f"{r['montadas']} tarjetas montadas, búsqueda {r['busqueda']['ms']:.0f} ms (quedan {r['busqueda']['quedan']}), "
        f"volver a todos {r['vuelta']['ms']:.0f} ms, memoria {r['memoria_mb']} MB"
    )
