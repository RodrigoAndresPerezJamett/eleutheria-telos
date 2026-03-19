const { defineConfig } = require('@playwright/test');

module.exports = defineConfig({
  testDir: './tests',
  timeout: 30_000,
  use: {
    // Serve UI static files from the ui/ directory
    baseURL: 'http://localhost:9191',
    viewport: { width: 1280, height: 800 },
    screenshot: 'off',  // we take manual screenshots in the test
  },
  webServer: {
    command: 'python3 -m http.server 9191 --directory ../ui',
    url: 'http://localhost:9191',
    reuseExistingServer: true,
    timeout: 5000,
  },
  projects: [
    {
      name: 'chromium',
      use: { browserName: 'chromium' },
    },
  ],
});
