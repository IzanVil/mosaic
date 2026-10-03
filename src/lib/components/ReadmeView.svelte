<script lang="ts">
  import { FileText, TriangleAlert } from '@lucide/svelte';
  import { onDestroy } from 'svelte';

  import { openExternal } from '../api/system';
  import type { Section } from '../stores/projectDetail';
  import type { ReadmePreview } from '../types';

  interface Props {
    /** README tal y como lo carga `stores/projectDetail.ts`. */
    section: Section<ReadmePreview | null>;
    /** La carpeta ya no está en disco: cambia el mensaje de «sin README». */
    missing: boolean;
  }

  let { section, missing }: Props = $props();

  /** Aviso efímero tras pulsar un enlace, para que ningún clic quede mudo. */
  let notice = $state<string | null>(null);
  let timer: ReturnType<typeof setTimeout> | undefined;
  onDestroy(() => clearTimeout(timer));

  function say(text: string) {
    clearTimeout(timer);
    notice = text;
    timer = setTimeout(() => (notice = null), 3000);
  }

  /**
   * Ningún enlace del README navega dentro de la ventana: eso sacaría a la
   * aplicación de sí misma. Los web se abren en el navegador; el resto
   * (rutas relativas, anclas, correo) se explica en vez de no hacer nada.
   */
  function onClick(event: MouseEvent) {
    const target = event.target;
    if (!(target instanceof Element)) return;
    const link = target.closest('a');
    if (!link) return;

    event.preventDefault();
    const href = link.getAttribute('href') ?? '';
    if (/^https?:\/\//i.test(href)) {
      openExternal(href).then(
        () => say('Abierto en el navegador.'),
        (error: unknown) => say(`No se pudo abrir el enlace: ${String(error)}`),
      );
    } else {
      say('Este enlace apunta dentro del repositorio. Mosaic solo abre enlaces web.');
    }
  }
</script>

{#if section.status === 'loading'}
  <p class="nota">Leyendo el README…</p>
{:else if section.status === 'error'}
  <p class="nota error">No se pudo leer el README: {section.message}</p>
{:else if section.data === null}
  <div class="vacio">
    <FileText size={24} strokeWidth={1.5} />
    <p>
      {missing
        ? 'La carpeta ya no está en disco, así que no hay README que leer.'
        : 'Este proyecto no tiene README en su carpeta raíz.'}
    </p>
  </div>
{:else}
  <div class="cabecera">
    <code>{section.data.file_name}</code>
    {#if notice}<span class="aviso" role="status">{notice}</span>{/if}
  </div>
  {#if section.data.truncated}
    <p class="recortado">
      <TriangleAlert size={14} strokeWidth={1.75} />
      El fichero pasa de 512 KB y solo se muestra el principio.
    </p>
  {/if}
  <!-- El clic solo intercepta los enlaces de dentro, que ya son focusables y se activan con Enter. -->
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions -->
  <article class="readme" onclick={onClick}>
    <!-- El HTML llega descartado de HTML crudo y saneado con ammonia en el backend (core/readme.rs). -->
    {@html section.data.html}
  </article>
{/if}

<style>
  .nota {
    margin: 0;
    font-size: var(--text-body);
    color: var(--text-tertiary);
  }
  .nota.error {
    color: var(--danger);
  }

  .vacio {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-7) var(--space-4);
    color: var(--text-tertiary);
    text-align: center;
  }
  .vacio p {
    max-width: 24rem;
    margin: 0;
    font-size: var(--text-body);
    line-height: var(--text-body-lh);
  }

  .cabecera {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    min-height: 24px;
    margin-bottom: var(--space-4);
    padding-bottom: var(--space-3);
    border-bottom: 1px solid var(--border-subtle);
  }
  .cabecera code {
    font-family: var(--font-mono);
    font-size: var(--text-code);
    color: var(--text-tertiary);
  }
  .aviso {
    font-size: var(--text-meta);
    color: var(--text-secondary);
  }

  .recortado {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin: 0 0 var(--space-4);
    font-size: var(--text-meta);
    color: var(--warning);
  }

  /*
   * Tipografía del README. Va con :global porque el HTML llega ya hecho y
   * Svelte no puede marcarlo con su hash; todo queda dentro de `.readme`.
   */
  .readme {
    max-width: 46rem;
    font-size: var(--text-body);
    line-height: 1.65;
    color: var(--text-secondary);
    overflow-wrap: anywhere;
  }
  .readme :global(:where(h1, h2, h3, h4, h5, h6)) {
    margin: var(--space-6) 0 var(--space-3);
    color: var(--text-primary);
    letter-spacing: var(--tracking-tight);
    line-height: 1.25;
  }
  .readme :global(> :first-child) {
    margin-top: 0;
  }
  .readme :global(h1) {
    font-size: var(--text-title);
    font-weight: var(--text-title-weight);
  }
  .readme :global(h2) {
    padding-bottom: var(--space-2);
    border-bottom: 1px solid var(--border-subtle);
    font-size: var(--text-section);
    font-weight: var(--text-section-weight);
  }
  .readme :global(:where(h3, h4, h5, h6)) {
    font-size: var(--text-card);
    font-weight: var(--text-card-weight);
  }
  .readme :global(:where(p, ul, ol, pre, blockquote, table)) {
    margin: 0 0 var(--space-4);
  }
  .readme :global(:where(ul, ol)) {
    padding-left: var(--space-5);
  }
  .readme :global(li + li) {
    margin-top: var(--space-1);
  }
  .readme :global(a) {
    color: var(--accent);
    text-decoration: underline;
    text-underline-offset: 2px;
    cursor: pointer;
  }
  .readme :global(a:hover) {
    color: var(--accent-hover);
  }
  .readme :global(code) {
    padding: 1px 5px;
    border-radius: var(--radius-sm);
    background: var(--surface-sunken);
    font-family: var(--font-mono);
    font-size: var(--text-code);
    color: var(--text-primary);
  }
  .readme :global(pre) {
    padding: var(--space-3) var(--space-4);
    overflow-x: auto;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    background: var(--surface-base);
  }
  .readme :global(pre code) {
    padding: 0;
    background: none;
    line-height: var(--text-code-lh);
  }
  .readme :global(blockquote) {
    padding-left: var(--space-3);
    border-left: 3px solid var(--border-default);
    color: var(--text-tertiary);
  }
  .readme :global(table) {
    display: block;
    max-width: 100%;
    overflow-x: auto;
    border-collapse: collapse;
  }
  .readme :global(:where(th, td)) {
    padding: var(--space-1) var(--space-3);
    border: 1px solid var(--border-subtle);
    text-align: left;
  }
  .readme :global(th) {
    background: var(--surface-sunken);
    color: var(--text-primary);
    font-weight: 600;
  }
  .readme :global(hr) {
    margin: var(--space-5) 0;
    border: 0;
    border-top: 1px solid var(--border-subtle);
  }
  .readme :global(:focus-visible) {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
</style>
