#![no_std]
#![deny(unsafe_code)]

use core::sync::atomic::AtomicU64;

#[derive(Debug, Default)]
pub struct CoreTelemetry {
    pub run_queue_len: AtomicU64,
    pub ipc_rx_depth: AtomicU64,
    pub ipc_tx_pressure: AtomicU64,
    pub timer_ticks: AtomicU64,
    pub idle_ticks: AtomicU64,
    pub migrations_in: AtomicU64,
    pub migrations_out: AtomicU64,
    pub cap_faults: AtomicU64,
    pub page_faults: AtomicU64,
    pub driver_irqs: AtomicU64,
    pub service_queue_depth: AtomicU64,
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
