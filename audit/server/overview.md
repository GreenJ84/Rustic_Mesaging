# Server Audit: Architecture and Completion

## Current Shape

The `server` crate is a Rocket 0.5.1 binary backed by synchronous Diesel/r2d2 and YugabyteDB's PostgreSQL interface. Code is divided into `routes/`, `service/`, `models/`, `db.rs`, and generated `schema.rs`; migrations define both schema and large fixture data. Rocket manages the database pool and in-memory WebSocket manager.

The repaired `#[launch] fn rocket()` in `server/src/main.rs:39` compiles and all server test targets build. Route/service separation is understandable, DTOs such as `MemberSafe` and `MemberShort` already exist, and server creation uses a transaction. These are useful foundations.

## Completion Assessment

The HTTP CRUD surface is broad: authentication, profiles, friendships, chats, messages, servers, channels, posts, reports, and WebSockets. However, breadth exceeds reliable completion:

- Real-time server routes exist, but the Yew client has no WebSocket code. README claims about authenticated real-time sessions do not match implementation.
- Several `PUT` routes intentionally return `501 Not Implemented`.
- Four server tests cover unauthorized member access, registration, duplicate registration, and login only.
- The normal test/startup path depends on a destructive database reset and seeded credentials.
- Error behavior varies by route; database errors are often returned to clients, while several `unwrap`/`expect` paths can terminate request handling.
- Message lookup renders the authenticated requester as the sender instead of the persisted message sender, corrupting response identity (`message_routes.rs:66-74`).
- `#![allow(unused)]` at crate level hides dead imports/code, including a commented Socket.IO prototype.

## Architecture Findings

### Authorization is route-local and inconsistent — P0

Authentication is represented by a Rocket request guard, but resource authorization is manually repeated. Some routes check membership, some ignore the token (`_token`), and others authorize against IDs supplied in the same untrusted form. This makes policy omissions routine.

Adopt typed policy witnesses:

- `AuthenticatedUser` derives identity from the token.
- `ServerMember` and `ServerOwner` load the referenced resource and verify access.
- `ChatMember` and `ResourceAuthor` bind authorization to the persisted row.
- Service mutations accept the authenticated actor or already-authorized resource, never an actor ID from a form.

Rocket's request guards are designed to prevent handlers from running until validation succeeds: <https://rocket.rs/guide/requests/#request-guards>.

### Database work and domain policy are coupled — P1

Routes fetch connections, perform policy checks, map database errors, and shape responses. Services mostly expose raw CRUD. Move multi-step use cases into transaction-scoped application services, for example `create_server(actor, input)` and `accept_friend_request(actor, request_id)`. Return a typed domain error that one responder maps to consistent HTTP statuses.

### WebSocket design can exhaust shared resources — P1

Each socket obtains and retains a pooled synchronous database connection for the connection lifetime (`web_sockets.rs:27-29`, `71-73`). Broadcast holds the global connection mutex while awaiting every client send (`models/web_socket.rs:69-73`). One slow client can serialize all sends, while enough idle sockets can exhaust the DB pool.

Authenticate the upgrade, verify membership, acquire DB connections only per operation, perform blocking Diesel work outside the async task, and give each socket a bounded outbound channel with timeout/backpressure policy. The unused broadcast sender should be removed or become the actual fan-out mechanism.

## Recommended Target

Keep Rocket as the composition layer, but introduce `authz`, `domain`, and `api` boundaries. Use explicit input DTOs without identity/ownership fields; safe output DTOs without secrets; transactional services for multi-row changes; and typed errors. This allows route handlers to become small, reviewable adapters and makes authorization testable independently of response formatting.
