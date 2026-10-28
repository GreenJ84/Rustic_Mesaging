//! Application realtime infrastructure.
//!
//! This layer owns delivery concerns such as WebSocket connections, event
//! routing, reconnect behavior, and eventually presence/notifications. Domain
//! services remain responsible for validation and persistence.

pub(crate) mod web_socket;
