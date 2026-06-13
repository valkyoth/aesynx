#![no_std]
#![deny(unsafe_code)]

#[cfg(test)]
extern crate alloc;

use core::fmt;

use aesynx_abi::{CapId, CoreId, MessageId, ObjectId};

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct MessageHeader {
    src: CoreId,
    dst: CoreId,
    kind: MessageKind,
    seq: u64,
    reply_to: Option<MessageId>,
}

impl MessageHeader {
    #[must_use]
    pub const fn stamp(request: MessageRequest, verified_src: CoreId, assigned_seq: u64) -> Self {
        Self {
            src: verified_src,
            dst: request.dst,
            kind: request.kind,
            seq: assigned_seq,
            reply_to: request.reply_to,
        }
    }

    #[must_use]
    pub const fn src(self) -> CoreId {
        self.src
    }

    #[must_use]
    pub const fn dst(self) -> CoreId {
        self.dst
    }

    #[must_use]
    pub const fn kind(self) -> MessageKind {
        self.kind
    }

    #[must_use]
    pub const fn seq(self) -> u64 {
        self.seq
    }

    #[must_use]
    pub const fn reply_to(self) -> Option<MessageId> {
        self.reply_to
    }
}

impl fmt::Debug for MessageHeader {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MessageHeader")
            .field("src", &"<redacted>")
            .field("dst", &"<redacted>")
            .field("kind", &self.kind)
            .field("seq", &self.seq)
            .field("reply_to", &self.reply_to.map(|_| "<redacted>"))
            .finish()
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct MessageRequest {
    /// Destination requested by the sender. Routers must validate this value
    /// against the live core set before using it as an index or queue selector.
    pub dst: CoreId,
    pub kind: MessageKind,
    pub reply_to: Option<MessageId>,
}

impl fmt::Debug for MessageRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MessageRequest")
            .field("dst", &"<redacted>")
            .field("kind", &self.kind)
            .field("reply_to", &self.reply_to.map(|_| "<redacted>"))
            .finish()
    }
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

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum MessagePayload {
    Empty,
    Cap(CapId),
    Object(ObjectId),
    Inline(InlineBytes),
}

impl fmt::Debug for MessagePayload {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("Empty"),
            Self::Cap(_) => formatter.write_str("Cap(<redacted>)"),
            Self::Object(_) => formatter.write_str("Object(<redacted>)"),
            Self::Inline(_) => formatter.write_str("Inline(<redacted>)"),
        }
    }
}

pub const MAX_INLINE_PAYLOAD_LEN: usize = 64;
const _: () = assert!(
    MAX_INLINE_PAYLOAD_LEN <= u8::MAX as usize,
    "MAX_INLINE_PAYLOAD_LEN must fit in InlineBytes::len"
);

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct InlineBytes {
    len: u8,
    bytes: [u8; MAX_INLINE_PAYLOAD_LEN],
}

impl fmt::Debug for InlineBytes {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("InlineBytes")
            .field("len", &self.len)
            .field("bytes", &"<redacted>")
            .finish()
    }
}

impl InlineBytes {
    pub fn new(src: &[u8]) -> Result<Self, IpcError> {
        if src.len() > MAX_INLINE_PAYLOAD_LEN {
            return Err(IpcError::PayloadTooLarge);
        }

        let mut bytes = [0u8; MAX_INLINE_PAYLOAD_LEN];
        bytes[..src.len()].copy_from_slice(src);
        let len = u8::try_from(src.len()).map_err(|_| IpcError::PayloadTooLarge)?;

        Ok(Self { len, bytes })
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
    pub fn as_slice(&self) -> &[u8] {
        &self.bytes[..self.len as usize]
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
    use alloc::format;

    use aesynx_abi::{CapId, CoreId, MessageId, ObjectId};

    use super::{
        InlineBytes, IpcError, MAX_INLINE_PAYLOAD_LEN, MessageHeader, MessageKind, MessagePayload,
        MessageRequest,
    };

    #[test]
    fn message_header_is_kernel_stamped() {
        let request = MessageRequest {
            dst: CoreId::new(2),
            kind: MessageKind::Ping,
            reply_to: Some(MessageId::new(9)),
        };
        let header = MessageHeader::stamp(request, CoreId::new(1), 42);

        assert_eq!(header.src(), CoreId::new(1));
        assert_eq!(header.dst(), CoreId::new(2));
        assert_eq!(header.kind(), MessageKind::Ping);
        assert_eq!(header.seq(), 42);
        assert_eq!(header.reply_to(), Some(MessageId::new(9)));
    }

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

    #[test]
    fn inline_payload_slice_is_length_bounded() {
        let payload = InlineBytes::new(&[1, 2, 3]);

        assert_eq!(payload.map(|value| value.as_slice() == [1, 2, 3]), Ok(true));
    }

    #[test]
    fn ipc_debug_output_redacts_authority_and_payload_values() -> Result<(), IpcError> {
        let inline = InlineBytes::new(&[1, 2, 3])?;
        let payload = MessagePayload::Inline(inline);
        let cap = MessagePayload::Cap(CapId::new(42));
        let object = MessagePayload::Object(ObjectId::new(99));
        let header = MessageHeader::stamp(
            MessageRequest {
                dst: CoreId::new(2),
                kind: MessageKind::WriteObject,
                reply_to: Some(MessageId::new(7)),
            },
            CoreId::new(1),
            9,
        );

        assert_eq!(format!("{payload:?}"), "Inline(<redacted>)");
        assert_eq!(format!("{cap:?}"), "Cap(<redacted>)");
        assert_eq!(format!("{object:?}"), "Object(<redacted>)");
        assert!(!format!("{header:?}").contains("CoreId"));
        assert!(!format!("{header:?}").contains("MessageId"));
        assert!(!format!("{inline:?}").contains("[1, 2, 3]"));

        Ok(())
    }
}
