use aesynx_arch::ArchCpu;

pub fn run_or_halt() {
    match crate::capability_ipc_smoke::run() {
        Ok(status) => {
            aesynx_arch_x86_64::serial_println!(
                "cap-ipc grant_seq={} revoke_seq={} receiver_occupied={} cap_ipc_grant_message_ok={} cap_ipc_receiver_read_ok={} cap_ipc_receiver_write_denied={} cap_ipc_sender_missing_grant_denied={} cap_ipc_revoke_message_ok={} cap_ipc_registry_epoch_bumped={} cap_ipc_receiver_revoked={} cap_ipc_audit_events={} cap_ipc_grant_audit_seen={}",
                status.grant_seq,
                status.revoke_seq,
                status.receiver_occupied,
                status.grant_message_ok,
                status.receiver_read_ok,
                status.receiver_write_denied,
                status.sender_missing_grant_denied,
                status.revoke_message_ok,
                status.registry_epoch_bumped,
                status.receiver_revoked,
                status.audit_events,
                status.grant_audit_seen
            );
            aesynx_arch_x86_64::serial::write_str("[TEST] cap-ipc=ok\n");
        }
        Err(error) => {
            aesynx_arch_x86_64::serial_println!("cap-ipc error={:?}", error);
            aesynx_arch_x86_64::serial::write_str("[TEST] cap-ipc=fail\n");
            aesynx_arch_x86_64::X86_64::halt_forever()
        }
    }
}
