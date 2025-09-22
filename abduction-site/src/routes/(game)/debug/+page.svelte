<script lang="ts">
	import { entityColor } from '$lib/display';
	import { game } from '$lib/game.svelte';
</script>

<section>
	<h2>Entities</h2>
	<ul class="debug-list">
		{#each game.entities as [entityId, entity] (entityId)}
			{@const isInspectable = entity.markers.includes('inspectable')}
			<li title={entityId} class:inspectable={isInspectable}>
				<details>
					<h3>Markers</h3>
					<ul class="markers">
						{JSON.stringify(entity.markers)}
						{#each entity.markers as marker (marker)}
							<li>{marker}</li>
						{/each}
					</ul>

					<h3>Attributes</h3>
					<pre><code>{JSON.stringify(entity.attributes, null, 2)}</code></pre>
					<summary>
						<strong style:color={entityColor(entity)}>
							{entity.name}
						</strong>
					</summary>
				</details>
			</li>
		{/each}
	</ul>
</section>

<style>
	section {
		margin-block: 2em;
		margin-inline: 1em;
		width: 100%;
	}

	h3 {
		margin-block: 0.5em;
	}

	code,
	pre {
		margin-block: 0.2em;
	}

	ul.debug-list {
		list-style-type: none;
		margin: 0;
		padding: 0;
		display: flex;
		gap: 0.5em;
		flex-wrap: wrap;

		& details summary::-webkit-details-marker,
		& details summary::marker {
			display: none;
		}

		& summary {
			list-style: none;
			cursor: pointer;
		}

		& li:not(.inspectable) {
			opacity: 0.5;
		}
	}

	ul.markers {
		list-style-type: none;
		margin: 0;
		padding: 0;
		display: flex;
		gap: 0.3em;
		flex-wrap: wrap;

		& li {
			&:not(:last-child)::after {
				content: ',';
			}
		}
	}
</style>
