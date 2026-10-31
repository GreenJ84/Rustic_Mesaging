# Rustic Messaging System Audit

**Audit date:** 2026-10-04  
**Scope:** current working tree, including uncommitted application repairs  
**Verdict:** the workspace compiles and the Yew production bundle builds, but the application is **not ready for untrusted users or persistent production data**.

## Executive Summary

The repository break is fixed. `cargo check --workspace`, `cargo test --workspace --no-run`, `docker compose config --quiet`, and the production `trunk build --release` all complete. The active Rocket `#[launch]` function is restored and the client's `getrandom` WebAssembly configuration works. This is a meaningful improvement over the previous broken baseline.

Security and operational readiness remain the release blockers. Several routes authenticate a JWT but do not bind the mutation to its subject. WebSocket endpoints perform no authentication and trust path and payload identities. Two author checks use `!member_id == author_id`; `!` is integer bitwise negation, not inequality, so those checks normally allow unauthorized edits. Friend-request responses serialize the full `Member`, including its password hash. Populated environment files are still tracked. Finally, normal Compose startup runs `diesel database reset`, destroying persisted data.

## Risk Register

| Priority | Finding | Impact | Primary location |
|---|---|---|---|
| P0 | WebSockets accept unauthenticated identities and writes | Impersonation, unauthorized reads/writes | `server/src/routes/web_sockets.rs` |
| P0 | Client-controlled owner/author/sender identifiers | Horizontal/vertical privilege escalation | server create routes |
| P0 | Bitwise-negation authorization checks | Unauthorized post/message changes | `post_routes.rs`, `message_routes.rs` |
| P0 | Full `Member` serialized in friend requests | Password-hash and email disclosure | `models/friend_request.rs` |
| P0 | Tracked populated `.env` files | Credential compromise, including Git history | `server/.env*` |
| P0 | Database reset on normal startup | Guaranteed production data loss | `docker-compose.yml` |
| P1 | Eight known dependency vulnerabilities | DoS, log injection, and vulnerable transitive code | `Cargo.lock` |
| P1 | Server reports exposed to any authenticated user | Member email/admin metadata disclosure | `routes/report_routes.rs` |
| P1 | Client treats failed DELETE/HEAD requests as success | Incorrect UI state and authorization confusion | `client/src/utils/api_requests.rs` |
| P1 | No runtime integration, browser, or authorization matrix tests | Regressions remain undetected | test suites |

P0 means release-blocking; P1 means fix before a public beta; P2 means planned hardening or maintainability work.

## Verification Snapshot

- Pass: `cargo check --workspace` (eight deprecated Diesel attribute warnings).
- Pass: `cargo test --workspace --no-run` (all four crate test targets compile).
- Pass: production client build with Trunk and the Docker-equivalent `getrandom` flag.
- Pass with warning: Compose configuration; top-level `version` is obsolete.
- Fail: `cargo fmt --all -- --check` with broad formatting drift.
- Fail: strict workspace Clippy; all crates report findings, including a potentially orphaned server child process.
- Fail: `cargo audit`; **8 vulnerabilities and 12 warnings** across 308 locked dependencies.
- Not run: database-backed tests and full Compose startup because both supported paths reset the configured database.

## Section Summaries

- [Server overview](server/overview.md): good domain separation and a restored Rocket launch path, but incomplete authorization boundaries and brittle error handling.
- [Server security](server/security.md): details the exploitable identity, response-data, JWT, WebSocket, and dependency findings.
- [Rocket and Rust practices](server/rocket-rust.md): recommends typed authorization guards, explicit state/configuration, safe async boundaries, and strict quality gates.
- [Data and testing](server/data-testing.md): covers destructive migrations, fixture placement, transactionality, privacy, and the missing test matrix.
- [Client overview](client/overview.md): functional CSR bundle with substantial completion gaps, including no WebSocket client despite real-time claims.
- [Client security and state](client/security-state.md): covers token storage, request semantics, race-prone effects, and route/context divergence.
- [Yew practices](client/yew-practices.md): proposes typed remote state, one router, focused contexts, and browser-level tests.
- [Docker audit](docker/overview.md): covers data loss, image provenance, root runtime, health checks, secret leakage, and WebSocket proxying.
- [Scripts audit](scripts/overview.md): covers fixed sleeps/names, process leaks, destructive defaults, cleanup, and replacement options.

## Development Plan

1. **Contain immediately:** rotate tracked secrets, remove them from the index/history, disable WebSocket routes or add authenticated membership guards, stop startup resets, replace all client-supplied actor IDs, fix `!=` checks, and return safe DTOs only.
2. **Establish security invariants:** centralize `AuthenticatedUser`, `ServerOwner`, `ServerMember`, `ChatMember`, and resource-owner guards; add negative tests for every mutation and report.
3. **Stabilize dependencies and quality gates:** upgrade the lockfile until RustSec is clean, pin direct dependencies, add a toolchain file, format the tree, and make check/fmt/Clippy/audit/test mandatory in CI.
4. **Complete state and real-time behavior:** implement an authenticated WebSocket client with reconnect/backoff and event reconciliation, or remove real-time claims until complete. Make route IDs the source of truth and model remote state as loading/ready/error.
5. **Harden delivery:** use non-root minimal images, lock and digest dependencies, add health checks, migrate without reset, add security headers/TLS guidance, and test a clean Compose deployment with persistent data.

Production readiness should be reconsidered only after all P0/P1 findings have regression tests and a clean-stack restore/restart exercise succeeds.
