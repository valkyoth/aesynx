#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoreIsa {
    X86_64,
    Aarch64,
    Riscv64,
    DeviceFabric,
    Unknown,
}

impl CoreIsa {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::X86_64 => "x86_64",
            Self::Aarch64 => "aarch64",
            Self::Riscv64 => "riscv64",
            Self::DeviceFabric => "device-fabric",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CorePerformanceClass {
    Control,
    Performance,
    Efficiency,
    Device,
    Unknown,
}

impl CorePerformanceClass {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Control => "control",
            Self::Performance => "performance",
            Self::Efficiency => "efficiency",
            Self::Device => "device",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CoreCapabilitySet {
    isa: CoreIsa,
    performance_class: CorePerformanceClass,
    local_timer: bool,
    ipi: bool,
    directed_irq: bool,
    shared_memory_atomics: bool,
}

impl CoreCapabilitySet {
    #[must_use]
    pub const fn new(isa: CoreIsa, performance_class: CorePerformanceClass) -> Self {
        Self {
            isa,
            performance_class,
            local_timer: false,
            ipi: false,
            directed_irq: false,
            shared_memory_atomics: false,
        }
    }

    #[must_use]
    pub const fn with_local_timer(mut self, enabled: bool) -> Self {
        self.local_timer = enabled;
        self
    }

    #[must_use]
    pub const fn with_ipi(mut self, enabled: bool) -> Self {
        self.ipi = enabled;
        self
    }

    #[must_use]
    pub const fn with_directed_irq(mut self, enabled: bool) -> Self {
        self.directed_irq = enabled;
        self
    }

    #[must_use]
    pub const fn with_shared_memory_atomics(mut self, enabled: bool) -> Self {
        self.shared_memory_atomics = enabled;
        self
    }

    #[must_use]
    pub const fn isa(self) -> CoreIsa {
        self.isa
    }

    #[must_use]
    pub const fn performance_class(self) -> CorePerformanceClass {
        self.performance_class
    }

    #[must_use]
    pub const fn has_local_timer(self) -> bool {
        self.local_timer
    }

    #[must_use]
    pub const fn supports_ipi(self) -> bool {
        self.ipi
    }

    #[must_use]
    pub const fn supports_directed_irq(self) -> bool {
        self.directed_irq
    }

    #[must_use]
    pub const fn supports_shared_memory_atomics(self) -> bool {
        self.shared_memory_atomics
    }
}
