use crate::dbc;
use super::protocol::Module;

// Every incoming CAN frame, from any PCB, collapses into one of these.
#[derive(Debug)]
pub enum CanRxEvent {
    Error { source: Module, critical: bool, code: u16 },
    ProcedureStatus { source: Module, status_raw: u8, kind_raw: u8 },
    ProcedureState { source: Module, state_raw: u8 },
    ProcedureCommand { source: Module, target: Module, command_raw: u8 },
    Data { source: Module },
}

pub fn source_of(event: &CanRxEvent) -> Module {
    match *event {
        CanRxEvent::Error { source, .. }
        | CanRxEvent::ProcedureStatus { source, .. }
        | CanRxEvent::ProcedureState { source, .. }
        | CanRxEvent::ProcedureCommand { source, .. }
        | CanRxEvent::Data { source } => source,
    }
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
