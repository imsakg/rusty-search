<script lang="ts">
	// Svelte
	import { onMount } from 'svelte';
	import { writable, derived } from 'svelte/store';
	import { getContext } from 'svelte';

	// Skeleton
	import { Pagination } from '@skeletonlabs/skeleton-svelte';
	import { Modal } from '@skeletonlabs/skeleton-svelte';
	import { type ToastContext } from '@skeletonlabs/skeleton-svelte';
	import { Progress } from '@skeletonlabs/skeleton-svelte';

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
	let progress: number | null = null;

	async function start_crawler() {
		const onEvent = new Channel();

		onEvent.onmessage = (message: any) => {
			console.log(`stopped event ${message}`);
		};

		if (!$crawlingStatus) {
			$crawlingStatus = true;

			const totalSources = $sourceData.length;
			for (let i = 0; i < totalSources; i++) {
				if (!$crawlingStatus) {
					break;
				}
				progress = (i / totalSources) * 100;
				await invoke('start_crawler', { targetUrl: $sourceData[i].url, reader: onEvent });
			}
		} else {
			await emit('status-changed', 'shutdown');
		}
		$crawlingStatus = false;
		progress = null;
	}
</script>

<div class="h-1/2 w-1/2 flex-col items-center justify-items-center space-y-8">
	<h4 class="h4 text-center">Crawler</h4>

	<Progress value={progress} max={100}
		>{progress !== null ? `${progress.toFixed(2)}%` : 'waiting to start'}</Progress
	>
	<div class="flex w-full place-items-center items-center justify-center">
		<input
			type="button"
			class="btn bg-primary-400-600"
			value={$crawlingStatus ? 'Stop Crawler' : 'Start Crawler'}
			on:click={start_crawler}
		/>
	</div>
</div>
