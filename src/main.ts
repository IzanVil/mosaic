import { mount } from 'svelte'
import './app.css'
import App from './App.svelte'
import { loadViewState, theme } from './lib/stores/filters'
import { applyTheme } from './lib/utils/theme'

/**
 * La vista guardada se lee antes de montar nada: así el tablero no se pinta
 * sin filtros para reordenarse un instante después, y el tema guardado ya está
 * puesto cuando aparece el primer píxel de la interfaz.
 */
async function start() {
  await loadViewState()

  let stopTheme = () => {}
  theme.subscribe((mode) => {
    stopTheme()
    stopTheme = applyTheme(mode)
  })

  mount(App, {
    target: document.getElementById('app')!,
  })
}

void start()
