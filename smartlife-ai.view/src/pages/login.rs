//! Login page with username/password form and JWT storage.

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use wasm_bindgen_futures::spawn_local;

/// Login page component. POSTs credentials to user service, stores JWT on success, redirects to dashboard.
#[component]
pub fn LoginPage() -> impl IntoView {
    let username = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let error = RwSignal::new(String::new());
    let navigate = use_navigate();

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let u = username.get();
        let p = password.get();
        let error = error;
        let navigate = navigate.clone();
        spawn_local(async move {
            let body = serde_json::json!({ "username": u, "password": p });
            match gloo_net::http::Request::post("/v1/user_service/users/login")
                .header("Content-Type", "application/json")
                .body(body.to_string())
                .unwrap()
                .send()
                .await
            {
                Ok(resp) => {
                    if resp.ok() {
                        if let Ok(data) = resp.json::<serde_json::Value>().await {
                            if let Some(token) = data["data"]["token"].as_str() {
                                crate::api::set_token(token);
                                let _ = navigate("/", Default::default());
                            }
                        }
                    } else {
                        error.set("Login failed".to_string());
                    }
                }
                Err(e) => error.set(e.to_string()),
            }
        });
    };

    view! {
        <div class="login-page">
            <div class="login-box">
                <h1>"CapeOS"</h1>
                <form on:submit=on_submit>
                    <div class="form-group">
                        <label>"Username"</label>
                        <input type="text"
                            on:input=move |ev| username.set(event_target_value(&ev))
                            prop:value=username
                        />
                    </div>
                    <div class="form-group">
                        <label>"Password"</label>
                        <input type="password"
                            on:input=move |ev| password.set(event_target_value(&ev))
                            prop:value=password
                        />
                    </div>
                    <button class="btn" type="submit">"Sign In"</button>
                    <Show when=move || !error.get().is_empty()>
                        <p class="error-msg">{move || error.get()}</p>
                    </Show>
                </form>
            </div>
        </div>
    }
}
