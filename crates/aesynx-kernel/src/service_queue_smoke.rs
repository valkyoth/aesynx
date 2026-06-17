use core::sync::atomic::Ordering;

use aesynx_abi::{CoreId, MessageId, ROOT_CORE};
use aesynx_ipc::{
    CompletionStatus, CoreValidationError, InlineBytes, LiveCoreSet, MessageKind, MessagePayload,
    QueueSetError, RingQueueError, ServiceCompletion, ServiceKind, ServiceQueueSet, ServiceRequest,
    ValidatedCoreId,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServiceQueueSmokeStatus {
    pub log_submitted: bool,
    pub log_observed: bool,
    pub completion_observed: bool,
    pub timer_pending: bool,
    pub object_pending: bool,
    pub release_acquire_ok: bool,
    pub unsupported_denied: bool,
    pub unsupported_pending_denied: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServiceQueueSmokeError {
    InlinePayload,
    Core(CoreValidationError),
    Queue(QueueSetError),
    Ring(RingQueueError),
    Request,
    UnexpectedState,
}

pub fn run() -> Result<ServiceQueueSmokeStatus, ServiceQueueSmokeError> {
    let caller =
        ValidatedCoreId::new(ROOT_CORE, &BootCoreSet).map_err(ServiceQueueSmokeError::Core)?;
    let mut queues =
        ServiceQueueSet::<4, 4>::new(ROOT_CORE).map_err(ServiceQueueSmokeError::Ring)?;
    let log = request(
        MessageId::new(1),
        caller,
        ServiceKind::Log,
        MessageKind::TelemetrySample,
        MessagePayload::Inline(
            InlineBytes::new(b"service-queue")
                .map_err(|_| ServiceQueueSmokeError::InlinePayload)?,
        ),
    )?;
    let timer = request(
        MessageId::new(2),
        caller,
        ServiceKind::Timer,
        MessageKind::TelemetrySample,
        MessagePayload::Empty,
    )?;
    let object = request(
        MessageId::new(3),
        caller,
        ServiceKind::Object,
        MessageKind::OpenObject,
        MessagePayload::Empty,
    )?;
    let unsupported = request(
        MessageId::new(4),
        caller,
        ServiceKind::Capability,
        MessageKind::GrantCap,
        MessagePayload::Empty,
    )?;

    queues
        .submit(ROOT_CORE, log)
        .map_err(ServiceQueueSmokeError::Queue)?;
    queues
        .submit(ROOT_CORE, timer)
        .map_err(ServiceQueueSmokeError::Queue)?;
    queues
        .submit(ROOT_CORE, object)
        .map_err(ServiceQueueSmokeError::Queue)?;
    let log_submitted = queues
        .pending_requests(ROOT_CORE, ServiceKind::Log)
        .map_err(ServiceQueueSmokeError::Queue)?
        == 1;
    let timer_pending = queues
        .pending_requests(ROOT_CORE, ServiceKind::Timer)
        .map_err(ServiceQueueSmokeError::Queue)?
        == 1;
    let object_pending = queues
        .pending_requests(ROOT_CORE, ServiceKind::Object)
        .map_err(ServiceQueueSmokeError::Queue)?
        == 1;
    let unsupported_denied =
        queues.submit(ROOT_CORE, unsupported) == Err(QueueSetError::UnsupportedService);
    let unsupported_pending_denied = queues.pending_requests(ROOT_CORE, ServiceKind::Capability)
        == Err(QueueSetError::UnsupportedService);
    let observed = queues
        .pop_request(ROOT_CORE, ServiceKind::Log)
        .map_err(ServiceQueueSmokeError::Queue)?;
    let release_acquire_ok = observed.ordering().producer_publish() == Ordering::Release
        && observed.ordering().consumer_observe() == Ordering::Acquire;
    let observed_log = observed.into_value();
    let log_observed = observed_log.id() == log.id()
        && observed_log.caller().get() == CoreId::new(0)
        && observed_log.service() == ServiceKind::Log
        && observed_log.kind() == MessageKind::TelemetrySample;

    queues
        .complete(
            ROOT_CORE,
            ServiceKind::Log,
            ServiceCompletion::new(log.id(), CompletionStatus::Completed, MessagePayload::Empty),
        )
        .map_err(ServiceQueueSmokeError::Queue)?;
    let completion = queues
        .pop_completion(ROOT_CORE, ServiceKind::Log)
        .map_err(ServiceQueueSmokeError::Queue)?
        .into_value();
    let completion_observed = completion.request_id() == log.id()
        && completion.status() == CompletionStatus::Completed
        && completion.payload() == MessagePayload::Empty;

    if !log_submitted
        || !log_observed
        || !completion_observed
        || !timer_pending
        || !object_pending
        || !release_acquire_ok
        || !unsupported_denied
        || !unsupported_pending_denied
    {
        return Err(ServiceQueueSmokeError::UnexpectedState);
    }

    Ok(ServiceQueueSmokeStatus {
        log_submitted,
        log_observed,
        completion_observed,
        timer_pending,
        object_pending,
        release_acquire_ok,
        unsupported_denied,
        unsupported_pending_denied,
    })
}

struct BootCoreSet;

impl LiveCoreSet for BootCoreSet {
    fn contains(&self, core: CoreId) -> bool {
        core == ROOT_CORE
    }
}

fn request(
    id: MessageId,
    caller: ValidatedCoreId,
    service: ServiceKind,
    kind: MessageKind,
    payload: MessagePayload,
) -> Result<ServiceRequest, ServiceQueueSmokeError> {
    ServiceRequest::new(id, caller, service, kind, payload)
        .map_err(|_| ServiceQueueSmokeError::Request)
}
