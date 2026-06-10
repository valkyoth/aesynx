use super::{PageMapping, TlbFlush};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MapOutcome {
    flush: TlbFlush,
}

impl MapOutcome {
    #[must_use]
    pub const fn new(flush: TlbFlush) -> Self {
        Self { flush }
    }

    #[must_use]
    pub const fn flush(self) -> TlbFlush {
        self.flush
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MapRangeOutcome {
    pages: u64,
    flush: TlbFlush,
}

impl MapRangeOutcome {
    #[must_use]
    pub const fn new(pages: u64, flush: TlbFlush) -> Self {
        Self { pages, flush }
    }

    #[must_use]
    pub const fn pages(self) -> u64 {
        self.pages
    }

    #[must_use]
    pub const fn flush(self) -> TlbFlush {
        self.flush
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnmapOutcome {
    mapping: PageMapping,
    flush: TlbFlush,
}

impl UnmapOutcome {
    #[must_use]
    pub const fn new(mapping: PageMapping, flush: TlbFlush) -> Self {
        Self { mapping, flush }
    }

    #[must_use]
    pub const fn mapping(self) -> PageMapping {
        self.mapping
    }

    #[must_use]
    pub const fn flush(self) -> TlbFlush {
        self.flush
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnmapRangeOutcome {
    pages: u64,
    flush: TlbFlush,
}

impl UnmapRangeOutcome {
    #[must_use]
    pub const fn new(pages: u64, flush: TlbFlush) -> Self {
        Self { pages, flush }
    }

    #[must_use]
    pub const fn pages(self) -> u64 {
        self.pages
    }

    #[must_use]
    pub const fn flush(self) -> TlbFlush {
        self.flush
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProtectOutcome {
    previous: PageMapping,
    current: PageMapping,
    flush: TlbFlush,
}

impl ProtectOutcome {
    #[must_use]
    pub const fn new(previous: PageMapping, current: PageMapping, flush: TlbFlush) -> Self {
        Self {
            previous,
            current,
            flush,
        }
    }

    #[must_use]
    pub const fn previous(self) -> PageMapping {
        self.previous
    }

    #[must_use]
    pub const fn current(self) -> PageMapping {
        self.current
    }

    #[must_use]
    pub const fn flush(self) -> TlbFlush {
        self.flush
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProtectRangeOutcome {
    pages: u64,
    flush: TlbFlush,
}

impl ProtectRangeOutcome {
    #[must_use]
    pub const fn new(pages: u64, flush: TlbFlush) -> Self {
        Self { pages, flush }
    }

    #[must_use]
    pub const fn pages(self) -> u64 {
        self.pages
    }

    #[must_use]
    pub const fn flush(self) -> TlbFlush {
        self.flush
    }
}
