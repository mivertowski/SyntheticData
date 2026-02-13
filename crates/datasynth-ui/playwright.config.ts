import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright configuration for E2E testing of synth-ui.
 *
 * This tests the SvelteKit web application in isolation (without Tauri backend).
 * For full application testing with Tauri, use Tauri's native testing capabilities.
 */
export default defineConfig({
	testDir: './e2e',
	/* Run tests in files in parallel */
	fullyParallel: true,
	/* Fail the build on CI if you accidentally left test.only in the source code. */
	forbidOnly: !!process.env.CI,
	/* Retry on CI only */
	retries: process.env.CI ? 2 : 0,
	/* Opt out of parallel tests on CI. */
	workers: process.env.CI ? 1 : undefined,
	/* Reporter to use. */
	reporter: 'html',
	/* Shared settings for all the projects below. */
	use: {
		/* Base URL to use in actions like `await page.goto('/')`. */
		baseURL: 'http://localhost:5173',

		/* Collect trace when retrying the failed test. */
		trace: 'on-first-retry',

		/* Take screenshot on failure */
		screenshot: 'only-on-failure',
	},

	/* Visual regression snapshot settings */
	snapshotPathTemplate: '{testDir}/__screenshots__/{testFilePath}/{arg}{ext}',

	expect: {
		timeout: 5 * 1000,
		toHaveScreenshot: {
			maxDiffPixelRatio: 0.01,
			animations: 'disabled',
		},
	},

	/* Configure projects for major browsers */
	projects: [
		{
			name: 'chromium',
			use: { ...devices['Desktop Chrome'] },
		},

		{
			name: 'firefox',
			use: { ...devices['Desktop Firefox'] },
		},

		{
			name: 'webkit',
			use: { ...devices['Desktop Safari'] },
		},

		{
			name: 'visual',
			testMatch: /visual-regression\.spec\.ts/,
			use: { ...devices['Desktop Chrome'] },
		},
	],

	/* Run your local dev server before starting the tests */
	webServer: {
		command: 'npm run dev',
		url: 'http://localhost:5173',
		reuseExistingServer: !process.env.CI,
		timeout: 120 * 1000,
	},

	/* Global timeout for each test */
	timeout: 30 * 1000,
});
