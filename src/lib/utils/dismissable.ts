interface Options {
  onDismiss: () => void;
  enabled?: boolean;
}

/** Acción de Svelte: Escape o un clic fuera cierran. Escucha en captura, así gana a la vista. */
export function dismissable(node: HTMLElement, options: Options) {
  let current = options;

  function onKeydown(event: KeyboardEvent) {
    if (current.enabled === false) return;
    if (event.key !== 'Escape') return;
    event.stopPropagation();
    current.onDismiss();
  }

  function onPointerdown(event: PointerEvent) {
    if (current.enabled === false) return;
    const target = event.target;
    if (target instanceof Node && node.contains(target)) return;
    current.onDismiss();
  }

  window.addEventListener('keydown', onKeydown, true);
  window.addEventListener('pointerdown', onPointerdown, true);

  return {
    update(next: Options) {
      current = next;
    },
    destroy() {
      window.removeEventListener('keydown', onKeydown, true);
      window.removeEventListener('pointerdown', onPointerdown, true);
    },
  };
}
