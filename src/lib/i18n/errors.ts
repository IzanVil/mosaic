/**
 * Errores del backend en los idiomas que no son el español.
 *
 * El backend ya escribe cada error en español (`message`), así que el español
 * no tiene tabla aquí: se enseña ese mensaje tal cual. Las claves son los
 * códigos de `errors::AppError`, y un test comprueba que no falte ninguno.
 */

import type { Lang } from './index';

type P = Record<string, string>;
/** Texto de un error: fijo, o una función de sus datos ya convertidos en texto. */
export type ErrorText = string | ((params: P) => string);

const en: Record<string, ErrorText> = {
  db: ({ detail }: P) => `Database error: ${detail}`,
  migration: ({ detail }: P) => `Error applying migrations: ${detail}`,
  io: ({ detail }: P) => `File system error: ${detail}`,
  git: ({ detail }: P) => `Git error: ${detail}`,

  path_missing: ({ path }: P) => `${path} no longer exists on disk`,
  path_not_dir: ({ path }: P) => `${path} is not a folder`,
  path_not_utf8: ({ path }: P) => `${path} isn't valid UTF-8`,
  path_unreadable: ({ path, detail }: P) => `Couldn't read ${path}: ${detail}`,

  project_not_found: ({ id }: P) => `Project ${id} not found`,
  tag_not_found: ({ id }: P) => `Tag ${id} not found`,
  scan_path_not_found: ({ id }: P) => `Scan folder ${id} not found`,
  project_or_tag_not_found: ({ project_id, tag_id }: P) =>
    `Project ${project_id} or tag ${tag_id} not found`,
  no_editor: 'No known IDE or editor was found on the PATH',
  no_terminal: 'No known terminal emulator was found on the PATH',

  launch_failed: ({ app, detail }: P) => `Couldn't launch ${app}: ${detail}`,
  task_failed: ({ detail }: P) => `A background task failed: ${detail}`,
  db_locked: 'Internal error: the database lock is poisoned',
  data_dir_unknown: "Internal error: couldn't find the system's data folder",
  scan_path_register: ({ path }: P) => `Internal error: couldn't register the folder ${path}`,
  backup_serialize: ({ detail }: P) => `Internal error: couldn't write the backup: ${detail}`,

  file_manager_fixed: 'The file manager is chosen by the system, not by Mosaic',
  view_state_too_big: ({ max }: P) => `The saved view can't be larger than ${max} bytes`,
  view_state_invalid: ({ detail }: P) => `The saved view isn't valid JSON: ${detail}`,

  depth_out_of_range: ({ min, max }: P) => `Depth has to be between ${min} and ${max}`,
  entries_out_of_range: ({ min, max }: P) => `The entry limit has to be between ${min} and ${max}`,
  git_interval_out_of_range: ({ max }: P) =>
    `The Git interval has to be between 0 and ${max} minutes`,
  excluded_too_many: ({ max }: P) => `There can't be more than ${max} excluded folders`,
  excluded_empty: "An excluded folder name can't be empty",
  excluded_is_path: ({ name }: P) =>
    `“${name}” is a path: exclude folder names, without slashes`,
  excluded_control: "An excluded folder name can't contain control characters",
  excluded_too_long: ({ max }: P) =>
    `An excluded folder name can't be longer than ${max} characters`,
  excluded_duplicate: ({ name }: P) => `“${name}” is already on the list`,

  notes_too_long: ({ max }: P) => `Notes can't be longer than ${max} characters`,

  tag_name_empty: "The tag name can't be empty",
  tag_name_too_long: ({ max, len }: P) =>
    `The tag name can't be longer than ${max} characters (it has ${len})`,
  tag_name_control: "The tag name can't contain control characters",
  tag_color_invalid: ({ color }: P) => `The color has to use the #RRGGBB format, not “${color}”`,
  tag_name_taken: ({ name }: P) => `There's already a tag called “${name}”`,

  url_invalid: ({ detail }: P) => `Invalid link: ${detail}`,
  url_scheme: ({ scheme }: P) => `Only http and https links are opened, not “${scheme}:”`,
  url_no_host: 'The link has no server',

  backup_not_mosaic: "The file isn't a Mosaic backup",
  backup_no_version: "The backup doesn't say its version",
  backup_newer: ({ version }: P) =>
    `The backup comes from a newer version of Mosaic (format ${version}); update Mosaic to import it`,
  backup_damaged: ({ cause }: P) => `The backup is damaged: ${cause}`,
  backup_malformed: ({ detail }: P) => `${detail}`,
  backup_empty_path: "there's an empty scan folder",
  backup_tag_repeated: ({ name }: P) => `the tag “${name}” is repeated`,
  backup_tag_missing: ({ project, name }: P) =>
    `the project ${project} has the tag “${name}”, which isn't in the backup`,
};

/** Los textos de cada idioma que no es el español. */
export const errorTexts: Partial<Record<Lang, Record<string, ErrorText>>> = { en };
