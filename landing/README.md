# dockworker.ai marketing (production)

Vendored Figma Make export served at **https://dockworker.ai/** via Cloudflare Pages.

- **Build:** `bun run build` (invoked from `frontend/` for Pages `root_dir: frontend`).
- **GitHub nav link:** `https://github.com/dockworker-ai/dockworker.ai`
- **Console SPA:** `frontend/` (dev only until a separate app host is wired).

Re-import from Figma when the design source is available in git; this bundle exists so production tracks `main` instead of a manual Pages rollback.
