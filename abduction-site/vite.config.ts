import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [sveltekit()],
	server: {
		proxy: {
			'/_/': {
				target: 'http://localhost:9944',
				ws: true,
				rewrite: (path) => path.replace('/_/', '/')
			}
		}
	}
});
