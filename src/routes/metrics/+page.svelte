<script lang="ts">
	// Svelte
	import { onMount } from 'svelte';

	// Tauri
	// When using the Tauri API npm package:
	import { invoke, Channel } from '@tauri-apps/api/core';

	// When using the Tauri global script (if not using the npm package)
	// Be sure to set `app.withGlobalTauri` in `tauri.conf.json` to true
	// const invoke = window.__TAURI__.core.invoke;

	let colors = ['White', 'Red', 'Yellow', 'Green', 'Blue', 'Black'];
	let selected;
	let searchTerm = '';
	const minSearchLength = 2;
	let suggestions = ['White', 'Red', 'Yellow', 'Green', 'Blue', 'Black'];

	onMount(() => {});

	async function query() {
		if (searchTerm.length > minSearchLength) {
			// let response = await invoke('query', { searchTerm });
		}
	}

	async function handleInput(event: Event) {
		searchTerm = (event.target as HTMLInputElement).value;
		if (searchTerm.length >= minSearchLength) {
			let response = await invoke('query_db', { query: searchTerm });
			console.log(response);
			suggestions = colors.filter((color) =>
				color.toLowerCase().includes(searchTerm.toLowerCase())
			);
		} else {
			suggestions = [];
		}
	}

	function selectSuggestion(color: string) {
		selected = color;
		searchTerm = color;
		suggestions = [];
	}
</script>

<div class="container mx-auto">
	<div class="flex flex-col items-center justify-center space-y-1">
		<h1>Metrics</h1>

		<!-- AutoComplete Search Input -->
		<input
			bind:value={searchTerm}
			on:input={handleInput}
			autocomplete="off"
			type="input text"
			placeholder="Search..."
			class="input w-64 rounded border p-2"
			id="searchbar"
		/>
		{#if suggestions.length}
			<ul class=" w-64 border">
				{#each suggestions as suggestion}
					<li
						class="cursor-pointer p-2 hover:bg-gray-100"
						on:click={() => selectSuggestion(suggestion)}
					>
						{suggestion}
					</li>
				{/each}
			</ul>
		{/if}
	</div>
</div>
