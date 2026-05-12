use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    EmptyField(&'static str),
    InvalidInput(&'static str),
    InvalidTransition,
    NotFound(&'static str),
    Forbidden(&'static str),
    Unauthorized(&'static str),
}

impl Display for DomainError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DomainError::EmptyField(field) => write!(f, "field '{field}' must not be empty"),
            DomainError::InvalidInput(field) => write!(f, "invalid value for '{field}'"),
            DomainError::InvalidTransition => write!(f, "invalid status transition"),
            DomainError::NotFound(entity) => write!(f, "{entity} not found"),
            DomainError::Forbidden(message) => write!(f, "{message}"),
            DomainError::Unauthorized(message) => write!(f, "{message}"),
        }
    }
}

impl Error for DomainError {}
