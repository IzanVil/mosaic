/**
 * Navegación entre vistas: una unión discriminada, sin router, URLs ni historial,
 * que tres pantallas no necesitan. No se guarda: la app abre siempre en el tablero.
 */

import { get, writable } from 'svelte/store';

/** La pantalla que se está viendo. */
export type Route =
  | { view: 'dashboard' }
  | { view: 'settings' }
  | { view: 'project'; projectId: number };

/** Pantalla actual. Se cambia solo con las funciones de este módulo. */
export const route = writable<Route>({ view: 'dashboard' });

/** Tarjeta a la que vuelve el foco al salir del detalle; se olvida al pasar por Ajustes. */
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

/** Vuelve al tablero y dice qué tarjeta recibe el foco; lo pone quien llama: aquí no hay DOM. */
export function leaveProjectDetail(): number | null {
  const target = get(route).view === 'project' ? returnFocusTo : null;
  returnFocusTo = null;
  route.set({ view: 'dashboard' });
  return target;
}

/** El gestor de etiquetas está abierto; vive aquí porque también se abre desde la paleta. */
export const tagManagerOpen = writable(false);

/** Abre el gestor de etiquetas, volviendo antes al tablero si hace falta. */
export function openTagManager(): void {
  if (get(route).view !== 'dashboard') route.set({ view: 'dashboard' });
  tagManagerOpen.set(true);
}
