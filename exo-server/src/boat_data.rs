use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct ModuleData{
    pub id: i32,
    pub status: String,
    pub estimaated_life: i32,
    pub voltage: i32,
    pub current: i32,
    pub temperature: i32,
}

#[derive(Serialize, Deserialize)]
pub struct BoatData{
    pub latitude: f64,
    pub longitude: f64,
    pub speed: f64,
    pub hydrogen_level: i32,
    pub modules: Vec<ModuleData>,
}