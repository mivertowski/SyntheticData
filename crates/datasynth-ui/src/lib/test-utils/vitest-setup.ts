/**
 * Vitest setup file for Svelte 5 component testing.
 *
 * This file runs before each test file and sets up the testing environment.
 */
import '@testing-library/svelte/vitest';
import { vi, afterEach } from 'vitest';

// Mock the Tauri API globally
vi.mock('@tauri-apps/api/core', () => ({
	invoke: vi.fn().mockRejectedValue(new Error('Tauri not available in tests')),
}));

// Mock crypto.randomUUID if not available
if (typeof globalThis.crypto === 'undefined') {
	Object.defineProperty(globalThis, 'crypto', {
		value: {
			randomUUID: () => `test-uuid-${Math.random().toString(36).substring(2, 15)}`,
		},
	});
} else if (typeof globalThis.crypto.randomUUID === 'undefined') {
	Object.defineProperty(globalThis.crypto, 'randomUUID', {
		value: () => `test-uuid-${Math.random().toString(36).substring(2, 15)}`,
		configurable: true,
	});
}

// Clean up after each test
afterEach(() => {
	vi.clearAllMocks();
});
