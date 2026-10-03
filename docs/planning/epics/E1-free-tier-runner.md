# E1. Free-tier build runner

A free build is a Kubernetes Job. The pod is non-root, uses a user namespace, runs daemonless rootless BuildKit, and is deleted shortly after it exits.

Depends on: nothing. This epic is first.

## E1-S1. Render one Job from a build request

As a platform operator, I want a single command that turns a repo, git ref, Dockerfile path, and image name into a Job manifest, so that every free build uses the same contract.

Acceptance:

- The command refuses a non-`https` repo URL, a URL with embedded credentials, a `..` Dockerfile path, and an image reference outside the allowed character set.
- The checkout is `git clone --depth 1 --branch <ref>` in exec form.
- The build container command is `buildctl-daemonless.sh`, then `build` with the Dockerfile frontend.
- The image is `moby/buildkit` rootless, pinned to a release tag.

## E1-S2. Isolate the pod from the node

As a platform operator, I want the build pod to be an unprivileged user on the host kernel, so that a breakout inside BuildKit does not start as host root.

Acceptance:

- `hostUsers: false` on Kubernetes 1.30 or newer.
- `runAsNonRoot: true`, uid and gid 1000, `automountServiceAccountToken: false`.
- `allowPrivilegeEscalation: false` and all capabilities dropped.
- Seccomp and AppArmor are `Unconfined` only because rootless BuildKit needs `clone` and `unshare`.
- The pod does not mount a service-account token.

## E1-S3. Stop a run at the free ceiling

As a platform operator, I want a stalled or oversized build to die on its own, so that one free tenant cannot hold a node.

Acceptance:

- `activeDeadlineSeconds` is 600.
- `backoffLimit` is 0.
- `ttlSecondsAfterFinished` is 120.
- CPU limit is `2000m`. Memory limit is `4Gi`.
- Ephemeral storage limit is `30Gi`.
- The workspace emptyDir cap is `14Gi` and the BuildKit state emptyDir cap is `15Gi`. With the 128Mi run volume, the sized volumes stay under the 30Gi pod limit.

## E1-S4. Apply the runner namespace once

As a platform operator, I want the namespace, BuildKit config, and default network policy applied before any Job, so that a Job cannot start in an open network.

Acceptance:

- Namespace `dockworker-runners` enforces the baseline Pod Security profile and warns on restricted.
- BuildKit runs with the OCI worker and process sandbox disabled, which rootless mode needs inside this pod.
- Garbage collection keeps at most 15 GB, inside the 15Gi state volume.
- A Job without the runner labels is not selected by the build network policy.

## E1-S5. Prove a real build on a cluster

As a platform operator, I want one public sample repository to build and push on the cluster, so that the manifest is known to run.

Acceptance:

- The sample Job reaches `Succeeded`.
- The pushed image has a digest.
- A second run of the same sample imports the registry cache from E2.
- The pod is gone within 120 seconds of completion.
- Logs from the run show `buildctl-daemonless.sh` starting BuildKit, not `buildkitd` rejecting `-frontend`.
