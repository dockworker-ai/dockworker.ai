import { test, expect } from '@playwright/test';

test.describe('Authentication Flow', () => {
  test('should display login/signup page', async ({ page }) => {
    await page.goto('/auth');
    const loginForm = page.locator('form');
    await expect(loginForm).toBeVisible();
  });

  test('should have GitHub OAuth button', async ({ page }) => {
    await page.goto('/auth');
    const githubButton = page.locator('button:has-text(/github|sign.*with github/i)');
    await expect(githubButton).toBeVisible();
  });

  test('should navigate to auth from CTA button', async ({ page }) => {
    await page.goto('/');
    const ctaButton = page.locator('button:has-text(/get started|sign up|start free/i)').first();
    await ctaButton.click();

    // Should redirect to auth or login page
    const url = page.url();
    expect(url).toMatch(/auth|login|signup/i);
  });

  test('should display email input field (if email signup available)', async ({ page }) => {
    await page.goto('/auth');
    const emailField = page.locator('input[type="email"]');
    if (await emailField.isVisible()) {
      await expect(emailField).toBeVisible();
      await expect(emailField).toHaveAttribute('placeholder', /email|email address/i);
    }
  });

  test('should display password field (if email signup available)', async ({ page }) => {
    await page.goto('/auth');
    const passwordField = page.locator('input[type="password"]');
    if (await passwordField.isVisible()) {
      await expect(passwordField).toBeVisible();
    }
  });
});
