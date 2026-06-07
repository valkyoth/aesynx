#![no_std]
#![deny(unsafe_code)]

use aesynx_abi::{CoreId, DeviceId, ObjectId};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeviceObject {
    pub id: DeviceId,
    pub object_id: ObjectId,
    pub bus: BusKind,
    pub owner_core: CoreId,
    pub state: DeviceState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BusKind {
    Pci,
    Usb,
    Acpi,
    DeviceTree,
    VirtioMmio,
    Platform,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeviceState {
    Discovered,
    Matched,
    Probing,
    Bound,
    Running,
    Quiescing,
    Draining,
    Stopped,
    Revoked,
    Crashed,
}
