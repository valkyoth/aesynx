use aesynx_abi::{CoreId, MessageId};
use aesynx_ipc::{MessageKind, MessagePayload};

use crate::ServiceKind;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServiceRequest {
    id: MessageId,
    caller: CoreId,
    service: ServiceKind,
    kind: MessageKind,
    payload: MessagePayload,
}

impl ServiceRequest {
    pub fn new(
        id: MessageId,
        caller: CoreId,
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

    #[cfg(test)]
    pub(crate) const fn from_validated_parts(
        id: MessageId,
        caller: CoreId,
        service: ServiceKind,
        kind: MessageKind,
        payload: MessagePayload,
    ) -> Self {
        Self {
            id,
            caller,
            service,
            kind,
            payload,
        }
    }

    #[must_use]
    pub const fn id(self) -> MessageId {
        self.id
    }

    #[must_use]
    pub const fn caller(self) -> CoreId {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RequestError {
    InvalidMessageId,
}
