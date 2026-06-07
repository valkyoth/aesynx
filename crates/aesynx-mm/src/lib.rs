#![no_std]
#![deny(unsafe_code)]

use aesynx_abi::PhysFrame;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AddressSpace {
    root: PhysFrame,
}

impl AddressSpace {
    #[must_use]
    pub const fn new(root: PhysFrame) -> Self {
        Self { root }
    }

    #[must_use]
    pub const fn root(self) -> PhysFrame {
        self.root
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GenericPageFlags {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
    pub user: bool,
    pub global: bool,
    pub device_memory: bool,
    pub cacheable: bool,
}
