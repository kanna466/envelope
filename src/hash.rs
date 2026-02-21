//! Content-addressed hashing for envelopes

use sha2::{Sha256, Digest};
use std::fmt;

/// A 256-bit content hash (SHA-256)
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hash256([u8; 32]);

impl Hash256 {
    /// Create a hash from raw bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
    
    /// Get the raw bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
    
    /// Hash arbitrary data
    pub fn hash(data: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&result);
        Self(bytes)
    }
    
    /// Hash multiple chunks of data
    pub fn hash_parts<'a>(parts: impl IntoIterator<Item = &'a [u8]>) -> Self {
        let mut hasher = Sha256::new();
        for part in parts {
            hasher.update(part);
        }
        let result = hasher.finalize();
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&result);
        Self(bytes)
    }
    
    /// Create from hex string
    pub fn from_hex(s: &str) -> Result<Self, hex::FromHexError> {
        let bytes = hex::decode(s)?;
        if bytes.len() != 32 {
            return Err(hex::FromHexError::InvalidStringLength);
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Self(arr))
    }
    
    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
    
    /// Short hex for display (first 8 chars)
    pub fn short(&self) -> String {
        self.to_hex()[..8].to_string()
    }
}

impl fmt::Display for Hash256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl fmt::Debug for Hash256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hash256({})", self.short())
    }
}

impl Default for Hash256 {
    fn default() -> Self {
        Self([0u8; 32])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hash_deterministic() {
        let data = b"hello world";
        let h1 = Hash256::hash(data);
        let h2 = Hash256::hash(data);
        assert_eq!(h1, h2);
    }
    
    #[test]
    fn test_hash_differs() {
        let h1 = Hash256::hash(b"hello");
        let h2 = Hash256::hash(b"world");
        assert_ne!(h1, h2);
    }
    
    #[test]
    fn test_hex_roundtrip() {
        let h = Hash256::hash(b"test");
        let hex = h.to_hex();
        let h2 = Hash256::from_hex(&hex).unwrap();
        assert_eq!(h, h2);
    }
}
