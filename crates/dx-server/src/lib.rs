//! # dx-server: The Holographic Server
//!
//! High-performance SSR & Edge Runtime for dx-www
//!
//! **Role:** Serve Binary Snapshots, Handle SSR Inflation (SEO), Manage State
//! **Philosophy:** "Write TSX, Serve Binary"
//!
//! ## Architecture
//!
//! ```text
//! ┌──────────────┐
//! │  TSX Files   │
//! └──────┬───────┘
//!        ↓
//! ┌──────────────┐
//! │ dx-compiler  │
//! └──────┬───────┘
//!        ↓
//! ┌──────────────┐
//! │  .dxb Files  │ ← Binary Format
//! └──────┬───────┘
//!        ↓
//! ┌──────────────┐
//! │  dx-server   │ ← YOU ARE HERE
//! └──────┬───────┘
//!        ↓
//! ┌──────────────┐
//! │   Browser    │
//! │  (dx-cache)  │
//! └──────────────┘
//! ```

pub mod delta;
pub mod handlers;
pub mod ssr;
pub mod stream;

use axum::{
    routing::{get, post},
    Router,
};
use dashmap::DashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};

/// Global server state
#[derive(Clone)]
pub struct ServerState {
    /// Binary snapshot cache (path -> bytes)
    pub binary_cache: Arc<DashMap<String, Vec<u8>>>,
    /// Template cache (id -> html)
    pub template_cache: Arc<DashMap<u32, String>>,
    /// Version hashes for delta patching
    pub version_cache: Arc<DashMap<String, String>>,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            binary_cache: Arc::new(DashMap::new()),
            template_cache: Arc::new(DashMap::new()),
            version_cache: Arc::new(DashMap::new()),
        }
    }
}

/// Build the Axum router with all routes
pub fn build_router(state: ServerState) -> Router {
    Router::new()
        // Static files
        .route("/", get(handlers::serve_index))
        .route("/*path", get(handlers::serve_static))
        // Binary endpoints
        .route("/api/binary/:app", get(handlers::serve_binary))
        .route("/api/delta/:app", get(handlers::serve_delta))
        // SSR endpoint (for SEO)
        .route("/ssr/*path", get(handlers::serve_ssr))
        // Health check
        .route("/health", get(handlers::health_check))
        // Add state
        .with_state(state)
        // Middleware
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}

/// Start the dx-server
pub async fn serve(addr: SocketAddr, state: ServerState) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("🚀 dx-server starting at {}", addr);

    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::info!("✨ dx-server ready - The Holographic Server is online");
    tracing::info!("📦 Binary streaming enabled");
    tracing::info!("🔍 SEO inflation ready");
    tracing::info!("⚡ Delta patching active");

    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_creation() {
        let state = ServerState::new();
        assert_eq!(state.binary_cache.len(), 0);
        assert_eq!(state.template_cache.len(), 0);
    }
}
