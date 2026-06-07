#![no_std]
#![deny(unsafe_code)]

use aesynx_abi::{CapId, CoreId, MessageId, ObjectId};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MessageHeader {
    pub src: CoreId,
    pub dst: CoreId,
    pub kind: MessageKind,
    pub seq: u64,
    pub reply_to: Option<MessageId>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MessageKind {
    Ping,
    Pong,
    SpawnTask,
    OpenObject,
    ReadObject,
    WriteObject,
    GrantCap,
    RevokeCap,
    MapMemory,
    UnmapMemory,
    DriverRequest,
    DriverReply,
    TelemetrySample,
    MigrateTask,
    SchedulerAdvice,
    ModelLoad,
    ModelReject,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MessagePayload {
    Empty,
    Cap(CapId),
    Object(ObjectId),
    Inline { len: u8, bytes: [u8; 64] },
}
