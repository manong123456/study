//! CapeOS Frontend
//!
//! Leptos-based web application running in client-side rendering (CSR) mode.
//! Provides login, dashboard, and file management UI for the CapeOS platform.

mod api;
mod pages;
mod components;

use leptos::prelude::*;
use leptos_router::{components::{Router, Routes, Route}, path};
use pages::{login::LoginPage, dashboard::DashboardPage, files::FilesPage};
use components::layout::Layout;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

/// Root application component with routing for login, dashboard, and files.
#[component]
fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes fallback=|| "Page not found">
                <Route path=path!("/login") view=LoginPage />
                <Route path=path!("/") view=|| view! { <Layout><DashboardPage/></Layout> } />
                <Route path=path!("/files") view=|| view! { <Layout><FilesPage/></Layout> } />
            </Routes>
        </Router>
    }
}
