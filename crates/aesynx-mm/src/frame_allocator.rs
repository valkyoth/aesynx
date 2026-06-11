use aesynx_abi::{PhysAddr, PhysFrame};

mod status;

pub use status::FrameAllocatorStatus;

pub const FRAME_SIZE: u64 = 4096;
const BITS_PER_WORD: u64 = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrameRegionKind {
    Free,
    Reserved,
    Kernel,
    Bootloader,
    Device,
    Acpi,
    Bad,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrameState {
    Unknown,
    Free,
    Used,
    Reserved,
    Kernel,
    Bootloader,
    Device,
    Acpi,
    Bad,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrameAllocatorError {
    EmptyAllocator,
    CapacityOverflow,
    CapacityTooSmall,
    AddressOverflow,
    CorruptAllocator,
    RegionOverlap,
    FrameOutsideAllocator,
    FrameOutsideKnownMap,
    FrameNotAllocatable,
    DoubleFree,
    InvalidFrameCount,
    OutOfFrames,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AllocatedFrames {
    start: PhysFrame,
    count: u64,
}

impl AllocatedFrames {
    #[must_use]
    pub const fn new(start: PhysFrame, count: u64) -> Self {
        Self { start, count }
    }

    #[must_use]
    pub const fn start(self) -> PhysFrame {
        self.start
    }

    #[must_use]
    pub const fn count(self) -> u64 {
        self.count
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FrameBitmap<const WORDS: usize> {
    words: [u64; WORDS],
}

impl<const WORDS: usize> FrameBitmap<WORDS> {
    const EMPTY: Self = Self { words: [0; WORDS] };

    fn get(&self, index: u64) -> bool {
        let word_index = (index / BITS_PER_WORD) as usize;
        let bit = index % BITS_PER_WORD;
        word_index < WORDS && (self.words[word_index] & (1_u64 << bit)) != 0
    }

    fn set(&mut self, index: u64, value: bool) -> Result<(), FrameAllocatorError> {
        let word_index = (index / BITS_PER_WORD) as usize;
        let bit = index % BITS_PER_WORD;
        let Some(word) = self.words.get_mut(word_index) else {
            return Err(FrameAllocatorError::CapacityTooSmall);
        };
        let mask = 1_u64 << bit;
        if value {
            *word |= mask;
        } else {
            *word &= !mask;
        }
        Ok(())
    }

    fn count_ones(&self, frames: u64) -> u64 {
        let mut count = 0u64;
        let mut index = 0u64;
        while index < frames {
            if self.get(index) {
                count += 1;
            }
            index += 1;
        }
        count
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BitmapFrameAllocator<const WORDS: usize> {
    base_frame: PhysFrame,
    total_frames: u64,
    known: FrameBitmap<WORDS>,
    free: FrameBitmap<WORDS>,
    reserved: FrameBitmap<WORDS>,
    kernel: FrameBitmap<WORDS>,
    bootloader: FrameBitmap<WORDS>,
    device: FrameBitmap<WORDS>,
    acpi: FrameBitmap<WORDS>,
    bad: FrameBitmap<WORDS>,
}

impl<const WORDS: usize> BitmapFrameAllocator<WORDS> {
    pub fn new(base_frame: PhysFrame, total_frames: u64) -> Result<Self, FrameAllocatorError> {
        if total_frames == 0 {
            return Err(FrameAllocatorError::EmptyAllocator);
        }
        let capacity = capacity_frames::<WORDS>()?;
        if total_frames > capacity {
            return Err(FrameAllocatorError::CapacityTooSmall);
        }
        if base_frame.get().checked_add(total_frames).is_none() {
            return Err(FrameAllocatorError::AddressOverflow);
        }

        Ok(Self {
            base_frame,
            total_frames,
            known: FrameBitmap::EMPTY,
            free: FrameBitmap::EMPTY,
            reserved: FrameBitmap::EMPTY,
            kernel: FrameBitmap::EMPTY,
            bootloader: FrameBitmap::EMPTY,
            device: FrameBitmap::EMPTY,
            acpi: FrameBitmap::EMPTY,
            bad: FrameBitmap::EMPTY,
        })
    }

    #[must_use]
    pub const fn base_frame(&self) -> PhysFrame {
        self.base_frame
    }

    #[must_use]
    pub const fn total_frames(&self) -> u64 {
        self.total_frames
    }

    pub fn mark_region(
        &mut self,
        start: PhysAddr,
        len: u64,
        kind: FrameRegionKind,
    ) -> Result<(), FrameAllocatorError> {
        if len == 0 {
            return Ok(());
        }
        let end = start
            .get()
            .checked_add(len)
            .ok_or(FrameAllocatorError::AddressOverflow)?;
        let aligned_start = align_up(start.get())?;
        let aligned_end = align_down(end);
        if aligned_start >= aligned_end {
            return Ok(());
        }

        let region_start = aligned_start / FRAME_SIZE;
        let region_end = aligned_end / FRAME_SIZE;
        let allocator_start = self.base_frame.get();
        let allocator_end = allocator_start
            .checked_add(self.total_frames)
            .ok_or(FrameAllocatorError::AddressOverflow)?;
        let mark_start = max(region_start, allocator_start);
        let mark_end = min(region_end, allocator_end);
        if mark_start >= mark_end {
            return Ok(());
        }

        self.validate_unknown_range(mark_start, mark_end, allocator_start)?;
        self.commit_region(mark_start, mark_end, allocator_start, kind)?;

        Ok(())
    }

    pub fn allocate_one(&mut self) -> Result<PhysFrame, FrameAllocatorError> {
        let frames = self.allocate_contiguous(1)?;
        Ok(frames.start())
    }

    pub fn allocate_contiguous(
        &mut self,
        count: u64,
    ) -> Result<AllocatedFrames, FrameAllocatorError> {
        if count == 0 || count > self.total_frames {
            return Err(FrameAllocatorError::InvalidFrameCount);
        }

        let mut run_start = 0u64;
        let mut run_len = 0u64;
        let mut index = 0u64;
        while index < self.total_frames {
            if self.free.get(index) {
                if run_len == 0 {
                    run_start = index;
                }
                run_len += 1;
                if run_len == count {
                    self.claim_run(run_start, count)?;
                    return Ok(AllocatedFrames::new(
                        PhysFrame::new(self.base_frame.get() + run_start),
                        count,
                    ));
                }
            } else {
                run_len = 0;
            }
            index += 1;
        }

        Err(FrameAllocatorError::OutOfFrames)
    }

    pub fn free(&mut self, frame: PhysFrame) -> Result<(), FrameAllocatorError> {
        let index = self.index_of(frame)?;
        match self.debug_state(frame) {
            FrameState::Used => self.free.set(index, true),
            FrameState::Free => Err(FrameAllocatorError::DoubleFree),
            FrameState::Unknown => Err(FrameAllocatorError::FrameOutsideKnownMap),
            FrameState::Reserved
            | FrameState::Kernel
            | FrameState::Bootloader
            | FrameState::Device
            | FrameState::Acpi
            | FrameState::Bad => Err(FrameAllocatorError::FrameNotAllocatable),
        }
    }

    pub fn free_contiguous(&mut self, frames: AllocatedFrames) -> Result<(), FrameAllocatorError> {
        if frames.count() == 0 {
            return Err(FrameAllocatorError::InvalidFrameCount);
        }
        self.validate_freeable_run(frames)?;
        let mut offset = 0u64;
        while offset < frames.count() {
            let frame = checked_frame_offset(frames.start(), offset)?;
            let index = self.index_of(frame)?;
            self.free.set(index, true)?;
            offset += 1;
        }
        Ok(())
    }

    pub fn debug_state(&self, frame: PhysFrame) -> FrameState {
        let Ok(index) = self.index_of(frame) else {
            return FrameState::Unknown;
        };
        if !self.known.get(index) {
            return FrameState::Unknown;
        }
        if self.free.get(index) {
            return FrameState::Free;
        }
        if self.kernel.get(index) {
            return FrameState::Kernel;
        }
        if self.bootloader.get(index) {
            return FrameState::Bootloader;
        }
        if self.device.get(index) {
            return FrameState::Device;
        }
        if self.acpi.get(index) {
            return FrameState::Acpi;
        }
        if self.bad.get(index) {
            return FrameState::Bad;
        }
        if self.reserved.get(index) {
            return FrameState::Reserved;
        }
        FrameState::Used
    }

    fn claim_run(&mut self, start: u64, count: u64) -> Result<(), FrameAllocatorError> {
        let mut offset = 0u64;
        while offset < count {
            self.free.set(start + offset, false)?;
            offset += 1;
        }
        Ok(())
    }

    fn index_of(&self, frame: PhysFrame) -> Result<u64, FrameAllocatorError> {
        let value = frame.get();
        let start = self.base_frame.get();
        let end = start
            .checked_add(self.total_frames)
            .ok_or(FrameAllocatorError::AddressOverflow)?;
        if value < start || value >= end {
            return Err(FrameAllocatorError::FrameOutsideAllocator);
        }
        Ok(value - start)
    }

    fn validate_unknown_range(
        &self,
        mark_start: u64,
        mark_end: u64,
        allocator_start: u64,
    ) -> Result<(), FrameAllocatorError> {
        let mut frame = mark_start;
        while frame < mark_end {
            let index = frame - allocator_start;
            if self.known.get(index) {
                return Err(FrameAllocatorError::RegionOverlap);
            }
            frame += 1;
        }
        Ok(())
    }

    fn commit_region(
        &mut self,
        mark_start: u64,
        mark_end: u64,
        allocator_start: u64,
        kind: FrameRegionKind,
    ) -> Result<(), FrameAllocatorError> {
        let mut frame = mark_start;
        while frame < mark_end {
            let index = frame - allocator_start;
            self.known.set(index, true)?;
            match kind {
                FrameRegionKind::Free => self.free.set(index, true)?,
                FrameRegionKind::Reserved => self.reserved.set(index, true)?,
                FrameRegionKind::Kernel => self.kernel.set(index, true)?,
                FrameRegionKind::Bootloader => self.bootloader.set(index, true)?,
                FrameRegionKind::Device => self.device.set(index, true)?,
                FrameRegionKind::Acpi => self.acpi.set(index, true)?,
                FrameRegionKind::Bad => self.bad.set(index, true)?,
            }
            frame += 1;
        }
        Ok(())
    }

    fn validate_freeable_run(&self, frames: AllocatedFrames) -> Result<(), FrameAllocatorError> {
        let mut offset = 0u64;
        while offset < frames.count() {
            let frame = checked_frame_offset(frames.start(), offset)?;
            match self.debug_state(frame) {
                FrameState::Used => {}
                FrameState::Free => return Err(FrameAllocatorError::DoubleFree),
                FrameState::Unknown => return Err(FrameAllocatorError::FrameOutsideKnownMap),
                FrameState::Reserved
                | FrameState::Kernel
                | FrameState::Bootloader
                | FrameState::Device
                | FrameState::Acpi
                | FrameState::Bad => return Err(FrameAllocatorError::FrameNotAllocatable),
            }
            offset += 1;
        }
        Ok(())
    }
}

fn capacity_frames<const WORDS: usize>() -> Result<u64, FrameAllocatorError> {
    (WORDS as u64)
        .checked_mul(BITS_PER_WORD)
        .ok_or(FrameAllocatorError::CapacityOverflow)
}

fn align_up(value: u64) -> Result<u64, FrameAllocatorError> {
    let mask = FRAME_SIZE - 1;
    value
        .checked_add(mask)
        .map(|rounded| rounded & !mask)
        .ok_or(FrameAllocatorError::AddressOverflow)
}

fn checked_frame_offset(frame: PhysFrame, offset: u64) -> Result<PhysFrame, FrameAllocatorError> {
    frame
        .get()
        .checked_add(offset)
        .map(PhysFrame::new)
        .ok_or(FrameAllocatorError::AddressOverflow)
}

const fn align_down(value: u64) -> u64 {
    value & !(FRAME_SIZE - 1)
}

const fn min(left: u64, right: u64) -> u64 {
    if left < right { left } else { right }
}

const fn max(left: u64, right: u64) -> u64 {
    if left > right { left } else { right }
}

#[cfg(test)]
mod tests;
