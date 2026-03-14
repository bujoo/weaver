/**
 * Dot-based landmark icon drawers.
 * Each landmark is drawn using small dots (same style as rice grains)
 * arranged in the recognizable shape of the landmark.
 *
 * Each function receives (ctx, x, baseY, h, dotSize, color) where:
 * - x: center X position
 * - baseY: bottom of the icon (ground level)
 * - h: total height in pixels (real-scale)
 * - dotSize: size of each dot (matches grain size at current zoom)
 * - color: dot color
 */

type IconDrawer = (
	ctx: CanvasRenderingContext2D,
	x: number,
	baseY: number,
	h: number,
	dotSize: number,
	color: string
) => void;

/** Draw a single dot (grain) at grid position */
function dot(ctx: CanvasRenderingContext2D, x: number, y: number, size: number) {
	ctx.fillRect(x, y, size, size);
}

/** Fill a row of dots from col startC to endC (inclusive) centered at x */
function dotRow(
	ctx: CanvasRenderingContext2D,
	x: number,
	y: number,
	startC: number,
	endC: number,
	dotSize: number
) {
	for (let c = startC; c <= endC; c++) {
		dot(ctx, x + c * dotSize, y, dotSize);
	}
}

// ── Buildings ────────────────────────────────────────────────

const house: IconDrawer = (ctx, x, baseY, h, ds, color) => {
	ctx.fillStyle = color;
	const rows = Math.max(4, Math.round(h / ds));
	const roofRows = Math.ceil(rows * 0.4);
	const bodyRows = rows - roofRows;
	const bodyW = Math.max(3, Math.round(rows * 0.6));

	// Body (rectangle)
	for (let r = 0; r < bodyRows; r++) {
		const y = baseY - (r + 1) * ds;
		dotRow(ctx, x, y, -Math.floor(bodyW / 2), Math.floor(bodyW / 2), ds);
	}
	// Roof (triangle)
	for (let r = 0; r < roofRows; r++) {
		const y = baseY - (bodyRows + r + 1) * ds;
		const halfW = Math.round((roofRows - r) * (bodyW / 2) / roofRows);
		dotRow(ctx, x, y, -halfW, halfW, ds);
	}
};

const statueOfLiberty: IconDrawer = (ctx, x, baseY, h, ds, color) => {
	ctx.fillStyle = color;
	const rows = Math.max(6, Math.round(h / ds));
	const pedastalRows = Math.ceil(rows * 0.35);
	const bodyRows = Math.ceil(rows * 0.5);
	const torchRows = rows - pedastalRows - bodyRows;
	const pedW = Math.max(2, Math.round(rows * 0.3));

	// Pedestal
	for (let r = 0; r < pedastalRows; r++) {
		const y = baseY - (r + 1) * ds;
		dotRow(ctx, x, y, -Math.floor(pedW / 2), Math.floor(pedW / 2), ds);
	}
	// Body (tapers)
	for (let r = 0; r < bodyRows; r++) {
		const y = baseY - (pedastalRows + r + 1) * ds;
		const t = r / bodyRows;
		const halfW = Math.max(0, Math.round((1 - t * 0.6) * pedW / 2));
		dotRow(ctx, x, y, -halfW, halfW, ds);
	}
	// Torch (offset right)
	for (let r = 0; r < torchRows + 1; r++) {
		const y = baseY - (pedastalRows + bodyRows + r + 1) * ds;
		dot(ctx, x + ds * 2, y, ds);
	}
};

const pyramid: IconDrawer = (ctx, x, baseY, h, ds, color) => {
	ctx.fillStyle = color;
	const rows = Math.max(3, Math.round(h / ds));

	for (let r = 0; r < rows; r++) {
		const y = baseY - (r + 1) * ds;
		// Width grows from 1 at top to full at bottom
		const halfW = Math.round((rows - r) * rows / rows / 2);
		dotRow(ctx, x, y, -halfW, halfW, ds);
	}
};

const eiffelTower: IconDrawer = (ctx, x, baseY, h, ds, color) => {
	ctx.fillStyle = color;
	const rows = Math.max(8, Math.round(h / ds));

	for (let r = 0; r < rows; r++) {
		const y = baseY - (r + 1) * ds;
		const t = r / rows; // 0=bottom, 1=top

		if (t < 0.85) {
			// Two legs that converge — only draw the outer dots
			const spread = Math.round((1 - t) * rows * 0.25);
			if (spread > 0) {
				// Left leg
				dot(ctx, x - spread * ds, y, ds);
				// Right leg
				dot(ctx, x + spread * ds, y, ds);
			} else {
				dot(ctx, x, y, ds);
			}
			// Cross beams at 30%, 55%, 75% height
			if (Math.abs(t - 0.3) < 1 / rows || Math.abs(t - 0.55) < 1 / rows || Math.abs(t - 0.75) < 1 / rows) {
				dotRow(ctx, x, y, -spread, spread, ds);
			}
		} else {
			// Antenna — single dot column
			dot(ctx, x, y, ds);
		}
	}
};

const empireState: IconDrawer = (ctx, x, baseY, h, ds, color) => {
	ctx.fillStyle = color;
	const rows = Math.max(8, Math.round(h / ds));

	for (let r = 0; r < rows; r++) {
		const y = baseY - (r + 1) * ds;
		const t = r / rows;

		let halfW: number;
		if (t < 0.55) {
			halfW = Math.max(1, Math.round(rows * 0.12));
		} else if (t < 0.75) {
			halfW = Math.max(1, Math.round(rows * 0.08));
		} else if (t < 0.88) {
			halfW = Math.max(0, Math.round(rows * 0.04));
		} else {
			halfW = 0; // antenna
		}
		dotRow(ctx, x, y, -halfW, halfW, ds);
	}
};

const taipei101: IconDrawer = (ctx, x, baseY, h, ds, color) => {
	ctx.fillStyle = color;
	const rows = Math.max(10, Math.round(h / ds));
	const sections = 8;
	const bodyRows = Math.ceil(rows * 0.8);
	const sectionH = Math.ceil(bodyRows / sections);
	const baseRows = Math.ceil(rows * 0.08);
	const spireRows = rows - bodyRows - baseRows;

	// Base (wide)
	for (let r = 0; r < baseRows; r++) {
		const y = baseY - (r + 1) * ds;
		const halfW = Math.max(2, Math.round(rows * 0.12));
		dotRow(ctx, x, y, -halfW, halfW, ds);
	}
	// Stacked sections (each slightly narrower)
	for (let r = 0; r < bodyRows; r++) {
		const y = baseY - (baseRows + r + 1) * ds;
		const section = Math.floor(r / sectionH);
		const halfW = Math.max(1, Math.round(rows * 0.1) - section);
		dotRow(ctx, x, y, -halfW, halfW, ds);
	}
	// Spire
	for (let r = 0; r < spireRows; r++) {
		const y = baseY - (baseRows + bodyRows + r + 1) * ds;
		dot(ctx, x, y, ds);
	}
};

const burjKhalifa: IconDrawer = (ctx, x, baseY, h, ds, color) => {
	ctx.fillStyle = color;
	const rows = Math.max(10, Math.round(h / ds));

	for (let r = 0; r < rows; r++) {
		const y = baseY - (r + 1) * ds;
		const t = r / rows;

		// Y-shaped taper with stepped setbacks
		let halfW: number;
		if (t < 0.4) {
			halfW = Math.max(1, Math.round(rows * 0.08));
		} else if (t < 0.65) {
			halfW = Math.max(1, Math.round(rows * 0.05));
		} else if (t < 0.85) {
			halfW = Math.max(0, Math.round(rows * 0.025));
		} else {
			halfW = 0; // spire
		}
		dotRow(ctx, x, y, -halfW, halfW, ds);
	}
};

// ── Mountains ────────────────────────────────────────────────

const mountainFn: IconDrawer = (ctx, x, baseY, h, ds, color) => {
	const rows = Math.max(4, Math.round(h / ds));

	for (let r = 0; r < rows; r++) {
		const y = baseY - (r + 1) * ds;
		const halfW = Math.round((rows - r) * 0.6);
		// Mountain body
		ctx.fillStyle = color;
		dotRow(ctx, x, y, -halfW, halfW, ds);
	}
	// Snow cap — top 20% in white
	ctx.fillStyle = '#ffffff';
	const snowRows = Math.max(1, Math.ceil(rows * 0.2));
	for (let r = rows - snowRows; r < rows; r++) {
		const y = baseY - (r + 1) * ds;
		const halfW = Math.round((rows - r) * 0.6);
		dotRow(ctx, x, y, -halfW, halfW, ds);
	}
};

// ── Sky / Space (capped size — these would be too tall at real scale) ──

const airplane: IconDrawer = (ctx, x, baseY, _h, ds, color) => {
	ctx.fillStyle = color;
	const s = Math.max(ds, 2);
	// Small fixed-size plane icon
	//    ·
	//  · · ·
	// ·· · ··
	//    ·
	dot(ctx, x, baseY - s * 4, s);
	dotRow(ctx, x, baseY - s * 3, -1, 1, s);
	dotRow(ctx, x, baseY - s * 2, -2, 2, s);
	dot(ctx, x, baseY - s, s);
};

const parachute: IconDrawer = (ctx, x, baseY, _h, ds, color) => {
	ctx.fillStyle = color;
	const s = Math.max(ds, 2);
	// Canopy
	dotRow(ctx, x, baseY - s * 5, -2, 2, s);
	dotRow(ctx, x, baseY - s * 4, -1, 1, s);
	// Lines
	dot(ctx, x - s * 2, baseY - s * 3, s);
	dot(ctx, x + s * 2, baseY - s * 3, s);
	dot(ctx, x - s, baseY - s * 2, s);
	dot(ctx, x + s, baseY - s * 2, s);
	// Person
	dot(ctx, x, baseY - s, s);
};

const karmanLine: IconDrawer = (ctx, x, baseY, _h, ds, color) => {
	ctx.fillStyle = color;
	const s = Math.max(ds, 2);
	// Dashed horizontal line of dots
	for (let i = -4; i <= 4; i += 2) {
		dot(ctx, x + i * s, baseY - s, s);
	}
};

const issIcon: IconDrawer = (ctx, x, baseY, _h, ds, color) => {
	ctx.fillStyle = color;
	const s = Math.max(ds, 2);
	// Solar panels + truss in dots
	// ··   · ·   ··
	// ·· · · · · ··
	// ··   · ·   ··
	dotRow(ctx, x, baseY - s * 3, -4, -3, s);
	dotRow(ctx, x, baseY - s * 3, 3, 4, s);
	dotRow(ctx, x, baseY - s * 2, -4, 4, s);
	dotRow(ctx, x, baseY - s, -4, -3, s);
	dotRow(ctx, x, baseY - s, 3, 4, s);
	dotRow(ctx, x, baseY - s, -1, 1, s);
	dotRow(ctx, x, baseY - s * 3, -1, 1, s);
};

const satellite: IconDrawer = (ctx, x, baseY, _h, ds, color) => {
	ctx.fillStyle = color;
	const s = Math.max(ds, 2);
	//  ·
	// ·· ··
	//  ·
	dot(ctx, x, baseY - s * 3, s);
	dotRow(ctx, x, baseY - s * 2, -2, -1, s);
	dotRow(ctx, x, baseY - s * 2, 1, 2, s);
	dot(ctx, x, baseY - s * 2, s);
	dot(ctx, x, baseY - s, s);
};

const moonIcon: IconDrawer = (ctx, x, baseY, _h, ds, color) => {
	ctx.fillStyle = color;
	const s = Math.max(ds, 2);
	//  · ·
	// ·
	// ·
	//  · ·
	dot(ctx, x - s, baseY - s * 4, s);
	dot(ctx, x, baseY - s * 4, s);
	dot(ctx, x - s * 2, baseY - s * 3, s);
	dot(ctx, x - s * 2, baseY - s * 2, s);
	dot(ctx, x - s, baseY - s, s);
	dot(ctx, x, baseY - s, s);
};

// ── Icon map ─────────────────────────────────────────────────

const ICON_MAP: Record<string, IconDrawer> = {
	'Two-story house': house,
	'Statue of Liberty': statueOfLiberty,
	'Great Pyramid of Giza': pyramid,
	'Eiffel Tower': eiffelTower,
	'Empire State Building': empireState,
	'Taipei 101': taipei101,
	'Burj Khalifa': burjKhalifa,
	'Mt. Fuji': mountainFn,
	'Yushan': mountainFn,
	'Mt. Kilimanjaro': mountainFn,
	'Mt. Everest': mountainFn,
	'Cruising altitude': airplane,
	"Baumgartner's skydive": parachute,
	'Kármán line': karmanLine,
	'ISS': issIcon,
	'Geostationary orbit': satellite,
	'The Moon': moonIcon,
};

/**
 * Draw a dot-based landmark icon.
 * For buildings/mountains, h is the real-scale pixel height and dotSize
 * matches the grain size at current zoom — so the landmark is composed
 * of the same dots as the rice tower, at true proportional scale.
 * For sky/space landmarks, h is ignored and a fixed-size icon is drawn.
 */
export function drawMilestoneIcon(
	ctx: CanvasRenderingContext2D,
	label: string,
	x: number,
	baseY: number,
	h: number,
	dotSize: number,
	color: string
): void {
	const drawer = ICON_MAP[label];
	if (drawer) {
		ctx.save();
		drawer(ctx, x, baseY, h, dotSize, color);
		ctx.restore();
	}
}
