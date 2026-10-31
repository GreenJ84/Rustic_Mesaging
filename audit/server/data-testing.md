# Server Audit: Data, Migrations, and Testing

## Data Integrity

### Production startup destroys the database — P0

Compose runs `diesel database reset && diesel migration run` before every server start. `reset` drops and recreates the database, then the migration history installs fixture accounts and data. A persistent volume does not protect data from this command.

Replace startup with an explicit, non-destructive migration step (`diesel migration run`). Make reset/seed opt-in development commands guarded by environment and confirmation. Test restart behavior against a sentinel record before any deployment.

### Fixtures are mixed with schema history — P1

`2024-09-19-150030_create_filler_data/up.sql` contains a large seeded dataset, repeated password hashes, and admin accounts. Its `down.sql` does not provide a meaningful rollback. Schema migrations must be deterministic, reversible where practical, and free of development credentials. Move fixtures to `server/fixtures/` or a dedicated seed command disabled in production.

### Multi-row operations need transactions — P1

Server creation is transactional; chat creation, friend acceptance, and related multi-step flows are not consistently atomic. A partial failure can leave an orphan chat, incomplete membership, or friendship/request inconsistency. Wrap each domain use case in one transaction and enforce critical invariants with foreign keys, unique/check constraints, and well-reviewed triggers.

### Privacy and export correctness — P1

Membership reports expose emails/admin state without authorization. CSV conversion also sets `joined_at` to `Default::default()` instead of the report timestamp (`models/report.rs:47-56`), producing incorrect exports. Define field-level privacy and add exact CSV tests, including formula-prefix escaping and content-disposition behavior.

## Existing Tests

There are four server tests in `server/src/tests.rs`. They use Rocket's blocking local client and exercise registration/login basics. Test targets compile, but runtime tests were not executed during this audit because the supported setup resets the configured database.

Current shortcomings:

- Tests share seeded state and fixed usernames, so they are not isolated or safely repeatable.
- There are no authorization tests for the high-risk route families.
- No WebSocket, report, migration, restart, concurrency, or failure-path tests exist.
- The client and helper crates contain no tests.
- No CI workflow enforces compilation, format, lint, audit, or tests.

## Required Test Matrix

For every protected resource, test at least: unauthenticated, unrelated authenticated user, member, owner/author, admin, missing resource, malformed input, and database failure. Prioritize:

1. WebSocket upgrade and frame identity/membership checks.
2. Server/channel/post/message creation and mutation subject binding.
3. Friend-request create/read/accept/deny actor rules.
4. Report owner/admin checks and response field minimization.
5. Password/hash absence from every serialized response.
6. Owner departure/deletion, duplicate joins, and transaction rollback.
7. JWT expiry, malformed scheme, wrong issuer/audience, role changes, and rate limiting.

## Test Infrastructure Plan

- Create a separate ephemeral test database with a unique name per run; never point tests at development or production URLs.
- Run migrations once, create fixtures through builders, and wrap each test in a rollback transaction where possible.
- Split unit tests (pure validation/policy), Rocket integration tests (HTTP contracts), and browser/socket end-to-end tests.
- Add migration tests that apply from empty, roll forward across versions, and verify non-destructive restart.
- Add property tests for identity ordering and membership invariants, plus bounded concurrency tests for WebSocket fan-out.
- Publish coverage for trend visibility, but gate on critical authorization scenarios rather than a percentage alone.

## Exit Criteria

The data layer is release-ready when normal startup never drops data, fixture credentials cannot enter production, migrations and rollback strategy are documented, all actor/resource combinations are covered, and tests can run repeatedly without shared-state failures.
