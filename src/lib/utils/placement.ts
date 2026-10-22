/** Colocación de paneles desplegables. */

/** Si un panel abre hacia arriba: solo si no cabe debajo y sí encima; si no cabe, abajo. */
export function opensUpward(
  trigger: { top: number; bottom: number },
  panelHeight: number,
  viewportHeight: number,
  gap = 4,
): boolean {
  const fitsBelow = trigger.bottom + gap + panelHeight <= viewportHeight;
  const fitsAbove = trigger.top - gap - panelHeight >= 0;
  return !fitsBelow && fitsAbove;
}
