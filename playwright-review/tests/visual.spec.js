// Phase 4.5 — Playwright Visual Review
// Screenshots every panel and saves to playwright-review/screenshots/
// Reads the real session token from ~/.local/share/eleutheria-telos/server.json

const { test } = require('@playwright/test');
const fs = require('fs');
const path = require('path');
const os = require('os');

// Read the real token + port from the running app
function getServerInfo() {
  const serverJsonPath = path.join(
    os.homedir(),
    '.local/share/eleutheria-telos/server.json'
  );
  try {
    return JSON.parse(fs.readFileSync(serverJsonPath, 'utf-8'));
  } catch (e) {
    return { port: 47821, token: 'dev-token' };
  }
}

const { port, token } = getServerInfo();
const screenshotsDir = path.join(__dirname, '..', 'screenshots');
if (!fs.existsSync(screenshotsDir)) fs.mkdirSync(screenshotsDir, { recursive: true });

const PANELS = [
  { id: 'clipboard',       label: 'Clipboard' },
  { id: 'notes',           label: 'Notes' },
  { id: 'voice',           label: 'Voice' },
  { id: 'ocr',             label: 'OCR' },
  { id: 'translate',       label: 'Translate' },
  { id: 'search',          label: 'Search' },
  { id: 'screen-recorder', label: 'Screen Recorder' },
  { id: 'audio-recorder',  label: 'Audio Recorder' },
  { id: 'photo-editor',    label: 'Photo Editor' },
  { id: 'video-processor', label: 'Video Processor' },
  { id: 'quick-actions',   label: 'Quick Actions' },
  { id: 'models',          label: 'Models' },
  { id: 'settings',        label: 'Settings' },
];

test.describe('Phase 4.5 Visual Review', () => {

  test.beforeEach(async ({ page }) => {
    // Inject the real token + port BEFORE any page scripts run
    await page.addInitScript(({ p, t }) => {
      window.__SESSION_TOKEN__ = t;
      window.__API_PORT__ = p;
    }, { p: port, t: token });
  });

  test('Shell and all panels', async ({ page }) => {
    // Navigate to the static shell
    await page.goto('/index.html');

    // Wait for initApp() default panel (clipboard) to load — look for any h1 in #tool-panel
    await page.waitForSelector('#tool-panel h1', { timeout: 15000 });
    await page.waitForTimeout(500); // icons settle

    // Screenshot: shell with the clipboard panel (default)
    await page.screenshot({
      path: path.join(screenshotsDir, '00-shell-clipboard.png'),
      fullPage: false,
    });
    console.log('✓  00-shell-clipboard.png (Clipboard default)');

    // Navigate to each panel and screenshot
    for (let i = 0; i < PANELS.length; i++) {
      const panel = PANELS[i];

      // Wait for the HTMX response before proceeding
      const responsePromise = page.waitForResponse(
        resp =>
          resp.url().includes(`/tools/${panel.id}`) &&
          resp.request().method() === 'GET' &&
          resp.status() === 200,
        { timeout: 10000 }
      );

      // Trigger htmx.ajax() navigation
      await page.evaluate(({ id, p, t }) => {
        htmx.ajax('GET', `http://127.0.0.1:${p}/tools/${id}`, {
          target: '#tool-panel',
          swap: 'innerHTML',
          headers: { 'Authorization': 'Bearer ' + t },
        });
      }, { id: panel.id, p: port, t: token });

      // Wait for the response
      await responsePromise;

      // Wait for DOM swap + Lucide icons + any sub-requests
      await page.waitForTimeout(600);

      const filename = `${String(i + 1).padStart(2, '0')}-${panel.id}.png`;
      await page.screenshot({
        path: path.join(screenshotsDir, filename),
        fullPage: false,
      });
      console.log(`✓  ${filename} — ${panel.label}`);
    }

    // Also screenshot the command palette
    await page.keyboard.press('Control+k');
    await page.waitForSelector('#palette-box', { timeout: 3000 });
    await page.waitForTimeout(300);
    await page.screenshot({
      path: path.join(screenshotsDir, '99-command-palette.png'),
      fullPage: false,
    });
    console.log('✓  99-command-palette.png');
  });

  // Sidebar states
  test('Sidebar collapse', async ({ page }) => {
    await page.goto('/index.html');
    await page.waitForSelector('#tool-panel h1', { timeout: 15000 });
    await page.waitForTimeout(500);

    // Screenshot: sidebar expanded (default)
    await page.screenshot({ path: path.join(screenshotsDir, 'sidebar-expanded.png') });
    console.log('✓  sidebar-expanded.png');

    // Click collapse toggle
    const collapseBtn = page.locator('#sidebar button[title*="Collapse"]');
    await collapseBtn.click();
    await page.waitForTimeout(350); // CSS transition: 200ms + buffer
    await page.screenshot({ path: path.join(screenshotsDir, 'sidebar-collapsed.png') });
    console.log('✓  sidebar-collapsed.png');
  });

});
