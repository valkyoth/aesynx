#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoreRole {
    Bootstrap,
    Scheduler,
    DriverService,
    Idle,
}

impl CoreRole {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Bootstrap => "bootstrap",
            Self::Scheduler => "scheduler",
            Self::DriverService => "driver-service",
            Self::Idle => "idle",
        }
    }

    #[must_use]
    pub const fn can_schedule_tasks(self) -> bool {
        matches!(self, Self::Bootstrap | Self::Scheduler)
    }

    #[must_use]
    pub const fn can_own_driver_irq(self) -> bool {
        matches!(self, Self::Bootstrap | Self::DriverService)
    }
}
