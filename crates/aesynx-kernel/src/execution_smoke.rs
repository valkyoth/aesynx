use aesynx_arch::ArchCpu;

pub fn run() {
    match crate::task_smoke::run() {
        Ok(status) => {
            aesynx_arch_x86_64::serial_println!(
                "task-model created={} runnable_before={} runnable_after={} message_wait_before={} message_wait_after={} timer_wait_before={} timer_wait_after={} fifo_ok={} wake_ok={} wrong_core_denied={} zero_id_denied={}",
                status.created_tasks,
                status.runnable_before,
                status.runnable_after,
                status.message_wait_before,
                status.message_wait_after,
                status.timer_wait_before,
                status.timer_wait_after,
                status.fifo_ok,
                status.wake_ok,
                status.wrong_core_denied,
                status.zero_id_denied
            );
            aesynx_arch_x86_64::serial::write_str("[TEST] task-model=ok\n");
        }
        Err(error) => {
            aesynx_arch_x86_64::serial_println!("task-model error={:?}", error);
            aesynx_arch_x86_64::serial::write_str("[TEST] task-model=fail\n");
            aesynx_arch_x86_64::X86_64::halt_forever()
        }
    }

    match crate::cooperative_sched_smoke::run() {
        Ok(status) => {
            aesynx_arch_x86_64::serial_println!(
                "cooperative-sched task_a={} task_b={} dispatched={} yielded={} slept={} woke={} run_queue={} timer_wait={} round_robin_ok={} sleep_wake_ok={}",
                status.task_a_steps,
                status.task_b_steps,
                status.dispatched,
                status.yielded,
                status.slept,
                status.woke,
                status.final_run_queue_len,
                status.final_timer_wait_len,
                status.round_robin_ok,
                status.sleep_wake_ok
            );
            aesynx_arch_x86_64::serial::write_str("[TEST] cooperative-sched=ok\n");
        }
        Err(error) => {
            aesynx_arch_x86_64::serial_println!("cooperative-sched error={:?}", error);
            aesynx_arch_x86_64::serial::write_str("[TEST] cooperative-sched=fail\n");
            aesynx_arch_x86_64::X86_64::halt_forever()
        }
    }

    match crate::scheduler_telemetry_smoke::run() {
        Ok(status) => {
            aesynx_arch_x86_64::serial_println!(
                "scheduler-telemetry decisions={} task_a_runs={} task_b_runs={} core_run_queue={} first_reason_round_robin={} last_reason_round_robin={} trace_ok={}",
                status.decisions,
                status.task_a_runs,
                status.task_b_runs,
                status.core_run_queue_len,
                status.first_reason_round_robin,
                status.last_reason_round_robin,
                status.trace_ok
            );
            aesynx_arch_x86_64::serial::write_str("[TEST] scheduler-telemetry=ok\n");
        }
        Err(error) => {
            aesynx_arch_x86_64::serial_println!("scheduler-telemetry error={:?}", error);
            aesynx_arch_x86_64::serial::write_str("[TEST] scheduler-telemetry=fail\n");
            aesynx_arch_x86_64::X86_64::halt_forever()
        }
    }

    match crate::telemetry_events_smoke::run() {
        Ok(status) => {
            aesynx_arch_x86_64::serial_println!(
                "telemetry-events schema={} events={} boot={} cap={} sched={} boot_id={} cap_id={} sched_id={} schema_ok={}",
                status.schema_version,
                status.events,
                status.boot_events,
                status.capability_events,
                status.scheduler_events,
                status.boot_event_id,
                status.capability_event_id,
                status.scheduler_event_id,
                status.schema_ok
            );
            aesynx_arch_x86_64::serial_println!(
                "trace-event schema=1 event=boot-phase sequence=0 core=0 phase={}",
                aesynx_telemetry::TelemetryBootPhase::Running.label()
            );
            aesynx_arch_x86_64::serial_println!(
                "trace-event schema=1 event=capability-fault sequence=1 core=0 kind={} total_cap_faults=1",
                aesynx_telemetry::CapFaultKind::MissingPermission.label()
            );
            aesynx_arch_x86_64::serial_println!(
                "trace-event schema=1 event=scheduler-decision sequence=2 core=0 selected_task=<redacted> reason={} runnable_before=2 runnable_before_saturated=false timer_wait_before=0 timer_wait_before_saturated=false",
                aesynx_telemetry::SchedulerDecisionReason::RoundRobinRunnable.label()
            );
            aesynx_arch_x86_64::serial::write_str("[TEST] telemetry-events=ok\n");
        }
        Err(error) => {
            aesynx_arch_x86_64::serial_println!("telemetry-events error={:?}", error);
            aesynx_arch_x86_64::serial::write_str("[TEST] telemetry-events=fail\n");
            aesynx_arch_x86_64::X86_64::halt_forever()
        }
    }

    match crate::ai_policy_smoke::run() {
        Ok(status) => {
            aesynx_arch_x86_64::serial_println!(
                "ai-policy schema={} accepted_manifest={} rejected_manifest={} fallback_used={} fallback_confidence={} fallback_core={} safety_gate_ok={}",
                status.schema_version,
                status.accepted_manifest,
                status.rejected_manifest,
                status.fallback_used,
                status.fallback_confidence,
                status.fallback_core,
                status.safety_gate_ok
            );
            aesynx_arch_x86_64::serial::write_str("[TEST] ai-policy=ok\n");
        }
        Err(error) => {
            aesynx_arch_x86_64::serial_println!("ai-policy error={:?}", error);
            aesynx_arch_x86_64::serial::write_str("[TEST] ai-policy=fail\n");
            aesynx_arch_x86_64::X86_64::halt_forever()
        }
    }
}
