/**
 * Pixel art icon drawers for each milestone landmark.
 * Each function draws a small icon at the given (x, y) center position.
 * Size adapts to the `s` parameter (icon size in px).
 */

type IconDrawer = (ctx: CanvasRenderingContext2D, x: number, y: number, s: number, color: string) => void;

// ── Buildings ────────────────────────────────────────────────

const house: IconDrawer = (ctx, x, y, s, color) => {
	ctx.fillStyle = color;
	// Roof triangle
	ctx.beginPath();
	ctx.moveTo(x, y - s * 0.5);
	ctx.lineTo(x - s * 0.5, y);
	ctx.lineTo(x + s * 0.5, y);
	ctx.fill();
	// Body
	ctx.fillRect(x - s * 0.35, y, s * 0.7, s * 0.5);
};

const statueOfLiberty: IconDrawer = (ctx, x, y, s, color) => {
	ctx.fillStyle = color;
	// Torch (raised arm)
	ctx.fillRect(x + s * 0.1, y - s * 0.7, s * 0.1, s * 0.3);
	// Flame
	ctx.beginPath();
	ctx.arc(x + s * 0.15, y - s * 0.75, s * 0.1, 0, Math.PI * 2);
	ctx.fill();
	// Body
	ctx.beginPath();
	ctx.moveTo(x, y - s * 0.4);
	ctx.lineTo(x - s * 0.25, y + s * 0.5);
	ctx.lineTo(x + s * 0.25, y + s * 0.5);
	ctx.fill();
	// Head
	ctx.beginPath();
	ctx.arc(x, y - s * 0.45, s * 0.12, 0, Math.PI * 2);
	ctx.fill();
};

const pyramid: IconDrawer = (ctx, x, y, s, color) => {
	ctx.fillStyle = color;
	ctx.beginPath();
	ctx.moveTo(x, y - s * 0.5);
	ctx.lineTo(x - s * 0.6, y + s * 0.3);
	ctx.lineTo(x + s * 0.6, y + s * 0.3);
	ctx.fill();
};

const eiffelTower: IconDrawer = (ctx, x, y, s, color) => {
	ctx.strokeStyle = color;
	ctx.lineWidth = Math.max(1, s * 0.08);
	// Main structure — two legs converging at top
	ctx.beginPath();
	ctx.moveTo(x - s * 0.35, y + s * 0.5);
	ctx.lineTo(x, y - s * 0.5);
	ctx.lineTo(x + s * 0.35, y + s * 0.5);
	ctx.stroke();
	// Cross beams
	ctx.beginPath();
	ctx.moveTo(x - s * 0.2, y + s * 0.15);
	ctx.lineTo(x + s * 0.2, y + s * 0.15);
	ctx.stroke();
	// Antenna
	ctx.beginPath();
	ctx.moveTo(x, y - s * 0.5);
	ctx.lineTo(x, y - s * 0.65);
	ctx.stroke();
};

const empireState: IconDrawer = (ctx, x, y, s, color) => {
	ctx.fillStyle = color;
	// Main body
	ctx.fillRect(x - s * 0.2, y - s * 0.1, s * 0.4, s * 0.6);
	// Upper section (narrower)
	ctx.fillRect(x - s * 0.12, y - s * 0.35, s * 0.24, s * 0.25);
	// Antenna spire
	ctx.fillRect(x - s * 0.03, y - s * 0.6, s * 0.06, s * 0.25);
};

const taipei101: IconDrawer = (ctx, x, y, s, color) => {
	ctx.fillStyle = color;
	// Stacked sections (getting narrower toward top)
	const sections = 5;
	const totalH = s * 0.8;
	const sectionH = totalH / sections;
	for (let i = 0; i < sections; i++) {
		const w = s * (0.4 - i * 0.04);
		const sy = y + s * 0.4 - i * sectionH;
		ctx.fillRect(x - w / 2, sy - sectionH, w, sectionH - 1);
	}
	// Spire
	ctx.fillRect(x - s * 0.02, y - s * 0.55, s * 0.04, s * 0.15);
};

const burjKhalifa: IconDrawer = (ctx, x, y, s, color) => {
	ctx.fillStyle = color;
	// Tall narrow tapered tower
	ctx.beginPath();
	ctx.moveTo(x, y - s * 0.65);
	ctx.lineTo(x - s * 0.2, y + s * 0.5);
	ctx.lineTo(x + s * 0.2, y + s * 0.5);
	ctx.fill();
	// Spire
	ctx.fillRect(x - s * 0.015, y - s * 0.8, s * 0.03, s * 0.15);
};

// ── Mountains ────────────────────────────────────────────────

const mountain: IconDrawer = (ctx, x, y, s, color) => {
	ctx.fillStyle = color;
	ctx.beginPath();
	ctx.moveTo(x, y - s * 0.5);
	ctx.lineTo(x - s * 0.5, y + s * 0.3);
	ctx.lineTo(x + s * 0.5, y + s * 0.3);
	ctx.fill();
	// Snow cap
	ctx.fillStyle = '#ffffff';
	ctx.beginPath();
	ctx.moveTo(x, y - s * 0.5);
	ctx.lineTo(x - s * 0.15, y - s * 0.2);
	ctx.lineTo(x + s * 0.15, y - s * 0.2);
	ctx.fill();
};

// ── Sky ──────────────────────────────────────────────────────

const airplane: IconDrawer = (ctx, x, y, s, color) => {
	ctx.fillStyle = color;
	// Fuselage
	ctx.fillRect(x - s * 0.35, y - s * 0.05, s * 0.7, s * 0.1);
	// Wings
	ctx.beginPath();
	ctx.moveTo(x - s * 0.1, y);
	ctx.lineTo(x, y - s * 0.35);
	ctx.lineTo(x + s * 0.1, y);
	ctx.fill();
	// Tail
	ctx.beginPath();
	ctx.moveTo(x - s * 0.3, y - s * 0.05);
	ctx.lineTo(x - s * 0.35, y - s * 0.2);
	ctx.lineTo(x - s * 0.2, y - s * 0.05);
	ctx.fill();
};

const parachute: IconDrawer = (ctx, x, y, s, color) => {
	ctx.strokeStyle = color;
	ctx.fillStyle = color;
	ctx.lineWidth = Math.max(1, s * 0.06);
	// Canopy (arc)
	ctx.beginPath();
	ctx.arc(x, y - s * 0.15, s * 0.3, Math.PI, 0);
	ctx.stroke();
	// Lines
	ctx.beginPath();
	ctx.moveTo(x - s * 0.3, y - s * 0.15);
	ctx.lineTo(x, y + s * 0.35);
	ctx.moveTo(x + s * 0.3, y - s * 0.15);
	ctx.lineTo(x, y + s * 0.35);
	ctx.stroke();
	// Person (dot)
	ctx.beginPath();
	ctx.arc(x, y + s * 0.4, s * 0.06, 0, Math.PI * 2);
	ctx.fill();
};

// ── Space ────────────────────────────────────────────────────

const karmanLine: IconDrawer = (ctx, x, y, s, color) => {
	ctx.strokeStyle = color;
	ctx.lineWidth = Math.max(1, s * 0.06);
	ctx.setLineDash([s * 0.1, s * 0.08]);
	ctx.beginPath();
	ctx.moveTo(x - s * 0.5, y);
	ctx.lineTo(x + s * 0.5, y);
	ctx.stroke();
	ctx.setLineDash([]);
};

const iss: IconDrawer = (ctx, x, y, s, color) => {
	ctx.fillStyle = color;
	ctx.strokeStyle = color;
	ctx.lineWidth = Math.max(1, s * 0.06);
	// Solar panels (two rectangles)
	ctx.fillRect(x - s * 0.5, y - s * 0.12, s * 0.3, s * 0.24);
	ctx.fillRect(x + s * 0.2, y - s * 0.12, s * 0.3, s * 0.24);
	// Central module
	ctx.fillRect(x - s * 0.15, y - s * 0.08, s * 0.3, s * 0.16);
	// Truss
	ctx.beginPath();
	ctx.moveTo(x - s * 0.5, y);
	ctx.lineTo(x + s * 0.5, y);
	ctx.stroke();
};

const satellite: IconDrawer = (ctx, x, y, s, color) => {
	ctx.fillStyle = color;
	// Body
	ctx.fillRect(x - s * 0.1, y - s * 0.1, s * 0.2, s * 0.2);
	// Dish
	ctx.strokeStyle = color;
	ctx.lineWidth = Math.max(1, s * 0.06);
	ctx.beginPath();
	ctx.arc(x + s * 0.2, y - s * 0.15, s * 0.15, -Math.PI * 0.3, Math.PI * 0.3);
	ctx.stroke();
	// Solar panels
	ctx.fillRect(x - s * 0.4, y - s * 0.06, s * 0.25, s * 0.12);
	ctx.fillRect(x + s * 0.15, y - s * 0.06, s * 0.25, s * 0.12);
};

const moon: IconDrawer = (ctx, x, y, s, color) => {
	ctx.fillStyle = color;
	// Crescent moon
	ctx.beginPath();
	ctx.arc(x, y, s * 0.35, 0, Math.PI * 2);
	ctx.fill();
	// Cut out a circle to make crescent
	ctx.fillStyle = '#000000';
	ctx.beginPath();
	ctx.arc(x + s * 0.15, y - s * 0.08, s * 0.28, 0, Math.PI * 2);
	ctx.fill();
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
	'Mt. Fuji': mountain,
	'Yushan': mountain,
	'Mt. Kilimanjaro': mountain,
	'Mt. Everest': mountain,
	'Cruising altitude': airplane,
	"Baumgartner's skydive": parachute,
	'Kármán line': karmanLine,
	'ISS': iss,
	'Geostationary orbit': satellite,
	'The Moon': moon,
};

/** Draw the pixel art icon for a milestone at the given position. */
export function drawMilestoneIcon(
	ctx: CanvasRenderingContext2D,
	label: string,
	x: number,
	y: number,
	size: number,
	color: string
): void {
	const drawer = ICON_MAP[label];
	if (drawer) {
		ctx.save();
		drawer(ctx, x, y, size, color);
		ctx.restore();
	}
}
