/** Texto del aviso de lo que un README tiene y Mosaic no muestra. */

import { get } from "svelte/store";

import { lang, translate, type Lang } from "../i18n";
import type { ReadmePreview } from "../types";

/** Qué se ha quedado fuera, con números, o `null`: si no se dice, parece que algo ha fallado. */
export function omittedText(
  readme: Pick<
    ReadmePreview,
    "images_omitted" | "badges_omitted" | "media_omitted"
  >,
  language: Lang = get(lang),
): string | null {
  const parts: string[] = [];
  if (readme.images_omitted > 0)
    parts.push(translate(language, "readme.imagenes", { n: readme.images_omitted }));
  if (readme.badges_omitted > 0)
    parts.push(translate(language, "readme.insignias", { n: readme.badges_omitted }));
  if (readme.media_omitted > 0)
    parts.push(translate(language, "readme.medios", { n: readme.media_omitted }));
  if (parts.length === 0) return null;
  const list =
    parts.length === 1
      ? parts[0]
      : `${parts.slice(0, -1).join(", ")} ${translate(language, "comun.y")} ${parts.at(-1)}`;
  return translate(language, "readme.omitido", { lista: list ?? "" });
}
