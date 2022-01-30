use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct MyError {
    pub details: Option<String>,
}

impl MyError {
    pub fn new<T: ToString>(msg: T) -> MyError {
        MyError {
            details: Some(msg.to_string()),
        }
    }

    pub fn empty() -> MyError {
        MyError { details: None }
    }
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.details {
            Some(s) => write!(f, "{}", s),
            None => write!(f, "Error"),
        }
    }
}

impl Error for MyError {
    fn description(&self) -> &str {
        match &self.details {
            Some(s) => s,
            None => "Error",
        }
    }
}

macro_rules! myError_impl {
    (From<$type:ty>) => {
        impl From<$type> for MyError {
            fn from(err: $type) -> Self {
                Self::new(err)
            }
        }
    };
}

myError_impl!(From<reqwest::Error>);
myError_impl!(From<quick_xml::Error>);
myError_impl!(From<quick_xml::DeError>);
myError_impl!(From<diesel::result::Error>);
myError_impl!(From<r2d2::Error>);
myError_impl!(From<tokio::task::JoinError>);
