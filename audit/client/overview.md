# Client Audit: Architecture and Completion

## Current Shape

The `client` crate is a Yew CSR application built with Trunk. Views are grouped under `views/`, reusable UI under `comps/`, remote/domain models under `models/`, and three reducer contexts under `contexts/`. Nginx serves the generated bundle and proxies `/api` to Rocket.

The repaired client builds in release mode with the Docker-equivalent `getrandom_backend="wasm_js"` flag. The UI covers login/registration, profile management, friends/requests, chats/messages, servers/channels/posts, discovery, and report downloads. Reducers give the application an initial state organization, and route types are strongly typed.

## Completion Gaps

### Real-time behavior is not implemented — P1

No client source creates a `WebSocket`, `ws://`, or `wss://` connection. Messages and posts are submitted through HTTP and threads are refetched. This contradicts README claims that the client uses authenticated WebSockets and avoids polling. Decide explicitly:

- implement authenticated sockets, reconnect/backoff, bounded event buffering, optimistic reconciliation, and logout cleanup; or
- describe the current product as request/refresh based and disable the insecure server sockets until the feature is ready.

Nginx also lacks WebSocket `Upgrade`/`Connection` proxy headers, so the current container path would not support upgrades without configuration changes.

### Route and context state can diverge — P1

`ChatRoom` receives `chat_id` but submits/refetches using `chat_ctx.current_chat.id`. Similar server screens receive route IDs while rendering mutable global context. Direct navigation, browser back/forward, and rapid selection can display or mutate the wrong entity.

Make the route parameter the canonical identity. A page should load `RemoteData<Entity>` keyed by that ID; context may cache by ID but must not override the route. Add tests for direct deep links and fast route changes.

### Network contract is too weak — P1

`Result<T, ()>` discards status/body/context. `api_delete` returns success for every HTTP response, and `api_head` reflects transport completion rather than successful status. POST sometimes converts non-2xx responses into `Ok(OnlyResponse)`. This causes the UI to optimistically remove data or mark a session valid after authorization/server errors.

Create one typed API client with:

```rust
enum ApiError {
    Unauthorized,
    Forbidden,
    Validation { message: String },
    NotFound,
    Conflict { message: String },
    Server { request_id: Option<String> },
    Network(String),
    Decode(String),
}
```

Every method must check `response.ok()` before decoding/success. Centralize 401 logout and preserve actionable errors for rendering.

## Maintainability

- There are 137 `unwrap`/`expect` calls in client source; DOM casts, missing contexts, storage access, and synthetic event creation can panic the WASM app.
- Crate-level `#![allow(unused)]` hides dead code/imports.
- Multiple components mix form extraction, request construction, navigation, modal manipulation, data refresh, and rendering.
- Large inline SVG maps and stringly typed icon/state values increase bundle/code size and typo risk.
- There are no unit, component, or browser tests.

## Recommended Boundaries

Organize by feature (`auth`, `friends`, `chat`, `servers`, `reports`) with each feature owning its API types, reducer, hooks, components, and tests. Keep shared infrastructure limited to routing, API transport, session management, design primitives, and error/loading components. This aligns code changes with user behavior and reduces broad context coupling.

## Near-Term Plan

1. Fix API status semantics and route/context identity correctness.
2. Add explicit loading/error/empty states and remove user-triggerable panics.
3. Decide and implement the real-time contract end-to-end.
4. Split contexts by update frequency and feature ownership.
5. Add component tests and browser flows for login, deep links, failed mutations, and reconnect behavior.
