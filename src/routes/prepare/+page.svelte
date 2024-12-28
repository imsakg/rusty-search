<script lang="ts">
	// Svelte
	import { onMount } from 'svelte';
	import { writable, derived } from 'svelte/store';
	import { getContext } from 'svelte';

	// Skeleton
	import { Pagination } from '@skeletonlabs/skeleton-svelte';
	import { Modal } from '@skeletonlabs/skeleton-svelte';
	import { type ToastContext } from '@skeletonlabs/skeleton-svelte';

	// Icons
	import IconArrowLeft from 'lucide-svelte/icons/arrow-left';
	import IconArrowRight from 'lucide-svelte/icons/arrow-right';
	import IconEllipsis from 'lucide-svelte/icons/ellipsis';
	import IconFirst from 'lucide-svelte/icons/chevrons-left';
	import IconLast from 'lucide-svelte/icons/chevron-right';

	// Tauri
	// When using the Tauri API npm package:
	import { invoke } from '@tauri-apps/api/core';

	// When using the Tauri global script (if not using the npm package)
	// Be sure to set `app.withGlobalTauri` in `tauri.conf.json` to true
	// const invoke = window.__TAURI__.core.invoke;

	// Libs
	import { sources } from '$lib/stores';

	// Stores
	const sourceData = sources;

	// State
	let page = writable(1);
	let size = writable(20);
	const slicedSource = derived([sourceData, page, size], ([$sourceData, $page, $size]) =>
		$sourceData.slice(($page - 1) * $size, $page * $size)
	);

	let modalOpenState = $state(false);
	export const toast: ToastContext = getContext('toast');

	onMount(async () => {
		if ($sourceData.length > 0) return;

		let sitelist: Array<string> = await invoke('get_sitelist');
		sourceData.update(($sourceData) => {
			sitelist.forEach((element) => {
				$sourceData.push({
					position: $sourceData.length + 1,
					url: element,
					active: true
				});
			});
			return $sourceData;
		});
	});

	async function add_new_url(url: string) {
		let result = await invoke('add_new_url', { url: url });
		modalOpenState = false;
		if (result) {
			sourceData.update(($sourceData) => {
				$sourceData.forEach((item, _) => {
					item.position += +1;
				});
				$sourceData.unshift({
					position: 1,
					url: url,
					active: true
				});
				return $sourceData;
			});
			await toast.create({
				title: 'Success',
				description: 'New URL added successfully',
				type: 'success'
			});
		} else {
			await toast.create({
				title: 'Error',
				description: 'Failed! It could be a duplicate or invalid URL',
				type: 'error'
			});
		}
	}
</script>

<div class="contianer h-full w-full">
	<section class="space-y-4 p-4">
		<!-- Table -->
		<div class="table-wrap">
			<table class="table table-fixed caption-bottom">
				<thead>
					<tr>
						<th>Position</th>
						<th>
							<p class="text-center">Name</p>
						</th>

						<th>
							<p class="text-right">Active</p>
						</th>
					</tr>
				</thead>
				<tbody class="hover:[&>tr]:preset-tonal-primary">
					{#each $slicedSource as row}
						<tr>
							<td>{row.position}</td>
							<td class="text-center">{row.url}</td>
							<td class="text-right">
								<input
									type="checkbox"
									checked={$sourceData[row.position - 1].active}
									onchange={() => {
										sourceData.update((data) => {
											data[row.position - 1].active = !data[row.position - 1].active;
											return data;
										});
										console.log($sourceData);
									}}
								/>
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
		<!-- Footer -->
		<footer class="flex items-center justify-between space-x-4">
			<!-- Items per page -->
			<select name="size" id="size" class="select h-12 max-w-[120px]" bind:value={$size}>
				{#each [5, 10, 20, 50, 100] as v}
					<option value={v}>Items {v}</option>
				{/each}
				<option value={$sourceData.length}>Show All</option>
			</select>
			<!-- Add new item -->
			<Modal
				bind:open={modalOpenState}
				triggerBase="btn preset-tonal"
				contentBase="card bg-surface-100-900 p-4 space-y-4 shadow-xl max-w-screen-sm"
				backdropClasses="backdrop-blur-sm"
			>
				{#snippet trigger()}Add New Item{/snippet}
				{#snippet content()}
					<header class="flex justify-center">
						<h4 class="h4">Add New Source</h4>
					</header>
					<div class="flex items-center justify-center space-x-4">
						<label for="url" class="label max-w-fit">Source URL</label>
						<input
							type="text"
							id="urlInput"
							name="url"
							class="input-primary input"
							placeholder="https://example.com"
						/>
					</div>
					<footer class="flex justify-end gap-4">
						<button
							type="button"
							class="btn preset-tonal"
							onclick={() => {
								modalOpenState = false;
							}}
						>
							Cancel
						</button>
						<button
							type="button"
							class="btn preset-filled"
							onclick={() => {
								const urlInput = document.getElementById('urlInput') as HTMLInputElement;
								console.log(urlInput.value);
								if (urlInput) add_new_url(urlInput.value);
							}}>Confirm</button
						>
					</footer>
				{/snippet}
			</Modal>

			<!-- Pagination -->
			<Pagination
				bind:data={$sourceData}
				bind:page={$page}
				bind:pageSize={$size}
				siblingCount={2}
				alternative
			>
				{#snippet labelEllipsis()}<IconEllipsis class="size-4" />{/snippet}
				{#snippet labelNext()}<IconArrowRight class="size-4" />{/snippet}
				{#snippet labelPrevious()}<IconArrowLeft class="size-4" />{/snippet}
				{#snippet labelFirst()}<IconFirst class="size-4" />{/snippet}
				{#snippet labelLast()}<IconLast class="size-4" />{/snippet}
			</Pagination>
		</footer>
	</section>
</div>
