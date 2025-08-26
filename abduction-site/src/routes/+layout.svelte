<script lang="ts">
	import favicon from '$lib/assets/favicon.svg';
	import { init_context } from '$lib/api';

	let { children } = $props();

	init_context();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

<div class="wrapper">
	{@render children?.()}
</div>
