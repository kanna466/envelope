//! Error types for envelope operations

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid envelope: {0}")]
    InvalidEnvelope(String),
    
    #[error("Hash mismatch: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },
    
    #[error("Object not found: {0}")]
    NotFound(String),
    
    #[error("Storage error: {0}")]
    Storage(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
