/**
 * El detalle de un proyecto: README, historial, ramas y las notas. El proyecto en
 * sí sale de `projects`, la única fuente.
 *
 * Las notas se guardan solas 1,5 s después de la última tecla y en el acto al
 * salir; en blanco son `NULL`, y si no han cambiado no se escribe nada.
 */

import { get, writable } from 'svelte/store';

import { getProjectBranches, getProjectHistory } from '../api/git';
import { getProjectReadme, setProjectNotes } from '../api/projects';
import { tNow } from '../i18n';
import type { Branches, CommitInfo, ReadmePreview } from '../types';
import { debounce } from '../utils/debounce';
import { errorText } from '../utils/errors';
import { projectsError, setProjectNotesLocally } from './projects';

/** Espera tras la última tecla antes de guardar las notas. */
export const NOTES_SAVE_DELAY_MS = 1500;

/** Estado de una sección que se carga del disco por separado. */
export type Section<T> =
  | { status: 'loading' }
  | { status: 'ready'; data: T }
  | { status: 'error'; message: string };

/** Qué ha pasado con las notas: un guardado que nadie ve es un guardado que no se cree. */
export type NotesStatus = 'saved' | 'pending' | 'saving' | 'error';

/** README renderizado; `null` dentro de `data` significa que no tiene. */
export const readme = writable<Section<ReadmePreview | null>>({ status: 'loading' });
/** Últimos commits, del más reciente al más antiguo. */
export const history = writable<Section<CommitInfo[]>>({ status: 'loading' });
/** Ramas locales y remotas conocidas. */
export const branches = writable<Section<Branches>>({ status: 'loading' });

/** Texto que hay ahora mismo en el editor de notas. */
export const notesDraft = writable('');
/** Qué ha pasado con lo último que se escribió. */
export const notesStatus = writable<NotesStatus>('saved');
/** Motivo del último fallo al guardar, para enseñarlo junto al editor. */
export const notesError = writable<string | null>(null);

/** Proyecto abierto. Las respuestas de otro proyecto se descartan. */
let currentId: number | null = null;
/** Lo último que el backend confirmó como guardado. */
let savedNotes: string | null = null;

const scheduleSave = debounce(() => void flushNotes(), NOTES_SAVE_DELAY_MS);

/** La misma regla que aplica el backend: en blanco es `NULL`. */
function normalize(text: string): string | null {
  return text.trim() === '' ? null : text;
}

function load<T>(
  target: { set: (value: Section<T>) => void },
  projectId: number,
  request: () => Promise<T>,
): void {
  target.set({ status: 'loading' });
  request().then(
    (data) => {
      if (currentId === projectId) target.set({ status: 'ready', data });
    },
    (error: unknown) => {
      if (currentId === projectId) target.set({ status: 'error', message: errorText(error) });
    },
  );
}

/** Abre el detalle: carga sus tres secciones en paralelo y pone las notas en el editor. */
export function openDetail(projectId: number, notes: string | null): void {
  scheduleSave.cancel();
  currentId = projectId;
  savedNotes = notes;
  notesDraft.set(notes ?? '');
  notesStatus.set('saved');
  notesError.set(null);

  load(readme, projectId, () => getProjectReadme(projectId));
  load(history, projectId, () => getProjectHistory(projectId));
  load(branches, projectId, () => getProjectBranches(projectId));
}

/** Registra una edición de las notas y programa su guardado. */
export function editNotes(text: string): void {
  notesDraft.set(text);
  if (normalize(text) === savedNotes) {
    // Volver a escribir lo que ya estaba guardado no es un cambio pendiente.
    scheduleSave.cancel();
    notesStatus.set('saved');
    return;
  }
  notesStatus.set('pending');
  scheduleSave();
}

/** Guarda las notas ya, sin esperar al plazo, si han cambiado. */
export async function flushNotes(): Promise<void> {
  scheduleSave.cancel();
  const projectId = currentId;
  if (projectId === null) return;

  const text = get(notesDraft);
  if (normalize(text) === savedNotes) {
    notesStatus.set('saved');
    return;
  }

  notesStatus.set('saving');
  try {
    const stored = await setProjectNotes(projectId, text);
    savedNotes = stored;
    setProjectNotesLocally(projectId, stored);
    if (currentId === projectId) {
      notesError.set(null);
      // Si se siguió escribiendo mientras se guardaba, queda algo pendiente.
      notesStatus.set(normalize(get(notesDraft)) === stored ? 'saved' : 'pending');
      if (get(notesStatus) === 'pending') scheduleSave();
    }
  } catch (error) {
    if (currentId === projectId) {
      notesStatus.set('error');
      notesError.set(errorText(error));
    }
  }
}

/** Cierra el detalle: guarda lo pendiente, espera a que acabe y olvida el proyecto. */
export async function closeDetail(): Promise<void> {
  await flushNotes();
  if (get(notesStatus) === 'error') {
    // La vista ya no está: sin esto se perderían en silencio. Sale en el aviso del tablero.
    projectsError.set(
      tNow('notas.errorAlSalir', { error: get(notesError) ?? tNow('notas.errorDesconocido') }),
    );
  }
  currentId = null;
}
