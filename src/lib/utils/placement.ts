/** Colocación de paneles desplegables. */

/**
 * Si un panel debe abrirse hacia arriba: solo cuando no cabe debajo del
 * disparador y sí cabe encima. Si no cabe en ninguno, abajo.
 */
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
