import {defineConfig} from 'vite';
import react from '@vitejs/plugin-react';
export default defineConfig({root:'ui',plugins:[react()],clearScreen:false,server:{strictPort:true},build:{outDir:'dist',emptyOutDir:true},resolve:{dedupe:['react','react-dom']}});
