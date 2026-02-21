//! Simple indexing for envelope queries
//! 
//! This is a naive in-memory implementation for exploration.
//! Production would use proper B-trees, LSM trees, etc.

use crate::envelope::{Envelope, IndexValue};
use crate::hash::Hash256;
use std::collections::{HashMap, HashSet};

/// A simple index supporting basic queries
#[derive(Debug, Default)]
pub struct Index {
    /// type_hash -> set of envelope hashes
    by_type: HashMap<Hash256, HashSet<Hash256>>,
    
    /// (field_name, string_value) -> set of envelope hashes
    by_string_field: HashMap<(String, String), HashSet<Hash256>>,
    
    /// relationship_type -> target_hash -> set of source envelope hashes
    /// This is the reverse index: "who references X?"
    by_relationship: HashMap<String, HashMap<Hash256, HashSet<Hash256>>>,
    
    /// target_hash -> set of source hashes (all relationship types)
    references_to: HashMap<Hash256, HashSet<Hash256>>,
}

impl Index {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Index an envelope
    pub fn add(&mut self, hash: Hash256, envelope: &Envelope) {
        // Index by type
        self.by_type
            .entry(envelope.type_hash)
            .or_default()
            .insert(hash);
        
        // Index string fields
        for (key, value) in &envelope.index {
            if let IndexValue::String(s) = value {
                self.by_string_field
                    .entry((key.clone(), s.clone()))
                    .or_default()
                    .insert(hash);
            }
        }
        
        // Index relationships (reverse index)
        for rel in &envelope.relationships {
            self.by_relationship
                .entry(rel.rel_type.clone())
                .or_default()
                .entry(rel.target)
                .or_default()
                .insert(hash);
            
            self.references_to
                .entry(rel.target)
                .or_default()
                .insert(hash);
        }
    }
    
    /// Remove an envelope from the index
    pub fn remove(&mut self, hash: &Hash256, envelope: &Envelope) {
        // Remove from type index
        if let Some(set) = self.by_type.get_mut(&envelope.type_hash) {
            set.remove(hash);
        }
        
        // Remove from string field indexes
        for (key, value) in &envelope.index {
            if let IndexValue::String(s) = value {
                if let Some(set) = self.by_string_field.get_mut(&(key.clone(), s.clone())) {
                    set.remove(hash);
                }
            }
        }
        
        // Remove from relationship indexes
        for rel in &envelope.relationships {
            if let Some(type_map) = self.by_relationship.get_mut(&rel.rel_type) {
                if let Some(set) = type_map.get_mut(&rel.target) {
                    set.remove(hash);
                }
            }
            if let Some(set) = self.references_to.get_mut(&rel.target) {
                set.remove(hash);
            }
        }
    }
    
    /// Find all envelopes of a given type
    pub fn by_type(&self, type_hash: &Hash256) -> impl Iterator<Item = &Hash256> {
        self.by_type
            .get(type_hash)
            .into_iter()
            .flat_map(|s| s.iter())
    }
    
    /// Find envelopes where field == value
    pub fn by_field(&self, field: &str, value: &str) -> impl Iterator<Item = &Hash256> {
        self.by_string_field
            .get(&(field.to_string(), value.to_string()))
            .into_iter()
            .flat_map(|s| s.iter())
    }
    
    /// Find envelopes that reference a target (reverse lookup)
    pub fn references_to(&self, target: &Hash256) -> impl Iterator<Item = &Hash256> {
        self.references_to
            .get(target)
            .into_iter()
            .flat_map(|s| s.iter())
    }
    
    /// Find envelopes with a specific relationship to a target
    pub fn by_relationship(&self, rel_type: &str, target: &Hash256) -> impl Iterator<Item = &Hash256> {
        self.by_relationship
            .get(rel_type)
            .and_then(|m| m.get(target))
            .into_iter()
            .flat_map(|s| s.iter())
    }
}

/// A store with integrated indexing
#[derive(Debug, Default)]
pub struct IndexedStore {
    store: crate::store::Store,
    index: Index,
}

impl IndexedStore {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Store an envelope and update indexes
    pub fn put(&mut self, envelope: &Envelope) -> crate::Result<Hash256> {
        let hash = self.store.put(envelope)?;
        self.index.add(hash, envelope);
        Ok(hash)
    }
    
    /// Retrieve an envelope by hash
    pub fn get(&self, hash: &Hash256) -> crate::Result<Envelope> {
        self.store.get(hash)
    }
    
    /// Check if an object exists
    pub fn contains(&self, hash: &Hash256) -> bool {
        self.store.contains(hash)
    }
    
    /// Query by type
    pub fn query_by_type(&self, type_hash: &Hash256) -> Vec<Hash256> {
        self.index.by_type(type_hash).copied().collect()
    }
    
    /// Query by field value
    pub fn query_by_field(&self, field: &str, value: &str) -> Vec<Hash256> {
        self.index.by_field(field, value).copied().collect()
    }
    
    /// Query reverse references
    pub fn query_references_to(&self, target: &Hash256) -> Vec<Hash256> {
        self.index.references_to(target).copied().collect()
    }
    
    /// Number of objects
    pub fn len(&self) -> usize {
        self.store.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_indexed_store() {
        let mut store = IndexedStore::new();
        
        let author_type = Hash256::hash(b"Author");
        let post_type = Hash256::hash(b"Post");
        
        // Create author
        let author = Envelope::builder(author_type, b"Alice".to_vec())
            .index("name", "Alice")
            .build();
        let author_hash = store.put(&author).unwrap();
        
        // Create posts by that author
        let post1 = Envelope::builder(post_type, b"Post 1".to_vec())
            .index("title", "First Post")
            .relationship("author", author_hash)
            .build();
        let post1_hash = store.put(&post1).unwrap();
        
        let post2 = Envelope::builder(post_type, b"Post 2".to_vec())
            .index("title", "Second Post")
            .relationship("author", author_hash)
            .build();
        let post2_hash = store.put(&post2).unwrap();
        
        // Query by type
        let authors: Vec<_> = store.query_by_type(&author_type);
        assert_eq!(authors.len(), 1);
        assert!(authors.contains(&author_hash));
        
        let posts: Vec<_> = store.query_by_type(&post_type);
        assert_eq!(posts.len(), 2);
        
        // Query by field
        let alice_results: Vec<_> = store.query_by_field("name", "Alice");
        assert_eq!(alice_results.len(), 1);
        
        // Reverse query: who references the author?
        let referencing: Vec<_> = store.query_references_to(&author_hash);
        assert_eq!(referencing.len(), 2);
        assert!(referencing.contains(&post1_hash));
        assert!(referencing.contains(&post2_hash));
    }
}
