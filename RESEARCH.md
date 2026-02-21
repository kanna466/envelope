# Research Notes

Surveying existing approaches to zero-copy serialization, content-addressed storage, and graph data models.

## Zero-Copy Serialization

### FlatBuffers (Google)
- Schema-compiled accessors for in-place reading
- Forward/backward compatible schema evolution
- No dynamic typing — schema required to interpret
- No references between buffers
- Read-only after construction (builder pattern)
- Offsets are relative (32-bit), buffer-local

### Cap'n Proto
- Similar to FlatBuffers but with RPC layer
- "Infinitely faster" — no encoding/decoding step
- Pointers are relative offsets
- Capabilities for RPC (object references, but network-level)
- Schema evolution via protocol

### Apache Arrow
- Columnar format for analytics
- Zero-copy across language boundaries
- Focus: tabular data, not graphs
- IPC format for shared memory / network

### rkyv (Rust)
- Archive/deserialize with zero-copy access
- Relative pointers within archive
- Derive-based, ergonomic API
- No cross-language support

**Gap:** All assume single-buffer scope. References beyond buffer boundary need something else.

## Content-Addressed Systems

### Git
- Objects identified by SHA-1/SHA-256 hash
- Trees reference blobs and other trees by hash
- Immutable — new commit = new hash
- Garbage collection of unreachable objects
- Pack files for efficient storage

### IPFS
- Content-addressed blocks (CIDs)
- DAG structure — objects reference others by CID
- Immutable data, mutable via IPNS
- Block size limits, chunking for large data

### Perkeep (Camlistore)
- Schema blobs (JSON) reference content blobs by hash
- Claims system for mutable properties
- Index system separate from storage

**Insight:** Content-addressing gives stable identity and natural deduplication, but makes cycles impossible and updates expensive (new hash cascades up).

## Graph Databases / Models

### RDF / Triple Stores
- Subject-predicate-object triples
- URIs for identity
- SPARQL for queries
- Schema via RDFS/OWL
- Verbose, but very general

### Property Graphs (Neo4j, etc.)
- Nodes with properties
- Edges with types and properties
- Query languages (Cypher, Gremlin)
- Mutable by design

### SQLite / Relational
- Tables as relations
- Foreign keys for references
- ACID transactions
- Indexes for query acceleration

**Insight:** Most graph systems assume mutability. Immutable graphs are less explored outside version control.

## Relevant Patterns

### Event Sourcing
- State as sequence of immutable events
- Current state = fold over events
- Natural audit trail
- Similar immutability philosophy

### Persistent Data Structures
- Structural sharing for efficient updates
- Copy-on-write semantics
- Hash array mapped tries (HAMT)
- Clojure, Immutable.js

### Memory-Mapped Files
- OS pages file into memory on demand
- Same bytes on disk and in memory
- Enables zero-copy if format is compatible

## Open Questions

1. **Identity:** Hash-based (content-addressed) vs UUID (location-independent) vs local offset?
2. **References:** Inline hash? Indirection table? Symbolic names?
3. **Schema:** Per-object type tag? External registry? Self-describing?
4. **Indexes:** Part of the storage format? Separate structures?
5. **Garbage collection:** Reference counting? Tracing? External process?
