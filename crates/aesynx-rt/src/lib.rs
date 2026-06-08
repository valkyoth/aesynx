#![no_std]
#![deny(unsafe_code)]

use aesynx_abi::CapId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Env {
    console_in: CapId,
    console_out: CapId,
    process_service: CapId,
    object_root: CapId,
}

impl Env {
    pub const fn new(
        console_in: CapId,
        console_out: CapId,
        process_service: CapId,
        object_root: CapId,
    ) -> Self {
        Self {
            console_in,
            console_out,
            process_service,
            object_root,
        }
    }

    pub const fn console_in(self) -> CapId {
        self.console_in
    }

    pub const fn console_out(self) -> CapId {
        self.console_out
    }

    pub const fn process_service(self) -> CapId {
        self.process_service
    }

    pub const fn object_root(self) -> CapId {
        self.object_root
    }
}

#[cfg(test)]
mod tests {
    use aesynx_abi::CapId;

    use super::Env;

    #[test]
    fn env_capability_ids_are_read_only() {
        let env = Env::new(CapId::new(1), CapId::new(2), CapId::new(3), CapId::new(4));

        assert_eq!(env.console_in(), CapId::new(1));
        assert_eq!(env.console_out(), CapId::new(2));
        assert_eq!(env.process_service(), CapId::new(3));
        assert_eq!(env.object_root(), CapId::new(4));
    }
}
