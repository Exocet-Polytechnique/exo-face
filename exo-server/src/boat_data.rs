use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ModuleData {
    pub id: i32,
    pub voltage: i32,
    pub current: i32,
    pub temperature: f32,
    // timestamp in milliseconds since process start when last error was seen for this module
    pub last_error_ms: u64,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct BoatData {
    pub speed: f32,
    pub hydrogen_level: i32,
    pub modules: Vec<ModuleData>,
}
