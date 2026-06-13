#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServiceKind {
    Log,
    Timer,
    Object,
    Capability,
    Memory,
    Driver,
    Telemetry,
}
