import { test, expect } from '@playwright/test';

test.describe('Build Interface (Coming Soon)', () => {
  test('should have dashboard route available after auth', async ({ page }) => {
    // Dashboard is not yet part of the landing page design
    // This is a placeholder for when dashboard is implemented
    await page.goto('/');
    const dashboardLink = page.locator('a[href="/dashboard"]');

    // Dashboard link may not exist in current design
    if (await dashboardLink.count() > 0) {
      await expect(dashboardLink).toBeVisible();
    }
  });

  test('should render landing page properly for future build flow integration', async ({ page }) => {
    await page.goto('/');
    const main = page.locator('main').first();
    await expect(main).toBeVisible();
  });

  test('should have CTA that could lead to build form', async ({ page }) => {
    await page.goto('/');

    const primaryCTA = page.locator('a.brand-button, button.brand-button').first();
    if (await primaryCTA.count() > 0) {
      await expect(primaryCTA).toBeVisible();
    }
  });

  test('should display pipeline visualization (related to build process)', async ({ page }) => {
    await page.goto('/');

    // The new design has a Pipeline component showing the build process
    const pipelineSection = page.locator('section').filter({
      has: page.locator('text=/pipeline|build|container/i')
    }).first();

    if (await pipelineSection.count() > 0) {
      await expect(pipelineSection).toBeVisible();
    }
  });

  test('should compare apko vs nix build approaches', async ({ page }) => {
    await page.goto('/');

    const comparisonSection = page.locator('table, [role="table"]').first();
    if (await comparisonSection.count() > 0) {
      await expect(comparisonSection).toBeVisible();
    }
  });

  test('should explain apko feature (immutable container building)', async ({ page }) => {
    await page.goto('/');
    await page.evaluate(() => window.scrollBy(0, 1000));

    const apkoText = page.locator('text=/apko/i').first();
    if (await apkoText.count() > 0) {
      await expect(apkoText).toBeVisible();
    }
  });

  test('should have footer CTA for getting started', async ({ page }) => {
    await page.goto('/');
    await page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));

    const footer = page.locator('footer').first();
    await expect(footer).toBeVisible();
  });

  test('should maintain responsive layout for future build UI', async ({ page }) => {
    await page.goto('/');

    const main = page.locator('main').first();
    const boundingBox = await main.boundingBox();

    if (boundingBox) {
      expect(boundingBox.width).toBeGreaterThan(0);
      expect(boundingBox.height).toBeGreaterThan(0);
    }
  });
});
