<script lang="ts">
	// Svelte
	import { onMount } from 'svelte';
	import { writable, derived } from 'svelte/store';
	import { getContext } from 'svelte';

	// Skeleton
	import { Pagination } from '@skeletonlabs/skeleton-svelte';
	import { Modal } from '@skeletonlabs/skeleton-svelte';
	import { type ToastContext } from '@skeletonlabs/skeleton-svelte';

	// Tauri
	// When using the Tauri API npm package:
	import { invoke, Channel } from '@tauri-apps/api/core';

	// When using the Tauri global script (if not using the npm package)
	// Be sure to set `app.withGlobalTauri` in `tauri.conf.json` to true
	// const invoke = window.__TAURI__.core.invoke;

	// Libs
	import { sources } from '$lib/stores';
	import { emit } from '@tauri-apps/api/event';

	// Stores
	const sourceData = sources;

	let crawlingStatus = writable(false);

	async function start_crawler() {
		const onEvent = new Channel();

		onEvent.onmessage = (message: any) => {
			console.log(`stopped event ${message.event}`);
		};

		if (!$crawlingStatus) {
			$crawlingStatus = true;
			await invoke('start_crawler', { targetUrl: 'https://python.org', reader: onEvent });
			$crawlingStatus = false;
		} else {
			await emit('status-changed', { status: 'stop' });
			$crawlingStatus = false;
		}
	}
</script>

<h1>Crawler</h1>
<input
	type="button"
	class="btn bg-primary-400-600"
	value={$crawlingStatus ? 'Stop Crawler' : 'Start Crawler'}
	on:click={start_crawler}
/>
