// Backend simulado con N proyectos inventados (perf.html?n=N), para medir el
// tablero sin Tauri. Etiquetas, lenguajes y estado de Git variados, como en uso real.
(function () {
  window.__inicio = 0;
  var n = Number(new URLSearchParams(location.search).get('n') || 500);
  window.__N = n;
  var colores = ['#3B82F6', '#F97316', '#8B5CF6', '#EF4444', '#22C55E', '#64748B', '#EAB308', '#06B6D4'];
  var nombres = ['cliente', 'trabajo', 'personal', 'urgente', 'aprendizaje', 'archivado', 'idea', 'web'];
  var tags = nombres.map(function (nombre, i) { return { id: i + 1, name: nombre, color: colores[i], created_at: 0 }; });
  var lenguajes = ['Rust', 'TypeScript', 'Python', 'Go', 'Java', 'Svelte'];
  var ahora = 1791470000;
  var proyectos = [];
  for (var i = 0; i < n; i++) {
    var id = i + 1;
    var etiquetas = tags.filter(function (t) { return (i + t.id) % 4 === 0; });
    proyectos.push({
      id: id, name: (i % 50 === 7 ? 'facturas-' : 'proyecto-') + String(i).padStart(5, '0'),
      path: '/home/izan/proyectos/grupo-' + (i % 40) + '/proyecto-' + String(i).padStart(5, '0'),
      is_git_repo: i % 10 < 7, primary_language: lenguajes[i % lenguajes.length],
      last_opened_at: i % 3 ? ahora - i * 600 : null, pinned: i < 6, notes: null, missing: i % 97 === 0,
      last_seen_at: ahora, created_at: ahora - i, updated_at: ahora - i, tags: etiquetas,
      git_status: i % 10 < 7 ? { project_id: id, refreshed_at: ahora, branch: 'main', ahead: i % 7 ? 0 : 2, behind: 0,
        is_dirty: i % 5 === 0, last_commit_sha: 'abc1234', last_commit_msg: 'x', last_commit_at: ahora, remote_url: null } : null,
    });
  }
  var R = {
    get_view_state: JSON.stringify({ density: 'comodo', theme: 'dark', sidebar_collapsed: false }),
    list_projects_with_tags: proyectos,
    list_scan_paths: [{ id: 1, path: '/home/izan/proyectos', enabled: true, created_at: ahora, last_scan_at: ahora }],
    list_tags: tags.map(function (t) { return Object.assign({ project_count: 0 }, t); }),
    list_detected_apps: [], get_preferred_apps: { ide: '', terminal: '' },
    list_git_status: proyectos.filter(function (p) { return p.git_status; }).map(function (p) { return p.git_status; }),
    refresh_all_git_status: { refreshed: 0, failed: 0, elapsed_ms: 1 },
  };
  var k = 0;
  window.__TAURI_INTERNALS__ = {
    transformCallback: function () { return ++k; },
    invoke: function (c) {
      if (c.indexOf('plugin:') === 0 || c === 'set_view_state') return Promise.resolve(null);
      if (c in R) return Promise.resolve(R[c]);
      return Promise.reject('no simulado: ' + c);
    },
    metadata: { currentWindow: { label: 'main' }, currentWebview: { label: 'main' } },
  };
  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener: function () {} };
})();
