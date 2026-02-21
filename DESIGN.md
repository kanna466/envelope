# Envelope Design Exploration

## Core Concept

An **envelope** wraps a zero-copy payload, adding:
- Identity (how to reference this object)
- Type (how to interpret the payload)
- Relationships (what this object connects to)
- Index hints (queryable metadata without parsing payload)

```
┌─────────────────────────────────────┐
│ ENVELOPE                            │
│ ┌─────────────────────────────────┐ │
│ │ Identity (hash, uuid, etc.)     │ │
│ ├─────────────────────────────────┤ │
│ │ Type descriptor                 │ │
│ ├─────────────────────────────────┤ │
│ │ Relationships (edges out)       │ │
│ ├─────────────────────────────────┤ │
│ │ Index fields                    │ │
│ ├─────────────────────────────────┤ │
│ │ Payload (FlatBuffer, etc.)      │ │
│ └─────────────────────────────────┘ │
└─────────────────────────────────────┘
```

## Identity Models

### Option A: Content Hash
- Identity = hash(envelope_without_identity)
- Natural deduplication
- Immutable by construction
- **Con:** Can't have cycles, updates cascade

### Option B: UUID
- Identity = random or structured UUID
- Allows mutable semantics (same ID, new version)
- Cycles possible
- **Con:** No natural deduplication, need versioning

### Option C: Hybrid
- Content hash for payload
- Separate object ID for envelope
- Reference by object ID, verify by content hash
- Flexible but complex

**Leaning toward:** Content hash for simplicity, accept the DAG constraint.

## Type System

Need to identify payload type without external schema.

### Option A: Type URI
- `type: "org.flatbuffers.Monster"`
- Globally unique, human-readable
- Schema resolved externally

### Option B: Type Hash
- `type: hash(schema)`
- Schema is content-addressed too
- Self-contained

### Option C: Inline Schema
- Type descriptor embedded in envelope
- Fully self-describing
- Larger envelopes

**Leaning toward:** Type hash with schema objects stored as envelopes themselves. Self-describing when you have the schema envelope.

## Relationships

Objects reference other objects. How?

### Edges as Envelope Fields
```
relationships: [
  { rel: "author", target: <hash> },
  { rel: "parent", target: <hash> },
  { rel: "tags", target: [<hash>, <hash>] }
]
```
- Explicit, queryable
- Separate from payload semantics
- Enables graph queries without payload parsing

### Edge Typing
- Relationships have types too
- `rel: "authored_by"` vs `rel: "approved_by"`
- Could reference relationship type envelopes

### Bidirectional?
- Store forward edges only (target doesn't know about source)
- Or maintain back-references (expensive to update)
- **Decision:** Forward only, indexes for reverse lookups

## Index Fields

Envelope-level fields for query acceleration:

```
index: {
  created: 1708523400,
  author: <hash>,
  tags: ["rust", "serialization"],
  size: 4096
}
```

- Queryable without payload access
- Schema defined by type
- **Trade-off:** Redundancy vs query speed

## Versioning

If we use content hashes, "updating" an object means:
1. Create new envelope with new payload
2. New hash, new identity
3. References to old hash still work
4. Need a way to say "this supersedes that"

### Version Chain
```
previous: <hash_of_previous_version>
```

Follow the chain for history. Latest = head of chain (tracked externally).

### Tombstones
Special envelope type marking deletion:
```
type: "envelope/tombstone"
supersedes: <hash>
reason: "deleted by user"
```

## Wire Format

Envelope itself needs to be zero-copy readable.

### FlatBuffer Envelope
- Envelope is a FlatBuffer
- Payload is nested bytes
- Uniform tooling

### Custom Format
- Minimal header: magic, version, sizes
- Fixed fields in known positions
- Variable fields with offset table
- Payload at end

**Leaning toward:** FlatBuffer for envelope too. Dogfooding.

## Example

A blog post envelope:

```
Envelope {
  id: sha256("..."),
  type: sha256(<BlogPost schema>),
  relationships: [
    { rel: "author", target: sha256(<Author envelope>) },
    { rel: "tags", target: [sha256(<Tag1>), sha256(<Tag2>)] }
  ],
  index: {
    title: "Zero-Copy Dreams",
    created: 1708523400,
    word_count: 1500
  },
  previous: null,
  payload: <FlatBuffer bytes for BlogPost>
}
```

## Storage Layout

Multiple envelopes in a file/database:

### Append-Only Log
- Just append envelopes
- Indexes maintained separately
- Compaction removes superseded versions

### Content-Addressed Store
- Hash → envelope bytes
- Simple key-value storage
- Deduplication automatic

### Hybrid
- Hot data in structured store
- Cold data in content-addressed blobs
- Move between tiers

## Next Steps

1. Define envelope FlatBuffer schema
2. Prototype in Rust
3. Build simple store (append log + index)
4. Test with a toy use case

## Open Questions

1. How big should index fields be allowed to get?
2. Should relationships be typed with schemas too?
3. Compression: per-envelope or in batches?
4. How to handle large payloads (chunking)?
5. Signing/authentication of envelopes?
