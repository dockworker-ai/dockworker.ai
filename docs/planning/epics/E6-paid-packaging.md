# E6. Paid packaging

Paid packaging is the same runner with a higher ceiling and a bill. It starts after a free build can succeed end to end.

Depends on: E1-S5, E4, E5.

## E6-S1. Define paid limits

As a customer, I want a published paid tier, so that I know what changes when I pay.

Acceptance:

- The public pricing table lists minutes, concurrency, max build time, architectures, and price.
- Free limits in that table match the free-tier contract in the planning index.
- Paid builds still run non-root, with user namespaces, and without a service-account token.

## E6-S2. Raise the ceiling on a paid account

As a paying customer, I want more than one concurrent build and a longer deadline.

Acceptance:

- The Job renderer accepts a tier and applies that tier's deadline, CPU, memory, and storage caps.
- A paid account can run its concurrency cap and no more.
- A free account cannot request paid caps by setting a field on `POST /builds`.

## E6-S3. Bill for the paid tier

As a paying customer, I want to subscribe and see that the paid ceiling is active.

Acceptance:

- Checkout creates or updates a subscription and flips the account tier only after payment is confirmed.
- A failed or cancelled payment leaves the account on the free ceiling.
- The quota endpoint shows the paid allowance.

## E6-S4. Offer multi-arch on the paid tier

As a paying customer, I want `linux/amd64` and `linux/arm64` from one build.

Acceptance:

- The paid Job requests both architectures without QEMU for the native one.
- The free tier still builds `linux/amd64` only.
- The manifest lists both digests.
