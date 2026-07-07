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
    pub telemetry_battery_charge: f32, // 0-100
    pub pcb_status: Vec<PcbStatus>,
}

impl Default for BoatData {
    fn default() -> Self {
        let names = ["Cockpit", "Batterie de Télémétrie", "Haute Puissance", "Interface Pilote"];
        BoatData {
            aux_battery_charge: 0.0,
            telemetry_battery_charge: 0.0,
            pcb_status: names.iter().map(|name| PcbStatus { name: name.to_string(), alert: None }).collect(),
        }
    }
}
