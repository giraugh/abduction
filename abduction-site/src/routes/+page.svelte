<script lang="ts">
	import type { QubitServer } from '$lib/api.gen';
	import { build_client, ws } from '@qubit-rs/client';

	import { onMount } from 'svelte';

	let events: string[] = $state([]);
	onMount(() => {
		const transport = ws('/_/rpc');
		const client = build_client<QubitServer>(transport);

		client.events_stream.subscribe((event) => {
			events = [...events, event].slice(-10);
		});
	});
</script>

<ul>
	{#each events as event (event)}
		<li>{event}</li>
	{/each}
</ul>
