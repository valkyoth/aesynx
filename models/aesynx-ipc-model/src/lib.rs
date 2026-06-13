#![deny(unsafe_code)]

mod completion;
mod ordering;
mod request;
mod ring;
mod service;

pub use completion::{CompletionStatus, ServiceCompletion};
pub use ordering::{CONSUMER_OBSERVE_ORDERING, PRODUCER_PUBLISH_ORDERING, QueueOrderingEvidence};
pub use request::{RequestError, ServiceRequest};
pub use ring::{ObservedEntry, RingQueueError, ServiceRingQueue};
pub use service::ServiceKind;

#[cfg(test)]
mod tests;
