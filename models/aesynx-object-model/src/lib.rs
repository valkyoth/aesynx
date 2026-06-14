#![forbid(unsafe_code)]

mod graph;
mod id;
mod kind;
mod node;

pub use graph::{ObjectGraph, ObjectGraphError, Reachability};
pub use id::ModelObjectId;
pub use kind::ObjectKind;
pub use node::{ImmutableNode, NodeRefError};

#[cfg(test)]
mod tests;
