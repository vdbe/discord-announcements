use std::error::Error;
use std::fmt;

use macros::quick_impl;

pub use db::DbError;
pub use feed::FeedError;

mod db;
mod feed;
mod macros;

/// The default error type for this crate
#[derive(Debug)]
pub enum MyError {
    /// Container for DbError
    Db(DbError),

    /// Container for FeedError
    Feed(FeedError),

    /// Just a generic error without dedicated variant,
    /// with a string to store a description
    Generic(String),

    /// An empty error without further info for when you are lazy
    Empty,
}

impl MyError {
    pub fn new<T: ToString>(msg: T) -> Self {
        Self::Generic(msg.to_string())
    }

    pub fn empty() -> Self {
        Self::Empty
    }
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MyError::Db(err) => write!(f, "Error: {err}"),
            MyError::Feed(err) => write!(f, "Error: {err}"),
            MyError::Generic(desc) => write!(f, "Error: {desc}"),
            MyError::Empty => write!(f, "Error"),
        }
    }
}

impl Error for MyError {
    fn description(&self) -> &str {
        match self {
            MyError::Db(_) => "DbError",
            MyError::Feed(_) => "FeedError",
            MyError::Generic(desc) => desc,
            MyError::Empty => "",
        }
    }
}

impl From<DbError> for MyError {
    fn from(err: DbError) -> Self {
        Self::Db(err)
    }
}

impl From<FeedError> for MyError {
    fn from(err: FeedError) -> Self {
        Self::Feed(err)
    }
}

quick_impl!(From<r2d2::Error> for MyError, MyError::Db);
quick_impl!(From<diesel::result::Error> for MyError, MyError::Db);

quick_impl!(From<reqwest::Error> for MyError, MyError::Feed);
quick_impl!(From<quick_xml::Error> for MyError, MyError::Feed);
quick_impl!(From<quick_xml::DeError> for MyError, MyError::Feed);

quick_impl!(From<tokio::task::JoinError> for MyError);
