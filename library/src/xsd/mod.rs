//! # XSD Subsystem
//!
//! Handles XSD schema document parsing and DOM structure/type validation.

pub mod validator;

pub use validator::{XsdElementRule, XsdRestriction, XsdValidator};
