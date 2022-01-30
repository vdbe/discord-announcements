use std::error::Error;
use std::fmt;

use super::quick_impl;

/// Errors that come from database interactions
#[derive(Debug)]
pub enum DbError {
    /// Query did not find and matches
    NotFound,

    /// Just a generic error without dedicated variant,
    /// with a string to store a description
    Generic(String),

    /// An empty error without further info for when you are lazy
    Empty,
}

impl DbError {
    pub fn new<T: ToString>(msg: T) -> Self {
        Self::Generic(msg.to_string())
    }

    pub fn empty<T: ToString>(msg: T) -> Self {
        Self::Generic(msg.to_string())
    }
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NotFound => write!(f, "DB error: not found"),
            Self::Generic(s) => write!(f, "DB error: {s}"),
            Self::Empty => write!(f, "DB error"),
        }
    }
}

impl Error for DbError {
    fn description(&self) -> &str {
        match self {
            Self::NotFound => "not found",
            Self::Generic(s) => s,
            Self::Empty => "",
        }
    }
}

impl From<diesel::result::Error> for DbError {
    fn from(e: diesel::result::Error) -> Self {
        match e {
            diesel::result::Error::NotFound => Self::NotFound,
            //diesel::result::Error::InvalidCString(_) => todo!(),
            //diesel::result::Error::DatabaseError(_, _) => todo!(),
            //diesel::result::Error::QueryBuilderError(_) => todo!(),
            //diesel::result::Error::DeserializationError(_) => todo!(),
            //diesel::result::Error::SerializationError(_) => todo!(),
            //diesel::result::Error::RollbackTransaction => todo!(),
            //diesel::result::Error::AlreadyInTransaction => todo!(),
            e => Self::new(e),
        }
    }
}

quick_impl!(From<r2d2::Error> for DbError);
