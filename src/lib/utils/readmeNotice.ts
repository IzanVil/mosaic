/** Texto del aviso de lo que un README tiene y Mosaic no muestra. */

import type { ReadmePreview } from "../types";

function count(n: number, one: string, many: string): string {
  return `${n} ${n === 1 ? one : many}`;
}

/**
 * Qué se ha quedado fuera, dicho con números, o `null` si no falta nada. Si el
 * README tiene imágenes y no se dice, el lector cree que no las tiene o que
 * algo ha fallado. Las insignias van aparte: no son contenido que se eche de
 * menos como una captura.
 */
export function omittedText(
  readme: Pick<
    ReadmePreview,
    "images_omitted" | "badges_omitted" | "media_omitted"
  >,
): string | null {
  const parts: string[] = [];
  if (readme.images_omitted > 0)
    parts.push(count(readme.images_omitted, "imagen", "imágenes"));
  if (readme.badges_omitted > 0)
    parts.push(count(readme.badges_omitted, "insignia", "insignias"));
  if (readme.media_omitted > 0) {
    parts.push(
      count(
        readme.media_omitted,
        "vídeo o elemento incrustado",
        "vídeos o elementos incrustados",
      ),
    );
  }
  if (parts.length === 0) return null;
  const list =
    parts.length === 1
      ? parts[0]
      : `${parts.slice(0, -1).join(", ")} y ${parts.at(-1)}`;
  return `Este README tiene ${list} que Mosaic no muestra.`;
}
