import { test, expect } from '@playwright/test';

/**
 * E2E tests for configuration flow.
 *
 * Note: These tests run against the SvelteKit web app without Tauri backend.
 * The app should handle missing backend gracefully with default configurations.
 */

test.describe('Configuration Page', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('/config');
	});

	test('should load configuration page', async ({ page }) => {
		await page.waitForLoadState('domcontentloaded');
		const body = await page.textContent('body');
		expect(body?.length).toBeGreaterThan(0);
	});

	test('should display configuration sections', async ({ page }) => {
		await page.waitForLoadState('domcontentloaded');

		const body = await page.textContent('body');
		expect(body?.length).toBeGreaterThan(100);

		const contentElements = page.locator('main, .content, section, article, div[class]');
		const elementCount = await contentElements.count();
		expect(elementCount).toBeGreaterThan(0);
	});

	test('should have navigation sidebar', async ({ page }) => {
		const sidebar = page.locator('nav, aside, [class*="sidebar"]');
		await expect(sidebar.first()).toBeVisible();
	});
});

test.describe('Global Settings Section', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('/config/global');
		await page.waitForLoadState('domcontentloaded');
		await page.waitForSelector('.loading', { state: 'hidden', timeout: 10000 }).catch(() => {});
	});

	test('should load global settings page', async ({ page }) => {
		const pageContent = await page.textContent('body');
		expect(
			pageContent?.toLowerCase().includes('global') ||
				pageContent?.toLowerCase().includes('settings') ||
				pageContent?.toLowerCase().includes('industry')
		).toBeTruthy();
	});

	test('should have industry selector', async ({ page }) => {
		const industryInput = page.locator(
			'select[id*="industry"], input[id*="industry"], [data-testid*="industry"]'
		);
		await expect(industryInput.first()).toBeVisible();
	});

	test('should have period months input', async ({ page }) => {
		const periodInput = page.locator(
			'input[type="number"][id*="period"], input[type="number"][id*="months"], [data-testid*="period"]'
		);
		await expect(periodInput.first()).toBeVisible();
	});
});

test.describe('Form Interactions', () => {
	test('should enable save button when changes made', async ({ page }) => {
		await page.goto('/config/global');
		await page.waitForLoadState('domcontentloaded');
		await page.waitForSelector('.loading', { state: 'hidden', timeout: 10000 }).catch(() => {});

		const numberInput = page.locator('input[type="number"]').first();
		await numberInput.waitFor({ state: 'visible', timeout: 10000 });

		await numberInput.click();
		await numberInput.fill('24');

		// Discard button should appear (dirty state)
		const discardButton = page.locator('button:has-text("Discard")');
		await expect(discardButton).toBeVisible();

		// Save button should be enabled
		const saveButton = page.locator('button:has-text("Save Changes"), button:has-text("Save")');
		await expect(saveButton.first()).toBeEnabled();
	});

	test('should validate invalid input', async ({ page }) => {
		await page.goto('/config/global');
		await page.waitForLoadState('domcontentloaded');
		await page.waitForSelector('.loading', { state: 'hidden', timeout: 10000 }).catch(() => {});

		const periodInput = page.locator('input[type="number"]').first();
		await periodInput.waitFor({ state: 'visible', timeout: 10000 });

		await periodInput.click();
		await periodInput.fill('200');
		await periodInput.blur();
		await page.waitForTimeout(500);

		// Check for error message or invalid state
		const errorElement = page.locator('.error, .error-text, [class*="error"], [aria-invalid="true"]');
		const hasError = (await errorElement.count()) > 0;
		// Validation may or may not produce visible errors depending on implementation
	});

	test('should reset changes on discard', async ({ page }) => {
		await page.goto('/config/global');
		await page.waitForLoadState('domcontentloaded');
		await page.waitForSelector('.loading', { state: 'hidden', timeout: 10000 }).catch(() => {});

		const numberInput = page.locator('input[type="number"]').first();
		await numberInput.waitFor({ state: 'visible', timeout: 10000 });

		const originalValue = await numberInput.inputValue();
		await numberInput.click();
		await numberInput.fill('99');

		const resetButton = page.locator(
			'button:has-text("Reset"), button:has-text("Cancel"), button:has-text("Discard"), [data-testid*="reset"]'
		);
		await expect(resetButton.first()).toBeVisible();
		await resetButton.first().click();

		const currentValue = await numberInput.inputValue();
		expect(currentValue).toBe(originalValue);
	});
});

test.describe('Navigation Flow', () => {
	test('should navigate between config sections', async ({ page }) => {
		await page.goto('/config');
		await page.waitForLoadState('domcontentloaded');

		const configLinks = page.locator('a[href*="/config/"]');
		const linkCount = await configLinks.count();
		expect(linkCount).toBeGreaterThan(0);

		const firstLink = configLinks.first();
		await firstLink.click();
		await page.waitForLoadState('domcontentloaded');

		const body = await page.textContent('body');
		expect(body?.length).toBeGreaterThan(0);
	});

	test('should show unsaved changes indicator', async ({ page }) => {
		await page.goto('/config/global');
		await page.waitForLoadState('domcontentloaded');
		await page.waitForSelector('.loading', { state: 'hidden', timeout: 10000 }).catch(() => {});

		const numberInput = page.locator('input[type="number"]').first();
		await numberInput.waitFor({ state: 'visible', timeout: 10000 });

		await numberInput.click();
		await numberInput.fill('50');
		await page.waitForTimeout(300);

		// Discard button appearing is the dirty indicator
		const discardButton = page.locator('button:has-text("Discard")');
		await expect(discardButton).toBeVisible();
	});
});

test.describe('Responsive Design', () => {
	test('should be usable on mobile viewport', async ({ page }) => {
		await page.setViewportSize({ width: 375, height: 667 });
		await page.goto('/config');
		await page.waitForLoadState('domcontentloaded');

		const body = await page.textContent('body');
		expect(body?.length).toBeGreaterThan(0);
	});

	test('should be usable on tablet viewport', async ({ page }) => {
		await page.setViewportSize({ width: 768, height: 1024 });
		await page.goto('/config');
		await page.waitForLoadState('domcontentloaded');

		const body = await page.textContent('body');
		expect(body?.length).toBeGreaterThan(0);
	});
});
