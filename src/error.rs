use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum XmlError {
    #[error("Syntax error at line {line}, col {col}: {message}")]
    SyntaxError {
        message: String,
        line: usize,
        col: usize,
    },

    #[error("Security limit exceeded: {0}")]
    SecurityLimitExceeded(String),

    #[error("Entity error: {0}")]
    EntityError(String),

    #[error("DTD Validation error: {0}")]
    DtdError(String),

    #[error("XSD Validation error: {0}")]
    XsdError(String),

    #[error("XPath error: {0}")]
    XPathError(String),

    #[error("IO error: {0}")]
    Io(String),

    #[error("Node error: {0}")]
    NodeError(String),
}

impl From<std::io::Error> for XmlError {
    fn from(err: std::io::Error) -> Self {
        XmlError::Io(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, XmlError>;
