<script lang="ts">
  import {
    editNotes,
    flushNotes,
    notesDraft,
    notesError,
    notesStatus,
  } from '../stores/projectDetail';

  /** El mismo tope que aplica el backend, para no dejar escribir de más. */
  const MAX_CHARS = 20_000;

  const STATUS_TEXT = {
    saved: 'Guardado',
    pending: 'Sin guardar',
    saving: 'Guardando…',
    error: 'No se pudo guardar',
  } as const;
</script>

<div class="notas">
  <textarea
    value={$notesDraft}
    oninput={(event) => editNotes(event.currentTarget.value)}
    onblur={() => void flushNotes()}
    maxlength={MAX_CHARS}
    rows="6"
    placeholder="Apuntes para ti: lo que falta, dónde lo dejaste, a quién pertenece…"
    aria-label="Notas del proyecto"
    aria-describedby="estado-notas"
  ></textarea>

  <!-- Sin botón de guardar: el estado dice siempre qué ha pasado con lo escrito. -->
  <p id="estado-notas" class="estado" class:error={$notesStatus === 'error'} aria-live="polite">
    {STATUS_TEXT[$notesStatus]}{#if $notesStatus === 'error' && $notesError}: {$notesError}{/if}
  </p>
</div>

<style>
  .notas {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  textarea {
    width: 100%;
    box-sizing: border-box;
    min-height: 7.5rem;
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    background: var(--surface-base);
    font: inherit;
    font-size: var(--text-body);
    line-height: var(--text-body-lh);
    color: var(--text-primary);
    resize: vertical;
    transition: border-color var(--duration-fast) var(--ease-out);
  }
  textarea::placeholder {
    color: var(--text-tertiary);
  }
  textarea:hover {
    border-color: var(--border-strong);
  }
  textarea:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
    border-color: var(--accent-border);
  }

  .estado {
    margin: 0;
    font-size: var(--text-meta);
    line-height: var(--text-meta-lh);
    color: var(--text-tertiary);
  }
  .estado.error {
    color: var(--danger);
  }
</style>
