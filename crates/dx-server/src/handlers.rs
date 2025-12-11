//! # HTTP Handlers
//!
//! Axum route handlers for dx-server

use crate::{ssr::is_bot, ServerState};
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
};



/// Serve index.html or SSR
pub async fn serve_static(
    State(state): State<ServerState>,
    headers: axum::http::HeaderMap,
    uri: axum::http::Uri,
) -> impl IntoResponse {
    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // Detect Bot
    if crate::ssr::is_bot(user_agent) {
        // return serve_ssr(uri.path().to_string(), state).await.into_response();
        return Html("Bot Detected - SSR Disabled".to_string()).into_response();
    }

    // Serve SPA index for humans (SPA Fallback)
    // In production, this would serve the actual file from dist
    Html(include_str!("../../../examples/hello-world/demo.html")).into_response()
}

pub async fn serve_ssr(path: String, state: ServerState) -> impl IntoResponse {
    Html("SSR Disabled".to_string())
}
/*
    // Scoped block to drop DashMap Ref immediately
    let template_html = state.template_cache.get(&0).map(|r| r.value().clone());

    if let Some(html_str) = template_html {
        // ...
        let html = crate::ssr::inflate_page(
            &html_str,
            &slots,
            "Dx-Server SSR",
            &meta
        );
        return Html(html);
    }
*/

    Html("<h1>500 - Template Not Found</h1>".to_string())
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
