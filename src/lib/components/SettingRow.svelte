<script lang="ts">
  import { Check, LoaderCircle, TriangleAlert } from '@lucide/svelte';
  import type { Snippet } from 'svelte';

  import { fieldStatus, type AdvancedField } from '../stores/advancedSettings';

  interface Props {
    /** Nombre del ajuste. */
    label: string;
    /** Id del control, para que la etiqueta lo active al pulsarla. */
    controlId?: string;
    /** Cuándo surte efecto un cambio: «en el próximo escaneo», etc. */
    effect: string;
    /** Campo cuyo estado de guardado se enseña al lado. */
    field: AdvancedField;
    /** El control en sí. */
    children: Snippet;
  }

  let { label, controlId, effect, field, children }: Props = $props();

  let status = $derived($fieldStatus[field]);
</script>

<div class="fila">
  <label class="nombre" for={controlId}>{label}</label>
  <div class="control">
    {@render children()}
    <span class="estado" aria-live="polite">
      {#if status?.state === 'saving'}
        <LoaderCircle size={14} strokeWidth={1.75} class="girando" /> Guardando…
      {:else if status?.state === 'saved'}
        <Check size={14} strokeWidth={1.75} /> Guardado
      {/if}
    </span>
  </div>
  {#if status?.state === 'error'}
    <p class="error" role="alert">
      <TriangleAlert size={14} strokeWidth={1.75} /> {status.message}
    </p>
  {:else}
    <p class="efecto">{effect}</p>
  {/if}
</div>

<style>
  .fila {
    display: grid;
    grid-template-columns: 11rem minmax(0, 1fr);
    align-items: start;
    gap: var(--space-1) var(--space-4);
  }

  /* Arriba y con el relleno de un campo: así casa con controles de una línea
     y no flota en medio de los altos, como la lista de carpetas. */
  .nombre {
    padding-top: 7px;
    font-size: var(--text-body);
    line-height: var(--text-body-lh);
    color: var(--text-secondary);
  }

  .control {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    min-height: 34px;
    gap: var(--space-3);
    min-width: 0;
  }

  .estado {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    min-height: 1em;
    font-size: var(--text-meta);
    line-height: var(--text-meta-lh);
    color: var(--accent);
  }
  /* Sin nada que contar no ocupa sitio; en la lista de carpetas saltaría a una
     línea propia y dejaría un hueco. */
  .estado:empty {
    display: none;
  }
  .estado :global(.girando) {
    animation: girar 900ms linear infinite;
  }

  .efecto,
  .error {
    grid-column: 2;
    display: flex;
    align-items: center;
    gap: var(--space-1);
    margin: 0;
    font-size: var(--text-meta);
    line-height: var(--text-meta-lh);
    color: var(--text-tertiary);
  }
  .error {
    color: var(--danger);
  }

  @keyframes girar {
    to {
      transform: rotate(360deg);
    }
  }

  @media (max-width: 640px) {
    .fila {
      grid-template-columns: minmax(0, 1fr);
    }
    .efecto,
    .error {
      grid-column: 1;
    }
  }
</style>
