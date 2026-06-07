#![no_std]
#![deny(unsafe_code)]

use aesynx_abi::{CoreId, ObjectId};

pub trait KernelObject {
    fn object_id(&self) -> ObjectId;
    fn object_type(&self) -> ObjectType;
    fn owner_core(&self) -> CoreId;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObjectType {
    MemoryRegion,
    Endpoint,
    AddressSpace,
    Task,
    Process,
    Device,
    Driver,
    Queue,
    BytecodeModule,
    PersistentNode,
    ModelObject,
    TelemetryStream,
}
