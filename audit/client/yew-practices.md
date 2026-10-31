# Client Audit: Yew and Rust Practices

## Component and Hook Design

Yew hooks are generally called at the top level and reducers are used for shared mutable state, which is a sound base. The main issue is responsibility density: components often own DOM extraction, networking, state refresh, modal control, and navigation at once.

Extract feature hooks such as `use_chat(chat_id)`, `use_mutation`, and `use_session`. Hooks should return typed state and callbacks, while components render that state. Yew supports custom hooks and reducer/equality variants; current state guidance distinguishes `use_state_eq` and `use_reducer_eq` for avoiding equal-value rerenders: <https://yew.rs/docs/concepts/function-components/state>.

## Context Discipline

Yew documentation warns that contexts are easy to overuse because every changed context value rerenders consumers. Use context for genuinely distant shared data such as the current session. Pass local data through props and split contexts when update frequencies differ: <https://yew.rs/docs/0.22/concepts/contexts>.

Recommended providers:

- `SessionProvider`: account, authentication state, logout.
- `ChatIndexProvider`: previews only.
- Selected chat/thread state owned by the route page or a keyed cache.
- `ServerIndexProvider`: server list only.
- Selected server/channel state owned by the route page or keyed cache.
- Toast/error UI as a separate, lightweight provider.

## Props, Identity, and Rendering

- Use route props immediately; do not accept `chat_id`/`server_id` and then rely on unrelated current context.
- Add stable `key` values to rendered lists so Yew preserves component identity correctly.
- Prefer `Callback::from`/`use_callback` with minimal captured state; avoid cloning entire contexts into every handler.
- Use controlled form state or a validated form model instead of repeated unchecked DOM casts.
- Replace stringly typed modal IDs, page names, and icon choices with enums/newtypes.
- Use semantic buttons/forms and keyboard/focus management; clickable list items and synthetic events are brittle and less accessible.

## Panic and Error Policy

Browser APIs and contexts are fallible. Replace `unwrap` with:

- conditional rendering when a provider/DOM node is unavailable;
- typed component contracts that guarantee required props;
- `ApiError` for transport/decode failures;
- logged diagnostics plus user-visible recovery for unexpected browser failures.

Keep `expect` only for static application invariants and include the invariant in its message.

## Dependencies and Build

The WebAssembly randomness repair is effective: release Trunk build succeeds with `getrandom`'s `wasm_js` backend. However, `rand 0.9.0-alpha.2` is pre-release, Yew/router are Git dependencies, and several crates are duplicated at different versions. Prefer stable released versions, pin exact Git revisions only when necessary, and make the Docker and local Trunk commands identical through a checked-in script or task runner.

Do not load `.env` at browser runtime expecting secrecy: all build-time client configuration is public. Name public settings accordingly (for example `PUBLIC_API_BASE_URL`) and never compile credentials into WASM.

## Testing Plan

1. Reducer unit tests for every action and stale-response rule.
2. API client tests for all relevant statuses and malformed bodies.
3. Component tests for loading/error/empty/ready states and accessibility.
4. Browser tests for registration/login/logout, route reload, direct links, create/edit/delete failures, and report download.
5. Two-client end-to-end tests for real-time ordering, reconnect, duplicates, and authorization denial.

Add formatting and Clippy after cleanup. The current strict Clippy output includes a no-effect closure in message editing, redundant patterns, arbitrary self syntax, clones of `Copy` values, and unchecked unit/error flows; these are good incremental cleanup targets after correctness fixes.
