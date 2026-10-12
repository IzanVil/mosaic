/**
 * Avisa cuando un elemento entra o sale de la zona cercana a lo que se ve.
 *
 * Un solo `IntersectionObserver` por contenedor con scroll, compartido por
 * todas las tarjetas: con miles de proyectos, uno por tarjeta sería otro coste
 * que crece con la lista.
 */

/** Margen alrededor de lo visible en el que una tarjeta ya cuenta como cerca. */
export const NEAR_MARGIN_PX = 800;

type Callback = (near: boolean) => void;

const observers = new Map<Element | null, IntersectionObserver>();
const callbacks = new WeakMap<Element, Callback>();

/** El ancestro que hace scroll, o `null` si es la propia ventana. */
export function scrollParent(node: Element): Element | null {
  for (let el = node.parentElement; el; el = el.parentElement) {
    const { overflowY } = getComputedStyle(el);
    if (overflowY === 'auto' || overflowY === 'scroll') return el;
  }
  return null;
}

function observerFor(root: Element | null): IntersectionObserver {
  let observer = observers.get(root);
  if (!observer) {
    // El margen se aplica al contenedor que hace scroll: medido contra la
    // ventana, el contenedor recortaría lo que queda debajo y nunca estaría cerca.
    observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) callbacks.get(entry.target)?.(entry.isIntersecting);
      },
      { root, rootMargin: `${NEAR_MARGIN_PX}px 0px` },
    );
    observers.set(root, observer);
  }
  return observer;
}

/** Acción de Svelte: llama a `onChange` cada vez que el nodo se acerca o se aleja. */
export function nearViewport(node: Element, onChange: Callback) {
  const observer = observerFor(scrollParent(node));
  callbacks.set(node, onChange);
  observer.observe(node);
  return {
    update(next: Callback) {
      callbacks.set(node, next);
    },
    destroy() {
      observer.unobserve(node);
      callbacks.delete(node);
    },
  };
}
