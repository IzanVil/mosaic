/**
 * La conversión de remoto a página web es lo que abre el navegador desde el
 * aviso del README: una URL mal formada o con credenciales no puede salir.
 */

import { describe, expect, it } from 'vitest';

import { remoteWebUrl } from '../../src/lib/utils/remote';

describe('remoteWebUrl', () => {
  it('entiende https, con y sin .git', () => {
    expect(remoteWebUrl('https://github.com/IzanVil/mosaic.git')).toEqual({
      url: 'https://github.com/IzanVil/mosaic',
      host: 'GitHub',
    });
    expect(remoteWebUrl('https://gitlab.com/grupo/sub/proyecto/')?.url).toBe(
      'https://gitlab.com/grupo/sub/proyecto',
    );
  });

  it('entiende la forma corta de SSH y ssh://', () => {
    expect(remoteWebUrl('git@github.com:IzanVil/mosaic.git')?.url).toBe(
      'https://github.com/IzanVil/mosaic',
    );
    expect(remoteWebUrl('ssh://git@codeberg.org:2222/yo/repo.git')).toEqual({
      url: 'https://codeberg.org/yo/repo',
      host: 'Codeberg',
    });
  });

  it('descarta las credenciales de la URL', () => {
    const web = remoteWebUrl('https://usuario:token@github.com/yo/privado.git');
    expect(web?.url).toBe('https://github.com/yo/privado');
    expect(web?.url).not.toContain('token');
  });

  it('da nombre solo a los servicios conocidos', () => {
    expect(remoteWebUrl('https://git.miempresa.es/equipo/app.git')).toEqual({
      url: 'https://git.miempresa.es/equipo/app',
      host: null,
    });
  });

  it('no inventa una web para remotos locales o vacíos', () => {
    for (const remote of [null, '', '   ', '/srv/git/repo.git', 'file:///srv/git/repo.git', '../otro', 'https://github.com/']) {
      expect(remoteWebUrl(remote), String(remote)).toBeNull();
    }
  });
});
