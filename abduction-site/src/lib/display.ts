import type { Entity } from '$lib/api.gen';

export const HEX_SIZE = 2;
export function axialToPixel(hex: [number, number], size = HEX_SIZE): [number, number] {
	const [q, r] = hex;
	const x = size * (Math.sqrt(3) * q + (Math.sqrt(3) / 2) * r);
	const y = size * ((3 / 2) * r);
	return [x, y];
}

export function hexagonPoints(hex: [number, number], size: number = HEX_SIZE) {
	const [cx, cy] = axialToPixel(hex);
	const points = [];
	for (let i = 0; i < 6; i++) {
		const angle = (Math.PI / 3) * (i - 0.5);
		const px = cx + size * Math.cos(angle);
		const py = cy + size * Math.sin(angle);
		points.push(`${px},${py}`);
	}

	return points.join(' ');
}

// CHATGPT CODE :|
export function axialHexRange(radius: number): [number, number][] {
	const results: [number, number][] = [];
	for (let q = -radius; q <= radius; q++) {
		for (let r = -radius; r <= radius; r++) {
			const s = -q - r;
			if (Math.abs(q) <= radius && Math.abs(r) <= radius && Math.abs(s) <= radius) {
				results.push([q, r]);
			}
		}
	}

	return results;
}

export function entityColor(entity: Entity, mode: 'hue' | 'lightness' = 'hue') {
	// Otherwise get the hue
	const hue = entity.attributes.display_color_hue;
	if (!hue) return 'grey';

	// And turn it into a color
	if (mode === 'hue') return `hsl(${hue}deg, 50%, 50%)`;
	if (mode === 'lightness') return `hsl(0deg, 0%, ${Math.floor((hue / 365) * 100)}%)`;
}
