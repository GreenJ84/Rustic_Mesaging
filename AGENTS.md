# Repository Guidelines

## Project Structure & Module Organization

This Cargo workspace contains the Rustic Messaging System:

- `server/` contains the Rocket API, WebSocket handling, authentication, Diesel models, and database migrations in `server/migrations/`.
- `client/` contains the Yew WebAssembly frontend. UI pages live under `client/src/views/`, reusable components under `client/src/comps/`, shared state under `client/src/contexts/`, and browser assets under `client/static/`.
- `app_startup/` starts YugabyteDB, resets the local database, launches the API, and serves the client with Trunk.
- `testing_startup/` starts the database and runs the server test suite.
- `docker-compose.yml` and the two Dockerfiles define the containerized stack; `data/` stores the local database volume.

Keep feature code near its owning module and add new migrations rather than editing existing migration history.

## Build, Test, and Development Commands

Run commands from the repository root unless noted otherwise:

```bash
cargo check --workspace                # Type-check all crates
cargo fmt --all -- --check             # Verify Rust formatting
cargo clippy --workspace --all-targets # Run Rust lint checks
cargo run -p app_startup               # Start DB, API, and Trunk locally
cargo run -p testing_startup           # Reset DB and run server tests
docker compose up --build              # Run the full stack
docker compose down                    # Stop the Compose stack
```

Local development requires Docker, Rust, Trunk, and Diesel CLI with PostgreSQL support. Configure `server/.env` and `server/.env.prod` before starting services.

## Coding Style & Naming Conventions

Use Rust 2021 conventions and four-space indentation. Run `cargo fmt` before committing. Use `snake_case` for modules, functions, and variables; `PascalCase` for types and Yew components. Keep client presentation in `views`/`comps`, persistence in server models and migrations, and request or WebSocket behavior in the relevant handlers.

## Testing Guidelines

Server tests are Rust tests in `server/src/tests.rs`, using Rocket’s blocking local client and names such as `prevent_unregistered_user`. Run `cargo test --workspace` for the complete suite; database-backed tests need YugabyteDB, so `cargo run -p testing_startup` is the supported setup path. Add tests for authentication failures, validation, and new API behavior.

## Commit & Pull Request Guidelines

Recent commits use concise, imperative descriptions such as `Update ...`, `Implement ...`, and `Remove ...`. Keep commits focused. Pull requests should explain the behavior change, identify setup or migration changes, list validation commands, and include screenshots or a short recording for visible client changes. Call out required environment variables and database changes.

## Security & Configuration

Never commit populated `.env` files, credentials, JWT secrets, or database dumps. Test both accepted and rejected authentication paths.
