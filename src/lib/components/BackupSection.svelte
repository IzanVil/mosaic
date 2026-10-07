<script lang="ts">
  import { open, save } from '@tauri-apps/plugin-dialog';
  import { Download, Upload, X } from '@lucide/svelte';

  import {
    backupState,
    confirmImport,
    dismissBackup,
    exportTo,
    previewFrom,
  } from '../stores/backup';
  import { backupFileName, describePlan } from '../utils/backup';
  import { formatRelativeTime } from '../utils/format';

  const FILTERS = [{ name: 'Copia de Mosaic', extensions: ['json'] }];

  let includeWork = $state(true);

  async function pickExport() {
    const path = await save({ defaultPath: backupFileName(new Date()), filters: FILTERS });
    if (path) await exportTo(path, includeWork);
  }

  async function pickImport() {
    const path = await open({ multiple: false, directory: false, filters: FILTERS });
    if (typeof path === 'string') await previewFrom(path);
  }

  const busy = $derived($backupState.step === 'busy');
</script>

<section class="bloque" aria-labelledby="titulo-copia">
  <div>
    <h2 id="titulo-copia">Copia de seguridad</h2>
    <p class="explica">
      Guarda tus ajustes en un fichero para llevarlos a otro equipo o recuperarlos. Importar
      solo añade: nunca borra nada de lo que ya tienes.
    </p>
  </div>

  <label class="casilla">
    <input type="checkbox" bind:checked={includeWork} />
    Incluir etiquetas y notas
  </label>

  <div class="acciones">
    <button type="button" class="boton" onclick={pickExport} disabled={busy}>
      <Download size={16} strokeWidth={1.75} />
      Exportar…
    </button>
    <button type="button" class="boton" onclick={pickImport} disabled={busy}>
      <Upload size={16} strokeWidth={1.75} />
      Importar…
    </button>
  </div>

  {#if $backupState.step === 'exported'}
    <p class="aviso exito" role="status">
      {$backupState.message}
      <button type="button" class="cerrar" onclick={dismissBackup} aria-label="Cerrar">
        <X size={14} strokeWidth={1.75} />
      </button>
    </p>
  {:else if $backupState.step === 'error'}
    <p class="aviso fallo" role="alert">
      No se pudo: {$backupState.message}
      <button type="button" class="cerrar" onclick={dismissBackup} aria-label="Cerrar">
        <X size={14} strokeWidth={1.75} />
      </button>
    </p>
  {:else if $backupState.step === 'preview' || $backupState.step === 'imported'}
    {@const state = $backupState}
    <div class="resumen" role={state.step === 'imported' ? 'status' : undefined}>
      <p class="resumen-titulo">
        {#if state.step === 'preview'}
          Esto es lo que hará importar la copia de {formatRelativeTime(state.plan.exported_at)}
          (Mosaic {state.plan.app_version}):
        {:else}
          Copia importada.
        {/if}
      </p>
      <ul>
        {#each describePlan(state.plan) as line, index (index)}
          <li class={line.tone}>{line.text}</li>
        {/each}
      </ul>
      <div class="acciones">
        {#if state.step === 'preview'}
          <button type="button" class="primario" onclick={() => confirmImport(state.path)}>
            Importar
          </button>
          <button type="button" class="boton" onclick={dismissBackup}>Cancelar</button>
        {:else}
          <button type="button" class="boton" onclick={dismissBackup}>Cerrar</button>
        {/if}
      </div>
    </div>
  {/if}
</section>

<style>
  .bloque {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    margin-top: var(--space-2);
    padding-top: var(--space-5);
    border-top: 1px solid var(--border-subtle);
  }

  h2 {
    margin: 0;
    font-size: var(--text-section);
    line-height: var(--text-section-lh);
    font-weight: var(--text-section-weight);
    letter-spacing: var(--tracking-tight);
    color: var(--text-primary);
  }

  .explica {
    margin: var(--space-1) 0 0;
    font-size: var(--text-body);
    line-height: var(--text-body-lh);
    color: var(--text-secondary);
  }

  .casilla {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-body);
    color: var(--text-primary);
    cursor: pointer;
  }
  .casilla input {
    margin: 0;
    accent-color: var(--accent);
  }

  .acciones {
    display: flex;
    gap: var(--space-2);
  }

  .boton,
  .primario {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-md);
    font: inherit;
    font-size: var(--text-body);
    line-height: var(--text-body-lh);
    font-weight: 500;
    cursor: pointer;
    transition: background var(--duration-fast) var(--ease-out);
  }
  .boton {
    border: 1px solid var(--border-default);
    background: var(--surface-raised);
    color: var(--text-primary);
  }
  .boton:hover:not(:disabled) {
    background: var(--surface-overlay);
  }
  .primario {
    border: 0;
    background: var(--accent);
    color: var(--text-on-accent);
  }
  .primario:hover {
    background: var(--accent-hover);
  }
  .boton:disabled {
    opacity: 0.6;
    cursor: progress;
  }

  .aviso {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    margin: 0;
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-md);
    font-size: var(--text-body);
    line-height: var(--text-body-lh);
  }
  .exito {
    border: 1px solid color-mix(in oklab, var(--success) 40%, transparent);
    background: var(--success-surface);
    color: var(--text-primary);
  }
  .fallo {
    border: 1px solid color-mix(in oklab, var(--danger) 40%, transparent);
    background: var(--danger-surface);
    color: var(--danger);
  }

  .cerrar {
    display: inline-flex;
    flex: none;
    padding: var(--space-1);
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: inherit;
    cursor: pointer;
  }

  .resumen {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    padding: var(--space-4);
    border: 1px solid var(--accent-border);
    border-radius: var(--radius-md);
    background: var(--accent-surface);
  }
  .resumen-titulo {
    margin: 0;
    font-size: var(--text-body);
    line-height: var(--text-body-lh);
    font-weight: 500;
    color: var(--text-primary);
  }
  .resumen ul {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    margin: 0;
    padding-left: var(--space-5);
    font-size: var(--text-body);
    line-height: var(--text-body-lh);
  }
  .change {
    color: var(--text-primary);
  }
  .note {
    color: var(--text-secondary);
  }
  .warning {
    color: var(--warning);
  }

  button:focus-visible,
  input:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
</style>
