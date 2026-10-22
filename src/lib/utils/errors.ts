/** Texto de un error para enseñarlo en la interfaz, en el idioma de esta. */

import { get } from 'svelte/store';

import { lang, type Lang } from '../i18n';
import { errorTexts } from '../i18n/errors';
import type { CommandError } from '../types';

/** Si lo que se ha capturado es un error de un comando del backend. */
export function isCommandError(value: unknown): value is CommandError {
  if (typeof value !== 'object' || value === null) return false;
  const candidate = value as Partial<CommandError>;
  return typeof candidate.code === 'string' && typeof candidate.message === 'string';
}

/** Texto de un error: por su código si el idioma lo tiene; si no, el mensaje del backend. */
export function errorText(error: unknown, language: Lang = get(lang)): string {
  if (isCommandError(error)) return commandErrorText(error, language);
  if (error instanceof Error) return error.message;
  return String(error);
}

function commandErrorText(error: CommandError, language: Lang): string {
  const text = errorTexts[language]?.[error.code];
  if (text === undefined) return error.message;
  if (typeof text === 'string') return text;

  const params: Record<string, string> = {};
  for (const [key, value] of Object.entries(error.params ?? {})) {
    // Una causa va dentro de otra frase: «The backup is damaged: the tag…».
    params[key] = isCommandError(value)
      ? lowerFirst(commandErrorText(value, language))
      : String(value);
  }
  return text(params);
}

function lowerFirst(text: string): string {
  return text.charAt(0).toLowerCase() + text.slice(1);
}
