# Client Audit: Security and State Handling

## Security Findings

### JWT persistence increases XSS impact — P1

The bearer token is stored in `localStorage` and read by client code. Any script executing in the origin can extract it, and it survives browser restarts. The application has no visible Content Security Policy or other response hardening in Nginx.

Preferred design: a `Secure`, `HttpOnly`, `SameSite` cookie with a deliberate CSRF strategy. If bearer storage must remain temporarily, keep access tokens short-lived, prefer memory storage, use refresh rotation, clear on logout/401, enforce a restrictive CSP, and avoid third-party scripts. Client storage is a risk reduction decision, not an authorization boundary.

### Failed operations are reported as success — P1

`api_delete` ignores HTTP status (`api_requests.rs:135-150`), while `api_head` does not prove authorization. Components then update reducer state as if deletion/leave succeeded. Correct the transport layer first; every mutation should update local state only after a 2xx response or reconcile from a server-confirmed result.

### CSV creation uses a data URL — P2

Report content is embedded directly into `data:text/csv` without explicit percent encoding. Use the Blob/Object URL APIs, revoke the URL, and ensure the server escapes spreadsheet formulas beginning with `=`, `+`, `-`, or `@`. Server authorization remains mandatory because hiding a button is not access control.

### Browser hardening is absent — P1

Add production headers at Nginx: a restrictive CSP, `X-Content-Type-Options: nosniff`, `Referrer-Policy`, `Permissions-Policy`, and framing protection (`frame-ancestors`). Deploy only over HTTPS and ensure WebSocket URLs use `wss://` outside local development.

## State Findings

### Remote state lacks lifecycle information — P1

Contexts store default domain objects and collections but not loading, error, stale, or request-generation state. `Chat::default()` acts as a sentinel. This makes “not loaded,” “missing,” and “empty” indistinguishable and encourages `unwrap`.

Use a typed state:

```rust
enum RemoteData<T> {
    NotAsked,
    Loading,
    Ready(T),
    Error(ApiError),
}
```

Reducers should model request started/succeeded/failed, retain the requested resource ID, and reject stale completions.

### Detached effects can race — P1

Context effects use `spawn_local` without cancellation or generation checks. If selection changes while a request is in flight, an older response can overwrite the newer entity's thread/channels. Track request IDs or selected IDs and verify them before dispatch; use effect teardown to invalidate obsolete work.

### Context updates cause broad rerenders — P2

Each provider combines collections, selection, and thread data. Any update changes the shared context and rerenders all consumers. Split session/account, chat index, selected chat/thread, server index, and selected server/channel state. Use `use_reducer_eq`, memoized selectors/components, and props for local parent-child data.

### Nested routers create competing history providers — P1

`Main` creates a `BrowserRouter`, and `Home` creates another. Keep one router at the application root and nest route enums/switches under it. Authentication redirects should use the same navigator/history source.

### Error handling is mostly logging — P2

Requests frequently log errors and leave stale/default UI with no retry path. Introduce a shared error presentation model, inline form errors, retry actions, disabled/pending mutation buttons, and accessible status announcements.

## State Transition Plan

1. Build and test the typed API client; make 401/403/error semantics reliable.
2. Replace default sentinels with `RemoteData` and explicit selected IDs.
3. Make route params canonical and reject stale async completions.
4. Split providers around feature boundaries and update frequency.
5. Add socket events as reducer actions so HTTP and WebSocket updates share one reconciliation path.
6. Add browser tests for reload, direct URL, two fast selections, failed delete, logout, expired token, and reconnect.
