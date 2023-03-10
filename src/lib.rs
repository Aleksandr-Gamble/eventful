//! In a microservice architecture, it is the responsibility of microservices to emit events
//! This module is intended to help abstract functinality associate with emitting and consuming events.
//! Making the production and consumption of events simple across various message queues.
//! 

pub mod err;
pub mod nsq;
pub mod sqs;
