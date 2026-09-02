//! # Error Types
//!
//! Custom error types for XML parsing, entity expansion, DTD, XSD, and XPath evaluation.

use thiserror::Error;

/// Result alias returning [`XmlError`].
pub type Result<T> = std::result::Result<T, XmlError>;

/// Enum representing all XML library error conditions.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum XmlError {
    /// XML syntax or malformed token error.
    #[error("Syntax error at line {line}, col {col}: {message}")]
    SyntaxError {
        message: String,
        line: usize,
        col: usize,
    },

    /// Security limit threshold exceeded.
    #[error("Security limit exceeded: {0}")]
    SecurityLimitExceeded(String),

    /// Entity resolution or reference expansion error.
    #[error("Entity error: {0}")]
    EntityError(String),

    /// DTD constraint validation error.
    #[error("DTD Validation error: {0}")]
    DtdError(String),

    /// XSD Schema constraint validation error.
    #[error("XSD Validation error: {0}")]
    XsdError(String),

    /// XPath syntax or evaluation error.
    #[error("XPath error: {0}")]
    XPathError(String),

    /// Invalid Node operation error.
    #[error("Node error: {0}")]
    NodeError(String),

    /// I/O error.
    #[error("IO error: {0}")]
    Io(String),
}
