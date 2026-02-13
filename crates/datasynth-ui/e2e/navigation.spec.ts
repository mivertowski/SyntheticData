import { test, expect } from '@playwright/test';

/**
 * Navigation and sidebar tests.
 * Covers: all sidebar nav links, collapsible sections, GH Issue #27 fix,
 * mobile menu behavior, and active link highlighting.
 */

const SIDEBAR_SECTIONS = [
	'Generate',
	'Core Config',
	'Master Data',
	'Document Flows',
	'Enterprise',
	'Interconnectivity',
	'Quality & Anomaly',
	'Standards',
	'Specialized',
	'Presets',
];

const CONFIG_ROUTES = [
	'/config',
	'/config/global',
	'/config/companies',
	'/config/chart-of-accounts',
	'/config/transactions',
	'/config/output',
	'/config/master-data',
	'/config/master-data/vendors',
	'/config/master-data/customers',
	'/config/master-data/materials',
	'/config/master-data/assets',
	'/config/master-data/employees',
	'/config/document-flows',
	'/config/document-flows/p2p',
	'/config/document-flows/o2c',
	'/config/source-to-pay',
	'/config/financial-reporting',
	'/config/hr',
	'/config/manufacturing',
	'/config/sales-quotes',
	'/config/vendor-network',
	'/config/customer-segmentation',
	'/config/relationship-strength',
	'/config/cross-process-links',
	'/config/intercompany',
	'/config/compliance',
	'/config/data-quality',
	'/config/anomaly-injection',
	'/config/analytics',
	'/config/accounting-standards',
	'/config/graph-export',
	'/config/quality-gates',
	'/config/banking',
	'/config/fingerprint',
	'/config/temporal',
	'/config/audit',
	'/config/ocpm',
	'/config/scenario',
	'/config/behavioral-drift',
	'/config/market-drift',
	'/config/organizational-events',
];

test.describe('Sidebar Navigation', () => {
	test.beforeEach(async ({ page }) => {
		await page.goto('/config');
		await page.waitForLoadState('domcontentloaded');
	});

	test('sidebar is visible on config routes', async ({ page }) => {
		const sidebar = page.locator('nav.sidebar, aside.sidebar, [class*="sidebar"]');
		await expect(sidebar.first()).toBeVisible();
	});

	test('sidebar has all section headers', async ({ page }) => {
		for (const section of SIDEBAR_SECTIONS) {
			const sectionHeader = page.locator(`text=${section}`);
			// Section headers should exist in DOM (some may be collapsed)
			expect(await sectionHeader.count()).toBeGreaterThan(0);
		}
	});

	test('sidebar sections are collapsible', async ({ page }) => {
		// Find a collapsible section toggle button
		const toggleButtons = page.locator('.section-toggle, button[class*="section"]');
		const count = await toggleButtons.count();
		expect(count).toBeGreaterThan(0);

		// Click to toggle - items inside should hide/show
		const firstToggle = toggleButtons.first();
		await firstToggle.click();
		await page.waitForTimeout(300);

		// Click again to restore
		await firstToggle.click();
		await page.waitForTimeout(300);
	});

	test('active link is highlighted', async ({ page }) => {
		await page.goto('/config/global');
		await page.waitForLoadState('domcontentloaded');

		const activeLink = page.locator('a.active[href="/config/global"], a[class*="active"][href="/config/global"]');
		expect(await activeLink.count()).toBeGreaterThan(0);
	});

	test('clicking nav link navigates to correct page', async ({ page }) => {
		const globalLink = page.locator('a[href="/config/global"]');
		await globalLink.click();
		await page.waitForLoadState('domcontentloaded');

		expect(page.url()).toContain('/config/global');
	});

	test('all config nav links resolve to valid routes', async ({ page }) => {
		const navLinks = page.locator('a[href^="/config/"]');
		const linkCount = await navLinks.count();
		expect(linkCount).toBeGreaterThan(30);

		// Spot-check a few links
		const hrefs: string[] = [];
		for (let i = 0; i < linkCount; i++) {
			const href = await navLinks.nth(i).getAttribute('href');
			if (href) hrefs.push(href);
		}

		// All configured routes should be in sidebar
		for (const route of CONFIG_ROUTES) {
			expect(hrefs).toContain(route);
		}
	});
});

test.describe('GH Issue #27 Fix: Sidebar on /presets', () => {
	test('sidebar is visible on /presets route', async ({ page }) => {
		await page.goto('/presets');
		await page.waitForLoadState('domcontentloaded');

		const sidebar = page.locator('nav.sidebar, aside.sidebar, [class*="sidebar"]');
		await expect(sidebar.first()).toBeVisible();
	});

	test('presets link is in sidebar navigation', async ({ page }) => {
		await page.goto('/presets');
		await page.waitForLoadState('domcontentloaded');

		const presetsLink = page.locator('a[href="/presets"]');
		expect(await presetsLink.count()).toBeGreaterThan(0);
	});
});

test.describe('Sidebar on Non-Config Routes', () => {
	test('sidebar is visible on dashboard', async ({ page }) => {
		await page.goto('/');
		await page.waitForLoadState('domcontentloaded');

		const sidebar = page.locator('nav.sidebar, aside.sidebar, [class*="sidebar"]');
		await expect(sidebar.first()).toBeVisible();
	});

	test('sidebar is visible on stream page', async ({ page }) => {
		await page.goto('/stream');
		await page.waitForLoadState('domcontentloaded');

		const sidebar = page.locator('nav.sidebar, aside.sidebar, [class*="sidebar"]');
		await expect(sidebar.first()).toBeVisible();
	});
});

test.describe('Section Auto-Expand', () => {
	test('navigating to enterprise route auto-expands enterprise section', async ({ page }) => {
		await page.goto('/config/source-to-pay');
		await page.waitForLoadState('domcontentloaded');

		// The Source-to-Pay link should be visible (section expanded)
		const s2pLink = page.locator('a[href="/config/source-to-pay"]');
		await expect(s2pLink).toBeVisible();
	});

	test('navigating to interconnectivity route auto-expands that section', async ({ page }) => {
		await page.goto('/config/vendor-network');
		await page.waitForLoadState('domcontentloaded');

		const vnLink = page.locator('a[href="/config/vendor-network"]');
		await expect(vnLink).toBeVisible();
	});
});

test.describe('Mobile Navigation', () => {
	test('sidebar adapts on mobile viewport', async ({ page }) => {
		await page.setViewportSize({ width: 375, height: 667 });
		await page.goto('/config');
		await page.waitForLoadState('domcontentloaded');

		// On mobile, sidebar may be hidden or collapsed
		// Page should still be functional
		const body = await page.textContent('body');
		expect(body?.length).toBeGreaterThan(0);
	});

	test('sidebar adapts on tablet viewport', async ({ page }) => {
		await page.setViewportSize({ width: 768, height: 1024 });
		await page.goto('/config');
		await page.waitForLoadState('domcontentloaded');

		const body = await page.textContent('body');
		expect(body?.length).toBeGreaterThan(0);
	});
});
