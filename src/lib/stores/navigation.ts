/**
 * Navegación entre las vistas de la aplicación.
 *
 * Una unión discriminada y nada más: no hay router, ni URLs, ni historial.
 * Mosaic tiene tres pantallas y ninguna necesita un enlace propio. Tampoco se
 * persiste: al abrir la aplicación se vuelve siempre al tablero, y el detalle
 * de un proyecto no sobrevive a un reinicio.
 */

import { get, writable } from 'svelte/store';

/** La pantalla que se está viendo. */
export type Route =
  | { view: 'dashboard' }
  | { view: 'settings' }
  | { view: 'project'; projectId: number };

/** Pantalla actual. Se cambia solo con las funciones de este módulo. */
export const route = writable<Route>({ view: 'dashboard' });

/**
 * Tarjeta a la que vuelve el foco al salir del detalle.
 *
 * Se olvida en cuanto la navegación pasa por Ajustes: volver al tablero
 * después de eso ya no es «volver de la tarjeta», y saltar a una tarjeta que
 * el usuario abrió hace dos pantallas sería una sorpresa.
 */
let returnFocusTo: number | null = null;

/** Abre el detalle de un proyecto, recordando de qué tarjeta se vino. */
export function openProjectDetail(projectId: number): void {
  returnFocusTo = projectId;
  route.set({ view: 'project', projectId });
}

/** Va a Ajustes. Corta el regreso del foco a la tarjeta. */
export function goToSettings(): void {
  returnFocusTo = null;
  route.set({ view: 'settings' });
}

/** Va al tablero sin devolver el foco a ninguna tarjeta. */
export function goToDashboard(): void {
  route.set({ view: 'dashboard' });
}

/**
 * Sale del detalle de vuelta al tablero.
 *
 * Devuelve el proyecto cuya tarjeta debe recibir el foco, o `null` si no toca
 * devolverlo. Quien llama se encarga del foco cuando el tablero ya esté
 * pintado: este módulo no sabe nada del DOM.
 */
export function leaveProjectDetail(): number | null {
  const target = get(route).view === 'project' ? returnFocusTo : null;
  returnFocusTo = null;
  route.set({ view: 'dashboard' });
  return target;
}

/**
 * El gestor de etiquetas está abierto. Vive aquí y no en el tablero porque
 * también se abre desde la paleta de comandos, estando en otra pantalla.
 */
export const tagManagerOpen = writable(false);

/** Abre el gestor de etiquetas, volviendo antes al tablero si hace falta. */
export function openTagManager(): void {
  if (get(route).view !== 'dashboard') route.set({ view: 'dashboard' });
  tagManagerOpen.set(true);
}
