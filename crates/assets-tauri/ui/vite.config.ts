import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [sveltekit()],
	
	// Tauri configuration
	clearScreen: false,
	server: {
		port: 5173,
		strictPort: true,
		host: true, // Listen on all addresses
		watch: {
			// Tell vite to ignore watching src-tauri
			ignored: ["**/src-tauri/**"]
		}
	},
	
	// Prevent vite from obscuring rust errors
	envPrefix: ['VITE_', 'TAURI_'],
});
