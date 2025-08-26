<script lang="ts">
	import type { EntityMarker, MotivatorKey } from '$lib/api.gen';
	import { axialHexRange, axialToPixel, entityColor, HEX_SIZE, hexagonPoints } from '$lib/display';
	import { game } from '$lib/game.svelte';
	import { capitalize, pluralize } from '@giraugh/tools';

	let selectedEntity = $state<string | null>(null);

	function axialToCompass(hex: [number, number]) {
		const [px, py] = axialToPixel(hex);
		return `${Math.floor(Math.abs(py) / HEX_SIZE)}°${py >= 0 ? 'S' : 'N'} ${Math.floor(Math.abs(px) / HEX_SIZE)}°${px >= 0 ? 'E' : 'W'}`;
	}

	// TODO: from some kind of config
	let worldRadius = $derived(game.config?.world_radius ?? 0);
	let limits = $derived(HEX_SIZE * worldRadius * 2);

	let entityCount = $derived(game.entities.size);
	let playerCount = $derived(
		Array.from(game.entities.values()).filter((e) => e.markers.includes('player')).length
	);

	function emojiFromMarkers(markers: EntityMarker[]) {
		if (markers.includes('corpse')) return '💀';
		if (markers.includes('hazard')) return '🔥';
		if (markers.includes('player')) return '🤷‍♂️';
		return '';
	}
</script>

<svelte:head>
	<title>Abduction</title>
</svelte:head>

<div class="wrapper">
	<div class="svg-container">
		<svg class="world-svg" viewBox={`${-limits} ${-limits} ${limits * 2} ${limits * 2}`}>
			{#each axialHexRange(worldRadius) as hex (hex.join(','))}
				{@const points = hexagonPoints(hex)}
				<polygon {points} fill="#555" />
			{/each}

			{#each game.entities as [entityId, entity] (entityId)}
				{@const selected = entityId === selectedEntity}
				{@const position = entity.attributes.hex}
				{#if position !== null}
					<!-- Right now we are rendering entities as hexagons but this doesnt make much sense tbh.
					The hex's should always render and the entities should be dots on top of them -->
					{@const points = hexagonPoints(position)}
					<polygon
						{points}
						fill={entityColor(entity)}
						stroke={selected ? 'white' : undefined}
						stroke-width={0.4}
					/>
				{/if}
			{/each}
		</svg>
	</div>

	<div class="sidebar">
		<!-- For now, just dump all the entities here -->
		<h2>
			<span>{entityCount} {pluralize('entity', entityCount, 'entities')}</span>
			<span>/</span>
			<span>{playerCount} {pluralize('player', playerCount, 'players')}</span>
		</h2>
		<ul class="entity-list">
			{#each Array.from(game.entities.keys()).toSorted() as entityId (entityId)}
				{@const entity = game.entities.get(entityId)!}
				{#if entity.markers.includes('viewable')}
					<li>
						<button
							class:selected={entityId === selectedEntity}
							onclick={() => {
								if (selectedEntity === entityId) {
									selectedEntity = null;
								} else {
									selectedEntity = entityId;
								}
							}}>{emojiFromMarkers(entity.markers)} {entity.name}</button
						>
					</li>
				{/if}
			{/each}
		</ul>

		{#if selectedEntity !== null}
			{@const entity = game.entities.get(selectedEntity)}
			{#if entity}
				{@const loc = entity.attributes.hex ? axialToCompass(entity.attributes.hex) : 'unknown'}
				{@const motivators = entity.attributes.motivators}
				<hr />
				<h2>
					{entity.name} <span class="color-dot" style:background={entityColor(entity)}></span>
				</h2>
				<table class="attribute-table">
					<tbody>
						{#if entity.attributes.age !== null}
							<tr><td>Age</td><td>{entity.attributes.age}</td></tr>
						{/if}

						{#if entity.attributes.hex !== null}
							<tr><td>Location</td><td>{loc}</td></tr>
						{/if}

						{#each Object.keys(motivators).toSorted() as motivatorKey (motivatorKey)}
							{@const key = motivatorKey as MotivatorKey}
							{@const motivator = motivators[key]!}
							{@const motivation = Math.floor(motivator.motivation * 100)}
							<tr><td>{capitalize(motivatorKey)}</td><td>{motivation}%</td></tr>
						{/each}
					</tbody>
				</table>
			{/if}
		{/if}
	</div>
</div>

<style>
	.wrapper {
		width: 100%;
		display: flex;
		align-items: center;
		justify-content: center;
		flex: 1;
		align-self: stretch;
	}

	.attribute-table {
		width: 100%;
		border: 3px solid #555;
		border-collapse: collapse;

		& td {
			padding: 0.5em;
		}

		& td:first-child {
			font-weight: bold;
		}

		& tr:nth-child(2n) {
			background: rgba(0.3, 0.3, 0.3, 10%);
		}
	}

	.color-dot {
		width: 0.7em;
		border-radius: 100%;
		aspect-ratio: 1;
		display: inline-block;
		vertical-align: center;
		margin-inline-start: 0.2em;
	}

	.entity-list {
		list-style-type: none;
		padding: 0;
		margin: 0;
		display: flex;
		flex-wrap: wrap;
		gap: 0.2em;

		& button {
			font-weight: normal;
			opacity: 0.6;
			cursor: pointer;
		}

		& button.selected {
			opacity: 1;
		}
	}

	.svg-container {
		flex: 3;
		min-height: 100%;
		max-height: 100%;

		& svg {
			width: min(95%, calc(100vh - var(--nav-height)));
		}
	}

	.sidebar {
		background: var(--surface);
		min-height: 100%;
		align-self: stretch;
		padding: 1em;
		flex: 1;
	}
</style>
