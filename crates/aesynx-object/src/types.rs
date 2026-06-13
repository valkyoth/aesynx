use core::fmt;

use aesynx_abi::{CoreId, ObjectId};
use aesynx_cap::CapKind;

pub trait KernelObject {
    fn object_id(&self) -> ObjectId;
    fn object_type(&self) -> ObjectType;
    fn owner_core(&self) -> CoreId;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObjectType {
    MemoryRegion,
    Endpoint,
    AddressSpace,
    Task,
    Process,
    Device,
    Driver,
    Queue,
    BytecodeModule,
    PersistentNode,
    ModelObject,
    TelemetryStream,
}

impl ObjectType {
    pub const fn cap_kind(self) -> CapKind {
        match self {
            Self::MemoryRegion => CapKind::Memory,
            Self::Endpoint => CapKind::Endpoint,
            Self::AddressSpace => CapKind::AddressSpace,
            Self::Task => CapKind::Task,
            Self::Process => CapKind::Process,
            Self::Device => CapKind::Device,
            Self::Driver => CapKind::Driver,
            Self::Queue => CapKind::Queue,
            Self::BytecodeModule => CapKind::Model,
            Self::PersistentNode => CapKind::Object,
            Self::ModelObject => CapKind::Model,
            Self::TelemetryStream => CapKind::Telemetry,
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct ObjectRecord {
    id: ObjectId,
    object_type: ObjectType,
    owner_core: CoreId,
    generation: u32,
}

impl ObjectRecord {
    pub(crate) const fn new(
        id: ObjectId,
        object_type: ObjectType,
        owner_core: CoreId,
        generation: u32,
    ) -> Self {
        Self {
            id,
            object_type,
            owner_core,
            generation,
        }
    }

    #[must_use]
    pub const fn generation(self) -> u32 {
        self.generation
    }

    #[must_use]
    pub const fn cap_kind(self) -> CapKind {
        self.object_type.cap_kind()
    }
}

impl KernelObject for ObjectRecord {
    fn object_id(&self) -> ObjectId {
        self.id
    }

    fn object_type(&self) -> ObjectType {
        self.object_type
    }

    fn owner_core(&self) -> CoreId {
        self.owner_core
    }
}

impl fmt::Debug for ObjectRecord {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ObjectRecord")
            .field("object_id", &"<redacted>")
            .field("object_type", &self.object_type)
            .field("owner_core", &self.owner_core)
            .field("generation", &self.generation)
            .finish()
    }
}
