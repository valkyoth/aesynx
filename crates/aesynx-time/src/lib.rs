#![no_std]
#![forbid(unsafe_code)]

use core::sync::atomic::{AtomicU64, Ordering};

use aesynx_abi::{CoreId, TaskId};

pub const NANOS_PER_SECOND: u64 = 1_000_000_000;

pub trait ClockSource {
    fn now_ticks(&self) -> Result<u64, ClockError>;
    fn monotonic_ns(&self) -> Result<u64, ClockError>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClockError {
    Unavailable,
    Uninitialized,
    InvalidRate,
    Overflow,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct MonotonicInstant {
    nanos: u64,
}

impl MonotonicInstant {
    #[must_use]
    pub const fn from_nanos(nanos: u64) -> Self {
        Self { nanos }
    }

    #[must_use]
    pub const fn nanos(self) -> u64 {
        self.nanos
    }

    pub const fn checked_add(self, duration: TimeSpan) -> Result<Self, ClockError> {
        match self.nanos.checked_add(duration.nanos()) {
            Some(nanos) => Ok(Self::from_nanos(nanos)),
            None => Err(ClockError::Overflow),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct TimeSpan {
    nanos: u64,
}

impl TimeSpan {
    #[must_use]
    pub const fn from_nanos(nanos: u64) -> Self {
        Self { nanos }
    }

    #[must_use]
    pub const fn nanos(self) -> u64 {
        self.nanos
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TickRate {
    hz: u64,
}

impl TickRate {
    pub const fn new(hz: u64) -> Result<Self, ClockError> {
        if hz == 0 {
            return Err(ClockError::InvalidRate);
        }

        Ok(Self { hz })
    }

    #[must_use]
    pub const fn hz(self) -> u64 {
        self.hz
    }

    pub const fn ticks_to_nanos(self, ticks: u64) -> Result<MonotonicInstant, ClockError> {
        let seconds = ticks / self.hz;
        let remainder = ticks % self.hz;
        let Some(second_nanos) = seconds.checked_mul(NANOS_PER_SECOND) else {
            return Err(ClockError::Overflow);
        };
        let Some(remainder_nanos) = remainder.checked_mul(NANOS_PER_SECOND) else {
            return Err(ClockError::Overflow);
        };
        let fractional = remainder_nanos / self.hz;

        match second_nanos.checked_add(fractional) {
            Some(nanos) => Ok(MonotonicInstant::from_nanos(nanos)),
            None => Err(ClockError::Overflow),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimerTick {
    core: CoreId,
    tick: u64,
    timestamp: u64,
}

impl TimerTick {
    #[must_use]
    #[allow(dead_code)]
    pub(crate) const fn new(core: CoreId, tick: u64, timestamp: u64) -> Self {
        Self {
            core,
            tick,
            timestamp,
        }
    }

    #[must_use]
    pub const fn core(self) -> CoreId {
        self.core
    }

    #[must_use]
    pub const fn tick(self) -> u64 {
        self.tick
    }

    #[must_use]
    pub const fn timestamp(self) -> u64 {
        self.timestamp
    }
}

pub struct TickCounter {
    ticks: AtomicU64,
    rate: TickRate,
}

impl TickCounter {
    #[must_use]
    pub const fn new(rate: TickRate) -> Self {
        Self {
            ticks: AtomicU64::new(0),
            rate,
        }
    }

    pub fn record_tick(&self, core: CoreId, timestamp: u64) -> Result<TimerTick, ClockError> {
        let next = self
            .ticks
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |value| {
                value.checked_add(1)
            })
            .map_err(|_| ClockError::Overflow)?
            + 1;
        Ok(TimerTick::new(core, next, timestamp))
    }

    #[must_use]
    pub fn ticks(&self) -> u64 {
        self.ticks.load(Ordering::Acquire)
    }

    pub fn monotonic_now(&self) -> Result<MonotonicInstant, ClockError> {
        self.rate.ticks_to_nanos(self.ticks())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WakeId(u64);

impl WakeId {
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SleepRequest {
    task: TaskId,
    deadline: MonotonicInstant,
    wake_id: WakeId,
}

impl SleepRequest {
    #[must_use]
    pub const fn new(task: TaskId, deadline: MonotonicInstant, wake_id: WakeId) -> Self {
        Self {
            task,
            deadline,
            wake_id,
        }
    }

    #[must_use]
    pub const fn task(self) -> TaskId {
        self.task
    }

    #[must_use]
    pub const fn deadline(self) -> MonotonicInstant {
        self.deadline
    }

    #[must_use]
    pub const fn wake_id(self) -> WakeId {
        self.wake_id
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SleepError {
    QueueFull,
}

pub struct SleepQueue<const N: usize> {
    slots: [Option<SleepRequest>; N],
}

impl<const N: usize> SleepQueue<N> {
    #[must_use]
    pub const fn new() -> Self {
        Self { slots: [None; N] }
    }

    pub fn schedule(&mut self, request: SleepRequest) -> Result<(), SleepError> {
        for slot in &mut self.slots {
            if slot.is_none() {
                *slot = Some(request);
                return Ok(());
            }
        }

        Err(SleepError::QueueFull)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.slots.iter().all(Option::is_none)
    }

    /// Returns the due request with the earliest deadline.
    ///
    /// If multiple requests share the same earliest deadline, the lowest slot
    /// index wins. Requests whose deadlines are still in the future are left in
    /// the queue.
    pub fn pop_due(&mut self, now: MonotonicInstant) -> Option<SleepRequest> {
        let mut earliest: Option<(usize, MonotonicInstant)> = None;

        for (index, slot) in self.slots.iter().enumerate() {
            let Some(request) = slot else {
                continue;
            };
            let deadline = request.deadline();
            if deadline > now {
                continue;
            }
            match earliest {
                Some((_earliest_index, earliest_deadline)) if earliest_deadline <= deadline => {}
                _ => earliest = Some((index, deadline)),
            }
        }

        let (index, _deadline) = earliest?;
        self.slots[index].take()
    }
}

impl<const N: usize> Default for SleepQueue<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Timeout {
    deadline: MonotonicInstant,
}

impl Timeout {
    #[must_use]
    pub const fn new(deadline: MonotonicInstant) -> Self {
        Self { deadline }
    }

    #[must_use]
    pub const fn deadline(self) -> MonotonicInstant {
        self.deadline
    }

    #[must_use]
    pub const fn expired(self, now: MonotonicInstant) -> bool {
        now.nanos() >= self.deadline.nanos()
    }
}

#[cfg(test)]
mod tests {
    use aesynx_abi::{CoreId, TaskId};

    use super::{
        ClockError, MonotonicInstant, SleepError, SleepQueue, SleepRequest, TickCounter, TickRate,
        TimeSpan, Timeout, TimerTick, WakeId,
    };

    #[test]
    fn timer_tick_exposes_kernel_stamped_values() {
        let tick = TimerTick::new(CoreId::new(1), 2, 3);

        assert_eq!(tick.core(), CoreId::new(1));
        assert_eq!(tick.tick(), 2);
        assert_eq!(tick.timestamp(), 3);
    }

    #[test]
    fn tick_counter_records_monotonic_ticks() {
        let counter = TickCounter::new(TickRate::new(100).unwrap_or(TickRate { hz: 1 }));

        let first = counter.record_tick(CoreId::new(0), 10);
        let second = counter.record_tick(CoreId::new(0), 20);

        assert_eq!(first.map(TimerTick::tick), Ok(1));
        assert_eq!(second.map(TimerTick::tick), Ok(2));
        assert_eq!(counter.ticks(), 2);
        assert_eq!(
            counter.monotonic_now().map(MonotonicInstant::nanos),
            Ok(20_000_000)
        );
    }

    #[test]
    fn tick_rate_rejects_zero_and_converts_to_nanoseconds() {
        assert_eq!(TickRate::new(0), Err(ClockError::InvalidRate));
        let rate = TickRate::new(100).unwrap_or(TickRate { hz: 1 });

        assert_eq!(rate.hz(), 100);
        assert_eq!(
            rate.ticks_to_nanos(1).map(MonotonicInstant::nanos),
            Ok(10_000_000)
        );
        assert_eq!(
            rate.ticks_to_nanos(150).map(MonotonicInstant::nanos),
            Ok(1_500_000_000)
        );
    }

    #[test]
    fn instant_checked_add_detects_overflow() {
        assert_eq!(
            MonotonicInstant::from_nanos(10)
                .checked_add(TimeSpan::from_nanos(5))
                .map(MonotonicInstant::nanos),
            Ok(15)
        );
        assert_eq!(
            MonotonicInstant::from_nanos(u64::MAX).checked_add(TimeSpan::from_nanos(1)),
            Err(ClockError::Overflow)
        );
    }

    #[test]
    fn sleep_queue_pops_due_requests() {
        let mut queue = SleepQueue::<2>::new();
        let request = SleepRequest::new(
            TaskId::new(7),
            MonotonicInstant::from_nanos(20),
            WakeId::new(3),
        );

        assert!(queue.is_empty());
        assert_eq!(queue.schedule(request), Ok(()));
        assert_eq!(queue.pop_due(MonotonicInstant::from_nanos(19)), None);
        assert_eq!(
            queue.pop_due(MonotonicInstant::from_nanos(20)),
            Some(request)
        );
        assert!(queue.is_empty());
    }

    #[test]
    fn sleep_queue_pops_earliest_due_request() {
        let mut queue = SleepQueue::<2>::new();
        let later = SleepRequest::new(
            TaskId::new(1),
            MonotonicInstant::from_nanos(30),
            WakeId::new(1),
        );
        let earlier = SleepRequest::new(
            TaskId::new(2),
            MonotonicInstant::from_nanos(20),
            WakeId::new(2),
        );

        assert_eq!(queue.schedule(later), Ok(()));
        assert_eq!(queue.schedule(earlier), Ok(()));
        assert_eq!(
            queue.pop_due(MonotonicInstant::from_nanos(30)),
            Some(earlier)
        );
        assert_eq!(queue.pop_due(MonotonicInstant::from_nanos(30)), Some(later));
    }

    #[test]
    fn sleep_queue_reports_full() {
        let mut queue = SleepQueue::<1>::new();
        let request = SleepRequest::new(
            TaskId::new(1),
            MonotonicInstant::from_nanos(1),
            WakeId::new(1),
        );

        assert_eq!(queue.schedule(request), Ok(()));
        assert_eq!(queue.schedule(request), Err(SleepError::QueueFull));
    }

    #[test]
    fn timeout_tracks_expiry() {
        let timeout = Timeout::new(MonotonicInstant::from_nanos(10));

        assert_eq!(timeout.deadline().nanos(), 10);
        assert!(!timeout.expired(MonotonicInstant::from_nanos(9)));
        assert!(timeout.expired(MonotonicInstant::from_nanos(10)));
    }
}
