#![no_std]
#![deny(unsafe_code)]

use aesynx_boot::PlatformKind;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlatformProfile {
    pub kind: PlatformKind,
    pub firmware: FirmwareKind,
    pub discovery: DiscoveryKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FirmwareKind {
    Uefi,
    DirectBoot,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiscoveryKind {
    Acpi,
    DeviceTree,
    BootloaderOnly,
    Unknown,
}
