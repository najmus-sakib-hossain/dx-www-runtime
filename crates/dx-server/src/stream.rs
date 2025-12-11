//! # Stream Module - Binary Chunking
//!
//! Implements HTTP/2 streaming for HTIP payloads
//!
//! **Strategy:** Send chunks in optimal order to minimize LCP

use bytes::Bytes;

/// Binary stream chunk types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkType {
    /// Global template dictionary (layouts)
    Templates = 1,
    /// Base memory snapshot (runtime state)
    BaseMemory = 2,
    /// Page-specific data
    PageData = 3,
}

/// A chunk of binary data to stream
#[derive(Debug, Clone)]
pub struct BinaryChunk {
    pub chunk_type: ChunkType,
    pub data: Bytes,
    pub priority: u8, // 0 = highest priority
}

impl BinaryChunk {
    pub fn new(chunk_type: ChunkType, data: Vec<u8>) -> Self {
        let priority = match chunk_type {
            ChunkType::Templates => 0,  // Highest - needed first
            ChunkType::BaseMemory => 1, // Second - runtime init
            ChunkType::PageData => 2,   // Third - page content
        };

        Self {
            chunk_type,
            data: Bytes::from(data),
            priority,
        }
    }
}

/// Stream manager for binary payloads
#[derive(Default)]
pub struct BinaryStreamer {
    chunks: Vec<BinaryChunk>,
}

impl BinaryStreamer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a chunk to the stream
    pub fn add_chunk(&mut self, chunk: BinaryChunk) {
        self.chunks.push(chunk);
    }

    /// Get chunks in optimal streaming order
    pub fn get_ordered_chunks(&self) -> Vec<&BinaryChunk> {
        let mut sorted = self.chunks.iter().collect::<Vec<_>>();
        sorted.sort_by_key(|c| c.priority);
        sorted
    }

    /// Split a binary payload into chunks
    pub fn split_binary(binary: &[u8]) -> Vec<BinaryChunk> {
        // TODO: Implement proper binary splitting
        // For now, return single chunk
        vec![BinaryChunk::new(ChunkType::PageData, binary.to_vec())]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_priority() {
        let chunk1 = BinaryChunk::new(ChunkType::PageData, vec![1, 2, 3]);
        let chunk2 = BinaryChunk::new(ChunkType::Templates, vec![4, 5, 6]);
        let chunk3 = BinaryChunk::new(ChunkType::BaseMemory, vec![7, 8, 9]);

        assert_eq!(chunk1.priority, 2);
        assert_eq!(chunk2.priority, 0);
        assert_eq!(chunk3.priority, 1);
    }

    #[test]
    fn test_chunk_ordering() {
        let mut streamer = BinaryStreamer::new();

        streamer.add_chunk(BinaryChunk::new(ChunkType::PageData, vec![1]));
        streamer.add_chunk(BinaryChunk::new(ChunkType::Templates, vec![2]));
        streamer.add_chunk(BinaryChunk::new(ChunkType::BaseMemory, vec![3]));

        let ordered = streamer.get_ordered_chunks();

        assert_eq!(ordered[0].chunk_type, ChunkType::Templates);
        assert_eq!(ordered[1].chunk_type, ChunkType::BaseMemory);
        assert_eq!(ordered[2].chunk_type, ChunkType::PageData);
    }
}
