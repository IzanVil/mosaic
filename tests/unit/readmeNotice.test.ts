import { describe, expect, it } from 'vitest';

import { omittedText } from '../../src/lib/utils/readmeNotice';

const nada = { images_omitted: 0, badges_omitted: 0, media_omitted: 0 };

describe('aviso de contenido omitido del README', () => {
  it('no dice nada si no falta nada', () => {
    expect(omittedText(nada)).toBeNull();
  });

  it('cuenta las insignias aparte de las imágenes, como en el README de Mosaic', () => {
    expect(omittedText({ ...nada, images_omitted: 9, badges_omitted: 12 })).toBe(
      'Este README tiene 9 imágenes y 12 insignias que Mosaic no muestra.',
    );
  });

  it('con solo insignias no habla de imágenes', () => {
    expect(omittedText({ ...nada, badges_omitted: 1 })).toBe(
      'Este README tiene 1 insignia que Mosaic no muestra.',
    );
  });

  it('con las tres cosas separa con comas y una «y» al final', () => {
    expect(omittedText({ images_omitted: 1, badges_omitted: 2, media_omitted: 1 })).toBe(
      'Este README tiene 1 imagen, 2 insignias y 1 vídeo o elemento incrustado que Mosaic no muestra.',
    );
  });
});
