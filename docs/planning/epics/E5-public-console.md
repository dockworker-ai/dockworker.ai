# E5. Public console

`https://dockworker.ai` becomes the product. A visitor can read the free tier. A signed-in customer can run a build and watch it.

Depends on: E3, E4.

## E5-S1. Explain the free tier on the home page

As a visitor, I want the home page to state what a free build includes, so that I know the limits before I sign in.

Acceptance:

- The page states 200 minutes, 1 concurrent build, 10 minutes, 2 vCPU, 4 GiB, and `linux/amd64`.
- The page states that the image is pushed to the free registry prefix and is subject to the retention rule from E2.
- The primary action is sign-in. There is no claim that a build is running until E1-S5 has succeeded.

## E5-S2. Show quota and build history

As a customer, I want a signed-in page with minutes remaining and my builds.

Acceptance:

- The page loads quota and the build list from the API.
- An empty history tells the customer how to start a build.
- Signing out returns to the public home page and drops the session.

## E5-S3. Watch one build

As a customer, I want a build page with status, digest, and a live log.

Acceptance:

- The page follows status until the build is terminal.
- On success it shows the image name and digest.
- On failure or timeout it shows the log and the status.
- The page requests only that customer's build.
