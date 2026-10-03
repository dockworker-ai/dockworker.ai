# End-to-End Tests for dockworker.ai

This directory contains Playwright tests for the dockworker.ai frontend deployment on Cloudflare Pages.

## Test Files

- **landing-page.spec.ts** — Tests for the homepage, hero section, pricing, features, and navigation
- **auth.spec.ts** — Tests for authentication flows, GitHub OAuth integration, and login/signup pages
- **build-flow.spec.ts** — Tests for the build trigger form, build history, and status indicators
- **performance-and-cdn.spec.ts** — Tests for CDN delivery, performance metrics, caching headers, and compression

## Running Tests

### Prerequisites

```bash
bun install
bunx playwright install
```

### Local Development

Test against a local dev server (requires `npm run dev` or equivalent):

```bash
bun run test:e2e:local
```

### Against Staging (CF Pages)

Test the staging deployment:

```bash
bun run test:e2e:staging
```

### Against Production (CF Pages)

Test the production deployment:

```bash
bun run test:e2e:prod
```

### Interactive UI Mode

Debug tests with Playwright Inspector:

```bash
bun run test:e2e:ui
```

### Debug Mode

Run tests with detailed logging:

```bash
bun run test:e2e:debug
```

## CI/CD Integration

Tests are automatically run via GitHub Actions in three scenarios:

1. **On Push/PR** — Runs against local dev server and staging
2. **On Main Push** — Runs against local dev server, staging, and waits for CF deployment
3. **Daily Schedule** — Runs production health check at 02:00 UTC

See `.github/workflows/playwright.yml` for configuration.

## Configuration

### playwright.config.ts

- Base URL defaults to `https://dockworker.ai` (overridable via `BASE_URL` env var)
- Runs tests in Chromium and Firefox
- Captures screenshots on failure
- Generates HTML report

### Environment Variables

- `BASE_URL` — Override the default base URL (e.g., `http://localhost:3000`)
- `CI` — Set automatically in CI environments; enables retries and single worker

## Reports

Test reports are generated in `playwright-report/` and can be viewed with:

```bash
bunx playwright show-report
```

## Debugging

### View HTML Report

```bash
bunx playwright show-report
```

### Enable Trace Debugging

Set `trace: 'on'` in playwright.config.ts to record full browser traces.

### Screenshots

Failed tests automatically capture screenshots in the report.

## Test Structure

Each test file follows this pattern:

```typescript
import { test, expect } from '@playwright/test';

test.describe('Feature Name', () => {
  test('should do something specific', async ({ page }) => {
    await page.goto('/');
    const element = page.locator('selector');
    await expect(element).toBeVisible();
  });
});
```

## Selectors Strategy

Tests use a mix of:
- **Text selectors** — `page.locator('button:has-text(/click me/i)')`
- **Data attributes** — `page.locator('[data-testid="button"]')` (when available)
- **CSS selectors** — Standard CSS when elements are stable

## Maintenance

When updating the frontend:

1. Run tests locally to ensure they still pass
2. Update selectors if UI elements change
3. Add new tests for new features
4. Commit tests alongside UI changes

## Performance Baseline

Tests check that:
- Page loads in < 5 seconds
- CSS and JS assets are available
- Images have alt text
- Caching headers are configured
- Content is gzip-compressed

Adjust thresholds in `performance-and-cdn.spec.ts` as needed.
