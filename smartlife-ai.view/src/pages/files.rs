use leptos::prelude::*;

#[component]
pub fn FilesPage() -> impl IntoView {
    view! {
        <h1 style="margin-bottom: 20px">"File Manager"</h1>
        <div class="card">
            <h2>"/"</h2>
            <table>
                <thead>
                    <tr><th>"Name"</th><th>"Size"</th><th>"Type"</th></tr>
                </thead>
                <tbody>
                    <tr>
                        <td>"Connect to backend to browse files"</td>
                        <td>"-"</td>
                        <td>"-"</td>
                    </tr>
                </tbody>
            </table>
        </div>
    }
}
