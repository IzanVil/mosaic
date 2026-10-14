"""Saca las capturas fijas del README y de la web.

Uso: capturas.py URL_BASE CARPETA_README [CARPETA_WEB]

URL_BASE sirve el frontend compilado con `ventana.html` y el backend simulado.
Cada captura carga la ventana con una vista guardada, hace sus pasos y guarda
la imagen; los recortes salen de la posición real de los elementos.
"""

import json
import os
import sys
from urllib.parse import quote

import gi

gi.require_version("Gtk", "3.0")
gi.require_version("WebKit2", "4.1")
from gi.repository import GLib, Gtk, WebKit2  # noqa: E402
from PIL import Image, ImageDraw  # noqa: E402

base, carpeta_readme = sys.argv[1], sys.argv[2]
carpeta_web = sys.argv[3] if len(sys.argv) > 3 else None
tmp = os.path.join(carpeta_readme, ".capturas")
os.makedirs(tmp, exist_ok=True)

CLIENTE, TRABAJO = 1, 2
FILTROS = {
    "query": "api",
    "tag_ids": [CLIENTE, TRABAJO],
    "languages": ["Rust", "Java"],
    "include_missing": False,
}

# Fuerza a la vista el «+» de una tarjeta, que solo aparece con el ratón encima.
ABRIR_SELECTOR = (
    "const d=doc();const s=d.createElement('style');"
    "s.textContent='article:has([data-proyecto=\"1\"]) .anadir{display:inline-flex!important}';"
    "d.head.appendChild(s);"
    "d.querySelector('[data-proyecto=\"1\"]').closest('article').querySelector('.disparador').click();"
)
RECORTE_SELECTOR = (
    "JSON.stringify((()=>{const a=rect('article:has([data-proyecto=\"1\"])'),p=rect('article:has([data-proyecto=\"1\"]) [role=dialog]');"
    "const x=Math.min(a.x,p.x)-16,y=Math.min(a.y,p.y)-16;"
    "return {x,y,w:Math.max(a.x+a.w,p.x+p.w)+16-x,h:Math.max(a.y+a.h,p.y+p.h)+16-y}})())"
)
RECORTE_GESTOR = (
    "JSON.stringify((()=>{const m=rect('.modal');return {x:m.x-20,y:m.y-20,w:m.w+40,h:m.h+40}})())"
)
AJUSTES = "clic('nav.pestanas button:nth-of-type(2)')"
# La copia: el diálogo simulado devuelve una ruta y Ajustes enseña lo que haría importarla.
IMPORTAR = "clic('section[aria-labelledby=titulo-copia] .acciones button:nth-of-type(2)')"
VER_COPIA = "doc().querySelector('section[aria-labelledby=titulo-copia]').scrollIntoView({block:'center'})"
RECORTE_COPIA = (
    "JSON.stringify((()=>{const s=rect('section[aria-labelledby=titulo-copia]'),t=rect('#titulo-copia');"
    "return {x:s.x-24,y:t.y-16,w:s.w+48,h:s.y+s.h-t.y+40}})())"
)
PALETA = (
    "const w=doc().defaultView;"
    "w.dispatchEvent(new w.KeyboardEvent('keydown',{key:'k',ctrlKey:true,bubbles:true}));"
)
RECORTE_PALETA = (
    "JSON.stringify((()=>{const p=rect('[role=dialog].paleta');return {x:p.x-16,y:p.y-16,w:p.w+32,h:p.h+32}})())"
)


# Idioma de las capturas: fijo, para que no dependa del sistema de quien las saca.
IDIOMA = os.environ.get("MOSAIC_IDIOMA", "es")


def vista(tema, **extra):
    v = {"density": "comodo", "theme": tema, "sidebar_collapsed": False, "lang": IDIOMA}
    v.update(extra)
    return v


def trabajos():
    for tema in ("dark", "light"):
        nombre = "oscuro" if tema == "dark" else "claro"
        yield f"tablero-{nombre}", 1280, 800, False, vista(tema), [], None
        yield f"filtros-{nombre}", 1280, 800, False, vista(tema, filters=FILTROS), [], None
        yield f"gestor-{nombre}", 1280, 800, False, vista(tema), ["clic('button.fila.gestionar')"], RECORTE_GESTOR
        yield f"selector-{nombre}", 1280, 800, False, vista(tema), [ABRIR_SELECTOR], RECORTE_SELECTOR
        yield f"marco-{nombre}", 1254, 784, True, vista(tema), [], None
    yield "filtros-readme", 1320, 840, False, vista("dark", filters=FILTROS), [], None
    yield "paleta", 1280, 890, False, vista("dark"), [PALETA], RECORTE_PALETA
    yield "copia", 1280, 800, False, vista("dark"), [AJUSTES, IMPORTAR, VER_COPIA], RECORTE_COPIA
    yield "compacto", 1370, 890, False, vista("light", density="compacto"), [], None


cola = list(trabajos())
estado = {}


def js(codigo, despues=None):
    def fin(v, res):
        r = v.evaluate_javascript_finish(res)
        if despues:
            despues(r.to_string() if r is not None else "")

    estado["view"].evaluate_javascript(codigo, -1, None, None, None, fin)


def siguiente():
    if estado.get("win"):
        estado["win"].destroy()
    if not cola:
        Gtk.main_quit()
        return False
    nombre, an, al, marco, v, pasos_, recorte = cola.pop(0)
    ancho, alto = (1600, 940) if marco else (an, al)
    # Una ventana nueva por captura: una ventana sin pantalla no siempre encoge al cambiar de tamaño.
    win = Gtk.OffscreenWindow()
    view = WebKit2.WebView()
    win.add(view)
    win.set_default_size(ancho, alto)
    win.show_all()
    estado.update(nombre=nombre, pasos=list(pasos_), recorte=recorte, win=win, view=view)
    view.connect(
        "load-changed",
        lambda v, ev: ev == WebKit2.LoadEvent.FINISHED and GLib.timeout_add(2500, pasos),
    )
    view.load_uri(f"{base}/ventana.html?an={an}&al={al}&marco={1 if marco else 0}&vista={quote(json.dumps(v))}")
    return False


def pasos():
    if estado["pasos"]:
        paso = estado["pasos"].pop(0)
        js(
            "window.clic=(s)=>{const e=doc().querySelector(s);e.click();return 'ok'};" + paso,
            lambda _: GLib.timeout_add(900, pasos),
        )
    elif estado["recorte"]:
        js(estado["recorte"], lambda r: foto(json.loads(r)))
    else:
        foto(None)
    return False


def foto(recorte):
    def hecha(v, res):
        ruta = os.path.join(tmp, estado["nombre"] + ".png")
        v.get_snapshot_finish(res).write_to_png(ruta)
        if recorte:
            im = Image.open(ruta)
            caja = (round(recorte["x"]), round(recorte["y"]), round(recorte["x"] + recorte["w"]), round(recorte["y"] + recorte["h"]))
            im.crop(caja).save(ruta)
        print(estado["nombre"], Image.open(ruta).size)
        GLib.idle_add(siguiente)

    estado["view"].get_snapshot(WebKit2.SnapshotRegion.VISIBLE, WebKit2.SnapshotOptions.NONE, None, hecha)


GLib.timeout_add_seconds(240, lambda: (print("TIEMPO AGOTADO"), Gtk.main_quit()))
GLib.idle_add(siguiente)
Gtk.main()


def abrir(nombre):
    return Image.open(os.path.join(tmp, nombre + ".png")).convert("RGB")


def diagonal(oscuro, claro):
    """Mitad oscura a la izquierda y clara a la derecha, partidas en diagonal."""
    an, al = oscuro.size
    mascara = Image.new("L", (an, al), 0)
    ImageDraw.Draw(mascara).polygon([(an * 0.62, 0), (an, 0), (an, al), (an * 0.38, al)], fill=255)
    mezcla = Image.composite(claro, oscuro, mascara)
    ImageDraw.Draw(mezcla).line([(an * 0.62, 0), (an * 0.38, al)], fill=(45, 212, 191), width=3)
    return mezcla


abrir("filtros-readme").save(os.path.join(carpeta_readme, "filtros.png"), optimize=True)
abrir("marco-oscuro").save(os.path.join(carpeta_readme, "portada-oscura.png"), optimize=True)
abrir("marco-claro").save(os.path.join(carpeta_readme, "portada-clara.png"), optimize=True)
abrir("paleta").save(os.path.join(carpeta_readme, "paleta.png"), optimize=True)
abrir("copia").save(os.path.join(carpeta_readme, "copia.png"), optimize=True)
abrir("compacto").save(os.path.join(carpeta_readme, "compacto.png"), optimize=True)
abrir("gestor-oscuro").save(os.path.join(carpeta_readme, "gestor-etiquetas.png"), optimize=True)
abrir("selector-oscuro").save(os.path.join(carpeta_readme, "selector-etiquetas.png"), optimize=True)
diagonal(abrir("marco-oscuro"), abrir("marco-claro")).save(os.path.join(carpeta_readme, "temas.png"), optimize=True)

if carpeta_web:
    for nombre in ("tablero", "filtros", "gestor", "selector"):
        for tema in ("claro", "oscuro"):
            abrir(f"{nombre}-{tema}").save(os.path.join(carpeta_web, f"{nombre}-{tema}.webp"), quality=88, method=6)
    abrir("marco-oscuro").save(os.path.join(carpeta_web, "og.jpg"), quality=86, optimize=True)
