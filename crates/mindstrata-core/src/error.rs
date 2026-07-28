//! Error types for the Mindstrata simulation.

/// Result alias using our error type.
pub type Result<T> = std::result::Result<T, Error>;

/// The unified error type for Mindstrata crates.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("fixed-point overflow")]
    FixedOverflow,

    #[error("entity not found: {0}")]
    EntityNotFound(String),

    #[error("component not found: {0}")]
    ComponentNotFound(String),

    #[error("invalid seed: cannot be zero")]
    InvalidSeed,

    #[error("simulation ran out of ticks")]
    TickLimitReached,

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("{0}")]
    Custom(String),
}
