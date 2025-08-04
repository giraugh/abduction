<script lang="ts">
	import { onMount } from 'svelte';
	import { get_api } from '$lib/api';
	import type { TickEvent } from '$lib/api.gen';

	let events: TickEvent[] = $state([]);
	onMount(() => {
		const client = get_api();

		client.events_stream.subscribe((event) => {
			events = [...events, event].slice(-10);
		});
	});
</script>

<ul>
	{#each events as event (event)}
		<li>{JSON.stringify(event)}</li>
	{/each}
</ul>
