//! Envelope: Zero-copy serialization with metadata for graphs
//!
//! This library explores adding an envelope layer on top of zero-copy
//! serialization formats like FlatBuffers, providing:
//!
//! - Dynamic typing (self-describing objects)
//! - References between objects (graph structures)
//! - Index fields for queryability
//! - Version chains for immutable updates

pub mod hash;
pub mod envelope;
pub mod store;
pub mod error;

pub use crate::envelope::{Envelope, EnvelopeBuilder};
pub use crate::hash::Hash256;
pub use crate::store::Store;
pub use crate::error::Error;

pub type Result<T> = std::result::Result<T, Error>;
