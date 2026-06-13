use core::sync::atomic::Ordering;

pub const PRODUCER_PUBLISH_ORDERING: Ordering = Ordering::Release;
pub const CONSUMER_OBSERVE_ORDERING: Ordering = Ordering::Acquire;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QueueOrderingEvidence {
    producer_publish: Ordering,
    consumer_observe: Ordering,
}

impl QueueOrderingEvidence {
    #[must_use]
    pub const fn new(producer_publish: Ordering, consumer_observe: Ordering) -> Self {
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

    #[must_use]
    pub const fn service_queue_contract() -> Self {
        Self::new(PRODUCER_PUBLISH_ORDERING, CONSUMER_OBSERVE_ORDERING)
    }
}
