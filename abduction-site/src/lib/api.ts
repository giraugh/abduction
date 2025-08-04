import { browser } from '$app/environment';
import type { QubitServer } from '$lib/api.gen';
import { create_qubit_api } from './qbsv';

export const { get_api, init_context, load_api } = create_qubit_api<QubitServer>('/_/rpc', {
	browser
});
