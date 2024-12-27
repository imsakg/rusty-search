import containerQueries from '@tailwindcss/container-queries';
import forms from '@tailwindcss/forms';
import typography from '@tailwindcss/typography';
import type { Config } from 'tailwindcss';

import { skeleton, contentPath } from '@skeletonlabs/skeleton/plugin';
import newyork from './src/themes/newyork';

export default {
	content: ['./src/**/*.{html,js,svelte,ts}', contentPath(import.meta.url, 'svelte')],

	theme: {
		extend: {}
	},

	plugins: [
		typography,
		forms,
		containerQueries,
		skeleton({
			// NOTE: each theme included will increase the size of your CSS bundle
			themes: [newyork]
		})
	]
} satisfies Config;
