# dockworker.ai

Commercial OCI packaging with a free tier.

Planning for epics and user stories: [docs/planning/README.md](docs/planning/README.md).

The first epic is the free-tier build runner. Accounts, the console, and billing come after a build can push an image.

## Figma Make

This repository is the Figma Make working copy of the current dockworker.ai page: Deterministic Container Packaging & OCI Build Engine, Space Grotesk, with the header, hero, feature cards (`#matrix`), architecture section (`#zero-dockerfile`), and footer. Connect Make to GitHub here: `dockworker-ai/dockworker.ai`. `.figma/make` is already in the tree.

`public/fonts/*.woff2` is not in the repo yet. The checkout zip has twelve self-hosted files (Inter, JetBrains Mono, and Space Grotesk, weights 400–700). The GitHub contents API UTF-8-encodes file bodies, so those binaries have to be pushed with git. `.gitattributes` marks `*.woff2` as Git LFS, and Make reads git blobs, so add the real font bytes with the LFS clean filter off:

```
git clone git@github.com:dockworker-ai/dockworker.ai.git
# copy public/fonts from the checkout zip
git -c filter.lfs.process= -c filter.lfs.required=false add public/fonts
git commit -m "Add self-hosted woff2 fonts"
git push
```

## Local

```
pnpm install
pnpm dev
```
