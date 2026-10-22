import { mount } from 'svelte'
import './app.css'
import App from './App.svelte'
import { lang } from './lib/i18n'
import { loadViewState, theme } from './lib/stores/filters'
import { applyTheme } from './lib/utils/theme'

/** Lee la vista guardada antes de montar: sin reordenar el tablero y con el tema ya puesto. */
async function start() {
  await loadViewState()

  let stopTheme = () => {}
  theme.subscribe((mode) => {
    stopTheme()
    stopTheme = applyTheme(mode)
  })
  lang.subscribe((language) => {
    document.documentElement.lang = language
  })

  mount(App, {
    target: document.getElementById('app')!,
  })
}

void start()
