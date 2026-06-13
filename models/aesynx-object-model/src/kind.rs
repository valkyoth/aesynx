#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObjectKind {
    Memory,
    Endpoint,
    Queue,
    Task,
    AddressSpace,
    Device,
    Driver,
    Package,
    SnapshotRoot,
    PersistentNode,
    TelemetryStream,
    WorldFact,
}

impl ObjectKind {
    pub const fn is_service_backed(self) -> bool {
        matches!(self, Self::Endpoint | Self::Queue | Self::Driver)
    }
}
