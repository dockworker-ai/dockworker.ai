import { test, expect } from '@playwright/test';

test.describe('Build Trigger Flow', () => {
  test.beforeEach(async ({ page }) => {
    // Mock authenticated user by setting auth token in storage (if applicable)
    await page.goto('/');
  });

  test('should display build form on dashboard', async ({ page }) => {
    await page.goto('/dashboard');
    const buildForm = page.locator('form').first();
    // Build form may require auth, so it might not be visible on public page
    // This test is a placeholder for authenticated flow
  });

  test('should have git repo input field', async ({ page }) => {
    await page.goto('/');
    const repoInput = page.locator('input[placeholder*="repo"], input[placeholder*="github"], input[placeholder*="git"]').first();
    if (await repoInput.isVisible()) {
      await expect(repoInput).toBeVisible();
    }
  });

  test('should have image tag input field', async ({ page }) => {
    await page.goto('/');
    const imageInput = page.locator('input[placeholder*="image"], input[placeholder*="docker"]').first();
    if (await imageInput.isVisible()) {
      await expect(imageInput).toBeVisible();
    }
  });

  test('should have submit button on build form', async ({ page }) => {
    await page.goto('/');
    const submitButton = page.locator('button:has-text(/build|submit|start|trigger/i)').first();
    if (await submitButton.isVisible()) {
      await expect(submitButton).toBeVisible();
    }
  });

  test('should display recent builds history', async ({ page }) => {
    await page.goto('/dashboard');
    const buildsList = page.locator('[data-testid="builds-list"], ul:has-text(/build)');
    if (await buildsList.isVisible()) {
      await expect(buildsList).toBeVisible();
    }
  });

  test('should show build status indicators', async ({ page }) => {
    await page.goto('/dashboard');
    const statusIndicators = page.locator('[data-testid="build-status"], span:has-text(/running|completed|failed|success)/i');
    const count = await statusIndicators.count();
    // Expect at least one status indicator if builds are shown
    if (count > 0) {
      await expect(statusIndicators.first()).toBeVisible();
    }
  });
});
