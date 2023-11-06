//! This module contains errors.
//! 

use std::{error::Error, fmt};

use aws_sdk_sqs::types::{SdkError};

use hyperactive::err::{HypErr};

// The GenericError encompasses almost every possible error type that could be passed.
// Asynchronous functions that return Result<T, GenericError> can call other functions and use the "?" operator to return the Err() variant as needed.
//pub type GenericError = Box<dyn std::error::Error + Send + Sync>;


/*#[derive(Debug)]
pub enum MessageErrSQS {
    Send(SendMessageError),
    Delete(DeleteMessageError),
} // use fmt::Debug instead */

/// The EventError is ergonomic to instantiate and contains a simple error message
#[derive(fmt::Debug)]
pub enum EventfulError {
    NSQ,
    SQS(String),
    Hyperactive(HypErr),
    SerdeJSON(serde_json::Error),
}

impl Error for EventfulError {}

impl fmt::Display for EventfulError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "EventError: {:?}", self)
    }
}


impl<T: fmt::Debug> From<SdkError<T>> for EventfulError {
    fn from(err: SdkError<T>) -> Self {
        EventfulError::SQS(format!("{:?}", err))
    }
}

impl From<HypErr> for EventfulError {
    fn from(err: HypErr) -> Self {
        EventfulError::Hyperactive(err)
    }
}


impl From<serde_json::Error> for EventfulError {
    fn from(err: serde_json::Error) -> Self {
        EventfulError::SerdeJSON(err)
    }
}


