use std::net::SocketAddr;
use axum::{Router, routing::get, Json};
use serde::Serialize;
use capeos_common::models::ApiResponse;
use capeos_common::paths::DEFAULT_RUNTIME_PATH;
use capeos_common::utils::{write_url_file, port::get_available_port};
use tracing_subscriber::EnvFilter;

#[derive(Serialize)]
struct ContainerInfo {
    id: String,
    name: String,
    image: String,
    state: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let port = get_available_port();
    let app = Router::new()
        .route("/ping", get(|| async { "pong" }))
        .route("/v1/app_management/container", get(list_containers));

    let listen_url = format!("http://127.0.0.1:{}", port);
    write_url_file(DEFAULT_RUNTIME_PATH, "app-management.url", &listen_url).await?;

    tracing::info!("AppManagement on 127.0.0.1:{}", port);
    let listener = tokio::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], port))).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn list_containers() -> Json<ApiResponse<Vec<ContainerInfo>>> {
    let docker = bollard::Docker::connect_with_local_defaults();
    match docker {
        Ok(docker) => {
            let opts = bollard::container::ListContainersOptions::<String> {
                all: true,
                ..Default::default()
            };
            match docker.list_containers(Some(opts)).await {
                Ok(containers) => {
                    let list: Vec<ContainerInfo> = containers.iter().map(|c| ContainerInfo {
                        id: c.id.clone().unwrap_or_default(),
                        name: c.names.as_ref().and_then(|n| n.first()).cloned().unwrap_or_default(),
                        image: c.image.clone().unwrap_or_default(),
                        state: c.state.clone().unwrap_or_default(),
                    }).collect();
                    Json(ApiResponse::ok(list))
                }
                Err(_) => Json(ApiResponse::ok(vec![])),
            }
        }
        Err(_) => Json(ApiResponse::ok(vec![])),
    }
}
