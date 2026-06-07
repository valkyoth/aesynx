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
    Inline(InlineBytes),
}

pub const MAX_INLINE_PAYLOAD_LEN: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InlineBytes {
    len: u8,
    bytes: [u8; MAX_INLINE_PAYLOAD_LEN],
}

impl InlineBytes {
    pub fn new(src: &[u8]) -> Result<Self, IpcError> {
        if src.len() > MAX_INLINE_PAYLOAD_LEN {
            return Err(IpcError::PayloadTooLarge);
        }

        let mut bytes = [0u8; MAX_INLINE_PAYLOAD_LEN];
        bytes[..src.len()].copy_from_slice(src);

        Ok(Self {
            len: src.len() as u8,
            bytes,
        })
    }

    #[must_use]
    pub const fn len(self) -> u8 {
        self.len
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.len == 0
    }

    #[must_use]
    pub fn as_full_buffer(&self) -> &[u8; MAX_INLINE_PAYLOAD_LEN] {
        &self.bytes
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IpcError {
    PayloadTooLarge,
}

#[cfg(test)]
mod tests {
    use super::{InlineBytes, IpcError, MAX_INLINE_PAYLOAD_LEN};

    #[test]
    fn inline_payload_rejects_overlong_input() {
        let bytes = [0u8; MAX_INLINE_PAYLOAD_LEN + 1];

        assert_eq!(InlineBytes::new(&bytes), Err(IpcError::PayloadTooLarge));
    }

    #[test]
    fn inline_payload_tracks_valid_length() {
        let payload = InlineBytes::new(&[1, 2, 3]);

        assert_eq!(payload.map(|value| value.len()), Ok(3));
    }
}
