# Server Audit: Rocket and Rust Practices

## Rocket Practices

### Use guards as policy proofs

The JWT `FromRequest` guard is a good starting point, but it proves only token validity. Introduce guards that combine route parameters, managed state, and persisted relationships. Rocket can cache request-local authorization work and only dispatch handlers after guards pass. This is safer than scattered route `if` statements and follows Rocket's guard model: <https://rocket.rs/guide/requests/#request-guards>.

Recommended route shape:

```rust
#[post("/<server_id>/channels", data = "<input>")]
async fn create_channel(
    owner: ServerOwner,
    input: Json<CreateChannel>,
    services: &State<AppServices>,
) -> ApiResult<Json<ChannelView>>
```

`CreateChannel` contains only mutable channel attributes. The owner and server identity come from the guard/path, not the body.

### Centralize managed state and configuration

Rocket currently manages an `Arc<Pool<...>>` and `Arc<WebSocketManager>`, while JWT/admin secrets are read from the process environment on each request. Load and validate a typed `AppConfig` once during ignition, manage it through Rocket, and fail startup with a clear configuration error. Rocket managed state is thread-safe and type-checked: <https://rocket.rs/guide/master/state/>.

Create one `build_rocket(config, services)` function usable by production and tests. Keep `#[launch]` as a thin wrapper. This enables test doubles and avoids environment-coupled unit tests.

### Use coherent responders and errors

Routes repeatedly construct `status::Custom<T>` and inconsistently map missing rows to 500, 424, or 404. Define `ApiError { code, message }` implementing `Responder`, with variants for unauthenticated, forbidden, validation, not found, conflict, and internal errors. Log internal causes with request IDs but return stable public messages.

### Respect async/blocking boundaries

Synchronous Diesel in normal synchronous Rocket handlers is acceptable when Rocket schedules it on blocking workers. It is unsafe for throughput inside long-lived async WebSocket loops. Use `spawn_blocking` or a dedicated data-access executor, keep pool leases short, and never hold a Tokio mutex while awaiting network I/O.

## Rust Practices

### Remove panic paths from request handling — P1

The server has 38 `unwrap`/`expect` calls. High-risk examples include WebSocket parsing/inserts, report database acquisition/querying, membership lookups, and serialization. Convert expected operational failures into typed errors. Reserve `expect` for startup invariants with actionable messages.

`ChannelService::has_authorization` and `PostService::has_authorization` unwrap both the query result and optional row, so a missing ID can panic instead of returning `404`. Policy helpers should return `Result<AuthorizationDecision, DomainError>` and preserve “not found” versus “forbidden” behavior without leaking resource existence where privacy requires it.

### Make impossible states unrepresentable

- Use newtypes such as `MemberId`, `ServerId`, and `ChatId` where identity mix-ups are plausible.
- Separate persistence rows from API inputs and views.
- Do not derive `Serialize` for secret-bearing entities.
- Represent authorization as values (`ServerOwner`) instead of booleans.
- Return `Result` from `has_membership` instead of collapsing DB errors through `unwrap` into process failures.

### Tighten dependency and toolchain policy — P1

Direct dependencies use wildcard versions and Git Yew sources. Replace `*` with reviewed semver ranges, pin Git revisions or use released crates, commit one workspace lockfile, and remove the separate untracked `server/Cargo.lock` workflow. Add `rust-toolchain.toml` and pin builder versions consistently.

### Restore quality gates — P2

`cargo fmt --all -- --check` currently fails across the tree. Strict Clippy fails in every crate; server output includes deprecated Diesel annotations, needless returns, identity mappings, and avoidable clones. Crate-wide `#![allow(unused)]` suppresses useful signals.

Adopt this merge gate after one cleanup commit:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo audit
```

Treat generated Diesel schema separately if formatting or lint policy requires it. Do not silence categories globally without a documented rationale.

## Sequence

1. Fix authorization correctness and secret-bearing serialization before refactoring style.
2. Introduce typed config/errors/guards with characterization tests.
3. Shorten DB leases and redesign socket fan-out/backpressure.
4. Pin dependencies, update Diesel attributes, format, and make Clippy clean.
5. Add tracing with request IDs and sanitized fields; define service health/readiness endpoints.
