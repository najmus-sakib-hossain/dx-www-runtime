//! # dx-packet: Binary Protocol Types
//!
//! Zero-dependency crate defining the memory layout contract between
//! dx-server (serializer) and dx-client (WASM runtime).
//!
//! All types are `#[repr(C)]` for predictable memory layout and zero-copy parsing.
//!
//! ## HTIP v2 Wire Format
//!
//! ```text
//! ┌────────────────────────────────────────┐
//! │  Ed25519 Signature (64 bytes)          │  ← Verified by JS loader BEFORE WASM
//! ├────────────────────────────────────────┤
//! │  HtipHeader (16 bytes)                 │
//! ├────────────────────────────────────────┤
//! │  String Table (variable)               │
//! ├────────────────────────────────────────┤
//! │  Template Dictionary (variable)        │
//! ├────────────────────────────────────────┤
//! │  Opcode Stream (variable)              │
//! └────────────────────────────────────────┘
//! ```

#![no_std]

// ============================================================================
// HEADER
// ============================================================================

/// HTIP Header - first 16 bytes after signature
/// 
/// Memory Layout:
/// ```text
/// Offset  Size  Field
/// 0       2     magic (0x4458 = "DX")
/// 2       1     version
/// 3       1     flags
/// 4       2     template_count
/// 6       2     string_count
/// 8       4     opcode_count
/// 12      4     payload_size
/// ```
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct HtipHeader {
    /// Magic bytes: 0x4458 ("DX" in little-endian)
    pub magic: u16,
    /// Protocol version (currently 2)
    pub version: u8,
    /// Flags: bit 0 = has_strings, bit 1 = has_templates
    pub flags: u8,
    /// Number of templates in dictionary
    pub template_count: u16,
    /// Number of strings in string table
    pub string_count: u16,
    /// Number of opcodes in stream
    pub opcode_count: u32,
    /// Total payload size (excluding signature)
    pub payload_size: u32,
}

impl HtipHeader {
    pub const MAGIC: u16 = 0x4458; // "DX"
    pub const VERSION: u8 = 2;
    pub const SIZE: usize = 16;
    
    /// Validate header magic and version
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.magic == Self::MAGIC && self.version == Self::VERSION
    }
}

// ============================================================================
// OPCODES
// ============================================================================

/// Opcode types for DOM manipulation
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpType {
    /// Clone template and append to parent
    Clone = 1,
    /// Update text content of node
    PatchText = 2,
    /// Update attribute value
    PatchAttr = 3,
    /// Toggle CSS class
    ClassToggle = 4,
    /// Remove node from DOM
    Remove = 5,
    /// Set style property
    SetStyle = 6,
    /// Batch start marker
    BatchStart = 7,
    /// Batch commit marker
    BatchCommit = 8,
}

impl OpType {
    #[inline]
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            1 => Some(Self::Clone),
            2 => Some(Self::PatchText),
            3 => Some(Self::PatchAttr),
            4 => Some(Self::ClassToggle),
            5 => Some(Self::Remove),
            6 => Some(Self::SetStyle),
            7 => Some(Self::BatchStart),
            8 => Some(Self::BatchCommit),
            _ => None,
        }
    }
}

/// Fixed-size opcode header (4 bytes)
/// 
/// Memory Layout:
/// ```text
/// Offset  Size  Field
/// 0       1     op_type
/// 1       1     reserved (alignment)
/// 2       2     target_id
/// ```
/// 
/// Payload follows inline based on op_type
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct OpcodeHeader {
    /// Operation type
    pub op_type: u8,
    /// Reserved for alignment
    pub reserved: u8,
    /// Target node ID (0 = root)
    pub target_id: u16,
}

impl OpcodeHeader {
    pub const SIZE: usize = 4;
}

// ============================================================================
// OPCODE PAYLOADS
// ============================================================================

/// Clone operation: instantiate template
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ClonePayload {
    /// Template ID to clone
    pub template_id: u16,
    /// Parent node ID
    pub parent_id: u16,
}

/// Text patch: update node text content
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PatchTextPayload {
    /// String table index for new text
    pub string_idx: u16,
    /// Reserved
    pub reserved: u16,
}

/// Attribute patch: update attribute value
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PatchAttrPayload {
    /// String table index for attribute name
    pub attr_name_idx: u16,
    /// String table index for attribute value
    pub attr_value_idx: u16,
}

/// Class toggle: add/remove CSS class
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ClassTogglePayload {
    /// String table index for class name
    pub class_name_idx: u16,
    /// 1 = add, 0 = remove
    pub enable: u8,
    /// Reserved
    pub reserved: u8,
}

/// Style set: update inline style
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SetStylePayload {
    /// String table index for property name
    pub prop_name_idx: u16,
    /// String table index for property value
    pub prop_value_idx: u16,
}

// ============================================================================
// STRING TABLE
// ============================================================================

/// String entry header in string table
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct StringEntry {
    /// Offset into string data region
    pub offset: u32,
    /// Length in bytes
    pub len: u16,
    /// Reserved
    pub reserved: u16,
}

impl StringEntry {
    pub const SIZE: usize = 8;
}

// ============================================================================
// TEMPLATE DICTIONARY
// ============================================================================

/// Template entry in template dictionary
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct TemplateEntry {
    /// Template ID
    pub id: u16,
    /// String table index for HTML content
    pub html_string_idx: u16,
    /// Number of slots in template
    pub slot_count: u8,
    /// Reserved
    pub reserved: [u8; 3],
}

impl TemplateEntry {
    pub const SIZE: usize = 8;
}

// ============================================================================
// ERROR CODES (No strings, just u8)
// ============================================================================

/// Error codes for dx-client (no string formatting)
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorCode {
    /// Success
    Ok = 0,
    /// Invalid magic bytes
    InvalidMagic = 1,
    /// Unsupported version
    UnsupportedVersion = 2,
    /// Invalid opcode
    InvalidOpcode = 3,
    /// Template not found
    TemplateNotFound = 4,
    /// String index out of bounds
    StringIndexOutOfBounds = 5,
    /// Node not found
    NodeNotFound = 6,
    /// Buffer too small
    BufferTooSmall = 7,
}

// ============================================================================
// CONSTANTS
// ============================================================================

/// Ed25519 signature size (verified by JS before WASM)
pub const SIGNATURE_SIZE: usize = 64;

/// Maximum templates
pub const MAX_TEMPLATES: u16 = 4096;

/// Maximum strings in table
pub const MAX_STRINGS: u16 = 65535;

/// Maximum nodes in registry
pub const MAX_NODES: u16 = 65535;
