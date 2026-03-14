/**
 * Pixel art icon drawers for each milestone landmark.
 * Each function draws an icon at (x, baseY) where baseY is the bottom of the icon.
 * The icon extends UPWARD by `h` pixels (the real-scale height).
 * `w` is derived from `h` to maintain proportions.
 */

type IconDrawer = (ctx: CanvasRenderingContext2D, x: number, baseY: number, h: number, color: string) => void;

// ── Buildings ────────────────────────────────────────────────

const house: IconDrawer = (ctx, x, baseY, h, color) => {
	const w = h * 1.2;
	ctx.fillStyle = color;
	// Body
	ctx.fillRect(x - w * 0.4, baseY - h * 0.5, w * 0.8, h * 0.5);
	// Roof
	ctx.beginPath();
	ctx.moveTo(x, baseY - h);
	ctx.lineTo(x - w * 0.5, baseY - h * 0.5);
	ctx.lineTo(x + w * 0.5, baseY - h * 0.5);
	ctx.fill();
};

const statueOfLiberty: IconDrawer = (ctx, x, baseY, h, color) => {
	const w = h * 0.35;
	ctx.fillStyle = color;
	// Pedestal
	ctx.fillRect(x - w * 0.6, baseY - h * 0.3, w * 1.2, h * 0.3);
	// Body (tapered)
	ctx.beginPath();
	ctx.moveTo(x - w * 0.4, baseY - h * 0.3);
	ctx.lineTo(x - w * 0.2, baseY - h * 0.85);
	ctx.lineTo(x + w * 0.2, baseY - h * 0.85);
	ctx.lineTo(x + w * 0.4, baseY - h * 0.3);
	ctx.fill();
	// Head
	ctx.beginPath();
	ctx.arc(x, baseY - h * 0.88, h * 0.05, 0, Math.PI * 2);
	ctx.fill();
	// Torch arm
	ctx.fillRect(x + w * 0.15, baseY - h, w * 0.12, h * 0.2);
	// Flame
	ctx.beginPath();
	ctx.arc(x + w * 0.21, baseY - h - h * 0.03, h * 0.03, 0, Math.PI * 2);
	ctx.fill();
};

const pyramid: IconDrawer = (ctx, x, baseY, h, color) => {
	const w = h * 1.4;
	ctx.fillStyle = color;
	ctx.beginPath();
	ctx.moveTo(x, baseY - h);
	ctx.lineTo(x - w * 0.5, baseY);
	ctx.lineTo(x + w * 0.5, baseY);
	ctx.fill();
};

const eiffelTower: IconDrawer = (ctx, x, baseY, h, color) => {
	const w = h * 0.35;
	ctx.strokeStyle = color;
	ctx.lineWidth = Math.max(1, h * 0.025);
	// Two legs converging
	ctx.beginPath();
	ctx.moveTo(x - w, baseY);
	ctx.lineTo(x, baseY - h * 0.85);
	ctx.lineTo(x + w, baseY);
	ctx.stroke();
	// Antenna
	ctx.beginPath();
	ctx.moveTo(x, baseY - h * 0.85);
	ctx.lineTo(x, baseY - h);
	ctx.stroke();
	// Cross beams
	const beams = [0.3, 0.55, 0.75];
	for (const t of beams) {
		const bw = w * (1 - t);
		ctx.beginPath();
		ctx.moveTo(x - bw, baseY - h * t);
		ctx.lineTo(x + bw, baseY - h * t);
		ctx.stroke();
	}
};

const empireState: IconDrawer = (ctx, x, baseY, h, color) => {
	const w = h * 0.22;
	ctx.fillStyle = color;
	// Main body
	ctx.fillRect(x - w, baseY - h * 0.6, w * 2, h * 0.6);
	// Upper setback
	ctx.fillRect(x - w * 0.65, baseY - h * 0.78, w * 1.3, h * 0.18);
	// Tower top
	ctx.fillRect(x - w * 0.35, baseY - h * 0.88, w * 0.7, h * 0.1);
	// Antenna
	ctx.fillRect(x - w * 0.08, baseY - h, w * 0.16, h * 0.12);
};

const taipei101: IconDrawer = (ctx, x, baseY, h, color) => {
	const w = h * 0.18;
	ctx.fillStyle = color;
	// 8 stacked sections
	const sections = 8;
	const bodyH = h * 0.75;
	const sectionH = bodyH / sections;
	for (let i = 0; i < sections; i++) {
		const sw = w * (1 - i * 0.06);
		const sy = baseY - h * 0.1 - i * sectionH;
		ctx.fillRect(x - sw, sy - sectionH, sw * 2, sectionH - 1);
	}
	// Base
	ctx.fillRect(x - w * 1.2, baseY - h * 0.1, w * 2.4, h * 0.1);
	// Spire
	ctx.fillRect(x - w * 0.08, baseY - h, w * 0.16, h * 0.15);
};

const burjKhalifa: IconDrawer = (ctx, x, baseY, h, color) => {
	const w = h * 0.12;
	ctx.fillStyle = color;
	// Tapered tower with stepped setbacks
	ctx.beginPath();
	ctx.moveTo(x, baseY - h);
	ctx.lineTo(x - w * 0.3, baseY - h * 0.9);
	ctx.lineTo(x - w * 0.6, baseY - h * 0.7);
	ctx.lineTo(x - w, baseY - h * 0.4);
	ctx.lineTo(x - w * 1.2, baseY);
	ctx.lineTo(x + w * 1.2, baseY);
	ctx.lineTo(x + w, baseY - h * 0.4);
	ctx.lineTo(x + w * 0.6, baseY - h * 0.7);
	ctx.lineTo(x + w * 0.3, baseY - h * 0.9);
	ctx.fill();
};

// ── Mountains ────────────────────────────────────────────────

const mountainFn: IconDrawer = (ctx, x, baseY, h, color) => {
	const w = h * 1.2;
	ctx.fillStyle = color;
	ctx.beginPath();
	ctx.moveTo(x, baseY - h);
	ctx.lineTo(x - w * 0.5, baseY);
	ctx.lineTo(x + w * 0.5, baseY);
	ctx.fill();
	// Snow cap
	ctx.fillStyle = '#ffffff';
	ctx.beginPath();
	ctx.moveTo(x, baseY - h);
	ctx.lineTo(x - w * 0.12, baseY - h * 0.75);
	ctx.lineTo(x + w * 0.12, baseY - h * 0.75);
	ctx.fill();
};

// ── Sky ──────────────────────────────────────────────────────

const airplane: IconDrawer = (ctx, x, baseY, h, color) => {
	// h is very large for cruising altitude — cap the visual icon
	const iconH = Math.min(h, 20);
	const w = iconH * 2;
	const cy = baseY - iconH * 0.5;
	ctx.fillStyle = color;
	// Fuselage
	ctx.fillRect(x - w * 0.35, cy - iconH * 0.08, w * 0.7, iconH * 0.16);
	// Wings
	ctx.beginPath();
	ctx.moveTo(x - w * 0.08, cy);
	ctx.lineTo(x, cy - iconH * 0.45);
	ctx.lineTo(x + w * 0.08, cy);
	ctx.fill();
	// Tail
	ctx.beginPath();
	ctx.moveTo(x - w * 0.28, cy - iconH * 0.08);
	ctx.lineTo(x - w * 0.35, cy - iconH * 0.3);
	ctx.lineTo(x - w * 0.2, cy - iconH * 0.08);
	ctx.fill();
};

const parachute: IconDrawer = (ctx, x, baseY, h, color) => {
	const iconH = Math.min(h, 20);
	const cy = baseY - iconH * 0.5;
	ctx.strokeStyle = color;
	ctx.fillStyle = color;
	ctx.lineWidth = Math.max(1, iconH * 0.06);
	// Canopy
	ctx.beginPath();
	ctx.arc(x, cy - iconH * 0.1, iconH * 0.35, Math.PI, 0);
	ctx.stroke();
	// Lines
	ctx.beginPath();
	ctx.moveTo(x - iconH * 0.35, cy - iconH * 0.1);
	ctx.lineTo(x, cy + iconH * 0.35);
	ctx.moveTo(x + iconH * 0.35, cy - iconH * 0.1);
	ctx.lineTo(x, cy + iconH * 0.35);
	ctx.stroke();
	// Person
	ctx.beginPath();
	ctx.arc(x, cy + iconH * 0.4, iconH * 0.07, 0, Math.PI * 2);
	ctx.fill();
};

// ── Space ────────────────────────────────────────────────────

const karmanLine: IconDrawer = (ctx, x, baseY, h, color) => {
	const iconH = Math.min(h, 16);
	ctx.strokeStyle = color;
	ctx.lineWidth = Math.max(1, iconH * 0.08);
	ctx.setLineDash([iconH * 0.15, iconH * 0.1]);
	ctx.beginPath();
	ctx.moveTo(x - iconH * 0.8, baseY - iconH * 0.5);
	ctx.lineTo(x + iconH * 0.8, baseY - iconH * 0.5);
	ctx.stroke();
	ctx.setLineDash([]);
};

const iss: IconDrawer = (ctx, x, baseY, h, color) => {
	const iconH = Math.min(h, 18);
	const cy = baseY - iconH * 0.5;
	ctx.fillStyle = color;
	ctx.strokeStyle = color;
	ctx.lineWidth = Math.max(1, iconH * 0.05);
	// Solar panels
	ctx.fillRect(x - iconH * 0.7, cy - iconH * 0.15, iconH * 0.35, iconH * 0.3);
	ctx.fillRect(x + iconH * 0.35, cy - iconH * 0.15, iconH * 0.35, iconH * 0.3);
	// Module
	ctx.fillRect(x - iconH * 0.18, cy - iconH * 0.1, iconH * 0.36, iconH * 0.2);
	// Truss
	ctx.beginPath();
	ctx.moveTo(x - iconH * 0.7, cy);
	ctx.lineTo(x + iconH * 0.7, cy);
	ctx.stroke();
};

const satellite: IconDrawer = (ctx, x, baseY, h, color) => {
	const iconH = Math.min(h, 16);
	const cy = baseY - iconH * 0.5;
	ctx.fillStyle = color;
	// Body
	ctx.fillRect(x - iconH * 0.12, cy - iconH * 0.12, iconH * 0.24, iconH * 0.24);
	// Panels
	ctx.fillRect(x - iconH * 0.55, cy - iconH * 0.08, iconH * 0.35, iconH * 0.16);
	ctx.fillRect(x + iconH * 0.2, cy - iconH * 0.08, iconH * 0.35, iconH * 0.16);
	// Dish
	ctx.strokeStyle = color;
	ctx.lineWidth = Math.max(1, iconH * 0.07);
	ctx.beginPath();
	ctx.arc(x + iconH * 0.25, cy - iconH * 0.2, iconH * 0.15, -Math.PI * 0.4, Math.PI * 0.3);
	ctx.stroke();
};

const moonIcon: IconDrawer = (ctx, x, baseY, h, color) => {
	const iconH = Math.min(h, 22);
	const cy = baseY - iconH * 0.5;
	const r = iconH * 0.4;
	ctx.fillStyle = color;
	ctx.beginPath();
	ctx.arc(x, cy, r, 0, Math.PI * 2);
	ctx.fill();
	// Crescent cutout
	ctx.fillStyle = '#000000';
	ctx.beginPath();
	ctx.arc(x + r * 0.4, cy - r * 0.2, r * 0.8, 0, Math.PI * 2);
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
	'Mt. Fuji': mountainFn,
	'Yushan': mountainFn,
	'Mt. Kilimanjaro': mountainFn,
	'Mt. Everest': mountainFn,
	'Cruising altitude': airplane,
	"Baumgartner's skydive": parachute,
	'Kármán line': karmanLine,
	'ISS': iss,
	'Geostationary orbit': satellite,
	'The Moon': moonIcon,
};

/**
 * Draw the pixel art icon for a milestone at the given position.
 * For buildings/mountains: `h` is the real-scale pixel height — icon
 * extends upward from baseY by that amount.
 * For sky/space: icon is capped to a fixed size since the real scale
 * would be too large.
 */
export function drawMilestoneIcon(
	ctx: CanvasRenderingContext2D,
	label: string,
	x: number,
	baseY: number,
	h: number,
	color: string
): void {
	const drawer = ICON_MAP[label];
	if (drawer) {
		ctx.save();
		drawer(ctx, x, baseY, h, color);
		ctx.restore();
	}
}
