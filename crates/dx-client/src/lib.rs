//! # dx-client: 9KB WASM Runtime
//!
//! The lean, mean, DOM-rendering machine.
//!
//! ## Architecture (Option D: Optimistic Guard)
//!
//! ```text
//! fetch(url) ──► JS Loader ──► SubtleCrypto.verify() ──► dx-client WASM
//!                    │                                        │
//!                    │  (signature verified BEFORE WASM)      │
//!                    ▼                                        ▼
//!              64-byte sig removed                  Zero-copy render
//! ```
//!
//! ## Security Model
//!
//! - Ed25519 signature is verified by JavaScript BEFORE this code runs
//! - The `ptr` passed to `render_stream` is ALREADY VERIFIED
//! - No crypto in WASM = 0 bytes of crypto bloat

// ============================================================================
// ULTRA-TINY ALLOCATOR (~100 bytes vs ~1KB for wee_alloc)
// ============================================================================

mod allocator;

#[global_allocator]
static ALLOC: allocator::BumpAlloc = allocator::BumpAlloc;

use core::ptr;
use dx_packet::*;
use wasm_bindgen::prelude::*;

mod node_registry;
mod renderer;
mod string_table;
mod template_cache;

pub use node_registry::NodeRegistry;
pub use renderer::Renderer;
pub use string_table::StringTableReader;
pub use template_cache::TemplateCache;

// ============================================================================
// GLOBAL STATE (thread_local for WASM single-thread)
// ============================================================================

use core::cell::RefCell;

thread_local! {
    static RENDERER: RefCell<Option<Renderer>> = RefCell::new(None);
}

// ============================================================================
// WASM EXPORTS
// ============================================================================

/// Initialize the renderer
///
/// Must be called once before render_stream
#[wasm_bindgen]
pub fn init() -> Result<(), u8> {
    RENDERER.with(|r| {
        *r.borrow_mut() = Some(Renderer::new()?);
        Ok(())
    })
}

/// Render a verified HTIP stream to DOM
///
/// # Arguments
/// * `data` - Raw bytes from server (signature already stripped by JS loader)
///
/// # Returns
/// * `Ok(())` on success
/// * `Err(error_code)` on failure (see dx_packet::ErrorCode)
///
/// # Safety
/// This function assumes the data has been signature-verified by the JS loader.
/// Passing unverified data is a security vulnerability.
#[wasm_bindgen]
pub fn render_stream(data: &[u8]) -> Result<(), u8> {
    // Validate minimum size
    if data.len() < HtipHeader::SIZE {
        return Err(ErrorCode::BufferTooSmall as u8);
    }

    // Zero-copy header read
    let header = unsafe { ptr::read_unaligned(data.as_ptr() as *const HtipHeader) };

    // Validate header
    if !header.is_valid() {
        return Err(ErrorCode::InvalidMagic as u8);
    }

    RENDERER.with(|r| {
        let mut renderer = r.borrow_mut();
        let renderer = renderer.as_mut().ok_or(ErrorCode::NodeNotFound as u8)?;

        renderer.process_stream(data, &header)
    })
}

/// Get current node count (for debugging)
#[wasm_bindgen]
pub fn get_node_count() -> u32 {
    RENDERER.with(|r| r.borrow().as_ref().map(|r| r.node_count()).unwrap_or(0))
}

/// Clear all state (for hot reload)
#[wasm_bindgen]
pub fn reset() {
    RENDERER.with(|r| {
        *r.borrow_mut() = None;
    });
    // Reset bump allocator to reclaim all memory
    unsafe {
        allocator::reset_heap();
    }
}
