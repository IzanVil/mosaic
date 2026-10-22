/**
 * Del remoto de Git a la página web del repositorio, donde sí se ven las imágenes del
 * README. No toca la red: solo reescribe la URL, y la abre el navegador si se pide.
 */

/** Página web de un repositorio y el nombre de su servicio, si se conoce. */
export interface RemoteWeb {
  /** URL `https://` (o `http://`, si el remoto lo era) de la página. */
  url: string;
  /** «GitHub», «GitLab», «Codeberg» o `null` si es otro servidor. */
  host: string | null;
}

const KNOWN_HOSTS: Record<string, string> = {
  'github.com': 'GitHub',
  'gitlab.com': 'GitLab',
  'codeberg.org': 'Codeberg',
  'bitbucket.org': 'Bitbucket',
};

/** La web de un remoto `https`, `ssh` o `git@host:ruta`; `null` si es local o no tiene ruta. */
export function remoteWebUrl(remote: string | null | undefined): RemoteWeb | null {
  // Las credenciales que lleve la URL se descartan: no deben acabar en el navegador.
  const raw = (remote ?? '').trim();
  if (raw === '') return null;

  let scheme = 'https';
  let host: string;
  let path: string;

  const scp = /^[\w.-]+@([^:/\s]+):(?!\/)(.+)$/.exec(raw);
  if (scp) {
    host = scp[1] ?? '';
    path = scp[2] ?? '';
  } else {
    let url: URL;
    try {
      url = new URL(raw);
    } catch {
      return null;
    }
    if (url.protocol === 'http:') scheme = 'http';
    else if (url.protocol !== 'https:' && url.protocol !== 'ssh:' && url.protocol !== 'git:') {
      return null;
    }
    host = url.hostname;
    path = url.pathname;
  }

  path = path.replace(/^\/+/, '').replace(/\/+$/, '').replace(/\.git$/, '');
  if (host === '' || path === '') return null;

  return {
    url: `${scheme}://${host}/${path}`,
    host: KNOWN_HOSTS[host.toLowerCase()] ?? null,
  };
}
