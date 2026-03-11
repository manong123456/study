//! Layout components including sidebar navigation and main content area.

use leptos::prelude::*;

/// Main sidebar navigation layout wrapping page content with Dashboard and Files links.
#[component]
pub fn Layout(children: Children) -> impl IntoView {
    view! {
        <div class="app-container">
            <nav class="sidebar">
                <h1>"CapeOS"</h1>
                <a href="/">"Dashboard"</a>
                <a href="/files">"Files"</a>
            </nav>
            <main class="main-content">
                {children()}
            </main>
        </div>
    }
}
