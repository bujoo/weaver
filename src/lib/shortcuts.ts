/**
 * Global keyboard shortcut registration for Weaver.
 * Handles Cmd+key (Mac) shortcuts for navigation, actions, and views.
 */

export interface ShortcutCallbacks {
	onSwitchMission: (index: number) => void;
	onJumpToMission: () => void;
	onOpenVSCode: () => void;
	onToggleStream: () => void;
	onBackToControl: () => void;
	onShowShortcuts: () => void;
	onSendGuidance: () => void;
}

export interface ShortcutEntry {
	key: string;
	description: string;
	category: 'Navigation' | 'Actions' | 'Views';
}

export const SHORTCUTS: ShortcutEntry[] = [
	// Navigation
	{ key: 'Cmd+1/2/3', description: 'Switch mission', category: 'Navigation' },
	{ key: 'Cmd+J', description: 'Jump to mission', category: 'Navigation' },
	{ key: 'Escape', description: 'Back to Mission Control', category: 'Navigation' },
	{ key: 'Tab', description: 'Cycle tabs', category: 'Navigation' },
	{ key: 'Arrow keys', description: 'Navigate lists', category: 'Navigation' },

	// Actions
	{ key: 'Cmd+Enter', description: 'Send guidance', category: 'Actions' },
	{ key: 'Cmd+O', description: 'Open in VS Code', category: 'Actions' },
	{ key: 'Cmd+Shift+R', description: 'Retry todo', category: 'Actions' },
	{ key: 'Cmd+Shift+K', description: 'Skip todo', category: 'Actions' },

	// Views
	{ key: 'Cmd+Shift+S', description: 'Toggle stream view', category: 'Views' },
	{ key: 'Cmd+,', description: 'Open settings', category: 'Views' },
	{ key: 'Cmd+/', description: 'Show keyboard shortcuts', category: 'Views' },
];

/**
 * Register global keyboard shortcuts on the window.
 * Returns a cleanup function to remove the listener.
 */
export function registerShortcuts(callbacks: ShortcutCallbacks): () => void {
	function handler(e: KeyboardEvent) {
		const meta = e.metaKey || e.ctrlKey;

		// Cmd+1/2/3 -- switch to mission by index
		if (meta && !e.shiftKey && e.key >= '1' && e.key <= '9') {
			e.preventDefault();
			callbacks.onSwitchMission(parseInt(e.key, 10) - 1);
			return;
		}

		// Cmd+J -- jump to mission (fuzzy search)
		if (meta && !e.shiftKey && e.key.toLowerCase() === 'j') {
			e.preventDefault();
			callbacks.onJumpToMission();
			return;
		}

		// Cmd+O -- open in VS Code
		if (meta && !e.shiftKey && e.key.toLowerCase() === 'o') {
			e.preventDefault();
			callbacks.onOpenVSCode();
			return;
		}

		// Cmd+Shift+S -- toggle stream view
		if (meta && e.shiftKey && e.key.toLowerCase() === 's') {
			e.preventDefault();
			callbacks.onToggleStream();
			return;
		}

		// Cmd+/ or Cmd+? -- show shortcuts overlay
		if (meta && (e.key === '/' || e.key === '?')) {
			e.preventDefault();
			callbacks.onShowShortcuts();
			return;
		}

		// Cmd+Enter -- send guidance (only when an input/textarea is focused)
		if (meta && e.key === 'Enter') {
			const active = document.activeElement;
			if (
				active instanceof HTMLInputElement ||
				active instanceof HTMLTextAreaElement
			) {
				e.preventDefault();
				callbacks.onSendGuidance();
				return;
			}
		}

		// Escape -- back to mission control (only when no overlay is open)
		// Overlays handle their own Escape, so this fires for the layout level
		if (e.key === 'Escape' && !meta && !e.shiftKey) {
			callbacks.onBackToControl();
			return;
		}
	}

	window.addEventListener('keydown', handler);
	return () => window.removeEventListener('keydown', handler);
}
