import { test, expect } from '@playwright/test';

test.describe('Landing Page', () => {
  test('should load homepage', async ({ page }) => {
    await page.goto('/');
    await expect(page).toHaveTitle(/dockworker/i);
  });

  test('should display hero section with CTA', async ({ page }) => {
    await page.goto('/');
    const heading = page.locator('h1');
    await expect(heading).toContainText(/build|deploy|container/i);

    const ctaButton = page.locator('button:has-text(/get started|sign up|start free/i)').first();
    await expect(ctaButton).toBeVisible();
  });

  test('should display pricing section', async ({ page }) => {
    await page.goto('/');
    const pricingSection = page.locator('section:has-text(/pricing|plans)/i');
    await expect(pricingSection).toBeVisible();

    const freeCard = page.locator('text=Free').first();
    await expect(freeCard).toBeVisible();
  });

  test('should display features section', async ({ page }) => {
    await page.goto('/');
    const featuresSection = page.locator('section:has-text(/features|build|container)/i');
    await expect(featuresSection).toBeVisible();
  });

  test('should have working navigation links', async ({ page }) => {
    await page.goto('/');
    const navLinks = page.locator('nav a');
    const count = await navLinks.count();
    expect(count).toBeGreaterThan(0);

    for (let i = 0; i < Math.min(count, 3); i++) {
      const link = navLinks.nth(i);
      await expect(link).toBeVisible();
    }
  });
});
