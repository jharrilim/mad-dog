import fs from 'node:fs'
import path from 'node:path'
import { defineConfig } from 'vite'
import react, { reactCompilerPreset } from '@vitejs/plugin-react'
import babel from '@rolldown/plugin-babel'
import tailwindcss from '@tailwindcss/vite'

const isGitHubPages = process.env.GITHUB_PAGES === 'true'

// https://vite.dev/config/
export default defineConfig({
  base: isGitHubPages ? '/mad-dog/' : '/',
  plugins: [
    react(),
    babel({ presets: [reactCompilerPreset()] }),
    tailwindcss(),
    {
      name: 'github-pages-spa-fallback',
      closeBundle() {
        if (!isGitHubPages) return
        const dist = path.resolve(__dirname, 'dist')
        fs.copyFileSync(
          path.join(dist, 'index.html'),
          path.join(dist, '404.html'),
        )
      },
    },
  ],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  build: {
    modulePreload: {
      resolveDependencies(_filename, deps) {
        return deps.filter(
          (dep) => !dep.includes('three-lab') && !dep.includes('/Lab-'),
        )
      },
    },
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (
            id.includes('node_modules/three') ||
            id.includes('node_modules/@react-three')
          ) {
            return 'three-lab'
          }
        },
      },
    },
  },
})
