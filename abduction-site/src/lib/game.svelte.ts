import type { Entity, MatchConfig, TickEvent } from '$lib/api.gen';
import { SvelteMap } from 'svelte/reactivity';

class Game {
	entities: SvelteMap<string, Entity>;
	tickId: number;
	loaded: boolean;
	config: MatchConfig | null;

	constructor() {
		this.entities = new SvelteMap();
		this.tickId = $state(0);
		this.loaded = $state(false);
		this.config = $state(null);
	}

	processEvent(event: TickEvent) {
		if (event?.kind === 'entity_changes') {
			for (const change of event.changes) {
				if (change.kind === 'set_entity') {
					this.entities.set(change.entity.entity_id, change.entity);
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
	}
}

export const game = new Game();
