#![no_std]
#![deny(unsafe_code)]

#[cfg(test)]
extern crate alloc;

mod registry;
mod types;

pub use registry::{ObjectCreate, ObjectRegistry, ObjectRegistryError};
pub use types::{KernelObject, ObjectRecord, ObjectType};

#[cfg(test)]
mod tests;
