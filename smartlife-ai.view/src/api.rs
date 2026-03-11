use gloo_storage::{LocalStorage, Storage};

pub fn get_token() -> Option<String> {
    LocalStorage::get::<String>("capeos_token").ok()
}

pub fn set_token(token: &str) {
    let _ = LocalStorage::set("capeos_token", token.to_string());
}

pub fn clear_token() {
    LocalStorage::delete("capeos_token");
}
