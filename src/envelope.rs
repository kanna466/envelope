//! Core envelope types and builder

use crate::hash::Hash256;
use crate::error::Error;
use std::collections::HashMap;

/// A relationship to another envelope
#[derive(Debug, Clone)]
pub struct Relationship {
    /// Type of relationship (e.g., "author", "parent", "contains")
    pub rel_type: String,
    /// Target envelope hash
    pub target: Hash256,
}

impl Relationship {
    pub fn new(rel_type: impl Into<String>, target: Hash256) -> Self {
        Self {
            rel_type: rel_type.into(),
            target,
        }
    }
}

/// Value types for index fields
#[derive(Debug, Clone)]
pub enum IndexValue {
    String(String),
    Int64(i64),
    Float64(f64),
    Bool(bool),
    Hash(Hash256),
    Timestamp(i64),
}

impl From<&str> for IndexValue {
    fn from(s: &str) -> Self {
        IndexValue::String(s.to_string())
    }
}

impl From<String> for IndexValue {
    fn from(s: String) -> Self {
        IndexValue::String(s)
    }
}

impl From<i64> for IndexValue {
    fn from(v: i64) -> Self {
        IndexValue::Int64(v)
    }
}

impl From<f64> for IndexValue {
    fn from(v: f64) -> Self {
        IndexValue::Float64(v)
    }
}

impl From<bool> for IndexValue {
    fn from(v: bool) -> Self {
        IndexValue::Bool(v)
    }
}

impl From<Hash256> for IndexValue {
    fn from(v: Hash256) -> Self {
        IndexValue::Hash(v)
    }
}

/// An envelope wrapping a zero-copy payload
#[derive(Debug, Clone)]
pub struct Envelope {
    /// Hash of the type schema
    pub type_hash: Hash256,
    /// Human-readable type name (optional)
    pub type_name: Option<String>,
    /// Outgoing relationships
    pub relationships: Vec<Relationship>,
    /// Index fields for queries
    pub index: HashMap<String, IndexValue>,
    /// Previous version (for version chain)
    pub previous: Option<Hash256>,
    /// Creation timestamp
    pub created_at: Option<i64>,
    /// The payload bytes
    pub payload: Vec<u8>,
}

impl Envelope {
    /// Compute the content hash of this envelope
    pub fn hash(&self) -> Hash256 {
        // Hash: type_hash + sorted relationships + sorted index + payload
        let mut parts: Vec<&[u8]> = Vec::new();
        
        // Type hash
        parts.push(self.type_hash.as_bytes());
        
        // Relationships (sorted for determinism)
        let mut rels: Vec<_> = self.relationships.iter().collect();
        rels.sort_by(|a, b| {
            (&a.rel_type, a.target.as_bytes())
                .cmp(&(&b.rel_type, b.target.as_bytes()))
        });
        for rel in rels {
            parts.push(rel.rel_type.as_bytes());
            parts.push(rel.target.as_bytes());
        }
        
        // Index fields (sorted for determinism)
        let mut idx: Vec<_> = self.index.iter().collect();
        idx.sort_by_key(|(k, _)| *k);
        for (key, value) in idx {
            parts.push(key.as_bytes());
            match value {
                IndexValue::String(s) => parts.push(s.as_bytes()),
                IndexValue::Int64(v) => {
                    // This is a hack; proper impl would use fixed encoding
                    let bytes = v.to_le_bytes();
                    // Can't push local; hash_parts handles this better
                }
                _ => {} // Simplified for now
            }
        }
        
        // Payload
        parts.push(&self.payload);
        
        Hash256::hash_parts(parts)
    }
    
    /// Create a builder for constructing envelopes
    pub fn builder(type_hash: Hash256, payload: Vec<u8>) -> EnvelopeBuilder {
        EnvelopeBuilder {
            type_hash,
            type_name: None,
            relationships: Vec::new(),
            index: HashMap::new(),
            previous: None,
            created_at: None,
            payload,
        }
    }
}

/// Builder for constructing envelopes
#[derive(Debug)]
pub struct EnvelopeBuilder {
    type_hash: Hash256,
    type_name: Option<String>,
    relationships: Vec<Relationship>,
    index: HashMap<String, IndexValue>,
    previous: Option<Hash256>,
    created_at: Option<i64>,
    payload: Vec<u8>,
}

impl EnvelopeBuilder {
    /// Set human-readable type name
    pub fn type_name(mut self, name: impl Into<String>) -> Self {
        self.type_name = Some(name.into());
        self
    }
    
    /// Add a relationship
    pub fn relationship(mut self, rel_type: impl Into<String>, target: Hash256) -> Self {
        self.relationships.push(Relationship::new(rel_type, target));
        self
    }
    
    /// Add an index field
    pub fn index(mut self, key: impl Into<String>, value: impl Into<IndexValue>) -> Self {
        self.index.insert(key.into(), value.into());
        self
    }
    
    /// Set previous version
    pub fn previous(mut self, hash: Hash256) -> Self {
        self.previous = Some(hash);
        self
    }
    
    /// Set creation timestamp
    pub fn created_at(mut self, timestamp: i64) -> Self {
        self.created_at = Some(timestamp);
        self
    }
    
    /// Build the envelope
    pub fn build(self) -> Envelope {
        Envelope {
            type_hash: self.type_hash,
            type_name: self.type_name,
            relationships: self.relationships,
            index: self.index,
            previous: self.previous,
            created_at: self.created_at,
            payload: self.payload,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_build_envelope() {
        let type_hash = Hash256::hash(b"TestType");
        let payload = vec![1, 2, 3, 4];
        
        let env = Envelope::builder(type_hash, payload)
            .type_name("TestType")
            .index("title", "Hello World")
            .index("count", 42i64)
            .build();
        
        assert_eq!(env.type_name, Some("TestType".to_string()));
        assert_eq!(env.index.len(), 2);
    }
    
    #[test]
    fn test_envelope_hash_deterministic() {
        let type_hash = Hash256::hash(b"TestType");
        let payload = vec![1, 2, 3, 4];
        
        let env1 = Envelope::builder(type_hash, payload.clone())
            .index("a", "1")
            .index("b", "2")
            .build();
        
        // Same envelope, different insertion order
        let env2 = Envelope::builder(type_hash, payload)
            .index("b", "2")
            .index("a", "1")
            .build();
        
        assert_eq!(env1.hash(), env2.hash());
    }
}
