<script lang="ts">
	import { onMount } from 'svelte';
	import { get_api } from '$lib/api';
	import type { TickEvent } from '$lib/api.gen';
	import { game } from '$lib/game.svelte';

	const { children } = $props();

	let events: TickEvent[] = $state([]);
	onMount(() => {
		const client = get_api();

		// Fetch the configuration for the current match
		client.get_match_config.query().then((config) => {
			game.config = config;
		});

		// Get the current state of all entities
		client.get_entity_states.query().then((states) => {
			if (states) {
				game.loaded = true;
				game.waitingForStart = false;
				for (const entity of states) {
					game.entities.set(entity.entity_id, entity);
				}
			} else {
				// TODO: hmm, should prob just go somewhere to poll
				game.waitingForStart = true;
			}
		});

		// Begin events stream and start adding them into a buffer
		const unsubEvents = client.events_stream.subscribe({
			on_data: (event) => {
				events.push(event);
			},
			on_error: (error) => {
				// TODO: handle this properly
				console.warn('Stream had an error', error);
			},
			on_end: () => {
				// TODO:?
				// NOTE: I think this is also called on cleanup...
			}
		});

		// Get logs
		const unsubLogs = client.game_log_stream.subscribe({
			on_data: (event) => {
				game.addLog(event);
			},
			on_error: () => {},
			on_end: () => {}
		});

		return () => {
			unsubEvents();
			unsubLogs();
		};
	});

	// Regularly check for events and apply them all to the current game state
	$effect(() => {
		const interval = setInterval(() => {
			// If stream is not initialised, just queue up events
			// (If we didnt load because no match on, do process events because we are listening for a match start event)
			if (!game.loaded && !game.waitingForStart) return;

			// Process all available events
			while (events.length > 0) {
				game.processEvent(events.shift()!);
			}
		}, 1);

		return () => clearInterval(interval);
	});
</script>

<div class="wrapper">
	<nav>
		<h1>Abduction</h1>
		<span>Tick {game.tickId}</span>
	</nav>
	<main>
		{#if game.loaded}
			{@render children()}
		{:else if game.waitingForStart}
			No match currently running. Match will start soon...
		{:else}
			Loading...
		{/if}
	</main>
</div>

<style>
	:global(body, html) {
		margin: 0;
		padding: 0;
		font-family: sans-serif;
		box-sizing: border-box;
		background: var(--bg);
		color-scheme: dark;

		--bg: #222;
		--surface: #333;
		--nav-height: 2em;
	}

	main,
	.wrapper {
		display: flex;
		flex-direction: column;
		height: 100%;
	}

	main {
		flex: 1;
	}

	nav {
		padding: 1em;
		background: var(--surface);
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding-inline-end: 1.5em;
		height: var(--nav-height);

		& h1 {
			margin: 0;
		}
	}

	.wrapper {
		width: 100%;
		min-height: 100vh;
	}
</style>
