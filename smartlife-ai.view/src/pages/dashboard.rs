use leptos::prelude::*;

#[component]
pub fn DashboardPage() -> impl IntoView {
    view! {
        <h1 style="margin-bottom: 20px">"Dashboard"</h1>
        <div class="stats-grid">
            <div class="stat-card">
                <div class="label">"CPU Usage"</div>
                <div class="value">"--"</div>
            </div>
            <div class="stat-card">
                <div class="label">"Memory"</div>
                <div class="value">"--"</div>
            </div>
            <div class="stat-card">
                <div class="label">"Services"</div>
                <div class="value">"--"</div>
            </div>
            <div class="stat-card">
                <div class="label">"Storage"</div>
                <div class="value">"--"</div>
            </div>
        </div>
        <div class="card">
            <h2>"System Information"</h2>
            <p>"Connect to the CapeOS backend to view live system stats."</p>
        </div>
    }
}
