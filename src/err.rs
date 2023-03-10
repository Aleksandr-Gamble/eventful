//! This module contains errors.
//! 

use std::{error::Error, fmt};
use serde::{Serialize, Deserialize};

/// The GenericError encompasses almost every possible error type that could be passed.
/// Asynchronous functions that return Result<T, GenericError> can call other functions and use the "?" operator to return the Err() variant as needed.
pub type GenericError = Box<dyn std::error::Error + Send + Sync>;


/// The EventError is ergonomic to instantiate and contains a simple error message
#[derive(Serialize, Deserialize, Debug)]
pub struct EventError {
    pub msg: String,
}

impl Error for EventError {}

impl fmt::Display for EventError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "EventError: {}", self.msg)
    }
}

impl EventError {
    pub fn new(msg: &str) -> Self {
        EventError{
            msg: msg.to_string()
        }
    }
}
