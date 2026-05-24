use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ModuleData {
    pub id: i32,
    pub voltage: i32,
    pub current: i32,
    pub temperature: f32,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct BoatData {
    pub speed: f64,
    pub hydrogen_level: i32,
    pub modules: Vec<ModuleData>,
}
