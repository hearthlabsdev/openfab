import { defineConfig } from 'vite';
import { resolve } from 'path';

export default defineConfig({
  build: {
    outDir: resolve(__dirname, '../static/js'),
    emptyOutDir: false,
    // Configure library mode to enforce a clean IIFE/ES bundle without wrapper exports
    lib: {
      entry: resolve(__dirname, 'src/main.ts'),
      name: 'PrinterFrontend',
      formats: ['es'],
      fileName: () => 'bundle.js',
    },
    rollupOptions: {
      // Lit is bundled directly so the browser doesn't need bare module import maps
      output: {
        inlineDynamicImports: true,
      },
    },
  },
});