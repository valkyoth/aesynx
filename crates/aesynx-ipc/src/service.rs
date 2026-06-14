use core::fmt;

use aesynx_abi::MessageId;

use crate::{MessageKind, MessagePayload, ValidatedCoreId};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServiceKind {
    Log,
    Timer,
    Object,
    Capability,
    Memory,
    Driver,
    Telemetry,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct ServiceRequest {
    id: MessageId,
    caller: ValidatedCoreId,
    service: ServiceKind,
    kind: MessageKind,
    payload: MessagePayload,
}

impl fmt::Debug for ServiceRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ServiceRequest")
            .field("id", &"<redacted>")
            .field("caller", &"<redacted>")
            .field("service", &self.service)
            .field("kind", &self.kind)
            .field("payload", &"<redacted>")
            .finish()
    }
}

impl ServiceRequest {
    pub fn new(
        id: MessageId,
        caller: ValidatedCoreId,
        service: ServiceKind,
        kind: MessageKind,
        payload: MessagePayload,
    ) -> Result<Self, RequestError> {
        if id.get() == 0 {
            return Err(RequestError::InvalidMessageId);
        }

        Ok(Self {
            id,
            caller,
            service,
            kind,
            payload,
        })
    }

    #[must_use]
    pub const fn id(self) -> MessageId {
        self.id
    }

    #[must_use]
    pub const fn caller(self) -> ValidatedCoreId {
        self.caller
    }

    #[must_use]
    pub const fn service(self) -> ServiceKind {
        self.service
    }

    #[must_use]
    pub const fn kind(self) -> MessageKind {
        self.kind
    }

    #[must_use]
    pub const fn payload(self) -> MessagePayload {
        self.payload
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct ServiceCompletion {
    request_id: MessageId,
    status: CompletionStatus,
    payload: MessagePayload,
}

impl fmt::Debug for ServiceCompletion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ServiceCompletion")
            .field("request_id", &"<redacted>")
            .field("status", &self.status)
            .field("payload", &"<redacted>")
            .finish()
    }
}

impl ServiceCompletion {
    #[must_use]
    pub const fn new(
        request_id: MessageId,
        status: CompletionStatus,
        payload: MessagePayload,
    ) -> Self {
        Self {
            request_id,
            status,
            payload,
        }
    }

    #[must_use]
    pub const fn request_id(self) -> MessageId {
        self.request_id
    }

    #[must_use]
    pub const fn status(self) -> CompletionStatus {
        self.status
    }

    #[must_use]
    pub const fn payload(self) -> MessagePayload {
        self.payload
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompletionStatus {
    Accepted,
    Completed,
    Rejected,
    Denied,
    NotFound,
    Failed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RequestError {
    InvalidMessageId,
}
