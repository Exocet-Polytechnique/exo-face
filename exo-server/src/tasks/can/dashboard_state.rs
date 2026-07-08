use crate::dbc;
use crate::boat_data::Alert;
use crate::SharedState;
use super::protocol::Module;

// Display name for the PCB alert panel. Hydrogen/Broadcast never reach here in practice.
fn display_name(m: Module) -> &'static str {
    match m {
        Module::Cockpit => "Cockpit",
        Module::TelemetryBattery => "Batterie de Télémétrie",
        Module::HighPower => "Haute Puissance",
        Module::DriverInterface => "Interface Pilote",
        Module::Hydrogen | Module::Broadcast => "Inconnu",
    }
}

// Human-readable titles for every (source, critical, code) combination.
// Code convention: 0x{PCB}{seq} — PCB is the sender's Module number (Cockpit=1,
// TelemetryBattery=3, HighPower=4, DriverInterface=5; Hydrogen=2 unused). Within a PCB's
// range, 0x_000-0x_299 are errors (critical, red in the UI) and 0x_300+ are warnings
// (yellow) — this replaces the dbc's stale VAL_ tables, which used small sequential values
// that don't match what HighPower's real firmware (or anything else, going forward) sends.
fn error_title(source: Module, critical: bool, code: u16) -> &'static str {
    match (source, critical, code) {
        (Module::Cockpit, true, 0x1000) => "Défaut du bus CAN",
        (Module::Cockpit, true, 0x1001) => "Défaut matériel",
        (Module::Cockpit, true, 0x1002) => "Délai de confirmation dépassé",

        (Module::TelemetryBattery, true, 0x3000) => "Défaut de la batterie",
        (Module::TelemetryBattery, true, 0x3001) => "Surchauffe",
        (Module::TelemetryBattery, true, 0x3002) => "Défaut du commutateur d'alimentation",
        (Module::TelemetryBattery, false, 0x3300) => "Charge faible",
        (Module::TelemetryBattery, false, 0x3301) => "Avertissement de température",

        // HighPower's real firmware ErrorType/WarningType — already follows this convention.
        (Module::HighPower, true, 0x4000) => "Capteur de température (batterie auxiliaire) manquant",
        (Module::HighPower, true, 0x4001) => "Capteur de température (batterie auxiliaire) déconnecté",
        (Module::HighPower, true, 0x4002) => "Température de la batterie auxiliaire trop élevée",
        (Module::HighPower, true, 0x4003) => "Défaut du DMS",
        (Module::HighPower, true, 0x4004) => "Défaut d'isolation",
        (Module::HighPower, false, 0x4300) => "Température élevée (batterie auxiliaire)",
        (Module::HighPower, false, 0x4301) => "Retirer le DMS",
        (Module::HighPower, false, 0x4302) => "Insérer le DMS",

        (Module::DriverInterface, true, 0x5000) => "Défaut système",
        (Module::DriverInterface, true, 0x5001) => "Défaut CAN",
        (Module::DriverInterface, false, 0x5300) => "Avertissement de communication",

        _ => "Erreur inconnue",
    }
}

// Updates `source`'s row in the PCB alert panel with its latest reported error/warning.
pub fn update_alert(state: &SharedState, source: Module, critical: bool, code: u16) {
    let mut data = state.lock().unwrap();
    if let Some(pcb) = data.pcb_status.iter_mut().find(|p| p.name == display_name(source)) {
        pcb.alert = Some(Alert {
            title: error_title(source, critical, code).to_string(),
            severity: if critical { "error" } else { "warning" }.to_string(),
        });
    }
}

// Decodes the battery signals the UI cares about straight off the raw message, since
// `categorize` only tags data frames with their source, not their decoded content.
// Aux battery charge has no CAN signal yet — exo_can.dbc only defines AuxBatteryTemperature
// for that PCB, a different quantity — so `aux_battery_charge` stays at its default until a
// real signal is added.
pub fn update_battery_gauges(state: &SharedState, msg: &mut dbc::Messages) {
    if let dbc::Messages::LpPcb03D(m) = msg {
        if let Ok(dbc::LpPcb03DSensor::M0(s)) = m.sensor() {
            state.lock().unwrap().telemetry_battery_charge = s.batt_so_c() as f32 / 255.0 * 100.0;
        }
    }
}
