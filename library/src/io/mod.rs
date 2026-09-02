//! # Input / Output Module
//!
//! Subsystem providing stream reader [`XmlSource`], stringifier destination [`XmlDestination`], and BOM encoding detection.

pub mod destination;
pub mod encoding;
pub mod source;

pub use destination::XmlDestination;
pub use encoding::Format;
pub use source::XmlSource;
