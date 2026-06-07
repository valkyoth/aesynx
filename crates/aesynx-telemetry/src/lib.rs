#![no_std]
#![deny(unsafe_code)]

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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TaskTelemetry {
    pub cpu_time_ns: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub object_reads: u64,
    pub object_writes: u64,
    pub cap_checks: u64,
    pub faults: u64,
    pub queue_wait_ns: u64,
}
