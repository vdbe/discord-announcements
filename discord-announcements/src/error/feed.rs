use std::error::Error;
use std::fmt;

use super::quick_impl;

/// Errors that come from receiving/parsing the Feed
#[derive(Debug)]
pub enum FeedError {
    /// Deserialization error
    De(String),

    /// Web error: 404 403 and other reqwest errors
    Web(String),

    /// Just a generic error without dedicated variant,
    /// with a string to store a description
    Generic(String),

    /// An empty error without further info for when you are lazy
    Empty,
}

impl FeedError {
    pub fn new<T: ToString>(msg: T) -> Self {
        Self::Generic(msg.to_string())
    }
}

impl fmt::Display for FeedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::De(s) => write!(f, "Feed deserialization error: {s}"),
            Self::Web(s) => write!(f, "Feed weberror: {s}"),
            Self::Generic(s) => write!(f, "Feed error: {s}"),
            Self::Empty => write!(f, "FeedError"),
        }
    }
}

impl Error for FeedError {
    fn description(&self) -> &str {
        match self {
            Self::De(s) => s,
            Self::Web(s) => s,
            Self::Generic(s) => s,
            Self::Empty => "",
        }
    }
}

impl From<quick_xml::Error> for FeedError {
    fn from(e: quick_xml::Error) -> Self {
        match e {
            //quick_xml::Error::Io(_) => todo!(),
            //quick_xml::Error::Utf8(_) => todo!(),
            //quick_xml::Error::UnexpectedEof(_) => todo!(),
            //quick_xml::Error::EndEventMismatch { expected, found } => todo!(),
            //quick_xml::Error::UnexpectedToken(_) => todo!(),
            //quick_xml::Error::UnexpectedBang => todo!(),
            //quick_xml::Error::TextNotFound => todo!(),
            //quick_xml::Error::XmlDeclWithoutVersion(_) => todo!(),
            //quick_xml::Error::NameWithQuote(_) => todo!(),
            //quick_xml::Error::NoEqAfterName(_) => todo!(),
            //quick_xml::Error::UnquotedValue(_) => todo!(),
            //quick_xml::Error::DuplicatedAttribute(_, _) => todo!(),
            //quick_xml::Error::EscapeError(_) => todo!(),
            e => Self::new(e),
        }
    }
}

impl From<quick_xml::DeError> for FeedError {
    fn from(e: quick_xml::DeError) -> Self {
        match e {
            //quick_xml::DeError::Custom(_) => todo!(),
            //quick_xml::DeError::Int(_) => todo!(),
            //quick_xml::DeError::Float(_) => todo!(),
            //quick_xml::DeError::Xml(_) => todo!(),
            //quick_xml::DeError::EndOfAttributes => todo!(),
            //quick_xml::DeError::Eof => todo!(),
            //quick_xml::DeError::InvalidBoolean(_) => todo!(),
            //quick_xml::DeError::InvalidUnit(_) => todo!(),
            //quick_xml::DeError::InvalidEnum(_) => todo!(),
            //quick_xml::DeError::Text => todo!(),
            //quick_xml::DeError::Start => todo!(),
            //quick_xml::DeError::End => todo!(),
            //quick_xml::DeError::Unsupported(_) => todo!(),
            e => Self::De(e.to_string()),
        }
    }
}

quick_impl!(From<reqwest::Error> for FeedError);
