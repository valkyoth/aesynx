use core::sync::atomic::Ordering;

use aesynx_abi::{CoreId, MessageId};

use crate::service_queue::{CONSUMER_OBSERVE_ORDERING, PRODUCER_PUBLISH_ORDERING};
use crate::{
    MessageHeader, MessageKind, MessagePayload, MessageRequest, ObservedEntry,
    QueueOrderingEvidence, ValidatedCoreId,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FabricMessage {
    header: MessageHeader,
    payload: MessagePayload,
}

impl FabricMessage {
    pub const fn new(header: MessageHeader, payload: MessagePayload) -> Self {
        Self { header, payload }
    }

    #[must_use]
    pub const fn header(self) -> MessageHeader {
        self.header
    }

    #[must_use]
    pub const fn payload(self) -> MessagePayload {
        self.payload
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PublishedFabricMessage {
    message: FabricMessage,
    publish_ordering: Ordering,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PairwiseSpscQueue<const CAPACITY: usize> {
    // Non-Sync by design: this is the single-producer/single-consumer fabric
    // model. A live multicore ring must replace the plain indices with
    // architecture-backed atomic slot ownership and cache-line layout rules.
    // Copy is model-only here because no sequence allocator lives in this type;
    // live shared-memory rings must become linear ownership objects.
    producer_core: CoreId,
    consumer_core: CoreId,
    slots: [Option<PublishedFabricMessage>; CAPACITY],
    head: usize,
    tail: usize,
    len: usize,
}

impl<const CAPACITY: usize> PairwiseSpscQueue<CAPACITY> {
    pub const fn new(
        producer_core: ValidatedCoreId,
        consumer_core: ValidatedCoreId,
    ) -> Result<Self, FabricError> {
        if CAPACITY == 0 {
            return Err(FabricError::ZeroCapacity);
        }

        if producer_core.get().get() == consumer_core.get().get() {
            return Err(FabricError::LoopbackPair);
        }

        Ok(Self {
            producer_core: producer_core.get(),
            consumer_core: consumer_core.get(),
            slots: [None; CAPACITY],
            head: 0,
            tail: 0,
            len: 0,
        })
    }

    pub fn push(&mut self, caller: CoreId, message: FabricMessage) -> Result<(), FabricError> {
        self.require_producer(caller)?;
        self.require_message_route(message)?;

        if self.is_full() {
            return Err(FabricError::Backpressure);
        }

        self.slots[self.tail] = Some(PublishedFabricMessage {
            message,
            publish_ordering: PRODUCER_PUBLISH_ORDERING,
        });
        self.tail = self.next_index(self.tail);
        self.len += 1;

        Ok(())
    }

    pub fn pop(&mut self, caller: CoreId) -> Result<ObservedEntry<FabricMessage>, FabricError> {
        self.require_consumer(caller)?;

        if self.is_empty() {
            return Err(FabricError::Empty);
        }

        // FUTURE-SAFETY: before Cap/Object/Inline payloads cross trust
        // domains, vacated slots must be explicitly scrubbed with a
        // zero-before-observe primitive rather than relying on `Option::take`.
        let Some(entry) = self.slots[self.head].take() else {
            return Err(FabricError::CorruptState);
        };
        self.head = self.next_index(self.head);
        self.len -= 1;

        Ok(ObservedEntry::new(
            entry.message,
            QueueOrderingEvidence::new(entry.publish_ordering, CONSUMER_OBSERVE_ORDERING),
        ))
    }

    #[must_use]
    pub const fn producer_core(&self) -> CoreId {
        self.producer_core
    }

    #[must_use]
    pub const fn consumer_core(&self) -> CoreId {
        self.consumer_core
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

    const fn require_producer(&self, caller: CoreId) -> Result<(), FabricError> {
        if caller.get() != self.producer_core.get() {
            return Err(FabricError::ProducerMismatch);
        }

        Ok(())
    }

    const fn require_consumer(&self, caller: CoreId) -> Result<(), FabricError> {
        if caller.get() != self.consumer_core.get() {
            return Err(FabricError::ConsumerMismatch);
        }

        Ok(())
    }

    const fn require_message_route(&self, message: FabricMessage) -> Result<(), FabricError> {
        let header = message.header();
        if header.src().get() != self.producer_core.get()
            || header.dst().get() != self.consumer_core.get()
        {
            return Err(FabricError::RouteMismatch);
        }

        Ok(())
    }

    #[must_use]
    const fn next_index(&self, index: usize) -> usize {
        let next = index.wrapping_add(1);
        if next == CAPACITY { 0 } else { next }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CorePairPingPong<const CAPACITY: usize> {
    // Deliberately non-Copy: `next_seq` is the sole authoritative sequence
    // allocator for this pair. Copying it would fork the sequence space.
    root_core: ValidatedCoreId,
    peer_core: ValidatedCoreId,
    root_to_peer: PairwiseSpscQueue<CAPACITY>,
    peer_to_root: PairwiseSpscQueue<CAPACITY>,
    next_seq: u64,
    backpressure_events: u64,
}

impl<const CAPACITY: usize> CorePairPingPong<CAPACITY> {
    pub const fn new(
        root_core: ValidatedCoreId,
        peer_core: ValidatedCoreId,
    ) -> Result<Self, FabricError> {
        let root_to_peer = match PairwiseSpscQueue::new(root_core, peer_core) {
            Ok(queue) => queue,
            Err(error) => return Err(error),
        };
        let peer_to_root = match PairwiseSpscQueue::new(peer_core, root_core) {
            Ok(queue) => queue,
            Err(error) => return Err(error),
        };

        Ok(Self {
            root_core,
            peer_core,
            root_to_peer,
            peer_to_root,
            next_seq: 1,
            backpressure_events: 0,
        })
    }

    pub fn run_once(&mut self) -> Result<PingPongReport, FabricError> {
        let ping_seq = push_message(
            &mut self.root_to_peer,
            &mut self.next_seq,
            self.root_core,
            self.peer_core,
            MessageKind::Ping,
            None,
        )?;
        let observed_ping = self.root_to_peer.pop(self.peer_core.get())?;
        let ordering = observed_ping.ordering();
        let ping = observed_ping.into_value();
        self.require_kind(ping, MessageKind::Ping)?;

        let pong_seq = push_message(
            &mut self.peer_to_root,
            &mut self.next_seq,
            self.peer_core,
            self.root_core,
            MessageKind::Pong,
            Some(MessageId::new(ping_seq)),
        )?;
        let observed_pong = self.peer_to_root.pop(self.root_core.get())?;
        let pong = observed_pong.into_value();
        self.require_kind(pong, MessageKind::Pong)?;
        self.require_reply_to(pong, MessageId::new(ping_seq))?;

        let backpressure_ok = self.prove_backpressure()?;
        let release_acquire_ok = ordering.producer_publish() == Ordering::Release
            && ordering.consumer_observe() == Ordering::Acquire;

        Ok(PingPongReport {
            ping_seq,
            pong_seq,
            backpressure_events: self.backpressure_events,
            backpressure_ok,
            release_acquire_ok,
        })
    }

    fn prove_backpressure(&mut self) -> Result<bool, FabricError> {
        let forward = prove_backpressure_on(
            &mut self.root_to_peer,
            &mut self.next_seq,
            &mut self.backpressure_events,
            self.root_core,
            self.peer_core,
        )?;
        let reverse = prove_backpressure_on(
            &mut self.peer_to_root,
            &mut self.next_seq,
            &mut self.backpressure_events,
            self.peer_core,
            self.root_core,
        )?;

        Ok(forward && reverse)
    }

    fn require_kind(
        &self,
        message: FabricMessage,
        expected: MessageKind,
    ) -> Result<(), FabricError> {
        if message.header().kind() != expected {
            return Err(FabricError::UnexpectedMessage);
        }

        Ok(())
    }

    fn require_reply_to(
        &self,
        message: FabricMessage,
        expected: MessageId,
    ) -> Result<(), FabricError> {
        if message.header().reply_to() != Some(expected) {
            return Err(FabricError::UnexpectedMessage);
        }

        Ok(())
    }
}

fn push_message<const CAPACITY: usize>(
    queue: &mut PairwiseSpscQueue<CAPACITY>,
    next_seq: &mut u64,
    src: ValidatedCoreId,
    dst: ValidatedCoreId,
    kind: MessageKind,
    reply_to: Option<MessageId>,
) -> Result<u64, FabricError> {
    let seq = *next_seq;
    let message = message_with_seq(seq, src, dst, kind, reply_to);
    queue.push(src.get(), message)?;
    *next_seq = seq.checked_add(1).ok_or(FabricError::SequenceOverflow)?;

    Ok(seq)
}

fn prove_backpressure_on<const CAPACITY: usize>(
    queue: &mut PairwiseSpscQueue<CAPACITY>,
    next_seq: &mut u64,
    backpressure_events: &mut u64,
    src: ValidatedCoreId,
    dst: ValidatedCoreId,
) -> Result<bool, FabricError> {
    let mut enqueued = 0usize;
    while !queue.is_full() {
        let _ = push_message(queue, next_seq, src, dst, MessageKind::Ping, None)?;
        enqueued += 1;
    }

    let blocked = message_with_seq(*next_seq, src, dst, MessageKind::Ping, None);
    let backpressure_ok = queue.push(src.get(), blocked) == Err(FabricError::Backpressure);
    if backpressure_ok {
        *backpressure_events = backpressure_events
            .checked_add(1)
            .ok_or(FabricError::SequenceOverflow)?;
    }

    while enqueued > 0 {
        let _ = queue.pop(dst.get())?;
        enqueued -= 1;
    }

    Ok(backpressure_ok)
}

fn message_with_seq(
    seq: u64,
    src: ValidatedCoreId,
    dst: ValidatedCoreId,
    kind: MessageKind,
    reply_to: Option<MessageId>,
) -> FabricMessage {
    let request = MessageRequest {
        dst: dst.get(),
        kind,
        reply_to,
    };
    FabricMessage::new(
        MessageHeader::stamp(request, src, seq, dst),
        MessagePayload::Empty,
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PingPongReport {
    pub ping_seq: u64,
    pub pong_seq: u64,
    pub backpressure_events: u64,
    pub backpressure_ok: bool,
    pub release_acquire_ok: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FabricError {
    ZeroCapacity,
    LoopbackPair,
    ProducerMismatch,
    ConsumerMismatch,
    RouteMismatch,
    Backpressure,
    Empty,
    CorruptState,
    SequenceOverflow,
    UnexpectedMessage,
}

#[cfg(test)]
mod tests {
    use aesynx_abi::CoreId;

    use crate::{CorePairPingPong, FabricError, LiveCoreSet, PairwiseSpscQueue, ValidatedCoreId};

    struct TwoCoreSet;

    impl LiveCoreSet for TwoCoreSet {
        fn contains(&self, core: CoreId) -> bool {
            core == CoreId::new(0) || core == CoreId::new(1)
        }
    }

    #[test]
    fn ping_pong_reports_sequences_and_backpressure() -> Result<(), FabricError> {
        let live = TwoCoreSet;
        let root = ValidatedCoreId::new(CoreId::new(0), &live)
            .map_err(|_| FabricError::UnexpectedMessage)?;
        let peer = ValidatedCoreId::new(CoreId::new(1), &live)
            .map_err(|_| FabricError::UnexpectedMessage)?;
        let mut fabric = CorePairPingPong::<1>::new(root, peer)?;

        let report = fabric.run_once()?;

        assert_eq!(report.ping_seq, 1);
        assert_eq!(report.pong_seq, 2);
        assert_eq!(report.backpressure_events, 2);
        assert!(report.backpressure_ok);
        assert!(report.release_acquire_ok);

        Ok(())
    }

    #[test]
    fn pairwise_queue_rejects_loopback_pairs() {
        let live = TwoCoreSet;
        let root =
            ValidatedCoreId::new(CoreId::new(0), &live).map_err(|_| FabricError::UnexpectedMessage);

        assert_eq!(
            root.and_then(|root| PairwiseSpscQueue::<1>::new(root, root)),
            Err(FabricError::LoopbackPair)
        );
    }

    #[test]
    fn ping_pong_backpressure_drains_generic_capacity() -> Result<(), FabricError> {
        let live = TwoCoreSet;
        let root = ValidatedCoreId::new(CoreId::new(0), &live)
            .map_err(|_| FabricError::UnexpectedMessage)?;
        let peer = ValidatedCoreId::new(CoreId::new(1), &live)
            .map_err(|_| FabricError::UnexpectedMessage)?;
        let mut fabric = CorePairPingPong::<4>::new(root, peer)?;

        let report = fabric.run_once()?;

        assert!(report.backpressure_ok);
        assert_eq!(report.backpressure_events, 2);
        assert_eq!(fabric.root_to_peer.len(), 0);
        assert_eq!(fabric.peer_to_root.len(), 0);

        Ok(())
    }
}
