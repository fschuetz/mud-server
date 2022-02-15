//! Specific errors for the game world and its assets
//!
//! Module that provides auth specific errors and mapping functionality
//! for errors of submodules used by the game world.
use std::error::Error as StdError;
use std::fmt;

/// A specialized result type for queries.
///
/// This type exists to avoid writing out `crate::errors`, and is
/// otherwise a direct mapping to `Result`.
pub type GameWorldResult<T> = Result<T, Error>;

/// Error type for auth errors
#[derive(Debug, Clone)]
pub enum Error {
    /// Command is not valid
    InvalidCommand,
    /// Data message is not valid
    InvalidDataMessage,
    /// Player does not exist
    PlayerDoesNotExist,
    /// No valid spawn point found
    NoSpawnpointFound,
    /// Converting into verb failed, unknown verb
    VerbUnknownError,
    /// Converting into verb failed due to wrong encoding
    VerbEncodingError,
    /// Conversion into property failed
    PropertyConversionFailed,
    /// Unknown error - typically used to map errors from other libraries
    /// that do not fit.
    UnknownError,
}

/// Implementation of Display trait for Error to enable printing errors
///
/// Generation of an error is completely separate from how it is displayed.
/// There's no need to be concerned about cluttering complex logic with the display style.
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::InvalidCommand => write!(f, "invalid command"),
            Error::InvalidDataMessage => write!(f, "invalid data message"),
            Error::PlayerDoesNotExist => write!(f, "player does not exist"),
            Error::NoSpawnpointFound => write!(f,"no valid spawnpoint found"),
            Error::VerbUnknownError => write!(f,"unknown verb"),
            Error::VerbEncodingError => write!(f,"unknown verb encoding"),
            Error::PropertyConversionFailed => write!(f, "property conversion failed"),
            Error::UnknownError => write!(f, "unknown error"),
        }

    }
}

impl StdError for Error {
    // Methods are deprecated, so we do not implement
}

/// Implementation of PartialEq trait
/// 
/// Defines what errors are equal. Note that UnknownError is not equal to 
/// UnknownError. As an UnknownError is unknown, two unknown errors are not
/// necessarily the same kind of error.
impl PartialEq for Error {
    fn eq(&self, other: &Error) -> bool {
        match (self, other) {
            (&Error::InvalidCommand, &Error::InvalidCommand) => true,
            (&Error::InvalidDataMessage, &Error::InvalidDataMessage) => true,
            (&Error::PlayerDoesNotExist, &Error::PlayerDoesNotExist) => true,
            (&Error::NoSpawnpointFound, &Error::NoSpawnpointFound) => true,
            (&Error::VerbUnknownError, &Error::VerbUnknownError) => true,
            (&Error::VerbEncodingError, &Error::VerbEncodingError) => true,
            (&Error::PropertyConversionFailed, &Error::PropertyConversionFailed) => true,
            _ => false,
        }
    }
}
