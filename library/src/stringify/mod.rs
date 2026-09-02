//! # Stringify Subsystem
//!
//! Handles serialization of [`Document`](crate::document::Document) DOM trees back to valid, formatted XML string output.

pub mod serializer;

pub use serializer::{SerializeOptions, XmlSerializer};
