/**
 * Las cuatro reglas de las notas y el regreso del foco desde el detalle.
 *
 * Las notas son lo único de la vista de detalle que escribe en la base de
 * datos, y lo hace solo, sin botón de guardar: si una regla se rompe, el
 * usuario pierde texto sin enterarse. Por eso cada regla tiene su test.
 */

import { get } from 'svelte/store';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('../../src/lib/api/projects', () => ({
  setProjectNotes: vi.fn(async (_id: number, notes: string) =>
    notes.trim() === '' ? null : notes,
  ),
  getProjectReadme: vi.fn(async () => null),
  listProjectsWithTags: vi.fn(async () => []),
  setProjectPinned: vi.fn(async () => undefined),
}));
vi.mock('../../src/lib/api/git', () => ({
  getProjectHistory: vi.fn(async () => []),
  getProjectBranches: vi.fn(async () => ({ local: [], remote: [] })),
}));

import { setProjectNotes } from '../../src/lib/api/projects';
import {
  NOTES_SAVE_DELAY_MS,
  closeDetail,
  editNotes,
  notesStatus,
  openDetail,
} from '../../src/lib/stores/projectDetail';
import {
  goToSettings,
  leaveProjectDetail,
  openProjectDetail,
  route,
} from '../../src/lib/stores/navigation';

const save = vi.mocked(setProjectNotes);

beforeEach(() => {
  vi.useFakeTimers();
  save.mockClear();
});

afterEach(async () => {
  await closeDetail();
  vi.useRealTimers();
});

describe('notas', () => {
  it('se guardan solas 1,5 s después de la última tecla, y no antes', async () => {
    openDetail(1, null);
    editNotes('p');
    editNotes('pe');
    editNotes('pendiente');

    await vi.advanceTimersByTimeAsync(NOTES_SAVE_DELAY_MS - 1);
    expect(save).not.toHaveBeenCalled();
    expect(get(notesStatus)).toBe('pending');

    await vi.advanceTimersByTimeAsync(1);
    expect(save).toHaveBeenCalledTimes(1);
    expect(save).toHaveBeenCalledWith(1, 'pendiente');
    expect(get(notesStatus)).toBe('saved');
  });

  it('al salir de la vista se guardan en el acto', async () => {
    openDetail(1, null);
    editNotes('a medio escribir');

    await closeDetail();

    expect(save).toHaveBeenCalledWith(1, 'a medio escribir');
  });

  it('vaciarlas guarda NULL, y dejarlas en blanco sin tener nada no escribe', async () => {
    openDetail(1, 'algo');
    editNotes('   ');
    await vi.advanceTimersByTimeAsync(NOTES_SAVE_DELAY_MS);
    expect(save).toHaveBeenCalledWith(1, '   ');
    expect(await save.mock.results[0]?.value).toBeNull();

    save.mockClear();
    openDetail(2, null);
    editNotes('\n\t ');
    await closeDetail();
    expect(save).not.toHaveBeenCalled();
  });

  it('no escriben nada si el texto vuelve a ser el guardado', async () => {
    openDetail(1, 'original');
    editNotes('originalx');
    editNotes('original');
    expect(get(notesStatus)).toBe('saved');

    await vi.advanceTimersByTimeAsync(NOTES_SAVE_DELAY_MS * 2);
    await closeDetail();
    expect(save).not.toHaveBeenCalled();
  });
});

describe('volver del detalle', () => {
  it('devuelve la tarjeta de la que se vino', () => {
    openProjectDetail(7);
    expect(get(route)).toEqual({ view: 'project', projectId: 7 });
    expect(leaveProjectDetail()).toBe(7);
    expect(get(route)).toEqual({ view: 'dashboard' });
  });

  it('olvida la tarjeta si la navegación pasó por Ajustes', () => {
    openProjectDetail(7);
    goToSettings();
    expect(leaveProjectDetail()).toBeNull();
  });
});
