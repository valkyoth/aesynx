#![no_std]
#![deny(unsafe_code)]

use aesynx_abi::{CoreId, DeviceId, ObjectId};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeviceObject {
    id: DeviceId,
    object_id: ObjectId,
    bus: BusKind,
    owner_core: CoreId,
    state: DeviceState,
}

impl DeviceObject {
    #[must_use]
    pub const fn new(id: DeviceId, object_id: ObjectId, bus: BusKind, owner_core: CoreId) -> Self {
        Self {
            id,
            object_id,
            bus,
            owner_core,
            state: DeviceState::Discovered,
        }
    }

    #[must_use]
    pub const fn id(self) -> DeviceId {
        self.id
    }

    #[must_use]
    pub const fn object_id(self) -> ObjectId {
        self.object_id
    }

    #[must_use]
    pub const fn bus(self) -> BusKind {
        self.bus
    }

    #[must_use]
    pub const fn owner_core(self) -> CoreId {
        self.owner_core
    }

    #[must_use]
    pub const fn state(self) -> DeviceState {
        self.state
    }

    pub const fn transition(&mut self, next: DeviceState) -> Result<(), DeviceError> {
        if !device_transition_allowed(self.state, next) {
            return Err(DeviceError::InvalidStateTransition);
        }

        self.state = next;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BusKind {
    Pci,
    Usb,
    Acpi,
    DeviceTree,
    VirtioMmio,
    Platform,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeviceState {
    Discovered,
    Matched,
    Probing,
    Bound,
    Running,
    Quiescing,
    Draining,
    Stopped,
    Revoked,
    Crashed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeviceError {
    InvalidStateTransition,
}

const fn device_transition_allowed(current: DeviceState, next: DeviceState) -> bool {
    matches!(
        (current, next),
        (DeviceState::Discovered, DeviceState::Matched)
            | (DeviceState::Matched, DeviceState::Probing)
            | (DeviceState::Probing, DeviceState::Bound)
            | (DeviceState::Bound, DeviceState::Running)
            | (DeviceState::Running, DeviceState::Quiescing)
            | (DeviceState::Quiescing, DeviceState::Draining)
            | (DeviceState::Draining, DeviceState::Stopped)
            | (DeviceState::Stopped, DeviceState::Running)
            | (_, DeviceState::Revoked)
            | (_, DeviceState::Crashed)
    )
}

#[cfg(test)]
mod tests {
    use aesynx_abi::{CoreId, DeviceId, ObjectId};

    use super::{BusKind, DeviceError, DeviceObject, DeviceState};

    #[test]
    fn device_state_transitions_are_checked() {
        let mut device = DeviceObject::new(
            DeviceId::new(1),
            ObjectId::new(2),
            BusKind::VirtioMmio,
            CoreId::new(0),
        );

        assert_eq!(
            device.transition(DeviceState::Running),
            Err(DeviceError::InvalidStateTransition)
        );
        assert_eq!(device.transition(DeviceState::Matched), Ok(()));
        assert_eq!(device.transition(DeviceState::Probing), Ok(()));
        assert_eq!(device.transition(DeviceState::Bound), Ok(()));
        assert_eq!(device.transition(DeviceState::Running), Ok(()));
        assert_eq!(device.transition(DeviceState::Crashed), Ok(()));
    }

    #[test]
    fn device_identity_and_owner_are_read_only() {
        let device = DeviceObject::new(
            DeviceId::new(1),
            ObjectId::new(2),
            BusKind::VirtioMmio,
            CoreId::new(0),
        );

        assert_eq!(device.id(), DeviceId::new(1));
        assert_eq!(device.object_id(), ObjectId::new(2));
        assert_eq!(device.bus(), BusKind::VirtioMmio);
        assert_eq!(device.owner_core(), CoreId::new(0));
    }
}
