use aesynx_abi::{CoreId, ROOT_CORE};
use aesynx_ipc::{
    CorePairPingPong, CoreValidationError, FabricError, LiveCoreSet, ValidatedCoreId,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IpcPingPongSmokeStatus {
    pub ping_seq: u64,
    pub pong_seq: u64,
    pub backpressure_events: u64,
    pub backpressure_ok: bool,
    pub release_acquire_ok: bool,
    pub pairwise_route_ok: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IpcPingPongSmokeError {
    Core(CoreValidationError),
    Fabric(FabricError),
    UnexpectedState,
}

pub fn run() -> Result<IpcPingPongSmokeStatus, IpcPingPongSmokeError> {
    let live = PingPongCoreSet;
    let root = ValidatedCoreId::new(ROOT_CORE, &live).map_err(IpcPingPongSmokeError::Core)?;
    let peer = ValidatedCoreId::new(CoreId::new(1), &live).map_err(IpcPingPongSmokeError::Core)?;
    let mut fabric =
        CorePairPingPong::<1>::new(root, peer).map_err(IpcPingPongSmokeError::Fabric)?;
    let report = fabric.run_once().map_err(IpcPingPongSmokeError::Fabric)?;
    let pairwise_route_ok = report.ping_seq == 1 && report.pong_seq == 2;

    if !pairwise_route_ok || !report.backpressure_ok || !report.release_acquire_ok {
        return Err(IpcPingPongSmokeError::UnexpectedState);
    }

    Ok(IpcPingPongSmokeStatus {
        ping_seq: report.ping_seq,
        pong_seq: report.pong_seq,
        backpressure_events: report.backpressure_events,
        backpressure_ok: report.backpressure_ok,
        release_acquire_ok: report.release_acquire_ok,
        pairwise_route_ok,
    })
}

struct PingPongCoreSet;

impl LiveCoreSet for PingPongCoreSet {
    fn contains(&self, core: CoreId) -> bool {
        core == ROOT_CORE || core == CoreId::new(1)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn ipc_pingpong_smoke_reports_expected_evidence() {
        let status = super::run();

        assert_eq!(status.map(|value| value.ping_seq), Ok(1));
        assert_eq!(status.map(|value| value.pong_seq), Ok(2));
        assert_eq!(status.map(|value| value.backpressure_events), Ok(2));
        assert_eq!(status.map(|value| value.backpressure_ok), Ok(true));
        assert_eq!(status.map(|value| value.release_acquire_ok), Ok(true));
        assert_eq!(status.map(|value| value.pairwise_route_ok), Ok(true));
    }
}
