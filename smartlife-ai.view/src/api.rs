//! API client module for CapeOS frontend.
//!
//! Provides token storage and HTTP utilities for communicating with backend services.

use gloo_storage::{LocalStorage, Storage};

/// Retrieves the stored JWT token from local storage.
pub fn get_token() -> Option<String> {
    LocalStorage::get::<String>("capeos_token").ok()
}

/// Stores the JWT token in local storage.
pub fn set_token(token: &str) {
    let _ = LocalStorage::set("capeos_token", token.to_string());
}

/// Removes the JWT token from local storage.
pub fn clear_token() {
    LocalStorage::delete("capeos_token");
}
