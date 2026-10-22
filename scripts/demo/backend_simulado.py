"""Backend simulado de Mosaic para la demostración.

Uso: backend_simulado.py salida.js vista_json [es|en]

Define `window.__TAURI_INTERNALS__` con proyectos, etiquetas y un detalle de
proyecto inventados, para que la interfaz compilada funcione sin Tauri. Los
datos salen en el idioma que se pida, para que las capturas en inglés no
enseñen proyectos con nombre en español.
"""
import json, sys, time
out, view = sys.argv[1], sys.argv[2]
EN = len(sys.argv) > 3 and sys.argv[3] == 'en'
# Nombres en inglés de proyectos y etiquetas; lo que no está aquí se llama igual.
NOMBRES_EN = {'api-facturas':'invoice-api','panel-clientes':'client-portal','analizador-logs':'log-analyzer','api-inventario':'inventory-api','app-recetas':'recipe-app','blog-estatico':'static-blog','bot-telegram':'telegram-bot','calendario-turnos':'shift-calendar','chat-websocket':'websocket-chat','cli-backups':'backup-cli','compilador-toy':'toy-compiler','conversor-divisas':'currency-converter','crm-interno':'internal-crm','ejercicios':'exercises','extension-vscode':'vscode-extension','generador-pdf':'pdf-generator','gestor-tareas':'task-manager','juego-snake':'snake-game','landing-cafe':'cafe-landing','lector-rss':'rss-reader','mapa-rutas':'route-map','migrador-sql':'sql-migrator',
  'cliente':'client','trabajo':'work','urgente':'urgent','aprendizaje':'learning','archivado':'archived'}
def tr(nombre): return NOMBRES_EN.get(nombre, nombre) if EN else nombre
CARPETA = '/home/izan/projects' if EN else '/home/izan/proyectos'
# Las fechas cuelgan del reloj actual: así «hace 3 horas» sale igual cualquier día
# y las capturas no cambian solo porque haya pasado el tiempo.
now = int(time.time())
TAGS = [('cliente','#3B82F6'),('trabajo','#F97316'),('personal','#8B5CF6'),('urgente','#EF4444'),('aprendizaje','#22C55E'),('archivado','#64748B'),('idea','#EAB308'),('web','#06B6D4'),('ml','#84CC16'),('demo','#D946EF')]
tags = [{'id':i+1,'name':tr(n),'color':c,'created_at':now} for i,(n,c) in enumerate(TAGS)]
T = {n:t for (n,_),t in zip(TAGS,tags)}
P = [('api-facturas','Rust',0,1,['cliente','trabajo','urgente']),('mosaic','Svelte',1,1,['personal']),('panel-clientes','Svelte',1,1,['cliente','web']),('analizador-logs','Python',0,0,['idea']),('api-inventario','Java',0,0,['cliente','trabajo']),('app-recetas','Svelte',0,0,['personal','web']),('blog-estatico','JavaScript',1,0,['archivado']),('bot-telegram','Python',0,0,['demo']),('calendario-turnos','Svelte',1,0,['urgente']),('chat-websocket','Go',0,0,['aprendizaje']),('cli-backups','Rust',1,0,['personal']),('compilador-toy','Rust',0,0,['aprendizaje','idea','ml']),('conversor-divisas','Python',0,0,['ml']),('crm-interno','Ruby',0,0,['cliente','trabajo','urgente']),('dotfiles','JavaScript',1,0,['personal']),('ejercicios','Java',1,0,['aprendizaje']),('extension-vscode','TypeScript',0,0,['demo','idea']),('generador-pdf','TypeScript',1,0,['trabajo']),('gestor-tareas','TypeScript',0,0,['web']),('juego-snake','Rust',0,0,['personal','aprendizaje']),('landing-cafe','JavaScript',0,0,['cliente','web']),('lector-rss','Rust',0,0,['personal']),('mapa-rutas','TypeScript',0,0,['ml']),('migrador-sql','Go',1,0,['trabajo'])]
NOTES = {'panel-clientes': 'To do: confirm the invoice format with the client before Friday.' if EN else 'Pendiente: confirmar con el cliente el formato de las facturas antes del viernes.'}
projects = []
for i,(n,l,d,pin,tg) in enumerate(P):
    pid = i+1
    projects.append({'id':pid,'name':tr(n),'path':f'{CARPETA}/{tr(n)}','is_git_repo':True,'primary_language':l,'last_opened_at':now-3600*(i+1),'pinned':bool(pin),'notes':NOTES.get(n),'missing':False,'last_seen_at':now,'created_at':now,'updated_at':now,'tags':[T[x] for x in tg],
      'git_status':{'project_id':pid,'refreshed_at':now,'branch':'main','ahead':1 if i%5==0 else 0,'behind':0,'is_dirty':bool(d),'last_commit_sha':'abc1234','last_commit_msg':'x','last_commit_at':now-500,'remote_url':None}})
counts = {t['id']:sum(1 for p in projects for x in p['tags'] if x['id']==t['id']) for t in tags}
adv = {"max_depth":4,"excluded_dirs":[".git","node_modules","target"],"max_entries_per_scan":50000,"scan_on_startup":False,"git_refresh_interval_minutes":5}

README_EN = ("<h1>client-portal</h1><p>Web portal where each client sees the status of their orders, their invoices and messages from the team.</p>"
    "<h2>Running locally</h2><pre><code>pnpm install\npnpm dev</code></pre><h2>What's inside</h2><ul>"
    "<li><strong>Orders</strong>: list with filters by status and date.</li>"
    "<li><strong>Invoices</strong>: PDF download from the billing API.</li>"
    "<li><strong>Messages</strong>: a thread per order, with email notifications.</li></ul>"
    "<h2>Deployment</h2><p>Every push to <code>main</code> goes to staging on its own. Production is published by hand from the releases page.</p>")
README = {"file_name": "README.md", "truncated": False, "images_omitted": 0, "badges_omitted": 0, "media_omitted": 0, "html": README_EN if EN else
    "<h1>panel-clientes</h1><p>Panel web para que cada cliente vea el estado de sus encargos, sus facturas y los mensajes del equipo.</p>"
    "<h2>Arrancar en local</h2><pre><code>pnpm install\npnpm dev</code></pre><h2>Qué hay dentro</h2><ul>"
    "<li><strong>Encargos</strong>: lista con filtros por estado y fecha.</li>"
    "<li><strong>Facturas</strong>: descarga en PDF desde la API de facturación.</li>"
    "<li><strong>Mensajes</strong>: hilo por encargo, con avisos por correo.</li></ul>"
    "<h2>Despliegue</h2><p>Cada push a <code>main</code> se publica solo en el entorno de pruebas. Producción se publica a mano desde la página de releases.</p>"}
HISTORY = [{"sha": "a1b2c3d4e5f6", "short_sha": "a1b2c3d", "summary": s, "author_name": "Izan", "committed_at": now - i * 5400} for i, s in enumerate([
    "feat: status filters in the order list", "fix: invoice totals rounded the cents wrong",
    "feat: download invoices as PDF", "refactor: split out the API client", "docs: how to run locally",
    "feat: message thread per order", "fix: the session expired when switching tabs", "chore: update dependencies"] if EN else [
    "feat: filtros por estado en la lista de encargos", "fix: el total de la factura redondeaba mal los céntimos",
    "feat: descarga de facturas en PDF", "refactor: separar el cliente de la API", "docs: cómo arrancar en local",
    "feat: hilo de mensajes por encargo", "fix: la sesión caducaba al cambiar de pestaña", "chore: actualizar dependencias"])]
BRANCHES = {"local": [{"name": "main", "is_head": True}, {"name": "order-filters" if EN else "filtros-encargos", "is_head": False}], "remote": [{"name": "origin/main", "is_head": False}]}
# Lo que enseña «Importar…» en Ajustes: el diálogo devuelve una ruta y la copia trae un poco de todo.
PLAN = {"exported_at": now - 86400, "app_version": "0.2.1", "has_work": True, "settings_changed": ["max_depth", "preferred_ide"],
        "git_interval_changed": False, "scan_paths_added": 1, "scan_paths_missing": ["/home/other/work" if EN else "/home/otro/trabajo"],
        "tags_added": 3, "tags_existing": 1, "projects_matched": 5, "projects_not_found": 2, "pins_added": 2,
        "assignments_added": 7, "notes_added": 3, "notes_conflicts": ["mosaic"]}
R = {"get_view_state": view, "list_projects_with_tags": projects, "list_scan_paths": [{"id":1,"path":CARPETA,"enabled":True,"created_at":now,"last_scan_at":now}],
     "list_tags": [dict(t, project_count=counts[t['id']]) for t in tags], "list_detected_apps": [{"id":"code","name":"VS Code","kind":"ide"},{"id":"konsole","name":"Konsole","kind":"terminal"}],
     "get_preferred_apps": {"ide":"","terminal":""}, "list_git_status": [p['git_status'] for p in projects],
     "get_advanced_settings": {"current":adv,"defaults":adv,"limits":{"max_depth_min":1,"max_depth_max":8,"max_entries_min":1000,"max_entries_max":500000,"git_refresh_max_minutes":60,"excluded_dirs_max":100,"excluded_dir_max_chars":255}},
     "refresh_all_git_status": {"refreshed":0,"failed":0,"elapsed_ms":1,"changed":[],"removed":[]},
     "get_project_readme": README, "get_project_history": HISTORY, "get_project_branches": BRANCHES,
     "plugin:dialog|open": "/home/izan/mosaic-backup.json" if EN else "/home/izan/mosaic-copia.json", "preview_import": PLAN}
js = ("(function(){var R=%s;var n=0;var V=new URLSearchParams(location.search).get('vista');if(V)R.get_view_state=V;window.__TAURI_INTERNALS__={transformCallback:function(){return ++n;},"
      "invoke:function(c,a){if(c in R)return Promise.resolve(R[c]);if(c.indexOf('plugin:')===0)return Promise.resolve(1);"
      "if(c==='set_view_state')return Promise.resolve(null);"
      "if(c==='set_project_notes')return Promise.resolve(a.notes);"
      "if(c in R)return Promise.resolve(R[c]);return Promise.reject('no simulado: '+c);},"
      "metadata:{currentWindow:{label:'main'},currentWebview:{label:'main'}}};"
      "window.__TAURI_EVENT_PLUGIN_INTERNALS__={unregisterListener:function(){}};})();") % json.dumps(R)
open(out,'w').write(js)
