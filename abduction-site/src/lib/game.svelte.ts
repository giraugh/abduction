import { goto, invalidateAll } from '$app/navigation';
import type { Entity, MatchConfig, TickEvent } from '$lib/api.gen';
import { SvelteMap } from 'svelte/reactivity';

class Game {
	entities: SvelteMap<string, Entity>;
	tickId: number;
	config: MatchConfig | null;
	loaded: boolean;
	finished: boolean;

	constructor() {
		this.entities = new SvelteMap();
		this.tickId = $state(0);
		this.loaded = $state(false);
		this.config = $state(null);
		this.finished = $state(false);
	}

	processEvent(event: TickEvent) {
		if (event.kind === 'start_of_match') {
			// For now, just reload the page, as we need to do a full reset anyway
			location.reload();
		} else if (this.finished) {
			// Once the match is finished, ignore any more events
			return;
		}

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

		if (event?.kind === 'end_of_match') {
			console.log('End of match');
			this.finished = true;
		}
	}
}

export const game = new Game();
