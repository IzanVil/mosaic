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

  // Idioma
  'idioma.titulo': 'Idioma · Language',
  'idioma.sistema': 'El del sistema',
  'idioma.es': 'Español',
  'idioma.en': 'English',
} satisfies Record<string, string | ((p: never) => string)>;
