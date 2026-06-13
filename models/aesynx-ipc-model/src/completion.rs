use aesynx_abi::MessageId;
use aesynx_ipc::MessagePayload;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServiceCompletion {
    request_id: MessageId,
    status: CompletionStatus,
    payload: MessagePayload,
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
