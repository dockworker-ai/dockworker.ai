# E4. Accounts and free quota

A free account is a person plus a ceiling. The control plane rejects work that would pass the ceiling before it creates a Job.

Depends on: E3.

## E4-S1. Sign in

As a customer, I want to sign in and receive a session, so that builds and quota belong to me.

Acceptance:

- Sign-in creates an account on first success and reuses it after that.
- The session is required on `POST /builds`, `GET /builds`, and `GET /user/quota`.
- A missing or expired session gets an unauthorized response and creates no Job.

## E4-S2. Show the free ceiling

As a customer, I want to see minutes remaining, the concurrency cap, and the per-build time cap.

Acceptance:

- A new account starts at 200 minutes, 1 concurrent build, and a 10 minute cap.
- `GET /user/quota` returns remaining minutes, the monthly allowance, and the reset time.
- The console and the API return the same numbers.

## E4-S3. Enforce one build at a time

As a platform operator, I want a second free build to wait or be rejected while the first is running.

Acceptance:

- A free account with a `QUEUED` or `RUNNING` build cannot start another.
- The response explains the concurrency cap.
- No second Job is created.

## E4-S4. Enforce the minute budget

As a platform operator, I want a build that would exceed the remaining minutes to be refused.

Acceptance:

- Submit is rejected when remaining minutes are 0.
- A finished build subtracts its runtime, rounded up to the next minute, and never below 1 minute for a build that started.
- A timed-out build subtracts 10 minutes.
- A rejected submit subtracts nothing.
- The counter resets on the account's monthly boundary.
