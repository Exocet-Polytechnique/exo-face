use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct Alert {
    pub title: String,
    pub severity: String, // "warning" | "error"
}

#[derive(Serialize, Clone)]
pub struct PcbStatus {
    pub name: String,
    pub alert: Option<Alert>,
}

#[derive(Serialize, Clone)]
pub struct BoatData {
    // Not yet fed by any CAN signal — exo_can.dbc doesn't define an aux battery charge
    // signal yet (only AuxBatteryTemperature exists, which is a different quantity).
    pub aux_battery_charge: f32, // 0-100
    pub aux_battery_temperature: f32, // °C — real signal (HighPower's AuxBatteryTemperature)
    // Not yet fed by any CAN signal — exo_can.dbc has no aux battery power/current/voltage
    // signal at all (HighPower's D-frame only reports AuxBatteryTemperature for it).
    pub aux_battery_power: f32, // W
    pub telemetry_battery_charge: f32, // 0-100
    pub telemetry_battery_voltage: f32, // V
    pub telemetry_battery_current: f32, // A, + = charging, - = discharging
    pub telemetry_battery_power: f32, // W
    pub telemetry_battery_temperature: f32, // °C
    pub boat_state: u8, // raw CurrentState value: Idle=0, Starting=1, Started=2, ShuttingDown=3
    pub pcb_status: Vec<PcbStatus>,
}

impl Default for BoatData {
    fn default() -> Self {
        let names = ["Cockpit", "Batterie de Télémétrie", "Haute Puissance", "Interface Pilote"];
        BoatData {
            aux_battery_charge: 0.0,
            aux_battery_temperature: 0.0,
            aux_battery_power: 0.0,
            telemetry_battery_charge: 0.0,
            telemetry_battery_voltage: 0.0,
            telemetry_battery_current: 0.0,
            telemetry_battery_power: 0.0,
            telemetry_battery_temperature: 0.0,
            boat_state: 0, // Idle
            pcb_status: names.iter().map(|name| PcbStatus { name: name.to_string(), alert: None }).collect(),
        }
    }
}
