/** Debounce cancelable. */

export interface Debounced<Args extends unknown[]> {
  (...args: Args): void;
  /** Descarta la llamada pendiente, si hay alguna. */
  cancel(): void;
}

/**
 * Devuelve una versión de `fn` que espera `delayMs` sin nuevas llamadas antes
 * de ejecutarse.
 *
 * El `cancel()` es obligatorio: sin él, un temporizador vivo dispararía después
 * de destruir el componente y escribiría estado que ya nadie mira. Quien lo use
 * dentro de un `$effect` debe devolver `cancel` como limpieza.
 */
export function debounce<Args extends unknown[]>(
  fn: (...args: Args) => void,
  delayMs: number,
): Debounced<Args> {
  let timer: ReturnType<typeof setTimeout> | undefined;

  const debounced = (...args: Args) => {
    if (timer !== undefined) clearTimeout(timer);
    timer = setTimeout(() => {
      timer = undefined;
      fn(...args);
    }, delayMs);
  };

  debounced.cancel = () => {
    if (timer !== undefined) clearTimeout(timer);
    timer = undefined;
  };

  return debounced;
}
