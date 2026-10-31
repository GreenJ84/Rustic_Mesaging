# Server Audit: Security

## Release-Blocking Findings

### SEC-01: WebSocket authentication and authorization are absent — P0

`GET /channel/ws/<channel_id>/<user_id>` and `/message/ws/<sender_id>/<receiver_id>` have no JWT guard. They trust URL identities, do not verify channel/chat membership or friendship, and persist `NewPost`/`NewMessage` values supplied by the peer. Any reachable client can impersonate a user, subscribe to guessed rooms, and write records for another account.

**Fix:** authenticate the upgrade, derive the user ID only from verified claims, resolve the room server-side, authorize membership, ignore actor IDs in frames, enforce payload size/rate limits, and close with policy-specific codes on violations.

### SEC-02: Mutations trust client-owned identity fields — P0

- Server creation ignores `_token` and persists form `owner_id` (`server_routes.rs:18-32`).
- Channel creation ignores `_token`; any account can create under any server (`channel_routes.rs:26-35`).
- Channel deletion uses a server-membership predicate while claiming to require an owner, so any member can delete a channel (`channel_routes.rs:107-123`, `channel_service.rs:53-66`).
- Post creation ignores `_token` and trusts `author_id`/`channel_id` (`post_routes.rs:26-35`).
- Message creation checks membership in submitted `chat_id` but persists submitted `sender_id` (`message_routes.rs:22-35`).
- Friend-request creation/accept/deny do not bind the token to sender or receiver.
- Direct `POST /friend/<id>` bypasses the request/accept workflow.
- Any authenticated user can join a guessed chat ID.

**Fix:** remove actor/owner fields from public forms. Derive them from the authenticated principal. Load the target resource and authorize against persisted relationships before mutation.

### SEC-03: Author checks use bitwise negation — P0

`if !token.claims.member_id == post.author_id()` and the equivalent message check apply bitwise NOT to an integer. They do not mean `member_id != author_id` and ordinarily permit unauthorized changes.

**Fix:** use `!=`, add owner-or-admin policy helpers, and add two-user negative tests for every update/delete route. Search and prohibit similar ad hoc comparisons.

### SEC-04: Password hashes are returned — P0

`Member` derives `Serialize` and contains `password` and `email` (`models/member.rs:63-80`). `RequestDetailed` embeds `Member`, and `get_member_requests` selects full members. `GET /member/requests` therefore serializes password hashes and private account fields.

**Fix:** make persistence entities non-serializable. Replace `Member` with `MemberShort` in all relationship responses. Add serialization snapshot tests asserting that `password`, `email`, and `is_admin` are absent where not explicitly required.

### SEC-05: Populated secrets are tracked — P0

`.gitignore` now ignores environment files, but `server/.env` and `server/.env.prod` remain in the Git index and contain populated configuration. Ignoring a tracked file does not remove it or its history.

**Fix:** rotate database, JWT, and admin credentials; replace tracked files with `.env.example` templates; remove secrets from the index; inspect/purge Git history and remote caches; enable secret scanning. Do not reuse the seeded admin passkey.

## High-Priority Findings

### Reports expose private membership data — P1

Any valid JWT can request any server membership report. No server membership/owner/admin check is performed, and rows include email and admin status. Restrict detailed/CSV reports to an explicit owner/admin policy and minimize fields.

### CORS is overbroad — P1

`AllowedOrigins::all()`, `AllowedHeaders::all()`, and credentials are enabled together. Configure an environment-specific allowlist and only required methods/headers. Same-origin Nginx `/api` deployments generally do not need permissive production CORS.

### JWT lifecycle is minimal — P1

Claims contain only member ID, admin flag, and one-day expiry. Authorization accepts a permissively stripped `Bearer` prefix; no issuer, audience, issued-at, token ID, revocation, or key rotation exists. Admin status is frozen into the token. Parse the scheme exactly, validate issuer/audience, use short access tokens plus a deliberate refresh/revocation design, and re-check sensitive roles server-side.

Registration supports an `Admin-Key` header. Treat this as a provisioning operation, not a public registration option; separate or disable it in production and rate-limit login/registration.

### Dependency advisories — P1

The refreshed 2026-10-04 RustSec scan reports 8 vulnerabilities: `bytes 1.7.2`, `diesel 2.2.4` (two), `h2 0.3.26`, `idna 0.5.0`, `ring 0.17.8`, `time 0.3.36`, and `tracing-subscriber 0.3.18`. It also reports 12 maintenance/unsound/yanked warnings. Upgrade direct constraints and transitive parents, then require a clean `cargo audit` in CI. Diesel's SQLite-specific advisories are not directly exercised by the PostgreSQL build, but the command-injection advisory and stale graph still require remediation.

## Additional Hardening

- Return generic client errors and structured server logs; do not expose raw database messages.
- Add request/body limits, login throttling, security event logging, and per-socket message limits.
- Use `403 Forbidden` for authenticated-but-disallowed actions; reserve `401` for authentication failure.
- Define privacy rules for email and activity exports, including audit logging and CSV formula-injection escaping.
- Threat-model ID enumeration, account deletion, owner departure, and last-owner invariants.
