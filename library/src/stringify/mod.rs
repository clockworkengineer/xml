//! # Stringify Subsystem
//!
//! Handles serialization of [`Document`] DOM trees back to valid, formatted XML string output
//! and W3C Canonical XML (C14N).

pub mod canonical;
pub mod serializer;

pub use canonical::{CanonicalOptions, CanonicalSerializer};
pub use serializer::{SerializeOptions, XmlSerializer};

use crate::alloc_prelude::*;
use crate::document::Document;

/// Serializes a [`Document`] to a canonical XML string using default C14N options (without comments).
pub fn canonicalize(doc: &Document) -> String {
    CanonicalSerializer::canonicalize(doc, &CanonicalOptions::default())
}
