use core::sync::atomic::Ordering;

use crate::{ServiceCompletion, ServiceKind, ServiceRequest};

pub const PRODUCER_PUBLISH_ORDERING: Ordering = Ordering::Release;
pub const CONSUMER_OBSERVE_ORDERING: Ordering = Ordering::Acquire;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PublishedEntry<T> {
    value: T,
    publish_ordering: Ordering,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ObservedEntry<T> {
    value: T,
    ordering: QueueOrderingEvidence,
}

impl<T> ObservedEntry<T> {
    #[must_use]
    pub fn into_value(self) -> T {
        self.value
    }

    #[must_use]
    pub const fn ordering(&self) -> QueueOrderingEvidence {
        self.ordering
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QueueOrderingEvidence {
    producer_publish: Ordering,
    consumer_observe: Ordering,
}

impl QueueOrderingEvidence {
    #[must_use]
    pub(crate) const fn new(producer_publish: Ordering, consumer_observe: Ordering) -> Self {
        Self {
            producer_publish,
            consumer_observe,
        }
    }

    #[must_use]
    pub const fn producer_publish(self) -> Ordering {
        self.producer_publish
    }

    #[must_use]
    pub const fn consumer_observe(self) -> Ordering {
        self.consumer_observe
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServiceRingQueue<T: Copy, const CAPACITY: usize> {
    slots: [Option<PublishedEntry<T>>; CAPACITY],
    head: usize,
    tail: usize,
    len: usize,
}

impl<T: Copy, const CAPACITY: usize> ServiceRingQueue<T, CAPACITY> {
    pub const fn new() -> Result<Self, RingQueueError> {
        if CAPACITY == 0 {
            return Err(RingQueueError::ZeroCapacity);
        }

        Ok(Self {
            slots: [None; CAPACITY],
            head: 0,
            tail: 0,
            len: 0,
        })
    }

    pub fn push(&mut self, value: T) -> Result<(), RingQueueError> {
        if self.is_full() {
            return Err(RingQueueError::Full);
        }

        self.slots[self.tail] = Some(PublishedEntry {
            value,
            publish_ordering: PRODUCER_PUBLISH_ORDERING,
        });
        self.tail = self.next_index(self.tail);
        self.len += 1;

        Ok(())
    }

    pub fn pop(&mut self) -> Result<ObservedEntry<T>, RingQueueError> {
        if self.is_empty() {
            return Err(RingQueueError::Empty);
        }

        let Some(entry) = self.slots[self.head].take() else {
            return Err(RingQueueError::CorruptState);
        };
        self.head = self.next_index(self.head);
        self.len -= 1;

        Ok(ObservedEntry {
            value: entry.value,
            ordering: QueueOrderingEvidence::new(entry.publish_ordering, CONSUMER_OBSERVE_ORDERING),
        })
    }

    #[must_use]
    pub const fn capacity(&self) -> usize {
        CAPACITY
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[must_use]
    pub const fn is_full(&self) -> bool {
        self.len == CAPACITY
    }

    #[must_use]
    const fn next_index(&self, index: usize) -> usize {
        let next = index.wrapping_add(1);
        if next == CAPACITY { 0 } else { next }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServiceQueuePair<const REQUESTS: usize, const COMPLETIONS: usize> {
    submit: ServiceRingQueue<ServiceRequest, REQUESTS>,
    complete: ServiceRingQueue<ServiceCompletion, COMPLETIONS>,
}

impl<const REQUESTS: usize, const COMPLETIONS: usize> ServiceQueuePair<REQUESTS, COMPLETIONS> {
    pub const fn new() -> Result<Self, RingQueueError> {
        let Ok(submit) = ServiceRingQueue::new() else {
            return Err(RingQueueError::ZeroCapacity);
        };
        let Ok(complete) = ServiceRingQueue::new() else {
            return Err(RingQueueError::ZeroCapacity);
        };

        Ok(Self { submit, complete })
    }

    pub fn push_request(&mut self, request: ServiceRequest) -> Result<(), RingQueueError> {
        self.submit.push(request)
    }

    pub fn pop_request(&mut self) -> Result<ObservedEntry<ServiceRequest>, RingQueueError> {
        self.submit.pop()
    }

    pub fn push_completion(&mut self, completion: ServiceCompletion) -> Result<(), RingQueueError> {
        self.complete.push(completion)
    }

    pub fn pop_completion(&mut self) -> Result<ObservedEntry<ServiceCompletion>, RingQueueError> {
        self.complete.pop()
    }

    #[must_use]
    pub const fn pending_requests(&self) -> usize {
        self.submit.len()
    }

    #[must_use]
    pub const fn pending_completions(&self) -> usize {
        self.complete.len()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServiceQueueSet<const REQUESTS: usize, const COMPLETIONS: usize> {
    log: ServiceQueuePair<REQUESTS, COMPLETIONS>,
    timer: ServiceQueuePair<REQUESTS, COMPLETIONS>,
    object: ServiceQueuePair<REQUESTS, COMPLETIONS>,
}

impl<const REQUESTS: usize, const COMPLETIONS: usize> ServiceQueueSet<REQUESTS, COMPLETIONS> {
    pub const fn new() -> Result<Self, RingQueueError> {
        let Ok(log) = ServiceQueuePair::new() else {
            return Err(RingQueueError::ZeroCapacity);
        };
        let Ok(timer) = ServiceQueuePair::new() else {
            return Err(RingQueueError::ZeroCapacity);
        };
        let Ok(object) = ServiceQueuePair::new() else {
            return Err(RingQueueError::ZeroCapacity);
        };

        Ok(Self { log, timer, object })
    }

    pub fn submit(&mut self, request: ServiceRequest) -> Result<(), QueueSetError> {
        self.queue_mut(request.service())?
            .push_request(request)
            .map_err(QueueSetError::Queue)
    }

    pub fn pop_request(
        &mut self,
        service: ServiceKind,
    ) -> Result<ObservedEntry<ServiceRequest>, QueueSetError> {
        self.queue_mut(service)?
            .pop_request()
            .map_err(QueueSetError::Queue)
    }

    pub fn complete(
        &mut self,
        service: ServiceKind,
        completion: ServiceCompletion,
    ) -> Result<(), QueueSetError> {
        self.queue_mut(service)?
            .push_completion(completion)
            .map_err(QueueSetError::Queue)
    }

    pub fn pop_completion(
        &mut self,
        service: ServiceKind,
    ) -> Result<ObservedEntry<ServiceCompletion>, QueueSetError> {
        self.queue_mut(service)?
            .pop_completion()
            .map_err(QueueSetError::Queue)
    }

    #[must_use]
    pub const fn pending_requests(&self, service: ServiceKind) -> usize {
        match service {
            ServiceKind::Log => self.log.pending_requests(),
            ServiceKind::Timer => self.timer.pending_requests(),
            ServiceKind::Object => self.object.pending_requests(),
            ServiceKind::Capability
            | ServiceKind::Memory
            | ServiceKind::Driver
            | ServiceKind::Telemetry => 0,
        }
    }

    fn queue_mut(
        &mut self,
        service: ServiceKind,
    ) -> Result<&mut ServiceQueuePair<REQUESTS, COMPLETIONS>, QueueSetError> {
        match service {
            ServiceKind::Log => Ok(&mut self.log),
            ServiceKind::Timer => Ok(&mut self.timer),
            ServiceKind::Object => Ok(&mut self.object),
            ServiceKind::Capability
            | ServiceKind::Memory
            | ServiceKind::Driver
            | ServiceKind::Telemetry => Err(QueueSetError::UnsupportedService),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RingQueueError {
    ZeroCapacity,
    Full,
    Empty,
    CorruptState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueueSetError {
    Queue(RingQueueError),
    UnsupportedService,
}
