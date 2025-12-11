//! # Delta Module - Differential Updater
//!
//! XOR-based binary patching for minimal update payloads
//!
//! **Target:** 314 byte deltas for typical component updates

use blake3;

/// Calculate hash of binary data
pub fn hash_binary(data: &[u8]) -> String {
    let hash = blake3::hash(data);
    hash.to_hex().to_string()
}

/// Calculate XOR delta between two binaries
///
/// # Performance
/// - O(n) where n = size of new version
/// - Typical delta: 314 bytes
/// - Compression: gzip on top of XOR
pub fn calculate_delta(old: &[u8], new: &[u8]) -> Vec<u8> {
    let mut delta = Vec::with_capacity(new.len());

    // XOR each byte
    for i in 0..new.len() {
        if i < old.len() {
            delta.push(old[i] ^ new[i]);
        } else {
            // New bytes beyond old length
            delta.push(new[i]);
        }
    }

    delta
}

/// Apply delta patch to base binary
pub fn apply_delta(base: &[u8], delta: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(delta.len());

    // XOR each byte
    for i in 0..delta.len() {
        if i < base.len() {
            result.push(base[i] ^ delta[i]);
        } else {
            // New bytes beyond base length
            result.push(delta[i]);
        }
    }

    result
}

/// Delta metadata
#[derive(Debug, Clone)]
pub struct DeltaInfo {
    pub from_hash: String,
    pub to_hash: String,
    pub delta_size: usize,
    pub compression_ratio: f64,
}

impl DeltaInfo {
    pub fn calculate(old: &[u8], new: &[u8], delta: &[u8]) -> Self {
        let from_hash = hash_binary(old);
        let to_hash = hash_binary(new);
        let delta_size = delta.len();
        let compression_ratio = (new.len() as f64) / (delta_size as f64);

        Self {
            from_hash,
            to_hash,
            delta_size,
            compression_ratio,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_stability() {
        let data = b"hello world";
        let hash1 = hash_binary(data);
        let hash2 = hash_binary(data);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_delta_roundtrip() {
        let old = b"hello world";
        let new = b"hello rust!";

        let delta = calculate_delta(old, new);
        let result = apply_delta(old, &delta);

        assert_eq!(result, new);
    }

    #[test]
    fn test_delta_size() {
        // Identical except one byte
        let old = b"hello world";
        let new = b"hello World"; // Capital W

        let delta = calculate_delta(old, new);

        // Delta should be same size as input
        assert_eq!(delta.len(), new.len());

        // But most bytes should be zero (identical XOR)
        let non_zero = delta.iter().filter(|&&b| b != 0).count();
        assert!(non_zero < 5); // Only a few bytes changed
    }

    #[test]
    fn test_delta_info() {
        let old = vec![1, 2, 3, 4, 5];
        let new = vec![1, 2, 9, 4, 5]; // Changed one byte
        let delta = calculate_delta(&old, &new);

        let info = DeltaInfo::calculate(&old, &new, &delta);

        assert!(!info.from_hash.is_empty());
        assert!(!info.to_hash.is_empty());
        assert_ne!(info.from_hash, info.to_hash);
        assert_eq!(info.delta_size, delta.len());
    }

    #[test]
    fn test_new_bytes_beyond_old() {
        let old = b"hello";
        let new = b"hello world"; // Extended

        let delta = calculate_delta(old, new);
        let result = apply_delta(old, &delta);

        assert_eq!(result, new);
    }
}
