/** Debounce cancelable. */

export interface Debounced<Args extends unknown[]> {
  (...args: Args): void;
  /** Descarta la llamada pendiente, si hay alguna. */
  cancel(): void;
}

/**
 * `fn` cuando pasan `delayMs` sin llamadas nuevas. Hay que llamar a `cancel` al
 * destruir: si no, el temporizador escribiría estado que ya nadie mira.
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
