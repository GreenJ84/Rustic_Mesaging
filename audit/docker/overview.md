# Docker and Provisioning Audit

## Current Topology

Compose defines YugabyteDB, the Rocket API, and an Nginx-served Yew client on separate frontend/backend networks. Images use multi-stage builds, the database is not published by Compose, and Nginx provides a same-origin `/api` gateway. `docker compose config --quiet` succeeds, with an obsolete top-level `version` warning.

## Release Blocker

### Normal startup resets persistent data — P0

The server command waits for PostgreSQL readiness, then runs:

```sh
diesel database reset && diesel migration run
```

`reset` drops/recreates the database and the migration history loads development fixtures. This defeats the mounted `data` volume and makes every API restart destructive.

**Fix:** run only forward migrations in a one-shot migration service or release job. Start the API only after that job succeeds. Put reset/seed behind an explicit development profile and never in the production command.

## Image Findings

### Build context can include secrets — P0

There is no `.dockerignore`. The server context includes tracked `.env` files and uses `COPY . .`; even though the final stage copies selected artifacts, secrets enter the build context and builder layers/cache. Build args are also promoted to `ENV`, which can persist sensitive values.

Add per-context `.dockerignore` files covering `.env*`, `target/`, `.git/`, IDE files, `dist/`, logs, and test data. Never pass secrets through `ARG`/`ENV` during build; use runtime secrets/configuration. Docker's current build guidance recommends `.dockerignore`, non-root runtime users, multi-stage builds, and secret mounts: <https://docs.docker.com/guides/lab-building-images/>.

### Server runtime is a build environment — P1

The Ubuntu runtime installs curl, compilers, rustup, Cargo, and Diesel CLI as root. This produces a large attack surface and performs an unauthenticated-at-image-level network bootstrap during build. Build Diesel CLI in a pinned builder stage or use a dedicated migration binary/image; copy only the API binary, migrations, CA certificates, and required PostgreSQL runtime libraries into a minimal final image.

### Containers run as root — P1

Neither final image declares `USER`. Create fixed unprivileged users, use read-only filesystems where possible, set writable tmpfs locations, drop Linux capabilities, and enable `no-new-privileges`. If binding port 80 requires privileges, have Nginx listen on an unprivileged container port and map the host port.

### Builds are not reproducible — P1

- Base images use mutable tags and no digest.
- Client installs unversioned Trunk and uses Git Yew dependencies.
- Server and client build from subdirectory contexts instead of the committed workspace lockfile; `server/Cargo.lock` is currently untracked and can drift from root `Cargo.lock`.
- Rust builder versions differ (1.80 and 1.81), with no `rust-toolchain.toml`.

Build both crates from the repository root with `cargo build --locked`, pin Trunk/toolchain versions, and pin reviewed base image digests. Add SBOM/provenance and image scanning in CI.

## Compose and Runtime Findings

### Readiness and recovery are incomplete — P1

`depends_on` expresses ordering, not health. Yugabyte has no health check; server uses an unbounded shell loop; client can start before API readiness; server/client have no restart policy or health check. Add bounded health checks and `condition: service_healthy` where supported. Expose `/health/live` and `/health/ready`; readiness should test DB access without leaking details.

### WebSocket proxying is missing — P1

Nginx proxies HTTP but does not set `proxy_http_version 1.1`, `Upgrade`, and `Connection` headers. Add a specific socket location and timeouts when the client feature is implemented. Authenticate before upgrade and use TLS (`wss`) in deployed environments.

### Portability and scaling — P2

Fixed `container_name` values prevent project-scoped replicas and collide with helper scripts. Remove them and use service DNS names. The local bind-mounted `./data` volume is host-specific; document ownership/backup or use a named volume. Add resource limits, log rotation, graceful stop periods, and explicit profiles (`dev`, `test`, `prod`). Remove obsolete `version`.

### Browser security — P1

Nginx lacks TLS configuration, HSTS guidance, CSP, nosniff, referrer/permissions policy, cache policy for hashed assets, and request-size/time limits. Add hardened response headers and production TLS at Nginx or a documented trusted ingress. Avoid permissive Rocket CORS when the browser uses same-origin `/api`.

## Target Provisioning Flow

1. Build API/client from the workspace root using the committed lockfile and pinned toolchain.
2. Scan dependencies and images; produce SBOM and signed provenance.
3. Inject secrets at runtime through the deployment platform, not files baked into contexts.
4. Start database and wait for health.
5. Run a one-shot, non-destructive migration job.
6. Start the non-root API and wait for readiness.
7. Start Nginx with security headers, WebSocket support, and TLS/ingress integration.
8. Verify persistence by creating a sentinel, restarting every service, and confirming it remains.

## Acceptance Checklist

- Restarting any service never resets or seeds data.
- No secret appears in Git, build context, image history, environment dump, or logs.
- Containers run non-root with minimal packages and pinned images.
- Health checks fail correctly when DB/API dependencies are unavailable.
- HTTP and WebSocket traffic work through the public gateway over TLS.
- Clean-machine `docker compose up --build` is reproducible with `--locked` dependencies.
