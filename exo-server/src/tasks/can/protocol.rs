// Module addresses, per exo-can/exo_can.dbc's `TargetModule` numbering (shared by every PCB).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Module {
    Broadcast = 0,
    Cockpit = 1,
    Hydrogen = 2,
    TelemetryBattery = 3,
    HighPower = 4,
    DriverInterface = 5,
}

impl Module {
    pub fn from_raw(raw: u8) -> Option<Self> {
        match raw {
            0 => Some(Module::Broadcast),
            1 => Some(Module::Cockpit),
            2 => Some(Module::Hydrogen),
            3 => Some(Module::TelemetryBattery),
            4 => Some(Module::HighPower),
            5 => Some(Module::DriverInterface),
            _ => None,
        }
    }
}

// Shared raw values of the `Command` signal (LP_PCBnn_P, MessageType M2), same on every PCB.
pub mod command {
    pub const START: u8 = 0;
    pub const SHUTDOWN: u8 = 1;
    #[allow(dead_code)] // not sent by us yet; reacted to as an incoming command from other PCBs
    pub const FORCE_SHUTDOWN: u8 = 2;
}

// Shared raw values of the `CurrentState` signal (LP_PCBnn_P, MessageType M1).
// DriverInterfaceHAT's own state doubles as the overall boat state.
pub mod state {
    pub const IDLE: u8 = 0;
    pub const STARTING: u8 = 1;
    pub const STARTED: u8 = 2;
    pub const SHUTTING_DOWN: u8 = 3;
}

// Our own ErrorType codes on HP_PCB05_E (we're PCB 5 — DriverInterfaceHAT). See the code
// convention note on `error_title` in dashboard_state.rs.
pub mod error {
    #[allow(dead_code)]
    pub const SYSTEM_FAULT: u16 = 0x5000;
    pub const CAN_FAULT: u16 = 0x5001;
}
