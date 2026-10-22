<script lang="ts">
  import { Palette } from '@lucide/svelte';

  import { t } from '../i18n';
  import { dismissable } from '../utils/dismissable';
  import { colorInputValue, TAG_COLORS } from '../utils/tagColors';

  interface Props {
    /** Color actual, el que enseña el selector personalizado. */
    value: string;
    /** Se ha elegido una de las muestras. */
    onPick: (hex: string) => void;
    /** El selector personalizado ha cambiado. */
    onCustom: (hex: string) => void;
    /** Avisar mientras se arrastra, no solo al soltar; no si ya existe: cada aviso la guarda. */
    live?: boolean;
    /** Escape o un clic fuera. */
    onDismiss: () => void;
  }

  let { value, onPick, onCustom, live = false, onDismiss }: Props = $props();

  function custom(event: Event & { currentTarget: HTMLInputElement }) {
    onCustom(event.currentTarget.value.toUpperCase());
  }
</script>

<div use:dismissable={{ onDismiss }} class="paleta">
  <div class="muestras">
    {#each TAG_COLORS as color (color.hex)}
      <button
        type="button"
        class="muestra"
        onclick={() => onPick(color.hex)}
        title={$t(`color.${color.id}`)}
        aria-label={$t('selector.color', { nombre: $t(`color.${color.id}`) })}
        style="background-color: {color.hex};"
      ></button>
    {/each}
  </div>
  <label class="personalizado">
    <Palette size={14} strokeWidth={1.75} />
    {$t('gestor.personalizado')}
    <input
      type="color"
      value={colorInputValue(value)}
      oninput={live ? custom : undefined}
      onchange={live ? undefined : custom}
    />
  </label>
</div>

<style>
  .paleta {
    position: absolute;
    top: 100%;
    left: 0;
    z-index: 50;
    width: 12rem;
    margin-top: var(--space-1);
    padding: var(--space-2);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    background: var(--surface-overlay);
    box-shadow: var(--shadow-lg);
  }

  .muestras {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-1);
  }

  .muestra {
    width: 20px;
    height: 20px;
    padding: 0;
    border: 0;
    border-radius: var(--radius-pill);
    cursor: pointer;
    transition: transform var(--duration-fast) var(--ease-out);
  }
  .muestra:hover {
    transform: scale(1.1);
  }

  .personalizado {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-top: var(--space-2);
    padding-top: var(--space-2);
    border-top: 1px solid var(--border-subtle);
    font-size: var(--text-meta);
    line-height: var(--text-meta-lh);
    color: var(--text-tertiary);
  }
  .personalizado input {
    width: 40px;
    height: 24px;
    margin-left: auto;
    padding: 0;
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    background: transparent;
    cursor: pointer;
  }

  button:focus-visible,
  input:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
</style>
