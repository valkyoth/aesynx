use core::sync::atomic::Ordering;

use crate::{CONSUMER_OBSERVE_ORDERING, QueueOrderingEvidence};

#[derive(Clone, Debug, Eq, PartialEq)]
struct PublishedEntry<T> {
    value: T,
    publish_ordering: Ordering,
}

#[derive(Clone, Debug, Eq, PartialEq)]
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServiceRingQueue<T> {
    slots: Vec<Option<PublishedEntry<T>>>,
    head: usize,
    tail: usize,
    len: usize,
}

impl<T> ServiceRingQueue<T> {
    pub fn with_capacity(capacity: usize) -> Result<Self, RingQueueError> {
        if capacity == 0 {
            return Err(RingQueueError::ZeroCapacity);
        }

        let mut slots = Vec::with_capacity(capacity);
        slots.resize_with(capacity, || None);

        Ok(Self {
            slots,
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
            publish_ordering: crate::PRODUCER_PUBLISH_ORDERING,
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
    pub fn capacity(&self) -> usize {
        self.slots.len()
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
    pub fn is_full(&self) -> bool {
        self.len == self.capacity()
    }

    #[must_use]
    fn next_index(&self, index: usize) -> usize {
        let next = index + 1;
        if next == self.capacity() { 0 } else { next }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RingQueueError {
    ZeroCapacity,
    Full,
    Empty,
    CorruptState,
}
