/** Rutas para enseñar, nunca para abrir: el backend trabaja siempre con la ruta entera. */

// La carpeta personal en Linux, macOS y Windows. Se adivina por la forma y no se le pide al
// sistema porque solo sirve para leer, y una llamada más por tarjeta no compensa.
const HOME = /^(?:\/home\/[^/]+|\/Users\/[^/]+|[A-Za-z]:\\Users\\[^\\]+)(?=[\\/]|$)/;

/** La carpeta que contiene `path`, con la personal abreviada a `~`: `~/proyectos`. */
export function parentFolder(path: string): string {
  const trimmed = path.replace(/(?<=.)[\\/]+$/, '');
  const cut = Math.max(trimmed.lastIndexOf('/'), trimmed.lastIndexOf('\\'));
  if (cut < 0) return trimmed;
  let parent = trimmed.slice(0, cut);
  // La raíz se queda con su separador: `/` y `C:\`, no una cadena vacía ni `C:`.
  if (parent === '' || /^[A-Za-z]:$/.test(parent)) parent = trimmed.slice(0, cut + 1);
  return parent.replace(HOME, '~');
}
