use socketcan::{CanFrame, CanSocket, EmbeddedFrame, Socket, StandardId};

use crate::dbc;

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
    fn from_raw(raw: u8) -> Option<Self> {
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
    #[allow(dead_code)] // not broadcast yet; STARTED/IDLE cover the current handshake
    pub const STARTING: u8 = 1;
    pub const STARTED: u8 = 2;
    #[allow(dead_code)]
    pub const SHUTTING_DOWN: u8 = 3;
}

// Every incoming CAN frame, from any PCB, collapses into one of these.
#[derive(Debug)]
pub enum CanRxEvent {
    Error { source: Module, critical: bool, code: u16 },
    ProcedureStatus { source: Module, status_raw: u8, kind_raw: u8 },
    ProcedureState { source: Module, state_raw: u8 },
    ProcedureCommand { source: Module, target: Module, command_raw: u8 },
    Data { source: Module },
}

// Normalizes any of the 18 generated message types into a `CanRxEvent`.
pub fn categorize(msg: dbc::Messages) -> Option<CanRxEvent> {
    use dbc::Messages::*;

    macro_rules! error_event {
        ($m:expr, $source:expr, $critical:expr, $code_fn:ident) => {
            Some(CanRxEvent::Error { source: $source, critical: $critical, code: $m.$code_fn() as u16 })
        };
    }

    // Each PCB's M0/M1/M2 structs are independent generated types, but all expose the same
    // `_raw()` accessors, so this helper is written once per PCB rather than once per signal.
    match msg {
        HpPcb01E(m) => error_event!(m, Module::Cockpit, true, error_type_raw),
        LpPcb01E(m) => error_event!(m, Module::Cockpit, false, warning_type_raw),
        LpPcb01P(mut m) => match m.message_type() {
            Ok(dbc::LpPcb01PMessageType::M0(s)) => Some(CanRxEvent::ProcedureStatus {
                source: Module::Cockpit, status_raw: s.procedure_status_raw(), kind_raw: s.procedure_type_raw(),
            }),
            Ok(dbc::LpPcb01PMessageType::M1(s)) => Some(CanRxEvent::ProcedureState {
                source: Module::Cockpit, state_raw: s.current_state_raw(),
            }),
            Ok(dbc::LpPcb01PMessageType::M2(s)) => Some(CanRxEvent::ProcedureCommand {
                source: Module::Cockpit,
                target: Module::from_raw(s.target_module_raw())?,
                command_raw: s.command_raw(),
            }),
            Err(_) => None,
        },

        HpPcb02E(m) => error_event!(m, Module::Hydrogen, true, error_type_raw),
        LpPcb02E(m) => error_event!(m, Module::Hydrogen, false, warning_type_raw),
        LpPcb02P(mut m) => match m.message_type() {
            Ok(dbc::LpPcb02PMessageType::M0(s)) => Some(CanRxEvent::ProcedureStatus {
                source: Module::Hydrogen, status_raw: s.procedure_status_raw(), kind_raw: s.procedure_type_raw(),
            }),
            Ok(dbc::LpPcb02PMessageType::M1(s)) => Some(CanRxEvent::ProcedureState {
                source: Module::Hydrogen, state_raw: s.current_state_raw(),
            }),
            Ok(dbc::LpPcb02PMessageType::M2(s)) => Some(CanRxEvent::ProcedureCommand {
                source: Module::Hydrogen,
                target: Module::from_raw(s.target_module_raw())?,
                command_raw: s.command_raw(),
            }),
            Err(_) => None,
        },
        LpPcb02D(_) => Some(CanRxEvent::Data { source: Module::Hydrogen }),

        HpPcb03E(m) => error_event!(m, Module::TelemetryBattery, true, error_type_raw),
        LpPcb03E(m) => error_event!(m, Module::TelemetryBattery, false, warning_type_raw),
        LpPcb03P(mut m) => match m.message_type() {
            Ok(dbc::LpPcb03PMessageType::M0(s)) => Some(CanRxEvent::ProcedureStatus {
                source: Module::TelemetryBattery, status_raw: s.procedure_status_raw(), kind_raw: s.procedure_type_raw(),
            }),
            Ok(dbc::LpPcb03PMessageType::M1(s)) => Some(CanRxEvent::ProcedureState {
                source: Module::TelemetryBattery, state_raw: s.current_state_raw(),
            }),
            Ok(dbc::LpPcb03PMessageType::M2(s)) => Some(CanRxEvent::ProcedureCommand {
                source: Module::TelemetryBattery,
                target: Module::from_raw(s.target_module_raw())?,
                command_raw: s.command_raw(),
            }),
            Err(_) => None,
        },
        LpPcb03D(_) => Some(CanRxEvent::Data { source: Module::TelemetryBattery }),

        HpPcb04E(m) => error_event!(m, Module::HighPower, true, error_type_raw),
        LpPcb04E(m) => error_event!(m, Module::HighPower, false, warning_type_raw),
        LpPcb04P(mut m) => match m.message_type() {
            Ok(dbc::LpPcb04PMessageType::M0(s)) => Some(CanRxEvent::ProcedureStatus {
                source: Module::HighPower, status_raw: s.procedure_status_raw(), kind_raw: s.procedure_type_raw(),
            }),
            Ok(dbc::LpPcb04PMessageType::M1(s)) => Some(CanRxEvent::ProcedureState {
                source: Module::HighPower, state_raw: s.current_state_raw(),
            }),
            Ok(dbc::LpPcb04PMessageType::M2(s)) => Some(CanRxEvent::ProcedureCommand {
                source: Module::HighPower,
                target: Module::from_raw(s.target_module_raw())?,
                command_raw: s.command_raw(),
            }),
            Err(_) => None,
        },
        LpPcb04D(_) => Some(CanRxEvent::Data { source: Module::HighPower }),

        HpPcb05E(m) => error_event!(m, Module::DriverInterface, true, error_type_raw),
        LpPcb05E(m) => error_event!(m, Module::DriverInterface, false, warning_type_raw),
        LpPcb05P(mut m) => match m.message_type() {
            Ok(dbc::LpPcb05PMessageType::M0(s)) => Some(CanRxEvent::ProcedureStatus {
                source: Module::DriverInterface, status_raw: s.procedure_status_raw(), kind_raw: s.procedure_type_raw(),
            }),
            Ok(dbc::LpPcb05PMessageType::M1(s)) => Some(CanRxEvent::ProcedureState {
                source: Module::DriverInterface, state_raw: s.current_state_raw(),
            }),
            Ok(dbc::LpPcb05PMessageType::M2(s)) => Some(CanRxEvent::ProcedureCommand {
                source: Module::DriverInterface,
                target: Module::from_raw(s.target_module_raw())?,
                command_raw: s.command_raw(),
            }),
            Err(_) => None,
        },
    }
}

// --- Sending: DriverInterfaceHAT (this binary) is only ever the legitimate transmitter of
// LP_PCB05_P (procedure). There is intentionally no send_error/send_data.

fn send_lp_pcb05_p(socket: &CanSocket, build: impl FnOnce() -> Result<dbc::LpPcb05P, dbc::CanError>) -> std::io::Result<()> {
    let frame = build().map_err(|_| std::io::Error::other("invalid LP_PCB05_P parameters"))?;
    let id = StandardId::new(dbc::LpPcb05P::MESSAGE_ID as u16)
        .ok_or_else(|| std::io::Error::other("invalid LP_PCB05_P id"))?;
    let can_frame = CanFrame::new(id, frame.raw())
        .ok_or_else(|| std::io::Error::other("failed to build CAN frame"))?;
    socket.write_frame(&can_frame)
}

pub fn send_state(socket: &CanSocket, state_raw: u8) -> std::io::Result<()> {
    send_lp_pcb05_p(socket, || {
        let mut m1 = dbc::LpPcb05PMessageTypeM1::new();
        m1.set_current_state(state_raw)?;
        let mut frame = dbc::LpPcb05P::new(1)?;
        frame.set_m1(m1)?;
        Ok(frame)
    })
}

#[allow(dead_code)] // rounds out procedure-frame sending; no PCB other than Cockpit is orchestrated yet
pub fn send_command(socket: &CanSocket, target: Module, command_raw: u8) -> std::io::Result<()> {
    send_lp_pcb05_p(socket, || {
        let mut m2 = dbc::LpPcb05PMessageTypeM2::new();
        m2.set_target_module(target as u8)?;
        m2.set_command(command_raw)?;
        let mut frame = dbc::LpPcb05P::new(2)?;
        frame.set_m2(m2)?;
        Ok(frame)
    })
}

#[allow(dead_code)] // rounds out procedure-frame sending; not needed by the Cockpit handshake yet
pub fn send_procedure_status(socket: &CanSocket, status_raw: u8, kind_raw: u8) -> std::io::Result<()> {
    send_lp_pcb05_p(socket, || {
        let mut m0 = dbc::LpPcb05PMessageTypeM0::new();
        m0.set_procedure_status(status_raw)?;
        m0.set_procedure_type(kind_raw)?;
        let mut frame = dbc::LpPcb05P::new(0)?;
        frame.set_m0(m0)?;
        Ok(frame)
    })
}
