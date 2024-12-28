<script lang="ts">
	import '../app.css';
	let { children } = $props();

	/// Skeleton Components
	import { AppBar, ToastProvider } from '@skeletonlabs/skeleton-svelte';
	import { Navigation } from '@skeletonlabs/skeleton-svelte';

	/// Icons
	import { icons } from 'lucide-svelte';

	/// Svelte
	import { page } from '$app/state';

	/// Tauri

	// when using `"withGlobalTauri": true`, you may use
	// const { getCurrentWindow } = window.__TAURI__.window;
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { slide } from 'svelte/transition';
	import { quintOut } from 'svelte/easing';

	const appWindow = getCurrentWindow();

	let sidebarStatus = $state(1);

	document.getElementById('titlebar')?.addEventListener('mousedown', (e) => {});
</script>

<div class="grid h-screen max-h-screen grid-rows-[auto_1fr_auto]">
	<!-- Header -->
	<header class="sticky top-0 z-50">
		<AppBar
			headlineClasses="hidden"
			centerClasses="hidden sm:block"
			padding="p-0"
			leadPadding="p-0"
		>
			{#snippet lead()}
				<Navigation.Tile
					id="menu-toggle"
					labelExpanded="Menu"
					rounded="false"
					onclick={(id: string) => {
						if (id === 'menu-toggle') {
							sidebarStatus += 1;
							if (sidebarStatus > 2) sidebarStatus = 0;
						}
					}}
					width="w-16"
				>
					<icons.Menu />
				</Navigation.Tile>
			{/snippet}
			{#snippet trail()}
				<Navigation.Tile
					id="menu-minimize"
					labelExpanded="Menu"
					rounded="false"
					onclick={() => appWindow.minimize()}
				>
					<icons.Minimize /></Navigation.Tile
				>
				<Navigation.Tile
					id="menu-maximize"
					labelExpanded="Menu"
					rounded="false"
					onclick={() => appWindow.toggleMaximize()}
				>
					<icons.Maximize /></Navigation.Tile
				>

				<Navigation.Tile
					id="menu-close"
					labelExpanded="Menu"
					rounded="false"
					onclick={() => appWindow.close()}
				>
					<icons.X /></Navigation.Tile
				>
			{/snippet}

			{#snippet headline()}{/snippet}
			<div
				data-tauri-drag-region
				class="titlebar flex h-full cursor-move items-center justify-center"
			>
				<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
				<h3
					id="titlebar"
					class="titlebar h3"
					role="tooltip"
					onmousedown={(e) => {
						if (e.buttons === 1) {
							// Primary (left) button
							e.detail === 2
								? appWindow.toggleMaximize() // Maximize on double click
								: appWindow.startDragging(); // Else start dragging
						}
					}}
				>
					Rusty Search
				</h3>
			</div>
		</AppBar>
	</header>
	<!-- Grid Columns -->
	<div class="grid h-full grid-cols-[auto_1fr_auto] overflow-hidden">
		<!-- Left Sidebar. -->
		<aside class="max-h-screen">
			{#if sidebarStatus > 0}
				<div
					class="card grid h-full w-full grid-cols-[auto_1fr]"
					transition:slide={{ delay: 20, duration: 300, easing: quintOut, axis: 'x' }}
				>
					<!-- Component -->
					<Navigation.Rail expanded={sidebarStatus == 2} width="w-16" widthExpanded="w-48">
						{#snippet header()}
							<Navigation.Tile
								id="0"
								href="/prepare"
								labelExpanded="Prepare"
								rounded="false"
								selected={page.url.pathname === '/prepare'}
								title="Prepare"
							>
								<icons.LayoutDashboard />
							</Navigation.Tile>
						{/snippet}
						{#snippet tiles()}
							<Navigation.Tile
								id="1"
								href="/crawler"
								labelExpanded="Crawler"
								rounded="false"
								selected={page.url.pathname === '/crawler'}
								title="Crawler"
							>
								<icons.Settings />
							</Navigation.Tile>
						{/snippet}
						{#snippet footer()}
							<Navigation.Tile
								id="2"
								href="/metrics"
								labelExpanded="Metrics"
								rounded="false"
								selected={page.url.pathname === '/metrics'}
								title="Metrics"
							>
								<icons.Activity />
							</Navigation.Tile>
						{/snippet}
					</Navigation.Rail>
				</div>
			{/if}
		</aside>
		<div
			class="flex h-full w-full flex-col overflow-y-auto scrollbar-thin scrollbar-track-slate-900 scrollbar-thumb-slate-600 scrollbar-track-rounded-full"
		>
			<!-- Main Content -->
			<main class=" flex h-full w-full flex-col items-center justify-center">
				<ToastProvider>
					{@render children()}
				</ToastProvider>
			</main>
		</div>
	</div>
</div>
