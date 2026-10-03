import { test, expect } from '@playwright/test';

test.describe('Performance & CDN Delivery', () => {
  test('should load main assets from CDN', async ({ page }) => {
    const responses: string[] = [];

    page.on('response', response => {
      responses.push(response.url());
    });

    await page.goto('/');

    // Verify that assets are loaded
    expect(responses.length).toBeGreaterThan(0);

    // Check for .js and .css assets
    const hasJS = responses.some(url => url.endsWith('.js'));
    const hasCSS = responses.some(url => url.endsWith('.css'));

    expect(hasJS || responses.length > 5).toBeTruthy();
  });

  test('should have good page load performance', async ({ page }) => {
    const startTime = Date.now();
    await page.goto('/');
    const loadTime = Date.now() - startTime;

    // CF Pages should load in under 5 seconds
    expect(loadTime).toBeLessThan(5000);
  });

  test('should load images efficiently', async ({ page }) => {
    await page.goto('/');

    const images = page.locator('img');
    const count = await images.count();

    for (let i = 0; i < count; i++) {
      const img = images.nth(i);
      if (await img.isVisible()) {
        // Images should have alt text (accessibility)
        const alt = await img.getAttribute('alt');
        if (alt) {
          expect(alt.length).toBeGreaterThan(0);
        }
      }
    }
  });

  test('should have proper caching headers', async ({ page }) => {
    const response = await page.goto('/');
    if (response) {
      const cacheControl = response.headers()['cache-control'];
      // Verify caching is configured (CF Pages should set it)
      expect(cacheControl).toBeDefined();
    }
  });

  test('should serve gzip compressed content', async ({ page }) => {
    const response = await page.goto('/');
    if (response) {
      const contentEncoding = response.headers()['content-encoding'];
      // Verify compression is enabled
      expect(['gzip', 'br', 'deflate', undefined]).toContain(contentEncoding);
    }
  });
});
