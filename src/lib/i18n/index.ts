/**
 * Idioma de la interfaz.
 *
 * La preferencia (`system`, `es` o `en`) se guarda con la vista; el idioma que
 * se usa de verdad sale de ella y, en modo sistema, de lo que diga el navegador.
 * Los componentes leen los textos con `$t('clave')`, así que al cambiar de
 * idioma se repintan solos.
 */

import { derived, get, writable, type Readable } from 'svelte/store';

import { en } from './en';
import { es } from './es';

/** Idioma de la interfaz. */
export type Lang = 'es' | 'en';
/** Lo que elige el usuario: un idioma o el del sistema. */
export type LangPreference = 'system' | Lang;
/** Las preferencias válidas, en el orden del selector. */
export const LANG_PREFERENCES: LangPreference[] = ['system', 'es', 'en'];

/** Clave de un texto. */
export type Key = keyof typeof es;
type Params<K extends Key> = (typeof es)[K] extends (p: infer P) => string ? [P] : [];

/**
 * El idioma del sistema: el primero de la lista del navegador que Mosaic sepa
 * hablar. Si no habla ninguno, inglés, que es el que más gente entiende.
 */
export function systemLang(languages: readonly string[] = navigatorLanguages()): Lang {
  for (const language of languages) {
    const base = language.toLowerCase().split(/[-_]/)[0];
    if (base === 'es' || base === 'en') return base;
  }
  return 'en';
}

function navigatorLanguages(): readonly string[] {
  if (typeof navigator === 'undefined') return [];
  return navigator.languages?.length ? navigator.languages : [navigator.language];
}

/** La preferencia guardada. */
export const langPreference = writable<LangPreference>('system');

/** El idioma que se usa ahora. */
export const lang = derived(langPreference, (preference) =>
  preference === 'system' ? systemLang() : preference,
);

/** Texto de una clave en un idioma concreto. */
export function translate<K extends Key>(language: Lang, key: K, ...params: Params<K>): string {
  const entry = (language === 'en' ? en : es)[key] as string | ((p: unknown) => string);
  return typeof entry === 'function' ? entry(params[0]) : entry;
}

/** Función de traducir: la clave y, si el texto lleva datos, sus datos. */
export type Translator = <K extends Key>(key: K, ...params: Params<K>) => string;

/** La función de traducir del idioma actual: `$t('clave')` o `$t('clave', { n })`. */
export const t: Readable<Translator> = derived(
  lang,
  (language): Translator =>
    (key, ...params) =>
      translate(language, key, ...params),
);

/** Lo mismo fuera de un componente, con el idioma de este momento. */
export function tNow<K extends Key>(key: K, ...params: Params<K>): string {
  return translate(get(lang), key, ...params);
}
