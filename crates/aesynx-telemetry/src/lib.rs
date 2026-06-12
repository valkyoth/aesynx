#![no_std]
#![deny(unsafe_code)]

use core::cell::Cell;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicU64, Ordering};

use aesynx_ai_policy::ScheduleFeatures;

pub const MAX_SCHEDULE_QUEUE_FEATURE: u32 = 4096;
pub const MAX_SCHEDULE_COUNTER_FEATURE: u32 = 1_000_000;
pub const MAX_SCHEDULE_RATIO_FEATURE: u32 = 10_000;

#[derive(Debug, Default)]
pub struct CoreTelemetry {
    run_queue_len: AtomicU64,
    ipc_rx_depth: AtomicU64,
    ipc_tx_pressure: AtomicU64,
    timer_ticks: AtomicU64,
    idle_ticks: AtomicU64,
    migrations_in: AtomicU64,
    migrations_out: AtomicU64,
    cap_faults: AtomicU64,
    page_faults: AtomicU64,
    driver_irqs: AtomicU64,
    service_queue_depth: AtomicU64,
}

impl CoreTelemetry {
    pub fn set_run_queue_len(&self, value: u64) {
        self.run_queue_len.store(value, Ordering::Relaxed);
    }

    pub fn set_ipc_rx_depth(&self, value: u64) {
        self.ipc_rx_depth.store(value, Ordering::Relaxed);
    }

    pub fn set_ipc_tx_pressure(&self, value: u64) {
        self.ipc_tx_pressure.store(value, Ordering::Relaxed);
    }

    pub fn set_service_queue_depth(&self, value: u64) {
        self.service_queue_depth.store(value, Ordering::Relaxed);
    }

    pub fn inc_timer_ticks(&self) {
        self.timer_ticks.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_idle_ticks(&self) {
        self.idle_ticks.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_migrations_in(&self) {
        self.migrations_in.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_migrations_out(&self) {
        self.migrations_out.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_cap_faults(&self) {
        self.cap_faults.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_page_faults(&self) {
        self.page_faults.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_driver_irqs(&self) {
        self.driver_irqs.fetch_add(1, Ordering::Relaxed);
    }

    #[must_use]
    /// Returns an advisory snapshot.
    ///
    /// Each counter is sampled independently with relaxed ordering. The result
    /// is suitable for best-effort telemetry and scheduling hints, but it is
    /// not a coherent multi-counter transaction under concurrent updates. Do
    /// not use this snapshot for real-time security decisions or fault
    /// admission on weakly ordered architectures.
    pub fn snapshot(&self) -> CoreTelemetrySnapshot {
        CoreTelemetrySnapshot {
            run_queue_len: self.run_queue_len.load(Ordering::Relaxed),
            ipc_rx_depth: self.ipc_rx_depth.load(Ordering::Relaxed),
            ipc_tx_pressure: self.ipc_tx_pressure.load(Ordering::Relaxed),
            timer_ticks: self.timer_ticks.load(Ordering::Relaxed),
            idle_ticks: self.idle_ticks.load(Ordering::Relaxed),
            migrations_in: self.migrations_in.load(Ordering::Relaxed),
            migrations_out: self.migrations_out.load(Ordering::Relaxed),
            cap_faults: self.cap_faults.load(Ordering::Relaxed),
            page_faults: self.page_faults.load(Ordering::Relaxed),
            driver_irqs: self.driver_irqs.load(Ordering::Relaxed),
            service_queue_depth: self.service_queue_depth.load(Ordering::Relaxed),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CoreTelemetrySnapshot {
    pub run_queue_len: u64,
    pub ipc_rx_depth: u64,
    pub ipc_tx_pressure: u64,
    pub timer_ticks: u64,
    pub idle_ticks: u64,
    pub migrations_in: u64,
    pub migrations_out: u64,
    pub cap_faults: u64,
    pub page_faults: u64,
    pub driver_irqs: u64,
    pub service_queue_depth: u64,
}

impl CoreTelemetrySnapshot {
    #[must_use]
    pub const fn redacted_schedule_features(self) -> ScheduleFeatures {
        ScheduleFeatures {
            run_queue_len: clamp_u64_to_u32(self.run_queue_len, MAX_SCHEDULE_QUEUE_FEATURE),
            ipc_depth: clamp_u64_to_u32(self.ipc_rx_depth, MAX_SCHEDULE_QUEUE_FEATURE),
            queue_pressure: clamp_u64_to_u32(
                max_u64(self.ipc_tx_pressure, self.service_queue_depth),
                MAX_SCHEDULE_QUEUE_FEATURE,
            ),
            object_locality_score: 0,
            cache_miss_rate: 0,
            idle_ratio: idle_ratio_basis_points(self.idle_ticks, self.timer_ticks),
            migration_cost: clamp_u64_to_u32(
                self.migrations_in.saturating_add(self.migrations_out),
                MAX_SCHEDULE_COUNTER_FEATURE,
            ),
            priority: 0,
        }
    }
}

#[must_use]
pub const fn redacted_schedule_features(snapshot: CoreTelemetrySnapshot) -> ScheduleFeatures {
    snapshot.redacted_schedule_features()
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct TaskTelemetry {
    cpu_time_ns: u64,
    messages_sent: u64,
    messages_received: u64,
    object_reads: u64,
    object_writes: u64,
    cap_checks: u64,
    faults: u64,
    queue_wait_ns: u64,
    _single_writer: PhantomData<Cell<()>>,
}

impl TaskTelemetry {
    pub fn add_cpu_time_ns(&mut self, value: u64) -> Result<(), TelemetryError> {
        self.cpu_time_ns = self
            .cpu_time_ns
            .checked_add(value)
            .ok_or(TelemetryError::CounterOverflow)?;
        Ok(())
    }

    pub fn inc_messages_sent(&mut self) -> Result<(), TelemetryError> {
        self.messages_sent = increment_counter(self.messages_sent)?;
        Ok(())
    }

    pub fn inc_messages_received(&mut self) -> Result<(), TelemetryError> {
        self.messages_received = increment_counter(self.messages_received)?;
        Ok(())
    }

    pub fn inc_object_reads(&mut self) -> Result<(), TelemetryError> {
        self.object_reads = increment_counter(self.object_reads)?;
        Ok(())
    }

    pub fn inc_object_writes(&mut self) -> Result<(), TelemetryError> {
        self.object_writes = increment_counter(self.object_writes)?;
        Ok(())
    }

    pub fn inc_cap_checks(&mut self) -> Result<(), TelemetryError> {
        self.cap_checks = increment_counter(self.cap_checks)?;
        Ok(())
    }

    pub fn inc_faults(&mut self) -> Result<(), TelemetryError> {
        self.faults = increment_counter(self.faults)?;
        Ok(())
    }

    pub fn add_queue_wait_ns(&mut self, value: u64) -> Result<(), TelemetryError> {
        self.queue_wait_ns = self
            .queue_wait_ns
            .checked_add(value)
            .ok_or(TelemetryError::CounterOverflow)?;
        Ok(())
    }

    #[must_use]
    pub const fn snapshot(&self) -> TaskTelemetrySnapshot {
        TaskTelemetrySnapshot {
            cpu_time_ns: self.cpu_time_ns,
            messages_sent: self.messages_sent,
            messages_received: self.messages_received,
            object_reads: self.object_reads,
            object_writes: self.object_writes,
            cap_checks: self.cap_checks,
            faults: self.faults,
            queue_wait_ns: self.queue_wait_ns,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TaskTelemetrySnapshot {
    pub cpu_time_ns: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub object_reads: u64,
    pub object_writes: u64,
    pub cap_checks: u64,
    pub faults: u64,
    pub queue_wait_ns: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelemetryError {
    CounterOverflow,
}

fn increment_counter(value: u64) -> Result<u64, TelemetryError> {
    value.checked_add(1).ok_or(TelemetryError::CounterOverflow)
}

const fn max_u64(left: u64, right: u64) -> u64 {
    if left > right { left } else { right }
}

const fn clamp_u64_to_u32(value: u64, max: u32) -> u32 {
    if value > max as u64 {
        max
    } else {
        value as u32
    }
}

const fn idle_ratio_basis_points(idle_ticks: u64, timer_ticks: u64) -> u32 {
    if timer_ticks == 0 {
        return 0;
    }

    let capped_idle = if idle_ticks > timer_ticks {
        timer_ticks
    } else {
        idle_ticks
    };
    ((capped_idle as u128 * MAX_SCHEDULE_RATIO_FEATURE as u128) / timer_ticks as u128) as u32
}

#[cfg(test)]
mod tests {
    use super::{
        CoreTelemetry, CoreTelemetrySnapshot, MAX_SCHEDULE_COUNTER_FEATURE,
        MAX_SCHEDULE_QUEUE_FEATURE, MAX_SCHEDULE_RATIO_FEATURE, TaskTelemetry, TelemetryError,
    };

    #[test]
    fn core_telemetry_records_counter_updates() {
        let telemetry = CoreTelemetry::default();

        telemetry.set_run_queue_len(3);
        telemetry.set_ipc_rx_depth(4);
        telemetry.set_ipc_tx_pressure(5);
        telemetry.set_service_queue_depth(6);
        telemetry.inc_timer_ticks();
        telemetry.inc_idle_ticks();
        telemetry.inc_migrations_in();
        telemetry.inc_migrations_out();
        telemetry.inc_cap_faults();
        telemetry.inc_page_faults();
        telemetry.inc_driver_irqs();

        let snapshot = telemetry.snapshot();
        assert_eq!(snapshot.run_queue_len, 3);
        assert_eq!(snapshot.ipc_rx_depth, 4);
        assert_eq!(snapshot.ipc_tx_pressure, 5);
        assert_eq!(snapshot.service_queue_depth, 6);
        assert_eq!(snapshot.timer_ticks, 1);
        assert_eq!(snapshot.idle_ticks, 1);
        assert_eq!(snapshot.migrations_in, 1);
        assert_eq!(snapshot.migrations_out, 1);
        assert_eq!(snapshot.cap_faults, 1);
        assert_eq!(snapshot.page_faults, 1);
        assert_eq!(snapshot.driver_irqs, 1);
    }

    #[test]
    fn core_telemetry_exports_redacted_schedule_features() {
        let features = CoreTelemetrySnapshot {
            run_queue_len: 7,
            ipc_rx_depth: 8,
            ipc_tx_pressure: 9,
            timer_ticks: 100,
            idle_ticks: 25,
            migrations_in: 2,
            migrations_out: 3,
            service_queue_depth: 10,
            ..CoreTelemetrySnapshot::default()
        }
        .redacted_schedule_features();

        assert_eq!(features.run_queue_len, 7);
        assert_eq!(features.ipc_depth, 8);
        assert_eq!(features.queue_pressure, 10);
        assert_eq!(features.idle_ratio, 2500);
        assert_eq!(features.migration_cost, 5);
        assert_eq!(features.object_locality_score, 0);
        assert_eq!(features.cache_miss_rate, 0);
        assert_eq!(features.priority, 0);
    }

    #[test]
    fn core_telemetry_redaction_clamps_untrusted_counters() {
        let features = CoreTelemetrySnapshot {
            run_queue_len: u64::MAX,
            ipc_rx_depth: u64::MAX,
            ipc_tx_pressure: 1,
            service_queue_depth: u64::MAX,
            timer_ticks: 10,
            idle_ticks: u64::MAX,
            migrations_in: u64::MAX,
            migrations_out: u64::MAX,
            ..CoreTelemetrySnapshot::default()
        }
        .redacted_schedule_features();

        assert_eq!(features.run_queue_len, MAX_SCHEDULE_QUEUE_FEATURE);
        assert_eq!(features.ipc_depth, MAX_SCHEDULE_QUEUE_FEATURE);
        assert_eq!(features.queue_pressure, MAX_SCHEDULE_QUEUE_FEATURE);
        assert_eq!(features.idle_ratio, MAX_SCHEDULE_RATIO_FEATURE);
        assert_eq!(features.migration_cost, MAX_SCHEDULE_COUNTER_FEATURE);
    }

    #[test]
    fn task_telemetry_updates_are_append_only() {
        let mut telemetry = TaskTelemetry::default();

        assert_eq!(telemetry.add_cpu_time_ns(10), Ok(()));
        assert_eq!(telemetry.inc_messages_sent(), Ok(()));
        assert_eq!(telemetry.inc_messages_received(), Ok(()));
        assert_eq!(telemetry.inc_object_reads(), Ok(()));
        assert_eq!(telemetry.inc_object_writes(), Ok(()));
        assert_eq!(telemetry.inc_cap_checks(), Ok(()));
        assert_eq!(telemetry.inc_faults(), Ok(()));
        assert_eq!(telemetry.add_queue_wait_ns(20), Ok(()));

        let snapshot = telemetry.snapshot();
        assert_eq!(snapshot.cpu_time_ns, 10);
        assert_eq!(snapshot.messages_sent, 1);
        assert_eq!(snapshot.messages_received, 1);
        assert_eq!(snapshot.object_reads, 1);
        assert_eq!(snapshot.object_writes, 1);
        assert_eq!(snapshot.cap_checks, 1);
        assert_eq!(snapshot.faults, 1);
        assert_eq!(snapshot.queue_wait_ns, 20);
    }

    #[test]
    fn task_telemetry_rejects_counter_overflow() {
        let mut telemetry = TaskTelemetry::default();

        assert_eq!(telemetry.add_cpu_time_ns(u64::MAX), Ok(()));
        assert_eq!(
            telemetry.add_cpu_time_ns(1),
            Err(TelemetryError::CounterOverflow)
        );
    }
}
