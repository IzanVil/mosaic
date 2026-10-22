/**
 * Los recorridos principales de la interfaz, de punta a punta, sobre la app
 * compilada y el backend simulado de `tests/fixtures/backend.ts`.
 *
 * Buscan los elementos por su papel y su texto, como los encontraría quien usa
 * la app, así que corren en español: `playwright.config.ts` fija el idioma.
 */

import { expect, test, type Page } from '@playwright/test';

import { installBackend, type Seed } from '../fixtures/backend';

const CARPETA = '/home/izan/proyectos';
const DISCO = [
  { name: 'api-facturas', language: 'Rust', dirty: true },
  { name: 'panel-clientes', language: 'Svelte', dirty: false },
  { name: 'mosaic', language: 'Svelte', dirty: false },
];

async function abrir(page: Page, seed: Partial<Seed> = {}) {
  await page.addInitScript(installBackend, { folder: CARPETA, disk: DISCO, scanned: true, ...seed });
  await page.goto('/');
}

function tarjeta(page: Page, nombre: string) {
  return page.locator('article', { has: page.getByRole('button', { name: nombre, exact: true }) });
}

test('añadir una carpeta, escanear y ver las tarjetas', async ({ page }) => {
  await abrir(page, { scanned: false });

  await page.getByRole('button', { name: 'Configurar rutas' }).click();
  await page.getByRole('button', { name: 'Añadir carpeta' }).first().click();
  await expect(page.getByText(CARPETA, { exact: true })).toBeVisible();

  await page.getByRole('button', { name: 'Proyectos', exact: true }).click();
  await page.getByRole('button', { name: 'Escanear ahora' }).click();

  for (const { name } of DISCO) {
    await expect(tarjeta(page, name)).toBeVisible();
  }
  await expect(page.getByText('3 proyectos', { exact: true })).toBeVisible();
});

test('crear una etiqueta desde una tarjeta y filtrar por ella', async ({ page }) => {
  await abrir(page);

  const panel = tarjeta(page, 'panel-clientes');
  await panel.hover();
  await panel.getByTitle('Asignar etiquetas').click();
  await page.getByPlaceholder('Buscar o crear…').fill('cliente');
  await page.getByRole('button', { name: 'Crear «cliente»' }).click();
  await expect(panel.getByTitle('cliente', { exact: true })).toBeVisible();

  const lateral = page.getByRole('navigation', { name: 'Colecciones' });
  await lateral.getByRole('button', { name: /^cliente/ }).click();

  await expect(page.locator('article')).toHaveCount(1);
  await expect(panel).toBeVisible();
});

test('las notas se guardan al salir del detalle, también con Escape', async ({ page }) => {
  await abrir(page);
  const notas = page.getByLabel('Notas del proyecto');

  // Con el botón de volver, el cuadro pierde el foco y eso ya las guarda.
  await page.getByRole('button', { name: 'mosaic', exact: true }).click();
  await notas.fill('Revisar el instalador');
  await page.getByTitle('Volver a proyectos (Esc)').click();
  await page.getByRole('button', { name: 'mosaic', exact: true }).click();
  await expect(notas).toHaveValue('Revisar el instalador');

  // Con Escape el foco no sale del cuadro: solo las guarda el cierre de la vista.
  await notas.fill('Revisar el instalador de Windows');
  await page.keyboard.press('Escape');
  await expect(tarjeta(page, 'mosaic')).toBeVisible();

  // Recargar es cerrar y abrir la app: lo que se ve ahora viene del backend.
  await page.reload();
  await page.getByRole('button', { name: 'mosaic', exact: true }).click();
  await expect(notas).toHaveValue('Revisar el instalador de Windows');
});

test('buscar un proyecto en la paleta y abrir su detalle', async ({ page }) => {
  await abrir(page);
  await expect(tarjeta(page, 'mosaic')).toBeVisible();

  await page.keyboard.press('ControlOrMeta+k');
  const paleta = page.getByRole('dialog', { name: 'Paleta de comandos' });
  await paleta.getByRole('combobox').fill('panel');
  await page.keyboard.press('Enter');

  await expect(paleta).toBeHidden();
  await expect(page.locator('#titulo-proyecto')).toHaveText('panel-clientes');
});
