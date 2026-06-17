use alloc::format;
use core::sync::atomic::Ordering;

use aesynx_abi::{CoreId, MessageId};

use crate::{
    CompletionStatus, InlineBytes, LiveCoreSet, MessageKind, MessagePayload, ObservedEntry,
    QueueSetError, RequestError, RingQueueError, ServiceCompletion, ServiceKind, ServiceQueueSet,
    ServiceRequest, ServiceRingQueue, ValidatedCoreId,
};

struct TestCoreSet;

impl LiveCoreSet for TestCoreSet {
    fn contains(&self, core: CoreId) -> bool {
        core == CoreId::new(1)
    }
}

#[test]
fn service_request_rejects_zero_message_id() {
    let caller = ValidatedCoreId::new(CoreId::new(1), &TestCoreSet);
    let request = caller.map(|caller| {
        ServiceRequest::new(
            MessageId::new(0),
            caller,
            ServiceKind::Object,
            MessageKind::OpenObject,
            MessagePayload::Empty,
        )
    });

    assert_eq!(request, Ok(Err(RequestError::InvalidMessageId)));
}

#[test]
fn service_request_keeps_kernel_stamped_metadata() {
    let live = TestCoreSet;
    let caller = ValidatedCoreId::new(CoreId::new(1), &live);
    let request = caller.and_then(|caller| {
        ServiceRequest::new(
            MessageId::new(7),
            caller,
            ServiceKind::Log,
            MessageKind::TelemetrySample,
            MessagePayload::Empty,
        )
        .map_err(|_| crate::CoreValidationError::UnknownCore)
    });

    assert_eq!(
        request.map(|value| {
            (
                value.id(),
                value.caller().get(),
                value.service(),
                value.kind(),
                value.payload(),
            )
        }),
        Ok((
            MessageId::new(7),
            CoreId::new(1),
            ServiceKind::Log,
            MessageKind::TelemetrySample,
            MessagePayload::Empty,
        ))
    );
}

#[test]
fn service_debug_output_redacts_request_identity_and_payloads() -> Result<(), RequestError> {
    let caller = ValidatedCoreId::new(CoreId::new(1), &TestCoreSet)
        .map_err(|_| RequestError::InvalidMessageId)?;
    let payload = MessagePayload::Inline(
        InlineBytes::new(&[1, 2, 3]).map_err(|_| RequestError::InvalidMessageId)?,
    );
    let request = ServiceRequest::new(
        MessageId::new(77),
        caller,
        ServiceKind::Object,
        MessageKind::WriteObject,
        payload,
    )?;
    let completion = ServiceCompletion::new(
        MessageId::new(77),
        CompletionStatus::Denied,
        MessagePayload::Object(aesynx_abi::ObjectId::new(99)),
    );

    let request_debug = format!("{request:?}");
    let completion_debug = format!("{completion:?}");

    assert!(request_debug.contains("payload: \"<redacted>\""));
    assert!(!request_debug.contains("MessageId"));
    assert!(!request_debug.contains("CoreId"));
    assert!(!request_debug.contains("[1, 2, 3]"));
    assert!(completion_debug.contains("payload: \"<redacted>\""));
    assert!(!completion_debug.contains("ObjectId"));

    Ok(())
}

#[test]
fn service_ring_queue_rejects_zero_capacity() {
    let queue = ServiceRingQueue::<ServiceRequest, 0>::new(CoreId::new(1));

    assert_eq!(queue, Err(RingQueueError::ZeroCapacity));
}

#[test]
fn service_ring_queue_preserves_fifo_across_wraparound() -> Result<(), RingQueueError> {
    let first = request(1, ServiceKind::Log)?;
    let second = request(2, ServiceKind::Log)?;
    let third = request(3, ServiceKind::Log)?;
    let owner = CoreId::new(1);
    let mut queue = ServiceRingQueue::<ServiceRequest, 2>::new(owner)?;

    assert_eq!(queue.push(owner, first), Ok(()));
    assert_eq!(queue.push(owner, second), Ok(()));
    assert_eq!(queue.pop(owner).map(ObservedEntry::into_value), Ok(first));
    assert_eq!(queue.push(owner, third), Ok(()));
    assert_eq!(queue.pop(owner).map(ObservedEntry::into_value), Ok(second));
    assert_eq!(queue.pop(owner).map(ObservedEntry::into_value), Ok(third));

    Ok(())
}

#[test]
fn service_ring_queue_full_push_does_not_mutate_state() -> Result<(), RingQueueError> {
    let first = request(1, ServiceKind::Log)?;
    let second = request(2, ServiceKind::Log)?;
    let owner = CoreId::new(1);
    let mut queue = ServiceRingQueue::<ServiceRequest, 1>::new(owner)?;

    assert_eq!(queue.push(owner, first), Ok(()));
    assert_eq!(queue.len(), 1);
    assert_eq!(queue.push(owner, second), Err(RingQueueError::Full));
    assert_eq!(queue.len(), 1);
    assert_eq!(queue.pop(owner).map(ObservedEntry::into_value), Ok(first));

    Ok(())
}

#[test]
fn service_ring_queue_empty_pop_does_not_mutate_state() -> Result<(), RingQueueError> {
    let owner = CoreId::new(1);
    let mut queue = ServiceRingQueue::<ServiceRequest, 1>::new(owner)?;

    assert_eq!(queue.len(), 0);
    assert_eq!(queue.pop(owner), Err(RingQueueError::Empty));
    assert_eq!(queue.len(), 0);

    Ok(())
}

#[test]
fn service_ring_queue_records_release_acquire_contract() -> Result<(), RingQueueError> {
    let owner = CoreId::new(1);
    let mut queue = ServiceRingQueue::<ServiceRequest, 1>::new(owner)?;

    assert_eq!(queue.push(owner, request(1, ServiceKind::Log)?), Ok(()));

    let observed = queue.pop(owner);

    assert_eq!(
        observed.map(|value| {
            (
                value.ordering().producer_publish(),
                value.ordering().consumer_observe(),
            )
        }),
        Ok((Ordering::Release, Ordering::Acquire))
    );

    Ok(())
}

#[test]
fn service_ring_queue_rejects_non_owner_without_mutation() -> Result<(), RingQueueError> {
    let owner = CoreId::new(1);
    let other = CoreId::new(2);
    let first = request(1, ServiceKind::Log)?;
    let second = request(2, ServiceKind::Log)?;
    let mut queue = ServiceRingQueue::<ServiceRequest, 1>::new(owner)?;

    assert_eq!(queue.owner_core(), owner);
    assert_eq!(queue.push(other, first), Err(RingQueueError::OwnerMismatch));
    assert_eq!(queue.len(), 0);
    assert_eq!(queue.pop(other), Err(RingQueueError::OwnerMismatch));
    assert_eq!(queue.len(), 0);
    assert_eq!(queue.push(owner, second), Ok(()));
    assert_eq!(queue.len(), 1);
    assert_eq!(queue.pop(other), Err(RingQueueError::OwnerMismatch));
    assert_eq!(queue.len(), 1);
    assert_eq!(queue.pop(owner).map(ObservedEntry::into_value), Ok(second));

    Ok(())
}

#[test]
fn service_queue_set_routes_log_timer_and_object_queues() -> Result<(), QueueSetError> {
    let owner = CoreId::new(1);
    let mut queues = ServiceQueueSet::<2, 2>::new(owner).map_err(QueueSetError::Queue)?;
    let log = request(1, ServiceKind::Log).map_err(QueueSetError::Queue)?;
    let timer = request(2, ServiceKind::Timer).map_err(QueueSetError::Queue)?;
    let object = request(3, ServiceKind::Object).map_err(QueueSetError::Queue)?;

    queues.submit(owner, log)?;
    queues.submit(owner, timer)?;
    queues.submit(owner, object)?;

    assert_eq!(queues.pending_requests(owner, ServiceKind::Log), Ok(1));
    assert_eq!(queues.pending_requests(owner, ServiceKind::Timer), Ok(1));
    assert_eq!(queues.pending_requests(owner, ServiceKind::Object), Ok(1));
    assert_eq!(
        queues
            .pop_request(owner, ServiceKind::Log)
            .map(ObservedEntry::into_value),
        Ok(log)
    );
    assert_eq!(
        queues
            .pop_request(owner, ServiceKind::Timer)
            .map(ObservedEntry::into_value),
        Ok(timer)
    );
    assert_eq!(
        queues
            .pop_request(owner, ServiceKind::Object)
            .map(ObservedEntry::into_value),
        Ok(object)
    );

    Ok(())
}

#[test]
fn service_queue_set_rejects_unsupported_services_without_mutation() -> Result<(), RingQueueError> {
    let owner = CoreId::new(1);
    let mut queues = ServiceQueueSet::<1, 1>::new(owner)?;
    let unsupported = [
        ServiceKind::Capability,
        ServiceKind::Memory,
        ServiceKind::Driver,
        ServiceKind::Telemetry,
    ];

    for (index, service) in unsupported.into_iter().enumerate() {
        let id = u64::try_from(index)
            .map(|value| value + 4)
            .map_err(|_| RingQueueError::CorruptState)?;
        let request = request(id, service)?;

        assert_eq!(
            queues.submit(owner, request),
            Err(QueueSetError::UnsupportedService)
        );
        assert_eq!(
            queues.pop_request(owner, service),
            Err(QueueSetError::UnsupportedService)
        );
        assert_eq!(
            queues.pending_requests(owner, service),
            Err(QueueSetError::UnsupportedService)
        );
        assert_eq!(queues.pending_requests(owner, ServiceKind::Log), Ok(0));
        assert_eq!(queues.pending_requests(owner, ServiceKind::Timer), Ok(0));
        assert_eq!(queues.pending_requests(owner, ServiceKind::Object), Ok(0));
    }

    Ok(())
}

#[test]
fn service_queue_set_rejects_non_owner_without_mutation() -> Result<(), RingQueueError> {
    let owner = CoreId::new(1);
    let other = CoreId::new(2);
    let mut queues = ServiceQueueSet::<1, 1>::new(owner)?;
    let log = request(1, ServiceKind::Log)?;

    assert_eq!(queues.owner_core(), owner);
    assert_eq!(
        queues.submit(other, log),
        Err(QueueSetError::Queue(RingQueueError::OwnerMismatch))
    );
    assert_eq!(
        queues.submit(other, request(2, ServiceKind::Capability)?),
        Err(QueueSetError::Queue(RingQueueError::OwnerMismatch))
    );
    assert_eq!(
        queues.pending_requests(other, ServiceKind::Capability),
        Err(QueueSetError::Queue(RingQueueError::OwnerMismatch))
    );
    assert_eq!(queues.pending_requests(owner, ServiceKind::Log), Ok(0));
    assert_eq!(
        queues.pop_request(other, ServiceKind::Log),
        Err(QueueSetError::Queue(RingQueueError::OwnerMismatch))
    );
    assert_eq!(
        queues.pending_requests(other, ServiceKind::Log),
        Err(QueueSetError::Queue(RingQueueError::OwnerMismatch))
    );
    assert_eq!(queues.submit(owner, log), Ok(()));
    assert_eq!(
        queues.complete(
            other,
            ServiceKind::Log,
            ServiceCompletion::new(log.id(), CompletionStatus::Completed, MessagePayload::Empty),
        ),
        Err(QueueSetError::Queue(RingQueueError::OwnerMismatch))
    );
    assert_eq!(queues.pending_requests(owner, ServiceKind::Log), Ok(1));

    Ok(())
}

fn request(id: u64, service: ServiceKind) -> Result<ServiceRequest, RingQueueError> {
    ServiceRequest::new(
        MessageId::new(id),
        ValidatedCoreId::new(CoreId::new(1), &TestCoreSet)
            .map_err(|_| RingQueueError::CorruptState)?,
        service,
        MessageKind::TelemetrySample,
        MessagePayload::Empty,
    )
    .map_err(|_| RingQueueError::CorruptState)
}
