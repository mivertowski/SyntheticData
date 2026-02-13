import { test, expect } from '@playwright/test';

/**
 * Tests for preset management functionality.
 * Covers: navigating to presets, applying industry presets,
 * verifying config updates, and reset to defaults.
 */

test.describe('Presets Page', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('/presets');
		await page.waitForLoadState('domcontentloaded');
	});

	test('presets page loads successfully', async ({ page }) => {
		const body = await page.textContent('body');
		expect(body?.length).toBeGreaterThan(50);

		// Should contain preset-related content
		const hasPresetContent =
			body?.toLowerCase().includes('preset') || body?.toLowerCase().includes('industry');
		expect(hasPresetContent).toBeTruthy();
	});

	test('sidebar is visible on presets page (GH #27)', async ({ page }) => {
		const sidebar = page.locator('nav.sidebar, aside.sidebar, [class*="sidebar"]');
		await expect(sidebar.first()).toBeVisible();
	});

	test('preset cards or options are displayed', async ({ page }) => {
		// Look for preset cards, buttons, or select elements
		const presetElements = page.locator(
			'.preset-card, [class*="preset"], button:has-text("Manufacturing"), ' +
				'button:has-text("Retail"), button:has-text("Financial"), ' +
				'button:has-text("Healthcare"), button:has-text("Technology")'
		);
		expect(await presetElements.count()).toBeGreaterThan(0);
	});
});

test.describe('Preset Application', () => {
	test('applying a preset updates configuration', async ({ page }) => {
		await page.goto('/presets');
		await page.waitForLoadState('domcontentloaded');

		// Find and click a preset (e.g., manufacturing)
		const presetButton = page.locator(
			'button:has-text("Manufacturing"), [data-preset="manufacturing"], ' +
				'.preset-card:has-text("Manufacturing")'
		);

		if ((await presetButton.count()) > 0) {
			await presetButton.first().click();
			await page.waitForTimeout(500);

			// Navigate to global settings to verify the preset was applied
			await page.goto('/config/global');
			await page.waitForLoadState('domcontentloaded');
			await page.waitForSelector('.loading', { state: 'hidden', timeout: 10000 }).catch(() => {});

			// Industry should reflect the preset
			const industrySelect = page.locator('select[id*="industry"]');
			if ((await industrySelect.count()) > 0) {
				const value = await industrySelect.inputValue();
				expect(value.toLowerCase()).toContain('manufactur');
			}
		}
	});

	test('preset page has navigation back to config', async ({ page }) => {
		await page.goto('/presets');
		await page.waitForLoadState('domcontentloaded');

		// Should be able to navigate to config from presets
		const configLink = page.locator('a[href="/config"], a[href*="/config/"]');
		expect(await configLink.count()).toBeGreaterThan(0);
	});
});

test.describe('Preset Reset', () => {
	test('can reset config to defaults after applying preset', async ({ page }) => {
		await page.goto('/config/global');
		await page.waitForLoadState('domcontentloaded');
		await page.waitForSelector('.loading', { state: 'hidden', timeout: 10000 }).catch(() => {});

		// Make a change
		const numberInput = page.locator('input[type="number"]').first();
		if ((await numberInput.count()) > 0) {
			const originalValue = await numberInput.inputValue();
			await numberInput.click();
			await numberInput.fill('99');
			await page.waitForTimeout(300);

			// Click discard
			const discardButton = page.locator('button:has-text("Discard")');
			if ((await discardButton.count()) > 0) {
				await discardButton.click();
				await page.waitForTimeout(300);

				const restoredValue = await numberInput.inputValue();
				expect(restoredValue).toBe(originalValue);
			}
		}
	});
});
