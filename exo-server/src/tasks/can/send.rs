use socketcan::{CanFrame, CanSocket, EmbeddedFrame, Socket, StandardId};

use crate::dbc;
use super::protocol::Module;

// --- Sending: DriverInterfaceHAT (this binary) is only ever the legitimate transmitter of
// LP_PCB05_P (procedure) and HP_PCB05_E (its own error reports). There is intentionally no
// send_data — we never originate sensor telemetry.

fn write_raw(socket: &CanSocket, message_id: u32, payload: &[u8]) -> std::io::Result<()> {
    let id = StandardId::new(message_id as u16)
        .ok_or_else(|| std::io::Error::other("invalid CAN id"))?;
    let can_frame = CanFrame::new(id, payload)
        .ok_or_else(|| std::io::Error::other("failed to build CAN frame"))?;
    socket.write_frame(&can_frame)
}

fn send_lp_pcb05_p(socket: &CanSocket, build: impl FnOnce() -> Result<dbc::LpPcb05P, dbc::CanError>) -> std::io::Result<()> {
    let frame = build().map_err(|_| std::io::Error::other("invalid LP_PCB05_P parameters"))?;
    write_raw(socket, dbc::LpPcb05P::MESSAGE_ID, frame.raw())
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

pub fn send_command(socket: &CanSocket, target: Module, command_raw: u8) -> std::io::Result<()> {
    if target == Module::Hydrogen {
        return Err(std::io::Error::other("Hydrogen PCB is no longer used"));
    }
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

// Announces our own detected fault (e.g. a PCB not confirming a state change) via HP_PCB05_E.
// There's no destination field on error frames — this is self-originated, not addressed at
// whichever PCB caused it.
pub fn send_error(socket: &CanSocket, error_type_raw: u16) -> std::io::Result<()> {
    let frame = dbc::HpPcb05E::new(error_type_raw)
        .map_err(|_| std::io::Error::other("invalid HP_PCB05_E parameters"))?;
    write_raw(socket, dbc::HpPcb05E::MESSAGE_ID, frame.raw())
}
