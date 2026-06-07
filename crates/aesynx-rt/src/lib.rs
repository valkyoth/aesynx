#![no_std]
#![deny(unsafe_code)]

use aesynx_abi::CapId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Env {
    pub console_in: CapId,
    pub console_out: CapId,
    pub process_service: CapId,
    pub object_root: CapId,
}
