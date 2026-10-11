"""Graba el recorrido de demostración.

Uso: grabar.py URL_DE_marco.html CARPETA

Carga la página en WebKitGTK sin pantalla, ejecuta los pasos de abajo y guarda
un fotograma por paso con su duración en CARPETA/fotogramas.json.
"""
import gi, sys, json, os
gi.require_version('Gtk', '3.0'); gi.require_version('WebKit2', '4.1')
from gi.repository import Gtk, WebKit2, GLib
url, out = sys.argv[1], sys.argv[2]
os.makedirs(out, exist_ok=True)
win = Gtk.OffscreenWindow(); view = WebKit2.WebView(); win.add(view); win.set_default_size(1280, 800); win.show_all()

# (js, espera_ms antes de la foto, duración del fotograma en s)
P = []
def paso(js, espera=250, dura=0.1): P.append((js, espera, dura))
def mover(sel, dura_final=0.25):
    paso(f"mover({json.dumps(sel)})", 110, 0.11); paso("", 110, 0.11); paso("", 140, dura_final)
def escribir(sel, texto, por_letra=0.16):
    for i in range(1, len(texto) + 1):
        paso(f"escribir({json.dumps(sel)},{json.dumps(texto[:i])})", 200, por_letra)

paso("", 400, 1.8)                                   # el tablero
mover("#busqueda")
paso("buscar('#busqueda').focus()", 100, 0.2)
escribir("#busqueda", "api")
paso("", 350, 1.6)                                   # dos resultados
escribir("#busqueda", "")
paso("escribir('#busqueda','');buscar('#busqueda').blur()", 350, 0.5)
mover("etiqueta:cliente")
paso("clic('etiqueta:cliente')", 350, 1.8)           # filtrado por etiqueta
paso("clic('etiqueta:cliente')", 350, 0.6)
paso("aparcar()", 110, 0.11); paso("", 220, 0.2)
paso("teclas('Ctrl','K');tecla('k',true)", 400, 1.0) # la paleta
paso("teclas()", 150, 0.3)
escribir("input[placeholder^=\"Busca un\"]", "panel", 0.15)
paso("", 300, 1.1)
paso("teclas('Enter');tecla('Enter')", 900, 0.9)     # el detalle
paso("teclas()", 300, 3.4)
paso("teclas('Esc');tecla('Escape')", 500, 0.7)
paso("teclas()", 300, 0.6)
mover(".tema")
paso("clic('.tema')", 450, 2.2)                      # tema claro
mover(".densidad")
paso("clic('.densidad')", 450, 2.4)                  # modo compacto

frames = []
def correr(i):
    if i == len(P):
        json.dump(frames, open(os.path.join(out, 'fotogramas.json'), 'w')); Gtk.main_quit(); return False
    js, espera, dura = P[i]
    def foto():
        def done(v, res):
            f = os.path.join(out, f'{i:04d}.png'); v.get_snapshot_finish(res).write_to_png(f)
            frames.append([f, dura]); GLib.idle_add(correr, i + 1)
        view.get_snapshot(WebKit2.SnapshotRegion.VISIBLE, WebKit2.SnapshotOptions.NONE, None, done)
        return False
    def tras_js(v, res):
        r = v.evaluate_javascript_finish(res)
        val = r.to_string() if r is not None else ''
        if val.startswith('NO '): print('FALLA', i, val)
        GLib.timeout_add(espera, foto)
    if js: view.evaluate_javascript(js, -1, None, None, None, tras_js)
    else: GLib.timeout_add(espera, foto)
    return False
view.connect('load-changed', lambda v, ev: ev == WebKit2.LoadEvent.FINISHED and GLib.timeout_add(2500, correr, 0))
GLib.timeout_add_seconds(120, lambda: (print('TIMEOUT'), Gtk.main_quit()))
view.load_uri(url); Gtk.main()
