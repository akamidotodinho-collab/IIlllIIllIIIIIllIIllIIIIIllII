import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'path';

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
      '@assets': path.resolve(__dirname, './attached_assets'),
    },
  },
  build: {
    outDir: 'dist',
    assetsDir: 'assets',
    sourcemap: false,
    minify: 'esbuild',
    target: 'esnext',
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ['react', 'react-dom'],
        },
      },
    },
  },
  server: {
    host: '0.0.0.0',
    port: 5000,
    strictPort: true,
    allowedHosts: [
      '.replit.dev',
      '.repl.co',
      '23da8a05-b05a-4581-bacf-c158de98a337-00-19py2itl64pku.kirk.replit.dev'
    ],
    hmr: {
      host: '0.0.0.0',
      port: 5000
    },
    watch: {
      ignored: ['**/.local/**', '**/src-tauri/**', '**/target/**', '**/.cargo/**']
    }
  },
  optimizeDeps: {
    include: ['react', 'react-dom'],
  },
});
