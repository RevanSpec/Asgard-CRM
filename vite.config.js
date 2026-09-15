import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  base: './',

  // Tauri se connecte à `devUrl` (http://localhost:5173). Sans `strictPort`,
  // Vite bascule sur un port libre quand 5173 est pris et la fenêtre reste
  // vide, sans message d'erreur exploitable.
  server: {
    port: 5173,
    strictPort: true,
  },

  // Laisse passer les variables que Tauri injecte à la compilation.
  envPrefix: ['VITE_', 'TAURI_ENV_'],

  build: {
    // Les webviews ciblées sont modernes : WebView2 sous Windows, WebKit
    // ailleurs. Inutile de transpiler pour des moteurs plus anciens.
    target: 'esnext',
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
})
