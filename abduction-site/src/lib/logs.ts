import type { AxialHexDirection, GameLog, MotivatorKey, Topic } from './api.gen';
import type { Game } from './game.svelte';

/** If global, shows up everywhere, if local only if scoped to the hex/entity */
export type GameLogLevel = 'global' | 'local';
export type BarkSeverity = 'moderate' | 'severe';

/** Given a game log, determine how important it is */
export function logLevel(log: GameLog) {
	if (log.involved_entities.length === 0) return 'global';
	if (log.kind === 'entity_death') return 'global';
	if (log.kind === 'lightning_strike') return 'global';
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

// eslint-disable-next-line @typescript-eslint/no-unused-vars
function unsnake(s: string) {
	return s.replaceAll('_', ' ');
}

function formatBark(name: string, motivator: MotivatorKey, severity: BarkSeverity) {
	if (severity === 'moderate') {
		return (
			{
				boredom: `${name} twiddles their thumbs`,
				hunger: `${name}'s stomach grumbles`,
				hurt: `${name} winces in pain`,
				thirst: `${name} licks their dry lips`,
				sickness: `${name} looks pale`,
				tiredness: `${name} yawns`,
				saturation: `${name} has water dripping off of them`,
				cold: `${name} is shivering`,
				sadness: `${name} is looking glum`,
				friendliness: 'NA'
			} satisfies Record<MotivatorKey, string>
		)[motivator];
	}

	if (severity === 'severe') {
		return (
			{
				boredom: `${name} walks in circles`,
				hunger: `${name}'s doubles over in hunger`,
				hurt: `${name} groans in pain`,
				thirst: `${name} coughs dryly`,
				sickness: `${name} vomits`,
				tiredness: `${name} is falling asleep`,
				saturation: `${name} looks absolutely drenched`,
				cold: `${name} looks extremely cold`,
				sadness: `${name} is quietly crying`,
				friendliness: 'NA'
			} satisfies Record<MotivatorKey, string>
		)[motivator];
	}
}

function formatChatTopic(topic: Topic): string {
	return (
		{
			alien_situation: 'the aliens',
			ambitions: 'their life ambitions',
			career: 'their past career',
			entertainment: 'a tv show they saw recently',
			family: 'their family back home',
			fears: 'their innermost fears',
			hopes: 'their hopes and dreams',
			news: 'the current going-ons',
			weather: 'this weather'
		} satisfies Record<Topic, string>
	)[topic];
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
	if (log.kind === 'weather_change') {
		return `The weather is now ${log.weather}`;
	}

	if (log.kind === 'time_of_day_change') {
		return `It is now ${log.time_of_day}`;
	}

	if (log.kind === 'entity_movement') {
		return `${primaryName} trekked ${formatDirection(log.by)}`;
	}

	if (log.kind === 'entity_consume') {
		return `${primaryName} consumed a ${secondaryName}`;
	}

	if (log.kind === 'entity_drink_from') {
		return `${primaryName} drank from the ${secondaryName}`;
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

	if (log.kind === 'entity_start_sleeping') {
		return `${primaryName} lied down and closed their eyes`;
	}

	if (log.kind === 'entity_keep_sleeping') {
		return `${primaryName} is sleeping soundly`;
	}

	if (log.kind === 'entity_stop_sleeping') {
		return `${primaryName} wakes up`;
	}

	if (log.kind === 'entity_go_downhill') {
		return `${primaryName} is heading downhill`;
	}

	if (log.kind === 'entity_go_to_adjacent_lush') {
		return `${primaryName} spotted a lush location nearby`;
	}

	if (log.kind === 'entity_fell_in_water_source') {
		return `${primaryName} fell into the ${secondaryName}`;
	}

	if (log.kind === 'entity_hesitate_before_consume') {
		return `${primaryName} goes to eat ${secondaryName}, but hesitates for a second`;
	}

	if (log.kind === 'entity_complain_about_taste') {
		return `${primaryName} makes a face. The ${secondaryName} tasted horrible!`;
	}

	if (log.kind === 'entity_cold_because_of_time') {
		return `${primaryName} shivers in the cold wind`;
	}

	if (log.kind === 'entity_warm_because_of_time') {
		return `${primaryName} warms up a bit in the sun`;
	}

	if (log.kind === 'entity_saturated_because_of_rain') {
		return `${primaryName} is getting thoroughly rained on`;
	}

	if (log.kind === 'entity_hit_by_lightning') {
		return `${primaryName} was struck by lightning!`;
	}

	if (log.kind === 'lightning_strike') {
		return `Lightning struck the ground and started a fire!`;
	}

	if (log.kind === 'entity_greet') {
		if (log.response) {
			if (log.bond === 0) return `${primaryName} waves back at ${secondaryName}`;
			else if (log.bond < 0.3) return `${primaryName} nods back at ${secondaryName}`;
			else if (log.bond < 0.6) return `${primaryName} verbally greets ${secondaryName} back`;
			else return `${primaryName} hugs ${secondaryName} back`;
		} else {
			if (log.bond === 0) return `${primaryName} waves at ${secondaryName}`;
			else if (log.bond < 0.3) return `${primaryName} nods at ${secondaryName}`;
			else if (log.bond < 0.6) return `${primaryName} verbally greets ${secondaryName}`;
			else return `${primaryName} hugs ${secondaryName}`;
		}
	}

	if (log.kind === 'entity_mourn_over_corpse') {
		return `${primaryName} has a quiet vigil for ${secondaryName.replaceAll('Corpse of', '')}`;
	}

	if (log.kind === 'entity_upset_by_death') {
		return `${primaryName} is upset by witnessing death`;
	}

	if (log.kind === 'entity_chat') {
		return `${primaryName} chats with ${secondaryName} about ${formatChatTopic(log.topic)}`;
	}

	if (log.kind === 'entity_lose_interest') {
		return `${primaryName} gets distracted from the conversation`;
	}

	if (log.kind === 'entity_farewell') {
		return `${primaryName} says farewell to ${secondaryName}`;
	}

	if (log.kind === 'entity_track_being') {
		return `${primaryName} follows some tracks on the ground`;
	}

	if (log.kind === 'entity_avoid') {
		return `${primaryName} is avoiding ${secondaryName}`;
	}

	if (log.kind === 'entity_ignore') {
		return `${primaryName} ignores ${secondaryName}`;
	}
}
