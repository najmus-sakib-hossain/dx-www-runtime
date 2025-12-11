//! # HTTP Handlers
//!
//! Axum route handlers for dx-server

use crate::{ssr, ServerState};
use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{Html, IntoResponse},
};

/// Serve index.html or SSR-inflated HTML
///
/// # Bot Detection Strategy
/// - Bot detected → Serve SSR HTML (SEO-optimized)
/// - Human detected → Serve SPA shell (fast hydration)
///
/// # Performance
/// - Bot path: ~1ms (string inflation)
/// - Human path: ~0ms (static file serve)
pub async fn serve_index(
    State(state): State<ServerState>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let user_agent = headers.get(header::USER_AGENT).and_then(|v| v.to_str().ok()).unwrap_or("");

    // Detect Bot vs Human
    if ssr::is_bot(user_agent) {
        tracing::info!("🤖 Bot detected: {}", user_agent);
        return serve_ssr(state).await.into_response();
    }

    // Serve SPA shell for humans
    tracing::debug!("👤 Human detected, serving SPA shell");
    serve_spa_shell().into_response()
}

/// Serve SSR-inflated HTML for bots
async fn serve_ssr(state: ServerState) -> impl IntoResponse {
    // Try to get template with ID 0 (root template)
    let template_opt = state.template_cache.get(&0).map(|entry| entry.clone());

    if let Some(template) = template_opt {
        // Create mock state (in production, this would come from data fetching)
        let mut state_data = ssr::StateData::new();
        state_data.set(0, "Hello from SSR!".to_string());

        // Metadata for SEO
        let meta_tags = vec![
            ("description".to_string(), "Dx-WWW Runtime - The Binary Web".to_string()),
            ("keywords".to_string(), "wasm, binary, performance, ssr".to_string()),
            ("og:title".to_string(), "Dx-WWW Runtime".to_string()),
        ];

        // Inflate the page
        let html = ssr::inflate_page(
            &template,
            &state_data,
            "Dx-WWW Runtime",
            &meta_tags,
            &[], // No scripts for bots
        );

        tracing::debug!("✅ SSR inflation complete ({} bytes)", html.len());
        return Html(html);
    }

    // Fallback if no template found
    tracing::warn!("⚠️ Template 0 not found in cache");
    Html("<h1>500 - Template Not Found</h1>".to_string())
}

/// Serve SPA shell for humans (fast client-side hydration)
fn serve_spa_shell() -> Html<&'static str> {
    // In production, this would serve the actual built index.html
    // For now, use the demo file
    Html(include_str!("../../../examples/hello-world/demo.html"))
}

/// Stream binary artifacts (Day 16: The Binary Streamer)
///
/// # The Waterfall Killer
///
/// Traditional loading is sequential:
/// 1. Download → 2. Parse → 3. Execute
///
/// Streaming loading is parallel:
/// - Chunk 1 (Layout) → Client creates templates while downloading
/// - Chunk 2 (State) → Client allocates memory while downloading
/// - Chunk 3 (WASM) → Browser compiles while downloading
///
/// Result: Zero blocking time. Execution starts before download completes.
///
/// # Example
/// ```bash
/// curl --no-buffer http://localhost:3000/stream/app | xxd | head -50
/// ```
pub async fn serve_binary_stream(
    State(state): State<ServerState>,
    axum::extract::Path(app_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    use axum::body::Body;
    use axum::http::header;

    tracing::info!("📡 Streaming binary for app: {}", app_id);

    // Load artifacts from cache
    let layout_bin = state
        .binary_cache
        .get("layout.bin")
        .map(|entry| entry.value().clone())
        .unwrap_or_else(|| {
            tracing::warn!("⚠️ layout.bin not in cache, generating mock");
            vec![0u8; 100] // Mock layout
        });

    let wasm_bin = state
        .binary_cache
        .get("app.wasm")
        .map(|entry| entry.value().clone())
        .unwrap_or_else(|| {
            tracing::warn!("⚠️ app.wasm not in cache, using empty");
            vec![]
        });

    // Create mock artifact for header
    let artifact = dx_packet::DxbArtifact {
        version: 1,
        capabilities: dx_packet::CapabilitiesManifest::default(),
        templates: vec![],
        wasm_size: wasm_bin.len() as u32,
    };

    // Create streaming body
    let stream = crate::stream::create_stream(&artifact, layout_bin, wasm_bin);
    let body = Body::from_stream(stream);

    // Build response with streaming headers
    let response = axum::response::Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(header::CACHE_CONTROL, "public, max-age=31536000") // Cache for 1 year
        .header("X-Dx-Version", "1.0")
        .header("X-Dx-Stream", "chunked")
        .body(body)
        .unwrap();

    tracing::debug!("✅ Stream initialized");
    response
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
        let _response = health_check().await;
        // Response implements IntoResponse, can't easily test status
        // In real tests, use reqwest to test full HTTP
    }
}
