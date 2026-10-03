# dockworker.ai

Commercial OCI packaging with a free tier.

A free build is a Kubernetes Job. The pod is non-root, uses a user namespace, runs daemonless rootless BuildKit, and is deleted shortly after it exits. Accounts, the console, and billing come after a build can push an image.

| Path | What it is |
| --- | --- |
| [`backend/`](backend/) | Control plane. |
| [`frontend/`](frontend/) | Console. |
| [`docs/planning/`](docs/planning/README.md) | Epics and user stories. The first epic is [E1, the free-tier runner](docs/planning/epics/E1-free-tier-runner.md). |
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | Production architecture. |
| [`design/site/`](design/site/) | Marketing page source. It is not this product. |

## Local

Control plane:

```
cd backend
cargo run
```

Console:

```
cd frontend
bun install
bun run dev
```
