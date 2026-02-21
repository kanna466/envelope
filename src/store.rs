//! Content-addressed storage for envelopes

use crate::envelope::Envelope;
use crate::hash::Hash256;
use crate::error::Error;
use crate::Result;
use std::collections::HashMap;
use std::path::Path;

/// A simple in-memory content-addressed store
/// 
/// For exploration only. Production would use mmap'd files.
#[derive(Debug, Default)]
pub struct Store {
    /// Hash -> serialized envelope
    objects: HashMap<Hash256, Vec<u8>>,
}

impl Store {
    /// Create a new empty store
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Store an envelope, returning its hash
    pub fn put(&mut self, envelope: &Envelope) -> Result<Hash256> {
        let bytes = self.serialize(envelope)?;
        let hash = Hash256::hash(&bytes);
        self.objects.insert(hash, bytes);
        Ok(hash)
    }
    
    /// Retrieve an envelope by hash
    pub fn get(&self, hash: &Hash256) -> Result<Envelope> {
        let bytes = self.objects
            .get(hash)
            .ok_or_else(|| Error::NotFound(hash.to_hex()))?;
        self.deserialize(bytes)
    }
    
    /// Check if an object exists
    pub fn contains(&self, hash: &Hash256) -> bool {
        self.objects.contains_key(hash)
    }
    
    /// Number of objects in the store
    pub fn len(&self) -> usize {
        self.objects.len()
    }
    
    /// Check if store is empty
    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }
    
    /// List all hashes in the store
    pub fn hashes(&self) -> impl Iterator<Item = &Hash256> {
        self.objects.keys()
    }
    
    // Serialization - simple format for now, would use FlatBuffers in production
    fn serialize(&self, envelope: &Envelope) -> Result<Vec<u8>> {
        // Simple binary format:
        // [type_hash: 32] [type_name_len: 4] [type_name: N]
        // [rel_count: 4] [rels...]
        // [index_count: 4] [index...]
        // [previous: 1 + 32?] [created_at: 1 + 8?]
        // [payload_len: 4] [payload: N]
        
        let mut buf = Vec::new();
        
        // Type hash
        buf.extend_from_slice(envelope.type_hash.as_bytes());
        
        // Type name (length-prefixed)
        match &envelope.type_name {
            Some(name) => {
                buf.extend_from_slice(&(name.len() as u32).to_le_bytes());
                buf.extend_from_slice(name.as_bytes());
            }
            None => {
                buf.extend_from_slice(&0u32.to_le_bytes());
            }
        }
        
        // Relationships
        buf.extend_from_slice(&(envelope.relationships.len() as u32).to_le_bytes());
        for rel in &envelope.relationships {
            buf.extend_from_slice(&(rel.rel_type.len() as u32).to_le_bytes());
            buf.extend_from_slice(rel.rel_type.as_bytes());
            buf.extend_from_slice(rel.target.as_bytes());
        }
        
        // Index fields (simplified - strings only for now)
        let string_index: Vec<_> = envelope.index.iter()
            .filter_map(|(k, v)| {
                match v {
                    crate::envelope::IndexValue::String(s) => Some((k, s)),
                    _ => None, // Skip non-string for now
                }
            })
            .collect();
        
        buf.extend_from_slice(&(string_index.len() as u32).to_le_bytes());
        for (key, value) in string_index {
            buf.extend_from_slice(&(key.len() as u32).to_le_bytes());
            buf.extend_from_slice(key.as_bytes());
            buf.extend_from_slice(&(value.len() as u32).to_le_bytes());
            buf.extend_from_slice(value.as_bytes());
        }
        
        // Previous (optional)
        match &envelope.previous {
            Some(hash) => {
                buf.push(1);
                buf.extend_from_slice(hash.as_bytes());
            }
            None => {
                buf.push(0);
            }
        }
        
        // Created at (optional)
        match envelope.created_at {
            Some(ts) => {
                buf.push(1);
                buf.extend_from_slice(&ts.to_le_bytes());
            }
            None => {
                buf.push(0);
            }
        }
        
        // Payload
        buf.extend_from_slice(&(envelope.payload.len() as u32).to_le_bytes());
        buf.extend_from_slice(&envelope.payload);
        
        Ok(buf)
    }
    
    fn deserialize(&self, bytes: &[u8]) -> Result<Envelope> {
        let mut cursor = 0;
        
        let read_u32 = |cursor: &mut usize| -> u32 {
            let v = u32::from_le_bytes(bytes[*cursor..*cursor+4].try_into().unwrap());
            *cursor += 4;
            v
        };
        
        let read_i64 = |cursor: &mut usize| -> i64 {
            let v = i64::from_le_bytes(bytes[*cursor..*cursor+8].try_into().unwrap());
            *cursor += 8;
            v
        };
        
        let read_hash = |cursor: &mut usize| -> Hash256 {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&bytes[*cursor..*cursor+32]);
            *cursor += 32;
            Hash256::from_bytes(arr)
        };
        
        let read_string = |cursor: &mut usize| -> String {
            let len = read_u32(cursor) as usize;
            let s = String::from_utf8_lossy(&bytes[*cursor..*cursor+len]).to_string();
            *cursor += len;
            s
        };
        
        // Type hash
        let type_hash = read_hash(&mut cursor);
        
        // Type name
        let type_name_len = read_u32(&mut cursor);
        let type_name = if type_name_len > 0 {
            let name = String::from_utf8_lossy(&bytes[cursor..cursor+(type_name_len as usize)]).to_string();
            cursor += type_name_len as usize;
            Some(name)
        } else {
            None
        };
        
        // Relationships
        let rel_count = read_u32(&mut cursor) as usize;
        let mut relationships = Vec::with_capacity(rel_count);
        for _ in 0..rel_count {
            let rel_type = read_string(&mut cursor);
            let target = read_hash(&mut cursor);
            relationships.push(crate::envelope::Relationship::new(rel_type, target));
        }
        
        // Index
        let idx_count = read_u32(&mut cursor) as usize;
        let mut index = HashMap::with_capacity(idx_count);
        for _ in 0..idx_count {
            let key = read_string(&mut cursor);
            let value = read_string(&mut cursor);
            index.insert(key, crate::envelope::IndexValue::String(value));
        }
        
        // Previous
        let has_previous = bytes[cursor] == 1;
        cursor += 1;
        let previous = if has_previous {
            Some(read_hash(&mut cursor))
        } else {
            None
        };
        
        // Created at
        let has_created = bytes[cursor] == 1;
        cursor += 1;
        let created_at = if has_created {
            Some(read_i64(&mut cursor))
        } else {
            None
        };
        
        // Payload
        let payload_len = read_u32(&mut cursor) as usize;
        let payload = bytes[cursor..cursor+payload_len].to_vec();
        
        Ok(Envelope {
            type_hash,
            type_name,
            relationships,
            index,
            previous,
            created_at,
            payload,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_store_roundtrip() {
        let mut store = Store::new();
        
        let type_hash = Hash256::hash(b"TestType");
        let envelope = Envelope::builder(type_hash, vec![1, 2, 3, 4])
            .type_name("TestType")
            .index("title", "Hello")
            .build();
        
        let hash = store.put(&envelope).unwrap();
        let retrieved = store.get(&hash).unwrap();
        
        assert_eq!(retrieved.type_name, envelope.type_name);
        assert_eq!(retrieved.payload, envelope.payload);
    }
    
    #[test]
    fn test_store_deduplication() {
        let mut store = Store::new();
        
        let type_hash = Hash256::hash(b"TestType");
        let payload = vec![1, 2, 3, 4];
        
        let env1 = Envelope::builder(type_hash, payload.clone()).build();
        let env2 = Envelope::builder(type_hash, payload).build();
        
        let hash1 = store.put(&env1).unwrap();
        let hash2 = store.put(&env2).unwrap();
        
        // Same content = same hash = deduplicated
        assert_eq!(hash1, hash2);
        assert_eq!(store.len(), 1);
    }
}
