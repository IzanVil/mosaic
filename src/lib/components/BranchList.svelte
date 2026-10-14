<script lang="ts">
  import type { Section } from '../stores/projectDetail';
  import type { Branches } from '../types';
  import { t } from '../i18n';

  interface Props {
    /** Ramas tal y como las carga `stores/projectDetail.ts`. */
    section: Section<Branches>;
    /** Si es `false`, la lista vacía se explica como «no es un repositorio». */
    isRepo: boolean;
  }

  let { section, isRepo }: Props = $props();
</script>

{#if section.status === 'loading'}
  <p class="nota">{$t('ramas.leyendo')}</p>
{:else if section.status === 'error'}
  <p class="nota error">{$t('ramas.error', { error: section.message })}</p>
{:else if !isRepo}
  <p class="nota">{$t('comun.noRepo')}</p>
{:else}
  <div class="grupos">
    <div>
      <h3>{$t('ramas.locales')}</h3>
      {#if section.data.local.length === 0}
        <p class="nota">{$t('ramas.sinLocales')}</p>
      {:else}
        <ul>
          {#each section.data.local as branch (branch.name)}
            <li class:actual={branch.is_head}>
              <code>{branch.name}</code>
              {#if branch.is_head}<span class="marca">{$t('ramas.actual')}</span>{/if}
            </li>
          {/each}
        </ul>
      {/if}
    </div>

    <div>
      <h3>{$t('ramas.remotas')}</h3>
      {#if section.data.remote.length === 0}
        <p class="nota">{$t('ramas.sinRemotas')}</p>
      {:else}
        <ul>
          {#each section.data.remote as branch (branch.name)}
            <li><code>{branch.name}</code></li>
          {/each}
        </ul>
      {/if}
      <!-- Mosaic no hace fetch: si no se dice, una rama que falta parece un fallo. -->
      <p class="nota">{$t('ramas.nota')}</p>
    </div>
  </div>
{/if}

<style>
  .grupos {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  h3 {
    margin: 0 0 var(--space-2);
    font-size: var(--text-meta);
    line-height: var(--text-meta-lh);
    font-weight: 500;
    color: var(--text-secondary);
  }

  ul {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-1);
    margin: 0 0 var(--space-2);
    padding: 0;
    list-style: none;
  }

  li {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    padding: 2px var(--space-2);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    background: var(--surface-base);
  }
  li.actual {
    border-color: var(--accent-border);
  }

  code {
    font-family: var(--font-mono);
    font-size: var(--text-code);
    line-height: var(--text-code-lh);
    color: var(--text-secondary);
  }
  li.actual code {
    color: var(--text-primary);
  }

  .marca {
    font-size: var(--text-meta);
    color: var(--accent);
  }

  .nota {
    margin: 0;
    font-size: var(--text-meta);
    line-height: var(--text-meta-lh);
    color: var(--text-tertiary);
  }
  .nota.error {
    color: var(--danger);
  }
</style>
