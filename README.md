# Envelope

An exploration of zero-copy data representation with envelope metadata for graph structures, dynamic typing, and database-like features.

## Motivation

Zero-copy serialization libraries (FlatBuffers, Cap'n Proto) provide efficient in-place access to structured data. But they intentionally omit:

- **Dynamic typing** — you need the schema to interpret data
- **References** — no way to link objects into graphs
- **Self-description** — data isn't introspectable without external context
- **Indexation** — no database-like query capabilities

The thesis: keep payloads zero-copy (use existing libraries), add an **envelope** layer that provides the missing metadata.

## Design Goals

1. **Same representation everywhere** — storage, memory, network
2. **Read-only payloads** — mutations create new versions
3. **Graph topology** — objects can reference other objects
4. **Self-describing** — type and structure discoverable from envelope
5. **Indexable** — envelope carries enough metadata for efficient queries

## Research

See [RESEARCH.md](./RESEARCH.md) for notes on existing systems.

## Design

See [DESIGN.md](./DESIGN.md) for envelope format exploration.

## Status

Early exploration. Nothing works yet.
