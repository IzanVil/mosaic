<script lang="ts">
  import { t } from '../i18n';
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
    saved: 'comun.guardado',
    pending: 'notas.sinGuardar',
    saving: 'comun.guardando',
    error: 'notas.fallo',
  } as const;
</script>

<div class="notas">
  <textarea
    value={$notesDraft}
    oninput={(event) => editNotes(event.currentTarget.value)}
    onblur={() => void flushNotes()}
    maxlength={MAX_CHARS}
    rows="6"
    placeholder={$t('notas.placeholder')}
    aria-label={$t('notas.etiqueta')}
    aria-describedby="estado-notas"
  ></textarea>

  <!-- Sin botón de guardar: el estado dice siempre qué ha pasado con lo escrito. -->
  <p id="estado-notas" class="estado" class:error={$notesStatus === 'error'} aria-live="polite">
    {$t(STATUS_TEXT[$notesStatus])}{#if $notesStatus === 'error' && $notesError}: {$notesError}{/if}
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
