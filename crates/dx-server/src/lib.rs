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

use axum::{routing::get, Router};
use dashmap::DashMap;
use dx_packet::Template;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};

/// Global server state
#[derive(Clone)]
pub struct ServerState {
    /// Binary snapshot cache (path -> bytes)
    pub binary_cache: Arc<DashMap<String, Vec<u8>>>,
    /// Template cache (id -> Template) - stores full Template structs
    pub template_cache: Arc<DashMap<u32, Template>>,
    /// Version hashes for delta patching
    pub version_cache: Arc<DashMap<String, String>>,
}

impl Default for ServerState {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            binary_cache: Arc::new(DashMap::new()),
            template_cache: Arc::new(DashMap::new()),
            version_cache: Arc::new(DashMap::new()),
        }
    }

    /// Load artifacts from build output directory
    pub fn load_artifacts(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("📦 Loading artifacts from {}", path.display());

        // Load layout.bin (Templates)
        let layout_path = path.join("layout.bin");
        if layout_path.exists() {
            let bytes = std::fs::read(&layout_path)?;
            let config = bincode::config::standard();
            let templates: Vec<Template> = bincode::decode_from_slice(&bytes, config)?.0;

            tracing::info!("  ✓ Loaded {} templates", templates.len());

            // Populate cache with full Template structs
            for template in templates {
                self.template_cache.insert(template.id, template);
            }
        } else {
            tracing::warn!("  ⚠️ layout.bin not found");
        }

        // Load app.wasm
        let wasm_path = path.join("app.wasm");
        if wasm_path.exists() {
            let bytes = std::fs::read(&wasm_path)?;
            let size = bytes.len();
            self.binary_cache.insert("app.wasm".to_string(), bytes);
            tracing::info!("  ✓ Loaded app.wasm ({} bytes)", size);
        }

        Ok(())
    }

    /// Register a template manually (for testing or dynamic loading)
    pub fn register_template(&self, template: Template) {
        let id = template.id;
        self.template_cache.insert(id, template);
        tracing::debug!("📄 Registered template {}", id);
    }
}

/// Build the Axum router with all routes
pub fn build_router(state: ServerState) -> Router {
    Router::new()
        // Root index (supports bot detection + SSR)
        .route("/", get(handlers::serve_index))
        // Health check
        .route("/health", get(handlers::health_check))
        // Binary endpoints (future)
        // .route("/api/binary/:app", get(handlers::serve_binary))
        // .route("/api/delta/:app", get(handlers::serve_delta))
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
