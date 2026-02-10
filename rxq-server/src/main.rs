use axum::{extract::Query, http::StatusCode, response::IntoResponse, routing::get, Router};
use serde::Deserialize;
use std::net::SocketAddr;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};

#[derive(Deserialize)]
struct ProxyParams {
    url: String,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("rxq_server=debug,tower_http=debug")
        .init();

    // Define static file service (serving the React app)
    // We assume the binary is run from workspace root, where 'www/dist' is located.
    let serve_dir = ServeDir::new("www/dist");

    // Define Router
    let app = Router::new()
        .route("/api/proxy", get(proxy_handler))
        .fallback_service(serve_dir)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http());

    // Run server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn proxy_handler(Query(params): Query<ProxyParams>) -> impl IntoResponse {
    let url = params.url;
    tracing::info!("Proxying request to: {}", url);

    // Create client with browser-like User-Agent
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build();

    let client = match client {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to create client: {}", e),
            )
                .into_response()
        }
    };

    match client.get(&url).send().await {
        Ok(response) => {
            let status = response.status();
            match response.text().await {
                Ok(text) => (
                    StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::OK),
                    text,
                )
                    .into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to read body: {}", e),
                )
                    .into_response(),
            }
        }
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            format!("Failed to fetch URL: {}", e),
        )
            .into_response(),
    }
}
