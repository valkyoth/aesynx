#![no_std]
#![deny(unsafe_code)]

use core::cell::Cell;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicU64, Ordering};

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

#[derive(Debug, Default, Eq, PartialEq)]
pub struct TaskTelemetry {
    pub cpu_time_ns: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub object_reads: u64,
    pub object_writes: u64,
    pub cap_checks: u64,
    pub faults: u64,
    pub queue_wait_ns: u64,
    _single_writer: PhantomData<Cell<()>>,
}

#[cfg(test)]
mod tests {
    use super::CoreTelemetry;

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
}
