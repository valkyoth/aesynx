#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PageTableStatus {
    pub total_tables: u64,
    pub used_tables: u64,
    pub mapped_pages: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PageTableAudit {
    total_tables: u64,
    used_tables: u64,
    reachable_tables: u64,
    mapped_pages: u64,
}

impl PageTableAudit {
    #[must_use]
    pub const fn new(
        total_tables: u64,
        used_tables: u64,
        reachable_tables: u64,
        mapped_pages: u64,
    ) -> Self {
        Self {
            total_tables,
            used_tables,
            reachable_tables,
            mapped_pages,
        }
    }

    #[must_use]
    pub const fn total_tables(self) -> u64 {
        self.total_tables
    }

    #[must_use]
    pub const fn used_tables(self) -> u64 {
        self.used_tables
    }

    #[must_use]
    pub const fn reachable_tables(self) -> u64 {
        self.reachable_tables
    }

    #[must_use]
    pub const fn mapped_pages(self) -> u64 {
        self.mapped_pages
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PageTableMappingSummary {
    pub(crate) total_pages: u64,
    pub(crate) kernel_pages: u64,
    pub(crate) user_pages: u64,
    pub(crate) writable_pages: u64,
    pub(crate) executable_pages: u64,
    pub(crate) global_pages: u64,
    pub(crate) device_pages: u64,
}

impl PageTableMappingSummary {
    #[must_use]
    pub const fn new(
        total_pages: u64,
        kernel_pages: u64,
        user_pages: u64,
        writable_pages: u64,
        executable_pages: u64,
        global_pages: u64,
        device_pages: u64,
    ) -> Self {
        Self {
            total_pages,
            kernel_pages,
            user_pages,
            writable_pages,
            executable_pages,
            global_pages,
            device_pages,
        }
    }

    #[must_use]
    pub const fn total_pages(self) -> u64 {
        self.total_pages
    }

    #[must_use]
    pub const fn kernel_pages(self) -> u64 {
        self.kernel_pages
    }

    #[must_use]
    pub const fn user_pages(self) -> u64 {
        self.user_pages
    }

    #[must_use]
    pub const fn writable_pages(self) -> u64 {
        self.writable_pages
    }

    #[must_use]
    pub const fn executable_pages(self) -> u64 {
        self.executable_pages
    }

    #[must_use]
    pub const fn global_pages(self) -> u64 {
        self.global_pages
    }

    #[must_use]
    pub const fn device_pages(self) -> u64 {
        self.device_pages
    }
}
