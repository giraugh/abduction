<script lang="ts">
	import type { Entity, MotivatorKey } from '$lib/api.gen';
	import { axialHexRange, axialToPixel, entityColor, HEX_SIZE, hexagonPoints } from '$lib/display';
	import { game } from '$lib/game.svelte';
	import { capitalize, pluralize } from '@giraugh/tools';
	import { SvelteMap } from 'svelte/reactivity';

	type Focus = { kind: 'entity'; entityId: string } | { kind: 'hex'; hex: [number, number] } | null;

	let focus = $state<Focus>(null);
	let focusedEntityId = $derived(focus?.kind === 'entity' ? focus.entityId : null);
	let focusedHex = $derived(focus?.kind === 'hex' ? focus.hex : null);
	let focusedEntity = $derived(game.entities.get(focusedEntityId ?? ''));

	let showAllEntities = $state(false);

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

	function entityEmoji(entity: Entity) {
		if (entity.markers.includes('player')) return '🤷‍♂️';
		if (entity.attributes.corpse) return '💀';
		if (entity.attributes.hazard) return '🔥';
		if (entity.attributes.location) return '📍';
		if (entity.attributes.food) return '🍽️';

		return '';
	}

	function sameHex(a: [number, number], b: [number, number]): boolean {
		return a[0] === b[0] && a[1] === b[1];
	}

	const logView = $derived.by(() => {
		return game.logs.filter(
			(l) =>
				l.level === 'global' ||
				(focusedEntityId !== null && l.involved_entities.includes(focusedEntityId)) ||
				(focusedHex !== null && l.involved_hexes.some((h) => sameHex(h, focusedHex))) ||
				(focusedEntity?.attributes.corpse &&
					l.involved_entities.includes(focusedEntity.attributes.corpse))
		);
	});

	function randomJitter(): [number, number] {
		const JITTER = 0.6;
		return [(2 * Math.random() - 1) * JITTER, (2 * Math.random() - 1) * JITTER];
	}

	// TODO: what I actually want to DO:
	// - track how many players in each hex
	//   and track which index each player is
	// - then lay out the players at equal fractions of a circle, based on the count

	let playerPositions = new SvelteMap<string, [number, number]>();
	let playerJitter = new SvelteMap<string, [number, number]>();

	$effect(() =>
		game.onUpdate((e) => {
			if (e.markers.includes('player')) {
				if (e.attributes.hex) {
					playerPositions.set(e.entity_id, e.attributes.hex);
				}

				if (!playerJitter.has(e.entity_id)) {
					playerJitter.set(e.entity_id, randomJitter());
				}
			}
		})
	);

	const hexCounts = $derived.by(() => {
		return game.entities
			.values()
			.filter((e) => e.markers.includes('player') && e.attributes.hex !== null)
			.map((e) => e.attributes.hex!)
			.map((h) => `${h[0]}:${h[1]}`)
			.reduce(
				(acc, hk) => {
					if (!(hk in acc)) {
						acc[hk] = 0;
					}
					acc[hk] += 1;
					return acc;
				},
				{} as Record<string, number>
			);
	});
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

			<!-- Render just the locations as hexs -->
			{@render entitiesAsHexes(
				game.entities.values().toArray(),
				'location',
				(e) => e.attributes.location !== undefined
			)}

			<!-- Then render the players as dots -->
			{#each playerPositions.entries() as [entityId, hex] (entityId)}
				{@const entity = game.entities.get(entityId)}
				{@const jitter = playerJitter.get(entityId)}
				{@const sharing = hexCounts[`${hex[0]}:${hex[1]}`] > 1}
				{#if hex && entity && jitter}
					{@const [cx, cy] = axialToPixel(hex)}
					<!-- svelte-ignore a11y_click_events_have_key_events -->
					<!-- svelte-ignore a11y_no_static_element_interactions -->
					<circle
						onclick={() => {
							if (focusedEntityId === entityId) {
								focus = null;
							} else {
								focus = { kind: 'entity', entityId };
							}
						}}
						class="player-circle"
						class:focused={focusedEntityId === entityId}
						r="0.5"
						cx={(sharing ? jitter[0] : 0) + cx}
						cy={(sharing ? jitter[1] : 0) + cy}
						fill={entityColor(entity)}
					/>
				{/if}
			{/each}
		</svg>
	</div>

	<div class="sidebar">
		<!-- For now, just dump all the entities here -->
		{#if focus?.kind !== 'hex'}
			<h2>
				<span>{entityCount} {pluralize('entity', entityCount, 'entities')}</span>
				<span>/</span>
				<span>{playerCount} {pluralize('player', playerCount, 'players')}</span>
			</h2>
			<div class="filter-controls">
				<label>
					Show all entities
					<input type="checkbox" bind:checked={showAllEntities} />
				</label>
			</div>
		{:else}
			<h2>
				Showing entities in {axialToCompass(focus.hex)}
			</h2>
			<button class="deselect" onclick={() => (focus = null)}>Deselect</button>
		{/if}
		<ul class="entity-list">
			{#each Array.from(game.entities.keys()).toSorted() as entityId (entityId)}
				{@const entity = game.entities.get(entityId)!}
				{@const inFocusedHex =
					focusedHex &&
					entity.attributes.hex !== undefined &&
					sameHex(entity.attributes.hex, focusedHex)}
				{@const isFocusedEntity = focusedEntityId && entity.entity_id === focusedEntityId}
				{@const canSeeGlobally = entity.markers.includes('default_inspectable') || showAllEntities}
				{@const showGlobal = focus?.kind !== 'hex'}
				{#if showGlobal ? canSeeGlobally : inFocusedHex}
					<li>
						<button
							class:selected={isFocusedEntity}
							onclick={() => {
								if (focusedEntityId === entityId) {
									focus = null;
								} else {
									focus = { kind: 'entity', entityId };
								}
							}}>{entityEmoji(entity)} {entity.name}</button
						>
					</li>
				{/if}
			{/each}
		</ul>
		<ul class="logs">
			{#each logView as log (log.id)}
				<li class:global={log.level === 'global'}>{log.message}</li>
			{/each}
			<div id="log-anchor"></div>
		</ul>

		{#if focusedEntityId !== null}
			{@const entity = game.entities.get(focusedEntityId)}
			{#if entity}
				{@const loc = entity.attributes.hex ? axialToCompass(entity.attributes.hex) : 'unknown'}
				{@const motivators = entity.attributes.motivators}
				<hr />
				<h2>
					{entity.name} <span class="color-dot" style:background={entityColor(entity)}></span>
				</h2>
				<div class="markers">
					{entity.markers.join(' ')}
				</div>
				<table class="attribute-table">
					<tbody>
						{#if entity.attributes.age !== null}
							<tr><td>Age</td><td>{entity.attributes.age}</td></tr>
						{/if}

						{#if entity.attributes.hex !== null}
							<tr
								><td>Location</td><td
									><button onclick={() => (focus = { kind: 'hex', hex: entity.attributes.hex! })}
										>{loc}</button
									></td
								></tr
							>
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

{#snippet entitiesAsHexes(entities: Entity[], hexClass: string, filter: (e: Entity) => boolean)}
	{#each entities as entity (entity.entity_id)}
		{#if filter(entity)}
			{@const position = entity.attributes.hex}
			{#if position !== undefined}
				<!-- Right now we are rendering entities as hexagons but this doesnt make much sense tbh.
					The hex's should always render and the entities should be dots on top of them -->
				{@const points = hexagonPoints(position)}
				<!-- svelte-ignore a11y_click_events_have_key_events -->
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<polygon
					onclick={() => {
						if (focusedHex && sameHex(position, focusedHex)) {
							focus = null;
						} else {
							focus = { kind: 'hex', hex: position };
						}
					}}
					class={`hex ${hexClass}`}
					class:focused={(focusedHex && sameHex(focusedHex, position)) ||
						focusedEntityId === entity.entity_id}
					style:--fill={entityColor(entity, 'hue')}
					style:--fill-l={entityColor(entity, 'lightness')}
					fill="var(--fill-l)"
					stroke-width={0.4}
					{points}
				/>
			{/if}
		{/if}
	{/each}
{/snippet}

<style>
	.wrapper {
		width: 100%;
		display: flex;
		align-items: center;
		justify-content: center;
		flex: 1;
		align-self: stretch;
	}

	.hex:hover,
	.hex.focused {
		fill: var(--fill);
	}

	.hex.focused {
		stroke: white;
	}

	.deselect {
		margin-block-end: 1em;
	}

	.markers {
		opacity: 0.3;
		margin-block-end: 1em;
	}

	.location {
		opacity: 0.2;
	}

	.player-circle {
		transition:
			cx 0.3s,
			cy 0.3s;

		&.focused {
			stroke: white;
			stroke-width: 0.2;
		}
	}

	.logs {
		height: 15em;
		box-shadow: inset 0px 0px 6px 1px #111;
		border-radius: 0.3em;

		overflow-y: auto;

		list-style-type: none;
		margin: 0;
		margin-block: 1em;
		padding: 1em;

		& * {
			overflow-anchor: none;
		}

		& #log-anchor {
			overflow-anchor: auto;
			height: 1px;
		}

		& li {
			padding-block: 0.1em;
			font-size: 0.9rem;

			&.global {
				font-weight: bold;
				padding-block: 1.5em;
			}

			&:not(.global) {
				color: #888;
			}
		}
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
		font-size: 0.7rem;

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
		min-width: 300px;

		& h2:first-of-type {
			margin-top: 0;
		}
	}

	.filter-controls {
		margin-block: 1em;
	}

	@media (max-width: 700px) {
		.wrapper {
			display: flex;
			flex-direction: column;
		}

		.svg-container {
			flex: 1;
			width: 100%;
		}
	}
</style>
