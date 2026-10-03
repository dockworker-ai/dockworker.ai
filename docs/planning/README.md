# dockworker.ai planning

dockworker.ai is a commercial OCI packaging service. A free account can build and publish a small `linux/amd64` image. A paid account buys more minutes, more concurrency, longer runs, and multi-arch builds.

The build runner is the first epic. Accounts, the console, and billing follow a runner that can push an image and reuse cache.

This set is the planning record for the public repository. It describes the product contract. It does not contain credentials, cluster addresses, or customer data.

## Epic order

| ID | Epic | Outcome |
| --- | --- | --- |
| [E1](epics/E1-free-tier-runner.md) | Free-tier build runner | One untrusted build runs to a pushed image, then the pod is gone. |
| [E2](epics/E2-registry-and-cache.md) | Registry and cache | The image and its layer cache live in the dockworker registry. |
| [E3](epics/E3-build-control-plane.md) | Build control plane | A request becomes a Job, a status, and a log. |
| [E4](epics/E4-accounts-and-quota.md) | Accounts and free quota | A person gets a free ceiling and cannot exceed it. |
| [E5](epics/E5-public-console.md) | Public console | dockworker.ai shows remaining minutes, builds, and logs. |
| [E6](epics/E6-paid-packaging.md) | Paid packaging | Paid limits and billing start after a free build succeeds. |
| [E7](epics/E7-tenant-isolation.md) | Tenant isolation | A free build cannot mine, scan, or read another tenant's cache. |

## Free-tier contract

These numbers are the acceptance bar for E1–E4.

| Limit | Free tier |
| --- | --- |
| Architecture | `linux/amd64` |
| Concurrent builds | 1 |
| Monthly minutes | 200 |
| Hard stop | 10 minutes (`activeDeadlineSeconds: 600`) |
| CPU | 2 vCPU |
| Memory | 4 GiB |
| Ephemeral storage | 30 GiB pod limit |
| Workspace volume | 14 GiB |
| BuildKit state volume | 15 GiB |
| Run scratch | 128 MiB memory |
| Image | pushed to the free registry prefix |
| Cache | registry cache, `mode=max`, keyed by image name |
| Pod lifetime after exit | 120 seconds |

## Story shape

Each story has an actor, a result, and acceptance checks. A story is done when every check is true on a real run, not only in a rendered manifest.
