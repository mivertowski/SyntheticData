import { test, expect } from '@playwright/test';

/**
 * Visual regression tests for all routes.
 * Captures full-page screenshots and compares against baselines.
 * Run with: npx playwright test --project=visual
 * Update baselines: npx playwright test --project=visual --update-snapshots
 */

const ALL_ROUTES = [
	{ route: '/', name: 'dashboard' },
	{ route: '/stream', name: 'stream-viewer' },
	{ route: '/presets', name: 'presets' },
	{ route: '/config', name: 'config-overview' },
	{ route: '/config/global', name: 'global-settings' },
	{ route: '/config/companies', name: 'companies' },
	{ route: '/config/chart-of-accounts', name: 'chart-of-accounts' },
	{ route: '/config/transactions', name: 'transactions' },
	{ route: '/config/output', name: 'output' },
	{ route: '/config/master-data', name: 'master-data-overview' },
	{ route: '/config/master-data/vendors', name: 'vendors' },
	{ route: '/config/master-data/customers', name: 'customers' },
	{ route: '/config/master-data/materials', name: 'materials' },
	{ route: '/config/master-data/assets', name: 'assets' },
	{ route: '/config/master-data/employees', name: 'employees' },
	{ route: '/config/document-flows', name: 'document-flows-overview' },
	{ route: '/config/document-flows/p2p', name: 'procure-to-pay' },
	{ route: '/config/document-flows/o2c', name: 'order-to-cash' },
	{ route: '/config/source-to-pay', name: 'source-to-pay' },
	{ route: '/config/financial-reporting', name: 'financial-reporting' },
	{ route: '/config/hr', name: 'hr-payroll' },
	{ route: '/config/manufacturing', name: 'manufacturing' },
	{ route: '/config/sales-quotes', name: 'sales-quotes' },
	{ route: '/config/vendor-network', name: 'vendor-network' },
	{ route: '/config/customer-segmentation', name: 'customer-segmentation' },
	{ route: '/config/relationship-strength', name: 'relationship-strength' },
	{ route: '/config/cross-process-links', name: 'cross-process-links' },
	{ route: '/config/intercompany', name: 'intercompany' },
	{ route: '/config/compliance', name: 'fraud-controls' },
	{ route: '/config/data-quality', name: 'data-quality' },
	{ route: '/config/anomaly-injection', name: 'anomaly-injection' },
	{ route: '/config/analytics', name: 'analytics' },
	{ route: '/config/accounting-standards', name: 'accounting-standards' },
	{ route: '/config/graph-export', name: 'graph-export' },
	{ route: '/config/quality-gates', name: 'quality-gates' },
	{ route: '/config/banking', name: 'banking' },
	{ route: '/config/fingerprint', name: 'fingerprint' },
	{ route: '/config/temporal', name: 'temporal-patterns' },
	{ route: '/config/audit', name: 'audit' },
	{ route: '/config/ocpm', name: 'process-mining' },
	{ route: '/config/scenario', name: 'scenario' },
	{ route: '/config/behavioral-drift', name: 'behavioral-drift' },
	{ route: '/config/market-drift', name: 'market-drift' },
	{ route: '/config/organizational-events', name: 'organizational-events' },
];

test.describe('Visual Regression: Full Page Screenshots', () => {
	for (const { route, name } of ALL_ROUTES) {
		test(`screenshot: ${name} (${route})`, async ({ page }) => {
			await page.goto(route);
			await page.waitForLoadState('domcontentloaded');
			// Wait for config to load and animations to settle
			await page.waitForSelector('.loading', { state: 'hidden', timeout: 10000 }).catch(() => {});
			await page.waitForTimeout(500);

			await expect(page).toHaveScreenshot(`${name}.png`, {
				fullPage: true,
				mask: [page.locator('footer')],
			});
		});
	}
});

test.describe('Visual Regression: Responsive Viewports', () => {
	const RESPONSIVE_ROUTES = [
		{ route: '/', name: 'dashboard' },
		{ route: '/config/global', name: 'global-settings' },
		{ route: '/config/compliance', name: 'fraud-controls' },
		{ route: '/config/banking', name: 'banking' },
	];

	const VIEWPORTS = [
		{ width: 375, height: 667, label: 'mobile' },
		{ width: 768, height: 1024, label: 'tablet' },
		{ width: 1440, height: 900, label: 'desktop' },
	];

	for (const { route, name } of RESPONSIVE_ROUTES) {
		for (const viewport of VIEWPORTS) {
			test(`responsive: ${name} @ ${viewport.label}`, async ({ page }) => {
				await page.setViewportSize({ width: viewport.width, height: viewport.height });
				await page.goto(route);
				await page.waitForLoadState('domcontentloaded');
				await page.waitForSelector('.loading', { state: 'hidden', timeout: 10000 }).catch(() => {});
				await page.waitForTimeout(500);

				await expect(page).toHaveScreenshot(`${name}-${viewport.label}.png`, {
					fullPage: true,
				});
			});
		}
	}
});
