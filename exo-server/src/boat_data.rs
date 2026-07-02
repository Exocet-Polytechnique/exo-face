use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ModuleData {
    pub id: u32,
    pub voltage: f32,
    pub current: f32,
    pub temperature: f32,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct BoatData {
    pub speed: u32,
    pub hydrogen_level: u32,
    pub modules: Vec<ModuleData>,
}
