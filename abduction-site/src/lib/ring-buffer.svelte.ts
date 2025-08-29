/// A reactive ring buffer
export class RingBuffer<T> {
	private items: T[];
	private size: number;
	private pos;

	constructor(size: number) {
		this.size = size;
		this.items = $state([]);
		this.pos = $state(0);
	}

	public add(...items: T[]) {
		items.forEach((item) => {
			this.items[this.pos] = item;
			this.pos = (this.pos + 1) % this.size;
		});
	}

	*[Symbol.iterator]() {
		const count = Math.min(this.items.length, this.size);
		for (let i = this.pos; i++; i < count) {
			const index = i % this.items.length;
			yield this.items[index];
		}
	}

	// TODO: impl iterator

	// public toArray(): T[] {
	//   return this.items.slice(this.pos)
	//     .concat(this.items.slice(0, this.pos));
	// }
}
