use super::{BitmapFrameAllocator, FrameAllocatorError};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrameAllocatorStatus {
    total_frames: u64,
    known_frames: u64,
    free_frames: u64,
    used_frames: u64,
    reserved_frames: u64,
    unknown_frames: u64,
}

impl FrameAllocatorStatus {
    #[must_use]
    pub const fn total_frames(self) -> u64 {
        self.total_frames
    }

    #[must_use]
    pub const fn known_frames(self) -> u64 {
        self.known_frames
    }

    #[must_use]
    pub const fn free_frames(self) -> u64 {
        self.free_frames
    }

    #[must_use]
    pub const fn used_frames(self) -> u64 {
        self.used_frames
    }

    #[must_use]
    pub const fn reserved_frames(self) -> u64 {
        self.reserved_frames
    }

    #[must_use]
    pub const fn unknown_frames(self) -> u64 {
        self.unknown_frames
    }
}

impl<const WORDS: usize> BitmapFrameAllocator<WORDS> {
    pub fn status(&self) -> FrameAllocatorStatus {
        let known_frames = self.known.count_ones(self.total_frames);
        let free_frames = self.free.count_ones(self.total_frames);
        let reserved_frames = self.reserved_count();
        let used_frames = known_frames
            .checked_sub(free_frames)
            .and_then(|remaining| remaining.checked_sub(reserved_frames))
            .unwrap_or_default();
        FrameAllocatorStatus {
            total_frames: self.total_frames,
            known_frames,
            free_frames,
            used_frames,
            reserved_frames,
            unknown_frames: self.total_frames.saturating_sub(known_frames),
        }
    }

    pub fn status_checked(&self) -> Result<FrameAllocatorStatus, FrameAllocatorError> {
        self.validate_status_bitmaps()?;
        let status = self.status();
        let accounted = status
            .free_frames
            .checked_add(status.used_frames)
            .and_then(|count| count.checked_add(status.reserved_frames))
            .ok_or(FrameAllocatorError::CorruptAllocator)?;
        if accounted != status.known_frames {
            return Err(FrameAllocatorError::CorruptAllocator);
        }
        Ok(status)
    }

    fn reserved_count(&self) -> u64 {
        self.reserved.count_ones(self.total_frames)
            + self.kernel.count_ones(self.total_frames)
            + self.bootloader.count_ones(self.total_frames)
            + self.device.count_ones(self.total_frames)
            + self.acpi.count_ones(self.total_frames)
            + self.bad.count_ones(self.total_frames)
    }

    pub(super) fn validate_status_bitmaps(&self) -> Result<(), FrameAllocatorError> {
        let mut index = 0u64;
        while index < self.total_frames {
            let free = self.free.get(index);
            let reserved = self.reserved.get(index);
            let kernel = self.kernel.get(index);
            let bootloader = self.bootloader.get(index);
            let device = self.device.get(index);
            let acpi = self.acpi.get(index);
            let bad = self.bad.get(index);
            let known = self.known.get(index);
            let classified = free || reserved || kernel || bootloader || device || acpi || bad;
            if classified && !known {
                return Err(FrameAllocatorError::CorruptAllocator);
            }
            let mut reserved_classes = 0u8;
            if reserved {
                reserved_classes += 1;
            }
            if kernel {
                reserved_classes += 1;
            }
            if bootloader {
                reserved_classes += 1;
            }
            if device {
                reserved_classes += 1;
            }
            if acpi {
                reserved_classes += 1;
            }
            if bad {
                reserved_classes += 1;
            }
            if free && reserved_classes > 0 {
                return Err(FrameAllocatorError::CorruptAllocator);
            }
            if reserved_classes > 1 {
                return Err(FrameAllocatorError::CorruptAllocator);
            }
            index += 1;
        }
        Ok(())
    }
}
