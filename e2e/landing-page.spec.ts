import { test, expect } from '@playwright/test';

test.describe('Landing Page (Figma Design)', () => {
  test('should load homepage with correct title', async ({ page }) => {
    await page.goto('/');
    await expect(page).toHaveTitle(/dockworker/i);
  });

  test('should display hero section with headline', async ({ page }) => {
    await page.goto('/');
    const h1 = page.locator('h1');
    await expect(h1).toContainText(/Build once/i);
    await expect(h1).toContainText(/Promote often/i);
  });

  test('should have hero CTA buttons', async ({ page }) => {
    await page.goto('/');

    const pipelineBtn = page.locator('a:has-text(/See the pipeline/)');
    await expect(pipelineBtn).toBeVisible();
    await expect(pipelineBtn).toHaveAttribute('href', '#how-it-works');

    const compareBtn = page.locator('a:has-text(/Compare the two paths/)');
    await expect(compareBtn).toBeVisible();
    await expect(compareBtn).toHaveAttribute('href', '#comparison');
  });

  test('should display hero proof points', async ({ page }) => {
    await page.goto('/');

    const proofPoints = [
      /apko in every path/i,
      /Nix when you want it/i,
      /Signed digest promotion/i,
    ];

    for (const point of proofPoints) {
      const matcher = point.source.replace(/\\|^\/|\/[igm]$/g, '');
      await expect(page.locator(`text=${matcher}`)).toBeVisible();
    }
  });

  test('should display Pipeline section on nav scroll', async ({ page }) => {
    await page.goto('/');
    await page.locator('a[href="#how-it-works"]').click();

    const pipelineSection = page.locator('section').first();
    await expect(pipelineSection).toBeVisible();
  });

  test('should display ComparisonTable section', async ({ page }) => {
    await page.goto('/');
    await page.locator('a[href="#comparison"]').click();

    const comparisonTable = page.locator('table, [role="table"]').first();
    await expect(comparisonTable).toBeVisible();
  });

  test('should display ApkoFeature section', async ({ page }) => {
    await page.goto('/');
    await page.evaluate(() => window.scrollBy(0, document.body.scrollHeight / 2));

    const featureText = page.locator('text=/apko|feature/i').first();
    await expect(featureText).toBeVisible();
  });

  test('should display footer', async ({ page }) => {
    await page.goto('/');
    await page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));

    const footer = page.locator('footer');
    await expect(footer).toBeVisible();
  });

  test('should have responsive navigation bar', async ({ page }) => {
    await page.goto('/');
    const nav = page.locator('nav').first();
    await expect(nav).toBeVisible();
  });

  test('should have hero badge with Kubernetes branding', async ({ page }) => {
    await page.goto('/');
    const badge = page.locator('text=/KUBERNETES CONTAINER PACKAGING/i');
    await expect(badge).toBeVisible();
  });
});
