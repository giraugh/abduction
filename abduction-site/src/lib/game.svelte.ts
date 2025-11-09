import type { AxialHex, Entity, GameLog, MatchConfig, TickEvent } from '$lib/api.gen';
import { SvelteMap } from 'svelte/reactivity';
import { logLevel, logMessage, type GameLogLevel } from './logs';

export const LOG_BUFFER_LIMIT = 1000;

export type DecoratedLog = GameLog & {
	message: string;
	level: GameLogLevel;
	id: number;
	involved_hexes: Array<AxialHex>;
};

type EntityUpdateHandler = (entity: Entity) => void;

export class Game {
	/** Map from entity ids to entity states -> only stores latest state */
	entities: SvelteMap<string, Entity>;

	/** Buffer storing last 1k logs or so. Does not contain all historical logs */
	logs: Array<DecoratedLog>;
	logCounter: number;

	tickId: number;
	config: MatchConfig | null;
	loaded: boolean;
	waitingForStart: boolean;

	entityUpdateHandlers: Array<EntityUpdateHandler> = [];

	constructor() {
		this.entities = new SvelteMap();
		this.logs = $state([]);
		this.logCounter = 0;

		this.tickId = $state(0);
		this.loaded = $state(false);
		this.config = $state(null);
		this.waitingForStart = $state(false);
	}

	onUpdate(handler: EntityUpdateHandler) {
		this.entityUpdateHandlers.push(handler);

		return () => {
			this.entityUpdateHandlers = this.entityUpdateHandlers.filter((h) => h !== handler);
		};
	}

	addLog(log: GameLog) {
		// TODO: limit the size of this buffer
		this.logs.push({
			...log,
			message: logMessage(log, this) ?? '...',
			level: logLevel(log, this),
			// @ts-ignore
			involved_hexes: log.involved_entities
				.map((e) => this.entities.get(e)?.attributes.hex)
				.filter(Boolean),
			id: this.logCounter++
		});
	}

	/* Load initial state of entities */
	loadEntities(entities: Entity[]) {
		game.loaded = true;
		for (const entity of entities) {
			this.entities.set(entity.entity_id, entity);
			this.entityUpdateHandlers.forEach((h) => h(entity));
		}
	}

	processEvent(event: TickEvent) {
		if (event.kind === 'start_of_match') {
			// For now, just reload the page, as we need to do a full reset anyway
			location.reload();
		} else if (this.waitingForStart) {
			// Once the match is finished, ignore any more events
			return;
		}

		if (event?.kind === 'entity_changes') {
			for (const change of event.changes) {
				if (change.kind === 'set_entity') {
					this.entities.set(change.entity.entity_id, change.entity);

					for (const handler of this.entityUpdateHandlers) {
						handler(change.entity);
					}
				}

				if (change.kind === 'remove_entity') {
					// TODO: there's probably something to be said about storing this differently
					// so that you can still browse everything about the entity in the client... hmmm...
					this.entities.delete(change.entity_id);
				}
			}
		}

		if (event?.kind === 'start_of_tick') {
			this.tickId = event.tick_id;
		}

		if (event?.kind === 'end_of_match') {
			console.log('End of match');
			this.waitingForStart = true;
		}
	}
}

export const game = new Game();
