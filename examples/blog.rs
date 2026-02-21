//! Example: A simple blog with posts, authors, and tags
//! 
//! Demonstrates how envelopes connect into a graph.

use envelope::{Envelope, Hash256, Store};

fn main() {
    let mut store = Store::new();
    
    // Create a schema "hash" (in practice, this would be the hash of a real schema)
    let author_type = Hash256::hash(b"schema:Author");
    let tag_type = Hash256::hash(b"schema:Tag");
    let post_type = Hash256::hash(b"schema:BlogPost");
    
    // Create an author
    // In a real system, the payload would be a FlatBuffer
    let author_payload = b"Alice".to_vec();
    let author = Envelope::builder(author_type, author_payload)
        .type_name("Author")
        .index("name", "Alice")
        .index("email", "alice@example.com")
        .build();
    let author_hash = store.put(&author).unwrap();
    println!("Author: {}", author_hash.short());
    
    // Create some tags
    let rust_tag = Envelope::builder(tag_type, b"rust".to_vec())
        .type_name("Tag")
        .index("name", "rust")
        .build();
    let rust_hash = store.put(&rust_tag).unwrap();
    println!("Tag 'rust': {}", rust_hash.short());
    
    let serialization_tag = Envelope::builder(tag_type, b"serialization".to_vec())
        .type_name("Tag")
        .index("name", "serialization")
        .build();
    let ser_hash = store.put(&serialization_tag).unwrap();
    println!("Tag 'serialization': {}", ser_hash.short());
    
    // Create a blog post that references author and tags
    let post_payload = b"Zero-copy serialization is the future...".to_vec();
    let post = Envelope::builder(post_type, post_payload)
        .type_name("BlogPost")
        .index("title", "Zero-Copy Dreams")
        .index("word_count", "1500")
        .relationship("author", author_hash)
        .relationship("tag", rust_hash)
        .relationship("tag", ser_hash)
        .created_at(1708523400)
        .build();
    let post_hash = store.put(&post).unwrap();
    println!("Post: {}", post_hash.short());
    
    // Retrieve and inspect
    println!("\n--- Retrieving post ---");
    let retrieved = store.get(&post_hash).unwrap();
    println!("Type: {:?}", retrieved.type_name);
    println!("Relationships:");
    for rel in &retrieved.relationships {
        println!("  {} -> {}", rel.rel_type, rel.target.short());
    }
    println!("Index fields:");
    for (k, v) in &retrieved.index {
        println!("  {}: {:?}", k, v);
    }
    
    // Update the post (creates new version)
    println!("\n--- Updating post ---");
    let updated_payload = b"Zero-copy serialization is definitely the future...".to_vec();
    let updated_post = Envelope::builder(post_type, updated_payload)
        .type_name("BlogPost")
        .index("title", "Zero-Copy Dreams")
        .index("word_count", "1600")
        .relationship("author", author_hash)
        .relationship("tag", rust_hash)
        .relationship("tag", ser_hash)
        .previous(post_hash)  // Link to previous version
        .created_at(1708609800)
        .build();
    let updated_hash = store.put(&updated_post).unwrap();
    println!("Updated post: {} (previous: {})", updated_hash.short(), post_hash.short());
    
    // Both versions exist
    println!("\nStore contains {} objects", store.len());
    
    // The old hash still works
    let old = store.get(&post_hash).unwrap();
    println!("Old version still accessible: {:?}", old.index.get("word_count"));
}
