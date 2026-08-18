import adapter from '@openworkers/adapter-sveltekit';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
export default {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter({ outDir: 'build' }),
    // Fixed: the default is Date.now(), which would change every asset hash and
    // every rendered byte, and the whole point here is a reproducible oracle.
    version: { name: 'conformance' }
  }
};
