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

// Human-readable titles for every (source, critical, code) combination defined in
// exo_can.dbc's ErrorType/WarningType VAL_ tables (Hydrogen omitted — no longer used).
fn error_title(source: Module, critical: bool, code: u16) -> &'static str {
    match (source, critical, code) {
        (Module::Cockpit, true, 0) => "Défaut du bus CAN",
        (Module::Cockpit, true, 1) => "Défaut matériel",
        (Module::Cockpit, false, 0) => "Délai CAN dépassé",

        (Module::TelemetryBattery, true, 0) => "Défaut de la batterie",
        (Module::TelemetryBattery, true, 1) => "Surchauffe",
        (Module::TelemetryBattery, true, 2) => "Défaut du commutateur d'alimentation",
        (Module::TelemetryBattery, false, 0) => "Charge faible",
        (Module::TelemetryBattery, false, 1) => "Avertissement de température",

        // HighPower's real firmware ErrorType/WarningType — the dbc's VAL_ table (0..5 / 0..3)
        // is stale and doesn't match what's actually transmitted.
        (Module::HighPower, true, 0x4000) => "Capteur de température (batterie auxiliaire) manquant",
        (Module::HighPower, true, 0x4001) => "Capteur de température (batterie auxiliaire) déconnecté",
        (Module::HighPower, true, 0x4002) => "Température de la batterie auxiliaire trop élevée",
        (Module::HighPower, true, 0x4003) => "Défaut du DMS",
        (Module::HighPower, true, 0x4004) => "Défaut d'isolation",
        (Module::HighPower, false, 0x4300) => "Température élevée (batterie auxiliaire)",
        (Module::HighPower, false, 0x4301) => "Retirer le DMS",
        (Module::HighPower, false, 0x4302) => "Insérer le DMS",

        (Module::DriverInterface, true, 0) => "Défaut système",
        (Module::DriverInterface, true, 1) => "Défaut CAN",
        (Module::DriverInterface, false, 0) => "Avertissement de communication",

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
