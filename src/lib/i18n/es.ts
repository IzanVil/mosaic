/**
 * Textos en español. Es el diccionario de referencia: `en.ts` tiene que tener
 * exactamente las mismas claves, y si falta alguna no compila.
 *
 * Los textos con números son funciones, para que cada idioma resuelva sus
 * plurales.
 */

type N = { n: number };

export const es = {
  // Tiempo relativo
  'tiempo.nunca': 'nunca',
  'tiempo.momento': 'hace un momento',
  'tiempo.minutos': ({ n }: N) => `hace ${n} ${n === 1 ? 'minuto' : 'minutos'}`,
  'tiempo.horas': ({ n }: N) => `hace ${n} ${n === 1 ? 'hora' : 'horas'}`,
  'tiempo.dias': ({ n }: N) => `hace ${n} ${n === 1 ? 'día' : 'días'}`,
  'tiempo.meses': ({ n }: N) => `hace ${n} ${n === 1 ? 'mes' : 'meses'}`,
  'tiempo.anios': ({ n }: N) => `hace ${n} ${n === 1 ? 'año' : 'años'}`,

  // Cabecera
  'app.secciones': 'Secciones',
  'app.proyectos': 'Proyectos',
  'app.ajustes': 'Ajustes',
  'app.numProyectos': ({ n }: N) => `${n} ${n === 1 ? 'proyecto' : 'proyectos'}`,
  'app.acciones': 'Acciones',
  'app.git': 'Git',
  'app.gitLeyendo': 'Leyendo…',
  'app.gitAyuda':
    'Relee del disco la rama, los cambios sin commitear y el último commit de cada repositorio. No hace fetch: no toca la red. También se refresca solo cada 5 minutos.',
  'app.gitLeyendoRepos': 'Leyendo los repositorios…',
  'app.gitLeidos': ({ n }: N) => `${n} ${n === 1 ? 'repositorio leído' : 'repositorios leídos'}`,
  'app.gitSinCambios': 'sin cambios',
  'app.gitActualizados': ({ n }: N) => `${n} ${n === 1 ? 'actualizado' : 'actualizados'}`,
  'app.gitIlegibles': ({ n }: N) => `${n} ${n === 1 ? 'ilegible' : 'ilegibles'}`,
  'app.gitLeidoHace': ({ cuando }: { cuando: string }) => `Git leído ${cuando}`,
  'app.escanear': 'Escanear',
  'app.escaneando': 'Escaneando…',
  'app.escanearAyuda': 'Recorrer las rutas configuradas y actualizar la lista de proyectos',
  'app.escanearSinRutas': 'Añade una carpeta en Ajustes para poder escanear',
  'app.temaOscuro': 'Tema oscuro',
  'app.temaClaro': 'Tema claro',
  'app.temaSistema': 'Tema del sistema',
  'app.temaSiguienteOscuro': 'oscuro',
  'app.temaSiguienteClaro': 'claro',
  'app.temaSiguienteSistema': 'el del sistema',
  'app.temaTitulo': ({ actual, siguiente }: { actual: string; siguiente: string }) =>
    `${actual}. Pulsa para pasar a ${siguiente}.`,
  'app.temaEtiqueta': ({ actual, siguiente }: { actual: string; siguiente: string }) =>
    `${actual}. Cambiar a ${siguiente}`,

  // Acciones de la paleta
  'paleta.git': 'Releer el estado de Git',
  'paleta.gitClaves': 'refrescar actualizar',
  'paleta.escanear': 'Escanear las rutas',
  'paleta.escanearClaves': 'buscar proyectos actualizar',
  'paleta.proyectos': 'Ir a Proyectos',
  'paleta.proyectosClaves': 'tablero inicio',
  'paleta.ajustes': 'Ir a Ajustes',
  'paleta.ajustesClaves': 'preferencias configuración',
  'paleta.etiquetas': 'Gestionar etiquetas',
  'paleta.etiquetasClaves': 'tags colores',
  'paleta.tema': 'Cambiar el tema',
  'paleta.temaClaves': 'claro oscuro sistema',
  'paleta.ayuda': 'Ver los atajos de teclado',
  'paleta.ayudaClaves': 'ayuda teclas',

  // Tablero
  'tablero.compacta': 'Cambiar a vista compacta',
  'tablero.comoda': 'Cambiar a vista cómoda',
  'tablero.coinciden': ({ n }: N) =>
    n === 1
      ? '1 proyecto coincide con los filtros aplicados.'
      : `${n} proyectos coinciden con los filtros aplicados.`,
  'tablero.cargando': 'Cargando proyectos…',
  'tablero.sinRutasTitulo': 'Todavía no hay rutas que escanear',
  'tablero.sinRutasTexto':
    'Añade la carpeta donde guardas tus proyectos y Mosaic los encontrará por ti.',
  'tablero.sinRutasAccion': 'Configurar rutas',
  'tablero.sinProyectosTitulo': 'Ningún proyecto encontrado',
  'tablero.sinProyectosTexto':
    'Las rutas están configuradas pero aún no se ha escaneado, o no hay proyectos dentro.',
  'tablero.escanearAhora': 'Escanear ahora',
  'tablero.ningunoCoincideTitulo': 'Ningún proyecto coincide con los filtros',
  'tablero.ningunoCoincideTexto':
    'Prueba con otra búsqueda, o quita los filtros para volver a ver todo.',
  'tablero.fijados': 'Fijados',
  'tablero.fijadosSeccion': 'Proyectos fijados',
  'tablero.restoSeccion': 'Resto de proyectos',

  // Panel lateral
  'lateral.colecciones': 'Colecciones',
  'lateral.desplegar': 'Desplegar el panel lateral',
  'lateral.plegar': 'Plegar el panel lateral',
  'lateral.todos': 'Todos los proyectos',
  'lateral.soloDestacados': 'Solo destacados',
  'lateral.filtrarPor': ({ nombre }: { nombre: string }) => `Filtrar por ${nombre}`,
  'lateral.gestionar': 'Gestionar etiquetas',
  'lateral.vistas': 'Vistas',
  'lateral.fijados': 'Fijados',
  'lateral.etiquetas': 'Etiquetas',
  'lateral.sinEtiquetas': 'Ninguna todavía. Etiqueta un proyecto desde su tarjeta.',

  // Búsqueda
  'busqueda.placeholder': 'Buscar por nombre o ruta…',
  'busqueda.etiqueta': 'Buscar proyectos',
  'busqueda.limpiar': 'Limpiar la búsqueda',
  'busqueda.cuenta': ({ visibles, total }: { visibles: number; total: number }) =>
    `${visibles} de ${total}`,

  // Filtros
  'filtros.boton': 'Filtros',
  'filtros.git': 'Estado Git',
  'filtros.gitCualquiera': 'Cualquiera',
  'filtros.gitConCambios': 'Con cambios',
  'filtros.gitSinCambios': 'Sin cambios',
  'filtros.gitSinRepo': 'Sin repositorio',
  'filtros.lenguaje': 'Lenguaje',
  'filtros.soloDestacados': 'Solo destacados',
  'filtros.ocultarAusentes': 'Ocultar ausentes',
  'filtros.masEtiquetas': ({ n }: N) => `+${n} etiquetas`,
  'filtros.quitarLenguaje': ({ lenguaje }: { lenguaje: string }) =>
    `Quitar el filtro de lenguaje ${lenguaje}`,
  'filtros.chipGit': ({ estado }: { estado: string }) => `Git: ${estado}`,
  'filtros.quitarGit': 'Quitar el filtro de estado Git',
  'filtros.quitarDestacados': 'Quitar el filtro de destacados',
  'filtros.ausentesOcultos': 'Ausentes ocultos',
  'filtros.mostrarAusentes': 'Volver a mostrar los proyectos ausentes',
  'filtros.limpiar': 'Limpiar filtros',

  // Orden
  'orden.nombre': 'Nombre',
  'orden.ultimaApertura': 'Última apertura',
  'orden.alta': 'Fecha de alta',
  'orden.actualizacion': 'Última actualización',
  'orden.cambiar': 'Cambiar la ordenación',
  'orden.ascACambiar': 'Orden ascendente, cambiar a descendente',
  'orden.descACambiar': 'Orden descendente, cambiar a ascendente',
  'orden.ascendente': 'Ascendente',
  'orden.descendente': 'Descendente',
  'orden.menu': 'Ordenar por',

  // Resumen del escaneo
  'escaneo.nuevos': ({ n }: N) => `${n} nuevos`,
  'escaneo.actualizados': ({ n }: N) => `${n} actualizados`,
  'escaneo.yaNoEstan': ({ n }: N) => `${n} ya no están`,
  'escaneo.recortado':
    'El escaneo se detuvo al alcanzar el tope de entradas, así que puede faltar algo. Súbelo en los ajustes o acota las rutas.',
  'escaneo.rutasNoExisten': ({ n }: N) =>
    `${n} ${n === 1 ? 'ruta no existe' : 'rutas no existen'} en el disco.`,
  'escaneo.sinRutas': 'No hay ninguna ruta habilitada que escanear.',
  'escaneo.encontrados': ({ n }: N) =>
    `${n} ${n === 1 ? 'proyecto encontrado' : 'proyectos encontrados'}`,
  'escaneo.duracion': ({ duracion }: { duracion: string }) => `en ${duracion}`,

  // Tarjeta
  'tarjeta.abrirEditor': 'Abrir en el editor',
  'tarjeta.abrirTerminal': 'Abrir en la terminal',
  'tarjeta.abrirCarpeta': 'Abrir la carpeta',
  'tarjeta.dejarDestacar': 'Dejar de destacar',
  'tarjeta.destacar': 'Destacar el proyecto',
  'tarjeta.dejarDestacarNombre': ({ nombre }: { nombre: string }) => `Dejar de destacar ${nombre}`,
  'tarjeta.destacarNombre': ({ nombre }: { nombre: string }) => `Destacar ${nombre}`,
  'tarjeta.pista': 'Click para abrir el detalle',
  'tarjeta.headSeparado': 'HEAD separado',
  'tarjeta.sucio': 'Tiene cambios sin commitear',
  'tarjeta.porDelante': 'Commits por delante',
  'tarjeta.porDetras': 'Commits por detrás',
  'tarjeta.ausente': 'El último escaneo no encontró esta carpeta',
  'tarjeta.noEncontrado': 'No encontrado',
  'tarjeta.sinAbrir': 'Sin abrir',
  'tarjeta.abierto': ({ cuando }: { cuando: string }) => `Abierto ${cuando}`,

  // Etiquetas en tarjetas y selector
  'chip.quitar': ({ nombre }: { nombre: string }) => `Quitar la etiqueta ${nombre}`,
  'selector.asignar': 'Asignar etiquetas',
  'selector.etiqueta': 'Etiqueta',
  'selector.buscar': 'Buscar o crear…',
  'selector.crear': ({ nombre }: { nombre: string }) => `Crear «${nombre}»`,
  'selector.color': ({ nombre }: { nombre: string }) => `Color ${nombre}`,

  // Colores de etiqueta
  'color.slate': 'Pizarra',
  'color.gray': 'Gris',
  'color.red': 'Rojo',
  'color.orange': 'Naranja',
  'color.amber': 'Ámbar',
  'color.yellow': 'Amarillo',
  'color.lime': 'Lima',
  'color.green': 'Verde',
  'color.emerald': 'Esmeralda',
  'color.teal': 'Turquesa',
  'color.cyan': 'Cian',
  'color.sky': 'Celeste',
  'color.blue': 'Azul',
  'color.indigo': 'Índigo',
  'color.violet': 'Violeta',
  'color.fuchsia': 'Fucsia',

  // Comunes
  'comun.cerrar': 'Cerrar',
  'comun.cancelar': 'Cancelar',
  'comun.editor': 'Editor',
  'comun.terminal': 'Terminal',
  'comun.guardando': 'Guardando…',
  'comun.guardado': 'Guardado',
  'comun.noRepo': 'No es un repositorio Git.',
  'comun.numEtiquetas': ({ n }: N) => `${n} ${n === 1 ? 'etiqueta' : 'etiquetas'}`,
  'comun.y': 'y',

  // Ajustes
  'ajustes.sinEditor': 'No se ha encontrado ningún editor reconocido en el PATH.',
  'ajustes.sinTerminal': 'No se ha encontrado ningún emulador de terminal reconocido en el PATH.',
  'ajustes.gitDesactivado': 'Desactivado',
  'ajustes.gitCadaMinuto': 'Cada minuto',
  'ajustes.gitCadaMinutos': ({ n }: N) => `Cada ${n} minutos`,
  'ajustes.gitCadaHora': 'Cada hora',
  'ajustes.rutasTitulo': 'Rutas de escaneo',
  'ajustes.rutasTexto': 'Carpetas donde Mosaic busca proyectos, hasta cuatro niveles de profundidad.',
  'ajustes.anadirCarpeta': 'Añadir carpeta',
  'ajustes.cargandoRutas': 'Cargando rutas…',
  'ajustes.sinRutasTitulo': 'Ninguna ruta configurada',
  'ajustes.sinRutasTexto': 'Elige la carpeta donde guardas tus proyectos para empezar.',
  'ajustes.ultimoEscaneo': ({ cuando }: { cuando: string }) => `Último escaneo: ${cuando}`,
  'ajustes.eliminarRuta': 'Eliminar la ruta (los proyectos ya descubiertos se conservan)',
  'ajustes.eliminarRutaNombre': ({ ruta }: { ruta: string }) => `Eliminar la ruta ${ruta}`,
  'ajustes.rutasNota':
    'Deshabilitar una ruta la excluye de los escaneos sin borrarla. Eliminarla no borra los proyectos ya descubiertos: conservan sus etiquetas y notas.',
  'ajustes.errorLeer': ({ error }: { error: string }) => `No se pudieron leer los ajustes: ${error}`,
  'ajustes.escaneoTitulo': 'Escaneo',
  'ajustes.escaneoTexto': 'Cómo recorre Mosaic tus carpetas. Cada cambio se guarda solo.',
  'ajustes.profundidad': 'Profundidad',
  'ajustes.profundidadEfecto': ({ min, max }: { min: number; max: number }) =>
    `Niveles por debajo de cada ruta, de ${min} a ${max}. Se aplica en el próximo escaneo.`,
  'ajustes.tope': 'Tope de entradas',
  'ajustes.topeEfecto':
    'Si una ruta tiene más, el escaneo se detiene y avisa. Se aplica en el próximo escaneo.',
  'ajustes.arranque': 'Escanear al arrancar',
  'ajustes.arranqueEfecto':
    'Recorre las rutas ocho segundos después de abrir Mosaic. Se aplica la próxima vez que lo abras.',
  'ajustes.excluidas': 'Carpetas excluidas',
  'ajustes.excluidasEfecto':
    'Nombres de carpeta que el escáner salta allí donde aparezcan. Se aplica en el próximo escaneo.',
  'ajustes.gitTexto': 'Mosaic relee el estado de tus repositorios en segundo plano, sin red.',
  'ajustes.refresco': 'Refresco automático',
  'ajustes.refrescoEfecto':
    'Se aplica en el acto: al cambiarlo, Mosaic relee Git y empieza a contar de nuevo.',
  'ajustes.appsTitulo': 'Aplicaciones',
  'ajustes.appsTexto':
    'Con qué se abren los proyectos. Mosaic detecta lo que hay instalado; si no eliges nada, usa el primero de la lista.',
  'ajustes.automatico': ({ nombre }: { nombre: string }) => `Automático (${nombre})`,
  'ajustes.explorador':
    'El explorador de archivos no se configura aquí: se usa el que tenga asociado el sistema.',

  // Carpetas excluidas
  'excluidas.escribeNombre': 'Escribe el nombre de una carpeta.',
  'excluidas.sinBarras': 'Es un nombre de carpeta, sin barras.',
  'excluidas.yaEsta': ({ nombre }: { nombre: string }) => `«${nombre}» ya está en la lista.`,
  'excluidas.noCaben': ({ n }: N) => `No caben más de ${n}.`,
  'excluidas.ninguna': 'Ninguna: el escáner lo recorre todo, incluido',
  'excluidas.dejar': ({ nombre }: { nombre: string }) => `Dejar de excluir ${nombre}`,
  'excluidas.placeholder': 'Nombre de carpeta, por ejemplo vendor',
  'excluidas.campo': 'Carpeta que excluir',
  'excluidas.anadir': 'Añadir',
  'excluidas.confirmar': 'Pulsa otra vez: se quitan tus cambios',
  'excluidas.restaurar': 'Restaurar las predeterminadas',

  // Copia de seguridad
  'copia.filtro': 'Copia de Mosaic',
  'copia.fichero': 'mosaic-copia',
  'copia.titulo': 'Copia de seguridad',
  'copia.texto':
    'Guarda tus ajustes en un fichero para llevarlos a otro equipo o recuperarlos. Importar solo añade: nunca borra nada de lo que ya tienes.',
  'copia.incluir': 'Incluir etiquetas y notas',
  'copia.exportar': 'Exportar…',
  'copia.importar': 'Importar…',
  'copia.error': ({ error }: { error: string }) => `No se pudo: ${error}`,
  'copia.previa': ({ cuando, version }: { cuando: string; version: string }) =>
    `Esto es lo que hará importar la copia de ${cuando} (Mosaic ${version}):`,
  'copia.importada': 'Copia importada.',
  'copia.confirmar': 'Importar',
  'copia.numRutas': ({ n }: N) => `${n} ${n === 1 ? 'ruta' : 'rutas'}`,
  'copia.guardadaAjustes': ({ rutas }: { rutas: string }) => `Copia guardada: ajustes y ${rutas}.`,
  'copia.guardadaTodo': ({ rutas, etiquetas, proyectos }: { rutas: string; etiquetas: string; proyectos: string }) =>
    `Copia guardada: ajustes, ${rutas}, ${etiquetas} y ${proyectos} con etiquetas, notas o fijados.`,
  'copia.ajuste.max_depth': 'profundidad',
  'copia.ajuste.excluded_dirs': 'carpetas excluidas',
  'copia.ajuste.max_entries_per_scan': 'tope de entradas',
  'copia.ajuste.scan_on_startup': 'escanear al arrancar',
  'copia.ajuste.git_refresh_interval_minutes': 'refresco de Git',
  'copia.ajuste.preferred_ide': 'editor',
  'copia.ajuste.preferred_terminal': 'terminal',
  'copia.planAjustes': ({ nombres }: { nombres: string }) => `Ajustes: cambia ${nombres}.`,
  'copia.planRutas': ({ n }: N) =>
    `Añade ${n} ${n === 1 ? 'ruta de escaneo' : 'rutas de escaneo'}.`,
  'copia.planEtiquetas': ({ n }: N) => `Crea ${n} ${n === 1 ? 'etiqueta' : 'etiquetas'}.`,
  'copia.planAsignaciones': ({ n }: N) =>
    `Pone ${n} ${n === 1 ? 'etiqueta' : 'etiquetas'} a tus proyectos.`,
  'copia.planFijados': ({ n }: N) => `Fija ${n} ${n === 1 ? 'proyecto' : 'proyectos'}.`,
  'copia.planNotas': ({ n }: N) => `Añade notas a ${n} ${n === 1 ? 'proyecto' : 'proyectos'}.`,
  'copia.planNada': 'No cambia nada: ya tienes todo lo que trae esta copia.',
  'copia.planConflictos': ({ nombres, n }: { nombres: string; n: number }) =>
    `${nombres}: ya ${n === 1 ? 'tiene' : 'tienen'} notas distintas. Se quedan las tuyas.`,
  'copia.planRutasFaltan': ({ rutas, n }: { rutas: string; n: number }) =>
    n === 1
      ? `1 ruta no existe en este equipo y se salta: ${rutas}.`
      : `${n} rutas no existen en este equipo y se saltan: ${rutas}.`,
  'copia.planEtiquetasExisten': ({ n }: N) =>
    n === 1
      ? '1 etiqueta ya existía y conserva su color.'
      : `${n} etiquetas ya existían y conservan su color.`,
  'copia.planNoEncontrados': ({ n }: N) =>
    `${n} ${n === 1 ? 'proyecto de la copia no está' : 'proyectos de la copia no están'} en este equipo. Escanea y vuelve a importar para completarlos.`,
  'copia.planSinTrabajo': 'La copia no trae etiquetas ni notas.',

  // Detalle de proyecto
  'detalle.cargando': 'Cargando el proyecto…',
  'detalle.yaNoEsta': 'Este proyecto ya no está en la lista.',
  'detalle.volver': 'Volver a proyectos',
  'detalle.volverTitulo': 'Volver a proyectos (Esc)',
  'detalle.sucio': 'Cambios sin commitear',
  'detalle.sinAbrir': 'Sin abrir desde Mosaic',
  'detalle.abrir': 'Abrir el proyecto',
  'detalle.carpeta': 'Carpeta',
  'detalle.carpetaAusente': 'La carpeta ya no está en disco',
  'detalle.ausente':
    'El último escaneo no encontró esta carpeta. Sus etiquetas y notas se conservan por si vuelve.',
  'detalle.notas': 'Notas',
  'detalle.ramas': 'Ramas',
  'detalle.commits': 'Últimos commits',

  // README
  'readme.abierto': 'Abierto en el navegador.',
  'readme.noAbrir': ({ error }: { error: string }) => `No se pudo abrir: ${error}`,
  'readme.noAbrirEnlace': ({ error }: { error: string }) => `No se pudo abrir el enlace: ${error}`,
  'readme.enlaceInterno': 'Este enlace apunta dentro del repositorio. Mosaic solo abre enlaces web.',
  'readme.leyendo': 'Leyendo el README…',
  'readme.error': ({ error }: { error: string }) => `No se pudo leer el README: ${error}`,
  'readme.carpetaAusente': 'La carpeta ya no está en disco, así que no hay README que leer.',
  'readme.sinReadme': 'Este proyecto no tiene README en su carpeta raíz.',
  'readme.abrirEn': ({ sitio }: { sitio: string }) => `Abrir en ${sitio}`,
  'readme.navegador': 'el navegador',
  'readme.recortado': 'El fichero pasa de 512 KB y solo se muestra el principio.',
  'readme.imagenes': ({ n }: N) => `${n} ${n === 1 ? 'imagen' : 'imágenes'}`,
  'readme.insignias': ({ n }: N) => `${n} ${n === 1 ? 'insignia' : 'insignias'}`,
  'readme.medios': ({ n }: N) =>
    `${n} ${n === 1 ? 'vídeo o elemento incrustado' : 'vídeos o elementos incrustados'}`,
  'readme.omitido': ({ lista }: { lista: string }) => `Este README tiene ${lista} que Mosaic no muestra.`,

  // Historial y ramas
  'historial.leyendo': 'Leyendo el historial…',
  'historial.error': ({ error }: { error: string }) => `No se pudo leer el historial: ${error}`,
  'historial.vacio': 'Esta rama todavía no tiene commits.',
  'ramas.leyendo': 'Leyendo las ramas…',
  'ramas.error': ({ error }: { error: string }) => `No se pudieron leer las ramas: ${error}`,
  'ramas.locales': 'Locales',
  'ramas.sinLocales': 'Ninguna todavía.',
  'ramas.actual': 'actual',
  'ramas.remotas': 'Remotas',
  'ramas.sinRemotas': 'Ninguna conocida.',
  'ramas.nota': 'Las que conoce tu copia desde el último fetch. Mosaic no consulta la red.',

  // Copiar la ruta
  'copiar.ruta': 'Copiar ruta',
  'copiar.hecho': 'Copiado',
  'copiar.fallo': 'No se pudo copiar',

  // Notas
  'notas.sinGuardar': 'Sin guardar',
  'notas.fallo': 'No se pudo guardar',
  'notas.placeholder': 'Apuntes para ti: lo que falta, dónde lo dejaste, a quién pertenece…',
  'notas.etiqueta': 'Notas del proyecto',
  'notas.errorAlSalir': ({ error }: { error: string }) => `No se pudieron guardar las notas: ${error}`,
  'notas.errorDesconocido': 'error desconocido',

  // Paleta de comandos
  'comandos.titulo': 'Paleta de comandos',
  'comandos.placeholder': 'Busca un proyecto o una acción…',
  'comandos.buscar': 'Buscar en la paleta',
  'comandos.nada': ({ texto }: { texto: string }) => `Nada coincide con «${texto}».`,
  'comandos.resultados': 'Resultados',
  'comandos.recientes': 'Abiertos hace poco',
  'comandos.moverse': 'moverse',
  'comandos.abrir': 'abrir',
  'comandos.enEditor': 'proyecto en el editor',
  'comandos.cerrar': 'cerrar',

  // Ayuda de atajos
  'atajos.titulo': 'Atajos de teclado',
  'atajos.paleta': 'Abrir la paleta de comandos',
  'atajos.git': 'Releer el estado de Git',
  'atajos.ajustes': 'Ir a Ajustes',
  'atajos.buscar': 'Buscar en el tablero',
  'atajos.ayuda': 'Ver esta lista de atajos',
  'atajos.esc': 'Cerrar lo que esté abierto, o volver del detalle',

  // Gestor de etiquetas
  'gestor.nombreVacio': 'El nombre de la etiqueta no puede estar vacío',
  'gestor.colorNueva': 'Elegir el color de la etiqueta nueva',
  'gestor.color': 'Color',
  'gestor.personalizado': 'Personalizado',
  'gestor.nueva': 'Nueva etiqueta…',
  'gestor.crear': 'Crear',
  'gestor.vacio': 'Todavía no hay etiquetas. Crea la primera arriba.',
  'gestor.cambiarColor': 'Cambiar el color',
  'gestor.cambiarColorDe': ({ nombre }: { nombre: string }) => `Cambiar el color de ${nombre}`,
  'gestor.nombreDe': ({ nombre }: { nombre: string }) => `Nombre de la etiqueta ${nombre}`,
  'gestor.borrar': 'Borrar la etiqueta',
  'gestor.borrarDe': ({ nombre }: { nombre: string }) => `Borrar la etiqueta ${nombre}`,
  'gestor.sinAsignar': 'Esta etiqueta no está asignada a ningún proyecto.',
  'gestor.quitaraDe': ({ n }: N) =>
    `Esto quitará la etiqueta de ${n} ${n === 1 ? 'proyecto' : 'proyectos'}. No se borran los proyectos.`,
  'gestor.confirmarBorrar': 'Borrar',
  'selector.vacio': 'Todavía no hay etiquetas. Escribe un nombre para crear la primera.',

  // Idioma
  'idioma.titulo': 'Idioma · Language',
  'idioma.sistema': 'El del sistema',
  'idioma.es': 'Español',
  'idioma.en': 'English',
} satisfies Record<string, string | ((p: never) => string)>;
