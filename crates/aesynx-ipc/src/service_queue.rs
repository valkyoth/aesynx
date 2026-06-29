use core::sync::atomic::Ordering;

use aesynx_abi::CoreId;

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
    pub(crate) const fn new(value: T, ordering: QueueOrderingEvidence) -> Self {
        Self { value, ordering }
    }

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
    // Non-Sync by design: the current queue is a single-owner `&mut self`
    // structure. Future shared-memory or SMP service rings must add external
    // synchronization and real atomic head/tail or slot-validity ordering.
    owner_core: CoreId,
    slots: [Option<PublishedEntry<T>>; CAPACITY],
    head: usize,
    tail: usize,
    len: usize,
}

impl<T: Copy, const CAPACITY: usize> ServiceRingQueue<T, CAPACITY> {
    pub const fn new(owner_core: CoreId) -> Result<Self, RingQueueError> {
        if CAPACITY == 0 {
            return Err(RingQueueError::ZeroCapacity);
        }

        Ok(Self {
            owner_core,
            slots: [None; CAPACITY],
            head: 0,
            tail: 0,
            len: 0,
        })
    }

    pub fn push(&mut self, caller: CoreId, value: T) -> Result<(), RingQueueError> {
        self.require_owner(caller)?;
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

    pub fn pop(&mut self, caller: CoreId) -> Result<ObservedEntry<T>, RingQueueError> {
        self.require_owner(caller)?;
        if self.is_empty() {
            return Err(RingQueueError::Empty);
        }

        let Some(entry) = self.slots[self.head].take() else {
            return Err(RingQueueError::CorruptState);
        };
        self.head = self.next_index(self.head);
        self.len -= 1;

        Ok(ObservedEntry::new(
            entry.value,
            QueueOrderingEvidence::new(entry.publish_ordering, CONSUMER_OBSERVE_ORDERING),
        ))
    }

    #[must_use]
    pub const fn owner_core(&self) -> CoreId {
        self.owner_core
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

    const fn require_owner(&self, caller: CoreId) -> Result<(), RingQueueError> {
        if caller.get() != self.owner_core.get() {
            return Err(RingQueueError::OwnerMismatch);
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServiceQueuePair<const REQUESTS: usize, const COMPLETIONS: usize> {
    submit: ServiceRingQueue<ServiceRequest, REQUESTS>,
    complete: ServiceRingQueue<ServiceCompletion, COMPLETIONS>,
}

impl<const REQUESTS: usize, const COMPLETIONS: usize> ServiceQueuePair<REQUESTS, COMPLETIONS> {
    pub const fn new(owner_core: CoreId) -> Result<Self, RingQueueError> {
        let Ok(submit) = ServiceRingQueue::new(owner_core) else {
            return Err(RingQueueError::ZeroCapacity);
        };
        let Ok(complete) = ServiceRingQueue::new(owner_core) else {
            return Err(RingQueueError::ZeroCapacity);
        };

        Ok(Self { submit, complete })
    }

    pub fn push_request(
        &mut self,
        caller: CoreId,
        request: ServiceRequest,
    ) -> Result<(), RingQueueError> {
        self.submit.push(caller, request)
    }

    pub fn pop_request(
        &mut self,
        caller: CoreId,
    ) -> Result<ObservedEntry<ServiceRequest>, RingQueueError> {
        self.submit.pop(caller)
    }

    pub fn push_completion(
        &mut self,
        caller: CoreId,
        completion: ServiceCompletion,
    ) -> Result<(), RingQueueError> {
        self.complete.push(caller, completion)
    }

    pub fn pop_completion(
        &mut self,
        caller: CoreId,
    ) -> Result<ObservedEntry<ServiceCompletion>, RingQueueError> {
        self.complete.pop(caller)
    }

    #[must_use]
    pub const fn pending_requests(&self) -> usize {
        self.submit.len()
    }

    #[must_use]
    pub const fn pending_completions(&self) -> usize {
        self.complete.len()
    }

    #[must_use]
    pub const fn owner_core(&self) -> CoreId {
        self.submit.owner_core()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServiceQueueSet<const REQUESTS: usize, const COMPLETIONS: usize> {
    log: ServiceQueuePair<REQUESTS, COMPLETIONS>,
    timer: ServiceQueuePair<REQUESTS, COMPLETIONS>,
    object: ServiceQueuePair<REQUESTS, COMPLETIONS>,
}

impl<const REQUESTS: usize, const COMPLETIONS: usize> ServiceQueueSet<REQUESTS, COMPLETIONS> {
    pub const fn new(owner_core: CoreId) -> Result<Self, RingQueueError> {
        let Ok(log) = ServiceQueuePair::new(owner_core) else {
            return Err(RingQueueError::ZeroCapacity);
        };
        let Ok(timer) = ServiceQueuePair::new(owner_core) else {
            return Err(RingQueueError::ZeroCapacity);
        };
        let Ok(object) = ServiceQueuePair::new(owner_core) else {
            return Err(RingQueueError::ZeroCapacity);
        };

        Ok(Self { log, timer, object })
    }

    pub fn submit(&mut self, caller: CoreId, request: ServiceRequest) -> Result<(), QueueSetError> {
        self.require_owner(caller)?;
        self.queue_mut(request.service())?
            .push_request(caller, request)
            .map_err(QueueSetError::Queue)
    }

    pub fn pop_request(
        &mut self,
        caller: CoreId,
        service: ServiceKind,
    ) -> Result<ObservedEntry<ServiceRequest>, QueueSetError> {
        self.require_owner(caller)?;
        self.queue_mut(service)?
            .pop_request(caller)
            .map_err(QueueSetError::Queue)
    }

    pub fn complete(
        &mut self,
        caller: CoreId,
        service: ServiceKind,
        completion: ServiceCompletion,
    ) -> Result<(), QueueSetError> {
        self.require_owner(caller)?;
        self.queue_mut(service)?
            .push_completion(caller, completion)
            .map_err(QueueSetError::Queue)
    }

    pub fn pop_completion(
        &mut self,
        caller: CoreId,
        service: ServiceKind,
    ) -> Result<ObservedEntry<ServiceCompletion>, QueueSetError> {
        self.require_owner(caller)?;
        self.queue_mut(service)?
            .pop_completion(caller)
            .map_err(QueueSetError::Queue)
    }

    pub const fn pending_requests(
        &self,
        caller: CoreId,
        service: ServiceKind,
    ) -> Result<usize, QueueSetError> {
        if let Err(error) = self.require_owner(caller) {
            return Err(error);
        }

        match service {
            ServiceKind::Log => Ok(self.log.pending_requests()),
            ServiceKind::Timer => Ok(self.timer.pending_requests()),
            ServiceKind::Object => Ok(self.object.pending_requests()),
            ServiceKind::Capability
            | ServiceKind::Memory
            | ServiceKind::Driver
            | ServiceKind::Telemetry => Err(QueueSetError::UnsupportedService),
        }
    }

    #[must_use]
    pub const fn owner_core(&self) -> CoreId {
        self.log.owner_core()
    }

    const fn require_owner(&self, caller: CoreId) -> Result<(), QueueSetError> {
        if caller.get() != self.owner_core().get() {
            return Err(QueueSetError::Queue(RingQueueError::OwnerMismatch));
        }

        Ok(())
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
    OwnerMismatch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueueSetError {
    Queue(RingQueueError),
    UnsupportedService,
}
