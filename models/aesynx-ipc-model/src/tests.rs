use core::sync::atomic::Ordering;
use std::format;

use aesynx_abi::{CoreId, MessageId};
use aesynx_ipc::{InlineBytes, MessageKind, MessagePayload};

use crate::{
    CompletionStatus, ObservedEntry, RequestError, RingQueueError, ServiceCompletion, ServiceKind,
    ServiceRequest, ServiceRingQueue,
};

#[test]
fn service_request_rejects_zero_message_id() {
    let request = ServiceRequest::new(
        MessageId::new(0),
        CoreId::new(1),
        ServiceKind::Object,
        MessageKind::OpenObject,
        MessagePayload::Empty,
    );

    assert_eq!(request, Err(RequestError::InvalidMessageId));
}

#[test]
fn service_request_keeps_kernel_stamped_metadata() {
    let request = ServiceRequest::new(
        MessageId::new(7),
        CoreId::new(2),
        ServiceKind::Capability,
        MessageKind::GrantCap,
        MessagePayload::Empty,
    );

    assert_eq!(
        request.map(|value| {
            (
                value.id(),
                value.caller(),
                value.service(),
                value.kind(),
                value.payload(),
            )
        }),
        Ok((
            MessageId::new(7),
            CoreId::new(2),
            ServiceKind::Capability,
            MessageKind::GrantCap,
            MessagePayload::Empty,
        ))
    );
}

#[test]
fn service_completion_preserves_request_identity() {
    let completion = ServiceCompletion::new(
        MessageId::new(11),
        CompletionStatus::Completed,
        MessagePayload::Empty,
    );

    assert_eq!(completion.request_id(), MessageId::new(11));
    assert_eq!(completion.status(), CompletionStatus::Completed);
    assert_eq!(completion.payload(), MessagePayload::Empty);
}

#[test]
fn service_debug_output_redacts_request_identity_and_payloads() -> Result<(), RequestError> {
    let payload = MessagePayload::Inline(
        InlineBytes::new(&[1, 2, 3]).map_err(|_| RequestError::InvalidMessageId)?,
    );
    let request = ServiceRequest::new(
        MessageId::new(77),
        CoreId::new(2),
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
fn ring_queue_rejects_zero_capacity() {
    let queue = ServiceRingQueue::<ServiceRequest>::with_capacity(0);

    assert_eq!(queue, Err(RingQueueError::ZeroCapacity));
}

#[test]
fn ring_queue_preserves_fifo_across_wraparound() -> Result<(), RingQueueError> {
    let first = request(1);
    let second = request(2);
    let third = request(3);
    let mut queue = ServiceRingQueue::with_capacity(2)?;

    assert_eq!(queue.push(first), Ok(()));
    assert_eq!(queue.push(second), Ok(()));
    assert_eq!(queue.pop().map(ObservedEntry::into_value), Ok(first));
    assert_eq!(queue.push(third), Ok(()));
    assert_eq!(queue.pop().map(ObservedEntry::into_value), Ok(second));
    assert_eq!(queue.pop().map(ObservedEntry::into_value), Ok(third));

    Ok(())
}

#[test]
fn ring_queue_full_push_does_not_mutate_state() -> Result<(), RingQueueError> {
    let first = request(1);
    let second = request(2);
    let mut queue = ServiceRingQueue::with_capacity(1)?;

    assert_eq!(queue.push(first), Ok(()));
    assert_eq!(queue.len(), 1);
    assert_eq!(queue.push(second), Err(RingQueueError::Full));
    assert_eq!(queue.len(), 1);
    assert_eq!(queue.pop().map(ObservedEntry::into_value), Ok(first));

    Ok(())
}

#[test]
fn ring_queue_empty_pop_does_not_mutate_state() -> Result<(), RingQueueError> {
    let mut queue = ServiceRingQueue::<ServiceRequest>::with_capacity(1)?;

    assert_eq!(queue.len(), 0);
    assert_eq!(queue.pop(), Err(RingQueueError::Empty));
    assert_eq!(queue.len(), 0);

    Ok(())
}

#[test]
fn ring_queue_records_release_acquire_contract() -> Result<(), RingQueueError> {
    let mut queue = ServiceRingQueue::with_capacity(1)?;

    assert_eq!(queue.push(request(1)), Ok(()));

    let observed = queue.pop();

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

fn request(id: u64) -> ServiceRequest {
    ServiceRequest::from_validated_parts(
        MessageId::new(id),
        CoreId::new(1),
        ServiceKind::Log,
        MessageKind::TelemetrySample,
        MessagePayload::Empty,
    )
}
