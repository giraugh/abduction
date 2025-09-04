import type { AxialHexDirection, GameLog, MotivatorKey } from './api.gen';
import type { Game } from './game.svelte';

/** If global, shows up everywhere, if local only if scoped to the hex/entity */
export type GameLogLevel = 'global' | 'local';
export type BarkSeverity = 'moderate' | 'severe';

/** Given a game log, determine how important it is */
export function logLevel(log: GameLog) {
	if (log.kind === 'entity_death') return 'global';
	return 'local';
}

function formatDirection(dir: AxialHexDirection) {
	return (
		{
			east: 'east',
			west: 'west',
			north_east: 'north east',
			north_west: 'north west',
			south_east: 'south east',
			south_west: 'south west'
		} satisfies Record<AxialHexDirection, string>
	)[dir];
}

function formatBark(name: string, motivator: MotivatorKey, severity: BarkSeverity) {
	if (severity === 'moderate') {
		return (
			{
				boredom: `${name} twiddles their thumbs`,
				hunger: `${name}'s stomach grumbles`,
				hurt: `${name} winces in pain`,
				thirst: `${name} licks their dry lips`,
				sickness: `${name} looks pale`
			} satisfies Record<MotivatorKey, string>
		)[motivator];
	}

	if (severity === 'severe') {
		return (
			{
				boredom: `${name} walks in circles`,
				hunger: `${name}'s doubles over in hunger`,
				hurt: `${name} groans in pain`,
				thirst: `${name} coughes dryly`,
				sickness: `${name} vomits`
			} satisfies Record<MotivatorKey, string>
		)[motivator];
	}
}

export function logMessage(log: GameLog, game: Game) {
	// Grab the full entity state for the entities associated with the log
	const entities = log.involved_entities.map((entityId) => {
		return game.entities.get(entityId);
	});

	// Get the name of the entity primarily associated with this log
	const primaryName = entities?.[0]?.name ?? 'Someone';
	const secondaryName = entities?.[1]?.name ?? 'Someone';

	// Now consider the kind
	if (log.kind === 'entity_movement') {
		return `${primaryName} trekked ${formatDirection(log.by)}`;
	}

	if (log.kind === 'entity_consume') {
		return `${primaryName} consumed a ${secondaryName}`;
	}

	if (log.kind === 'entity_motivator_bark') {
		const severity = log.motivation > 0.75 ? 'severe' : 'moderate';
		return formatBark(primaryName, log.motivator, severity);
	}

	if (log.kind === 'entity_death') {
		return `${primaryName} has died`;
	}

	if (log.kind === 'hazard_hurt') {
		return `${secondaryName} was damaged by ${primaryName}`;
	}
}
