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
	let suggestions: object[] = [];

	onMount(() => {});

	async function query() {
		if (searchTerm.length > minSearchLength) {
			// let response = await invoke('query', { searchTerm });
		}
	}

	async function handleInput(event: Event) {
		searchTerm = (event.target as HTMLInputElement).value;
		if (searchTerm.length >= minSearchLength) {
			let response: object[] = await invoke('query_db', { query: searchTerm });
			console.log(response);
			suggestions = response;
			// suggestions = colors.filter((color) =>
			// 	color.toLowerCase().includes(searchTerm.toLowerCase())
			// );
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

<div class="container mx-auto h-full">
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
		<!-- {#if suggestions.length}
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
		{/if} -->
	</div>
	<div class="table-wrap h-fit scrollbar-thin">
		<table class="table w-full caption-bottom">
			<caption class="pt-4">A list of elements from the periodic table.</caption>
			<thead>
				<tr>
					<th>Row ID</th>
					<th>highlight</th>
					<th>URL</th>
					<th class="!text-right">Score</th>
				</tr>
			</thead>
			<tbody class="hover:[&>tr]:preset-tonal-primary">
				{#each suggestions as row}
					<tr>
						<td>{row.id.tb} - {row.id.id.String}</td>
						<td class=" overflow-ellipsis">{row.highlight}</td>
						<td>{row.address}</td>
						<td class="text-right">{row.score}</td>
					</tr>
				{/each}
			</tbody>
			<tfoot>
				<tr>
					<td colspan="3">Total</td>
					<td class="text-right">{suggestions.length} Elements</td>
				</tr>
			</tfoot>
		</table>
	</div>
</div>
