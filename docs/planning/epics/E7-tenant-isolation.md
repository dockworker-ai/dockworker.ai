# E7. Tenant isolation

Free build capacity will be probed. Isolation is part of the runner, not a later hardening pass.

Depends on: E1. Egress tightening can ship before the console.

## E7-S1. Block cloud metadata and cluster networks

As a platform operator, I want the build pod to have no path to node credentials or cluster-internal services.

Acceptance:

- The network policy selects pods labeled `app.kubernetes.io/component: build-runner`.
- Ingress is empty.
- Egress to `10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`, and `169.254.169.254/32` is denied.
- DNS to kube-dns on UDP and TCP 53 is allowed.
- A build that tries to read the cloud metadata address fails.

## E7-S2. Narrow public egress to build destinations

As a platform operator, I want outbound traffic limited to registries and package mirrors, so that a free build cannot use port 443 as an open proxy.

Acceptance:

- Allowed HTTPS destinations are an explicit list: the dockworker registry, and the package and base-image mirrors the tier supports.
- SMTP, SSH, and non-listed ports are denied.
- A test build that connects to an unlisted host on port 443 fails.
- Adding a mirror is a reviewed change to that list.

## E7-S3. Keep cache and credentials per image prefix

As a customer, I want my cache and push credential unable to overwrite another customer's image.

Acceptance:

- The free push credential can push only under the free prefix.
- Cache refs are derived from the image name and are not a single shared tag.
- A build for one image name cannot export cache onto another image name.
- The Job does not receive another tenant's Docker config.

## E7-S4. Reject abusive builds before they run

As a platform operator, I want obvious abuse to consume no cluster time.

Acceptance:

- Invalid repo URLs, refs, and Dockerfile paths never create a Job.
- More than one concurrent free build never creates a second Job.
- A build over the minute budget never creates a Job.
- Repeated rejected submits from one account are rate limited.
