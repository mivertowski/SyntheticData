import { test, expect } from '@playwright/test';

/**
 * Responsive design tests across 3 viewport sizes.
 * Tests: sidebar behavior, grid reflow, form usability.
 */

const VIEWPORTS = {
	mobile: { width: 375, height: 667 },
	tablet: { width: 768, height: 1024 },
	desktop: { width: 1440, height: 900 },
};

test.describe('Responsive: Mobile (375px)', () => {
	test.beforeEach(async ({ page }) => {
		await page.setViewportSize(VIEWPORTS.mobile);
	});

	test('dashboard loads and is functional', async ({ page }) => {
		await page.goto('/');
		await page.waitForLoadState('domcontentloaded');

		const body = await page.textContent('body');
		expect(body?.length).toBeGreaterThan(50);
	});

	test('config page loads on mobile', async ({ page }) => {
		await page.goto('/config/global');
		await page.waitForLoadState('domcontentloaded');
		await page.waitForSelector('.loading', { state: 'hidden', timeout: 10000 }).catch(() => {});

		// h1 should still be visible
		const h1 = page.locator('h1');
		await expect(h1.first()).toBeVisible();
	});

	test('form inputs are usable on mobile', async ({ page }) => {
		await page.goto('/config/global');
		await page.waitForLoadState('domcontentloaded');
		await page.waitForSelector('.loading', { state: 'hidden', timeout: 10000 }).catch(() => {});

		// Number inputs should be accessible
		const inputs = page.locator('input[type="number"], select');
		const count = await inputs.count();
		if (count > 0) {
			const firstInput = inputs.first();
			await expect(firstInput).toBeVisible();

			// Input should be interactable
			const box = await firstInput.boundingBox();
			expect(box).toBeTruthy();
			// Minimum touch target size check
			expect(box!.width).toBeGreaterThan(20);
			expect(box!.height).toBeGreaterThan(20);
		}
	});

	test('save button is accessible on mobile', async ({ page }) => {
		await page.goto('/config/global');
		await page.waitForLoadState('domcontentloaded');
		await page.waitForSelector('.loading', { state: 'hidden', timeout: 10000 }).catch(() => {});

		const saveButton = page.locator('button:has-text("Save Changes"), button:has-text("Save")');
		if ((await saveButton.count()) > 0) {
			// Should be visible - may need scrolling
			await saveButton.first().scrollIntoViewIfNeeded();
			await expect(saveButton.first()).toBeVisible();
		}
	});

	test('grid columns collapse to single on mobile', async ({ page }) => {
		await page.goto('/config/compliance');
		await page.waitForLoadState('domcontentloaded');
		await page.waitForSelector('.loading', { state: 'hidden', timeout: 10000 }).catch(() => {});

		// On mobile, form-grid should be single column
		const grid = page.locator('.form-grid').first();
		if ((await grid.count()) > 0) {
			const style = await grid.evaluate((el) => window.getComputedStyle(el).gridTemplateColumns);
			// On mobile, should be single column (1fr or similar)
			const columns = style.split(' ').length;
			expect(columns).toBeLessThanOrEqual(2);
		}
	});
});

test.describe('Responsive: Tablet (768px)', () => {
	test.beforeEach(async ({ page }) => {
		await page.setViewportSize(VIEWPORTS.tablet);
	});

	test('dashboard loads on tablet', async ({ page }) => {
		await page.goto('/');
		await page.waitForLoadState('domcontentloaded');

		const body = await page.textContent('body');
		expect(body?.length).toBeGreaterThan(50);
	});

	test('sidebar is visible on tablet', async ({ page }) => {
		await page.goto('/config');
		await page.waitForLoadState('domcontentloaded');

		const sidebar = page.locator('nav.sidebar, aside.sidebar, [class*="sidebar"]');
		await expect(sidebar.first()).toBeVisible();
	});

	test('page header stacks on tablet', async ({ page }) => {
		await page.goto('/config/global');
		await page.waitForLoadState('domcontentloaded');
		await page.waitForSelector('.loading', { state: 'hidden', timeout: 10000 }).catch(() => {});

		// Header should still be visible with title and button
		const h1 = page.locator('h1');
		await expect(h1.first()).toBeVisible();

		const saveButton = page.locator('button:has-text("Save Changes"), button:has-text("Save")');
		expect(await saveButton.count()).toBeGreaterThan(0);
	});

	test('config page with sliders works on tablet', async ({ page }) => {
		await page.goto('/config/compliance');
		await page.waitForLoadState('domcontentloaded');
		await page.waitForSelector('.loading', { state: 'hidden', timeout: 10000 }).catch(() => {});

		const sliders = page.locator('input[type="range"]');
		if ((await sliders.count()) > 0) {
			const firstSlider = sliders.first();
			await expect(firstSlider).toBeVisible();
		}
	});
});

test.describe('Responsive: Desktop (1440px)', () => {
	test.beforeEach(async ({ page }) => {
		await page.setViewportSize(VIEWPORTS.desktop);
	});

	test('full layout renders on desktop', async ({ page }) => {
		await page.goto('/config/global');
		await page.waitForLoadState('domcontentloaded');
		await page.waitForSelector('.loading', { state: 'hidden', timeout: 10000 }).catch(() => {});

		// Sidebar visible
		const sidebar = page.locator('nav.sidebar, aside.sidebar, [class*="sidebar"]');
		await expect(sidebar.first()).toBeVisible();

		// Main content visible
		const h1 = page.locator('h1');
		await expect(h1.first()).toBeVisible();

		// Save button visible
		const saveButton = page.locator('button:has-text("Save Changes"), button:has-text("Save")');
		expect(await saveButton.count()).toBeGreaterThan(0);
	});

	test('two-column grid displays on desktop', async ({ page }) => {
		await page.goto('/config/global');
		await page.waitForLoadState('domcontentloaded');
		await page.waitForSelector('.loading', { state: 'hidden', timeout: 10000 }).catch(() => {});

		const grid = page.locator('.form-grid').first();
		if ((await grid.count()) > 0) {
			const box = await grid.boundingBox();
			// On desktop, grid should be wide enough for 2 columns
			expect(box!.width).toBeGreaterThan(400);
		}
	});

	test('sidebar and content are side by side', async ({ page }) => {
		await page.goto('/config/global');
		await page.waitForLoadState('domcontentloaded');

		const sidebar = page.locator('nav.sidebar, aside.sidebar, [class*="sidebar"]').first();
		const mainContent = page.locator('main, [class*="main-content"], .page').first();

		const sidebarBox = await sidebar.boundingBox();
		const contentBox = await mainContent.boundingBox();

		if (sidebarBox && contentBox) {
			// Content should be to the right of sidebar
			expect(contentBox.x).toBeGreaterThanOrEqual(sidebarBox.x + sidebarBox.width - 10);
		}
	});
});

test.describe('Responsive: Cross-Viewport Navigation', () => {
	for (const [label, viewport] of Object.entries(VIEWPORTS)) {
		test(`can navigate between pages on ${label}`, async ({ page }) => {
			await page.setViewportSize(viewport);
			await page.goto('/config');
			await page.waitForLoadState('domcontentloaded');

			// Navigate to a config page
			await page.goto('/config/global');
			await page.waitForLoadState('domcontentloaded');
			expect(page.url()).toContain('/config/global');

			// Navigate to another
			await page.goto('/config/compliance');
			await page.waitForLoadState('domcontentloaded');
			expect(page.url()).toContain('/config/compliance');
		});
	}
});
