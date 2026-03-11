//! Port allocation utilities.
//!
//! Provides helpers to find available ports and check port availability.

use std::net::TcpListener;

/// Binds to an ephemeral port on localhost and returns the allocated port number.
///
/// Uses `127.0.0.1:0` to let the OS assign an available port.
pub fn get_available_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind");
    listener.local_addr().unwrap().port()
}

/// Returns `true` if the given port is available for binding on localhost.
///
/// # Arguments
///
/// * `port` - The port number to check.
pub fn is_port_available(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_available_port() {
        let port = get_available_port();
        assert!(port > 0);
    }

    #[test]
    fn test_is_port_available() {
        let port = get_available_port();
        let listener = TcpListener::bind(("127.0.0.1", port)).unwrap();
        assert!(!is_port_available(port));
        drop(listener);
        assert!(is_port_available(port));
    }
}
