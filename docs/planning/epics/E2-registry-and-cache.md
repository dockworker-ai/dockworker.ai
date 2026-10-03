# E2. Registry and cache

The image and the layer cache outlive the pod. Local `emptyDir` cache is not the free-tier cache, because that disk disappears when the Job ends.

Depends on: E1.

## E2-S1. Push the image

As a customer, I want the finished image pushed to the free registry prefix, so that I can pull a digest after the pod is gone.

Acceptance:

- Build output is `type=image,name=<image>,push=true`.
- The image name is under the free prefix on `registry.dockworker.ai`.
- The Job mounts a Docker config from a registry credential that can push only that prefix.
- The credential value is not stored in git, in the Job env, or in this repository.
- The control plane records the digest after the push.

## E2-S2. Reuse layers across runs

As a customer, I want a second build of the same project to reuse layers, so that a small change does not start from an empty cache.

Acceptance:

- Export cache is `type=registry` with `mode=max`.
- Import cache uses the same ref.
- For image `registry.dockworker.ai/free/<name>:<tag>`, the cache ref is `registry.dockworker.ai/free/cache/<name>:cache`.
- Two different image names do not share a cache ref.
- A cold build and a warm build of the sample both succeed, and the warm build pulls cache from the registry.

## E2-S3. Keep free images bounded

As a platform operator, I want free images and cache tags to expire, so that the free prefix cannot grow without a limit.

Acceptance:

- Free image tags and cache tags have a documented retention window.
- Retention deletes tags. It does not delete a digest that a paid account has pinned.
- A customer can read the retention rule before the first build.
