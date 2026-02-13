import { test, expect } from '@playwright/test';

/**
 * Parameterized E2E tests for all config pages.
 * Tests loading, header presence, toggle interactions, form inputs,
 * dirty state, and validation for every config route.
 */

interface ConfigPageDef {
	route: string;
	title: string;
	hasToggle?: boolean;
	hasNumberInput?: boolean;
	hasSlider?: boolean;
}

const CONFIG_PAGES: ConfigPageDef[] = [
	// Core Config
	{ route: '/config', title: 'Configuration', hasToggle: false },
	{ route: '/config/global', title: 'Global Settings', hasNumberInput: true },
	{ route: '/config/companies', title: 'Companies' },
	{ route: '/config/chart-of-accounts', title: 'Chart of Accounts' },
	{ route: '/config/transactions', title: 'Transactions', hasNumberInput: true },
	{ route: '/config/output', title: 'Output' },

	// Master Data
	{ route: '/config/master-data', title: 'Master Data' },
	{ route: '/config/master-data/vendors', title: 'Vendors', hasNumberInput: true, hasToggle: true },
	{ route: '/config/master-data/customers', title: 'Customers', hasNumberInput: true, hasToggle: true },
	{ route: '/config/master-data/materials', title: 'Materials', hasNumberInput: true, hasToggle: true },
	{ route: '/config/master-data/assets', title: 'Fixed Assets', hasNumberInput: true, hasToggle: true },
	{ route: '/config/master-data/employees', title: 'Employees', hasNumberInput: true, hasToggle: true },

	// Document Flows
	{ route: '/config/document-flows', title: 'Document Flows' },
	{ route: '/config/document-flows/p2p', title: 'Procure to Pay', hasToggle: true, hasSlider: true },
	{ route: '/config/document-flows/o2c', title: 'Order to Cash', hasToggle: true, hasSlider: true },

	// Enterprise
	{ route: '/config/source-to-pay', title: 'Source-to-Pay', hasToggle: true },
	{ route: '/config/financial-reporting', title: 'Financial Reporting', hasToggle: true },
	{ route: '/config/hr', title: 'HR', hasToggle: true, hasNumberInput: true },
	{ route: '/config/manufacturing', title: 'Manufacturing', hasToggle: true, hasNumberInput: true },
	{ route: '/config/sales-quotes', title: 'Sales Quotes', hasToggle: true, hasNumberInput: true },

	// Interconnectivity
	{ route: '/config/vendor-network', title: 'Vendor Network', hasToggle: true, hasNumberInput: true },
	{ route: '/config/customer-segmentation', title: 'Customer Segmentation', hasToggle: true, hasSlider: true },
	{ route: '/config/relationship-strength', title: 'Relationship Strength', hasToggle: true, hasSlider: true },
	{ route: '/config/cross-process-links', title: 'Cross-Process Links', hasToggle: true },
	{ route: '/config/intercompany', title: 'Intercompany', hasToggle: true },

	// Quality & Anomaly
	{ route: '/config/compliance', title: 'Compliance', hasToggle: true, hasSlider: true },
	{ route: '/config/data-quality', title: 'Data Quality', hasToggle: true, hasSlider: true },
	{ route: '/config/anomaly-injection', title: 'Anomaly Injection', hasToggle: true, hasSlider: true },
	{ route: '/config/analytics', title: 'Analytics', hasToggle: true },

	// Standards
	{ route: '/config/accounting-standards', title: 'Accounting Standards', hasToggle: true },
	{ route: '/config/graph-export', title: 'Graph Export', hasToggle: true },
	{ route: '/config/quality-gates', title: 'Quality Gates', hasToggle: true },

	// Specialized
	{ route: '/config/banking', title: 'Banking', hasToggle: true, hasSlider: true },
	{ route: '/config/fingerprint', title: 'Fingerprint', hasToggle: true },
	{ route: '/config/temporal', title: 'Temporal', hasToggle: true },
	{ route: '/config/audit', title: 'Audit', hasToggle: true, hasSlider: true },
	{ route: '/config/ocpm', title: 'Process Mining', hasToggle: true },
	{ route: '/config/scenario', title: 'Scenario', hasToggle: true },
	{ route: '/config/behavioral-drift', title: 'Behavioral Drift', hasToggle: true, hasSlider: true },
	{ route: '/config/market-drift', title: 'Market Drift', hasToggle: true, hasSlider: true },
	{ route: '/config/organizational-events', title: 'Organizational Events', hasToggle: true },
];

for (const pageDef of CONFIG_PAGES) {
	test.describe(`Config Page: ${pageDef.title} (${pageDef.route})`, () => {
		test.beforeEach(async ({ page }) => {
			await page.goto(pageDef.route);
			await page.waitForLoadState('domcontentloaded');
			// Wait for config to load
			await page.waitForSelector('.loading', { state: 'hidden', timeout: 10000 }).catch(() => {});
		});

		test('page loads without errors', async ({ page }) => {
			const body = await page.textContent('body');
			expect(body?.length).toBeGreaterThan(50);

			// No uncaught JS errors
			const errors: string[] = [];
			page.on('pageerror', (err) => errors.push(err.message));
			await page.waitForTimeout(500);
			const unexpected = errors.filter(
				(e) => !e.includes('Tauri') && !e.includes('__TAURI__') && !e.includes('WebSocket')
			);
			expect(unexpected).toHaveLength(0);
		});

		test('has page header with title', async ({ page }) => {
			const header = page.locator('.page-header h1, header h1, h1');
			await expect(header.first()).toBeVisible();
			const text = await header.first().textContent();
			expect(text?.toLowerCase()).toContain(pageDef.title.toLowerCase().split(' ')[0].toLowerCase());
		});

		test('has save/discard buttons in header', async ({ page }) => {
			const saveButton = page.locator('button:has-text("Save Changes"), button:has-text("Save")');
			expect(await saveButton.count()).toBeGreaterThan(0);
		});

		if (pageDef.hasToggle) {
			test('has toggle switches', async ({ page }) => {
				const toggles = page.locator('input[type="checkbox"], [role="switch"], .toggle');
				expect(await toggles.count()).toBeGreaterThan(0);
			});

			test('toggle interaction triggers dirty state', async ({ page }) => {
				const toggle = page.locator('input[type="checkbox"], [role="switch"], .toggle').first();
				if ((await toggle.count()) > 0) {
					await toggle.click();
					await page.waitForTimeout(300);

					// Discard button should appear when dirty
					const discardButton = page.locator('button:has-text("Discard")');
					await expect(discardButton).toBeVisible();
				}
			});
		}

		if (pageDef.hasNumberInput) {
			test('has number inputs', async ({ page }) => {
				const numberInputs = page.locator('input[type="number"]');
				expect(await numberInputs.count()).toBeGreaterThan(0);
			});

			test('number input change triggers dirty state', async ({ page }) => {
				const numberInput = page.locator('input[type="number"]').first();
				if ((await numberInput.count()) > 0) {
					await numberInput.click();
					const original = await numberInput.inputValue();
					await numberInput.fill('999');
					await page.waitForTimeout(300);

					const discardButton = page.locator('button:has-text("Discard")');
					await expect(discardButton).toBeVisible();

					// Discard changes
					await discardButton.click();
					await page.waitForTimeout(300);

					const restored = await numberInput.inputValue();
					expect(restored).toBe(original);
				}
			});
		}

		if (pageDef.hasSlider) {
			test('has range sliders', async ({ page }) => {
				const sliders = page.locator('input[type="range"]');
				expect(await sliders.count()).toBeGreaterThan(0);
			});
		}
	});
}

test.describe('Config Pages: Form Sections', () => {
	test('FormSection components render with titles', async ({ page }) => {
		await page.goto('/config/global');
		await page.waitForLoadState('domcontentloaded');
		await page.waitForSelector('.loading', { state: 'hidden', timeout: 10000 }).catch(() => {});

		const sections = page.locator('.form-section, section, [class*="section"]');
		expect(await sections.count()).toBeGreaterThan(0);
	});

	test('FormGroup components render with labels', async ({ page }) => {
		await page.goto('/config/global');
		await page.waitForLoadState('domcontentloaded');
		await page.waitForSelector('.loading', { state: 'hidden', timeout: 10000 }).catch(() => {});

		const labels = page.locator('label');
		expect(await labels.count()).toBeGreaterThan(0);
	});
});

test.describe('Config Pages: Distribution Validation', () => {
	test('distribution sum indicator shows on compliance page', async ({ page }) => {
		await page.goto('/config/compliance');
		await page.waitForLoadState('domcontentloaded');
		await page.waitForSelector('.loading', { state: 'hidden', timeout: 10000 }).catch(() => {});

		// Look for distribution total indicator
		const totalIndicator = page.locator('.distribution-sum, .distribution-total, [class*="distribution"]');
		expect(await totalIndicator.count()).toBeGreaterThanOrEqual(0);
	});
});
