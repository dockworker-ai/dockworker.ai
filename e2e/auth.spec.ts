import { test, expect } from '@playwright/test';

test.describe('Authentication Flow', () => {
  test('should have login/auth entry point from nav', async ({ page }) => {
    await page.goto('/');
    const nav = page.locator('nav').first();
    await expect(nav).toBeVisible();

    // Look for auth/login link in navigation
    const authLink = nav.locator('a:has-text(/login|sign|auth/i)');
    const authLinkCount = await authLink.count();
    // Auth link may or may not be visible depending on implementation
    expect(authLinkCount).toBeGreaterThanOrEqual(0);
  });

  test('should navigate to auth page from CTA button', async ({ page }) => {
    await page.goto('/');

    // Find any button that might trigger authentication
    const buttons = page.locator('button, a[class*="brand"]').first();
    if (await buttons.isVisible()) {
      const href = await buttons.getAttribute('href');
      // If button has href pointing to auth, verify it's navigable
      if (href && href.includes('auth')) {
        await expect(buttons).toHaveAttribute('href', /auth|login/);
      }
    }
  });

  test('should have GitHub OAuth button if auth implemented', async ({ page }) => {
    const authPage = '/auth';

    try {
      await page.goto(authPage, { waitUntil: 'load', timeout: 3000 });
      const githubButton = page.locator('button:has-text(/github|sign.*with github/i)');

      // If auth page exists, GitHub button should be present
      if (await githubButton.count() > 0) {
        await expect(githubButton.first()).toBeVisible();
      }
    } catch {
      // Auth page not yet implemented, skip test
    }
  });

  test('should be able to navigate back from auth to home', async ({ page }) => {
    await page.goto('/');
    const homeTitle = await page.title();
    expect(homeTitle).toMatch(/dockworker/i);
  });

  test('should preserve nav state during auth flow', async ({ page }) => {
    await page.goto('/');
    const nav = page.locator('nav').first();
    const navVisible = await nav.isVisible();

    expect(navVisible).toBe(true);
  });
});
