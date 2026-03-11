//! Data models shared across CapeOS services.
//!
//! Includes API response wrappers, route definitions, and event types.

pub mod response;
pub mod route;
pub mod event;

/// Re-export of the standard API response wrapper.
pub use response::ApiResponse;
/// Re-export of the route definition for gateway routing.
pub use route::Route;
