use std::net::TcpListener;

pub fn get_available_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind");
    listener.local_addr().unwrap().port()
}

pub fn is_port_available(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_ok()
}
