# E3. Build control plane

The runner renders a Job. The control plane is what a customer calls. It accepts a build, creates the Job, and reports status and logs.

Depends on: E1, E2.

## E3-S1. Submit a build

As a customer, I want to submit a repository, git ref, Dockerfile path, and image name, so that I do not write a Kubernetes manifest.

Acceptance:

- `POST /builds` validates the same inputs the Job renderer validates.
- The response is a build id and status `QUEUED`.
- The service creates one Job in `dockworker-runners` and stores the Job name on the build.
- A second request with the same id does not create a second Job.

## E3-S2. Read build status

As a customer, I want to see whether a build is queued, running, succeeded, failed, or timed out.

Acceptance:

- `GET /builds/{id}` returns that status.
- A Job that hits the 600 second deadline is `TIMEOUT`.
- A Job that exits 0 after a push is `SUCCEEDED` and includes the image digest.
- A Job that exits non-zero is `FAILED` and includes the exit code.
- Status for one account is not readable by another account.

## E3-S3. Stream logs

As a customer, I want the build log while the Job is running, so that I can see a failing Dockerfile step.

Acceptance:

- `GET /builds/{id}/logs` streams plain-progress BuildKit output.
- The stream ends when the Job reaches a terminal status.
- Log lines are ordered.
- Logs for one account are not readable by another account.

## E3-S4. List my builds

As a customer, I want a list of my builds with image name, status, and duration.

Acceptance:

- `GET /builds` returns only the caller's builds, newest first.
- Each row includes id, image, status, creation time, and duration when the build has finished.
