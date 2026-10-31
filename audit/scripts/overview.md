# Startup and Test Automation Audit

## Scope

The repository has two Rust helper binaries:

- `app_startup` runs a Yugabyte container, waits, resets the database, starts Rocket as a child, changes directory, and blocks on `trunk serve`.
- `testing_startup` runs Yugabyte, waits, resets the database, runs server tests, then stops/removes the container.

They compile, but strict Clippy fails in both. These helpers duplicate behavior already represented by Compose and embed destructive lifecycle policy in imperative code.

## Critical Findings

### Database reset is the default — P0

Both helpers invoke `diesel database reset`. A user can accidentally erase whichever database `server/.env` points to. Replace reset with non-destructive migration for ordinary development. Provide a separately named command such as `dev-reset-db` that requires an explicit development URL check and interactive confirmation (with a deliberate CI override).

### Global container and port collisions — P1

Both use the fixed name `yugabyte` and publish five fixed host ports. Runs collide with each other, Compose, and unrelated local services. Use a unique Compose project name and only publish ports actually needed. Prefer `docker compose up -d yugabyte` over reproducing its definition in Rust.

### Fixed sleeps are unreliable — P1

Ten- and five-second sleeps can be too short on a cold machine and waste time on a fast one. Poll `pg_isready` with a timeout, report progress, and fail nonzero with the last diagnostic. Readiness is different from container process startup.

### Child lifecycle is unsafe — P1

`app_startup` spawns `cargo run` and never waits for or terminates the child. Clippy reports `zombie_processes`; closing Trunk or hitting an error can leave Rocket and Yugabyte running. Track all children, forward termination signals, kill/wait in reverse startup order, and run cleanup from a scope guard. Do not perform async/background process orchestration without ownership.

### Failures often return success — P1

Helpers print errors and `return` from `main`, yielding exit code 0. Panics bypass cleanup. `testing_startup::shutdown` prints even success to stderr and cleanup failures do not affect the final status. Return `Result`/`ExitCode`, preserve the original error, run best-effort cleanup, and exit nonzero if setup, test, or cleanup fails.

## Maintainability Findings

- Both scripts assume invocation from the repository root and mutate global current directory. Resolve paths from `CARGO_MANIFEST_DIR` or use each command's `current_dir`.
- Command prerequisites and versions are not checked up front.
- Startup output comments/messages disagree with actual wait durations.
- Test startup runs only the server directory rather than an explicit workspace/test plan.
- No mode separation exists for local development, CI, and production-like smoke tests.
- No logs are captured or summarized when a child fails.
- Repeated `.args(&[...])` calls fail strict Clippy; style drift also fails rustfmt.

## Recommended Replacement

Use Compose as the single service lifecycle definition and a small task runner (`just`, `make`, `cargo-make`, or carefully scoped scripts) as the command interface. Example behavior:

```text
dev-db-up       -> compose up -d yugabyte; wait with timeout
dev-migrate     -> non-destructive migration run
dev             -> ensure DB, migrate, supervise API + Trunk
test            -> create isolated test DB, migrate, run workspace tests, cleanup
dev-reset-db    -> validate local test/dev URL, confirm, reset and seed
smoke-compose   -> clean build, health checks, HTTP/socket smoke tests, restart persistence check
```

If the Rust binaries are retained, share a library for command execution, readiness polling, child supervision, cleanup, and typed errors. Use unique project/container names generated from repository and process identity; never shell-concatenate user-controlled values.

## Implementation Plan

1. Remove reset from `app_startup`; add URL/environment guards to the explicit reset path.
2. Replace raw `docker run` with Compose profiles and health checks.
3. Return nonzero exit codes and include command/context in every error.
4. Add signal-aware cleanup and wait on all child processes.
5. Create isolated test databases and run `cargo test --workspace` with deterministic fixtures.
6. Pin/check required tool versions and document one canonical command set in `README.md` and `AGENTS.md`.
7. Add CI that invokes the same tasks, proving local and automated workflows do not diverge.

## Acceptance Criteria

- Interrupting startup leaves no orphan process/container.
- Repeated and concurrent runs do not collide.
- Slow database startup is handled within a bounded timeout.
- Any failed command yields a nonzero process exit.
- Normal development and tests cannot target or reset production data.
- CI and local workflows use the same migration, build, lint, audit, and test entry points.
