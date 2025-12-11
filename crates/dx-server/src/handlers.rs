//! # HTTP Handlers
//!
//! Axum route handlers for dx-server

use axum::{
    body::Body,
    extract::{Path, State},
    http::{StatusCode, header},
    response::{Html, IntoResponse, Response},
};
use crate::{ServerState, ssr::is_bot};

/// Serve index.html
pub async fn serve_index() -> impl IntoResponse {
    Html(include_str!("../../../examples/hello-world/demo.html"))
}

/// Serve static files
pub async fn serve_static(
    Path(path): Path<String>,
) -> impl IntoResponse {
    // TODO: Implement proper static file serving
    (StatusCode::NOT_FOUND, "Not found")
}

/// Serve binary payload
pub async fn serve_binary(
    Path(app): Path<String>,
    State(state): State<ServerState>,
) -> impl IntoResponse {
    match state.binary_cache.get(&app) {
        Some(binary) => {
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/dx-binary")
                .header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
                .body(Body::from(binary.clone()))
                .unwrap()
        }
        None => {
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Binary not found"))
                .unwrap()
        }
    }
}

/// Serve delta patch
pub async fn serve_delta(
    Path(app): Path<String>,
    State(state): State<ServerState>,
) -> impl IntoResponse {
    // TODO: Implement delta serving
    // 1. Check If-None-Match header for old hash
    // 2. Calculate delta
    // 3. Serve with application/dx-patch MIME type
    
    (StatusCode::NOT_IMPLEMENTED, "Delta patching coming soon")
}

/// Serve SSR (for SEO bots)
pub async fn serve_ssr(
    Path(path): Path<String>,
    State(state): State<ServerState>,
) -> impl IntoResponse {
    // TODO: Implement SSR
    // 1. Check User-Agent
    // 2. If bot, inflate binary to HTML
    // 3. Otherwise, redirect to SPA
    
    Html("<html><body><h1>SSR Placeholder</h1></body></html>")
}

/// Health check endpoint
pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "dx-server is healthy")
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_health_check() {
        let response = health_check().await;
        // Response implements IntoResponse, can't easily test status
        // In real tests, use reqwest to test full HTTP
    }
}
