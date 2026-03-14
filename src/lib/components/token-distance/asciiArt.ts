/** ASCII art frames for each scene category.
 *  Each frame is an array of lines, rendered in monospace.
 */

export interface AsciiFrame {
	lines: string[];
	/** CSS class for coloring: 'ground', 'mountain', 'sky', 'space' */
	theme: string;
}

export const SCENE_GROUND: AsciiFrame = {
	theme: 'ground',
	lines: [
		'                                    ',
		'        ┌──┐                        ',
		'        │▓▓│  ┌───┐                 ',
		'    ┌───┤▓▓├──┤   │                 ',
		'    │   │▓▓│  │   │   ┌─┐           ',
		'────┴───┴──┴──┴───┴───┴─┴──────     ',
		'▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓     ',
	],
};

export const SCENE_BUILDINGS: AsciiFrame = {
	theme: 'ground',
	lines: [
		'            │█│                      ',
		'           ┌┤█├┐                     ',
		'        ╱╲ │ █ │  ┌──┐               ',
		'       ╱  ╲│ █ │  │██│               ',
		'      ╱    ╲ █ │  │██│  ┌┐           ',
		'     ╱──────╲█ │  │██│  ││  △        ',
		'    ╱        ╲ │  │██│  ││ ╱ ╲       ',
		'───╱──────────╲┴──┴──┴──┴┴╱───╲──    ',
		'▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓    ',
	],
};

export const SCENE_MOUNTAINS: AsciiFrame = {
	theme: 'mountain',
	lines: [
		'                 /\\                  ',
		'        /\\      /  \\    /\\           ',
		'   /\\  /  \\    /    \\  /  \\          ',
		'  /  \\/    \\  /      \\/    \\         ',
		' /    \\     \\/        \\     \\        ',
		'/      \\    /\\         \\     \\       ',
		'        \\  /  \\         \\     \\      ',
		'─────────\\/────\\─────────\\─────\\──   ',
		'  ☁        ☁         ☁        ☁     ',
	],
};

export const SCENE_SKY: AsciiFrame = {
	theme: 'sky',
	lines: [
		'                                     ',
		'         ── ✈ ──                     ',
		'                                     ',
		'    ☁            ☁                   ',
		'         ☁                ☁          ',
		'                                     ',
		'   ☁         ☁        ☁             ',
		'                                     ',
		'  ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─     ',
	],
};

export const SCENE_SPACE: AsciiFrame = {
	theme: 'space',
	lines: [
		'         ·    ✦         ·            ',
		'    ✦          ·    ·        ✦       ',
		'         ┌─╦═╦─┐                     ',
		'    ═════╡ ISS ╞═════               ',
		'         └─╩═╩─┘                     ',
		'  ·          ✦       ·       ·       ',
		'       ·          ·      ✦           ',
		'    ✦       ·          ·             ',
		'                 ✦           ·       ',
	],
};

export const SCENE_MOON: AsciiFrame = {
	theme: 'space',
	lines: [
		'              .  .                    ',
		'           .        .                ',
		'         .    🌙     .               ',
		'        .              .              ',
		'         .            .               ',
		'           .        .                ',
		'     ✦       .  .        ✦           ',
		'         ·          ·                ',
		'   ·         ✦           ·           ',
	],
};

/** Return the appropriate scene for a given token count */
export function getSceneForTokens(tokens: number): AsciiFrame {
	if (tokens < 30_000) return SCENE_GROUND;
	if (tokens < 166_000) return SCENE_BUILDINGS;
	if (tokens < 2_000_000) return SCENE_MOUNTAINS;
	if (tokens < 20_000_000) return SCENE_SKY;
	if (tokens < 7_000_000_000) return SCENE_SPACE;
	return SCENE_MOON;
}
