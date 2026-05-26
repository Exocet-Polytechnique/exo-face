#[macro_use]
extern crate rocket;

use std::sync::{Arc, Mutex};
use rocket::futures::SinkExt;
use socketcan::{CanFrame, CanSocket, EmbeddedFrame, Frame, Socket};

mod boat_data;
use boat_data::{BoatData, ModuleData};

mod dbc {
    #![allow(unused, non_snake_case, non_camel_case_types, clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/dbc_gen.rs"));
}

// TODO: set these to the actual data_type values defined in your protocol
mod dtype {
    pub const SPEED: u8          = 0x01;
    pub const HYDROGEN_LEVEL: u8 = 0x02;
    pub const VOLTAGE: u8        = 0x10;
    pub const CURRENT: u8        = 0x11;
    pub const TEMPERATURE: u8    = 0x12;
}

// How often the WebSocket sender pushes a snapshot to the frontend
const SEND_INTERVAL_MS: u64 = 1000;

type SharedState = Arc<Mutex<BoatData>>;

// Extract (dest_module, data_type, raw_data) only from d-frames; returns None for all other variants.
fn extract_d_frame(msg: dbc::Messages) -> Option<(u8, u8, u64)> {
    match msg {
        dbc::Messages::FrameP0d(m) => Some((m.dest_module_raw(), m.data_type_raw(), m.data_raw())),
        dbc::Messages::FrameP1d(m) => Some((m.dest_module_raw(), m.data_type_raw(), m.data_raw())),
        dbc::Messages::FrameP2d(m) => Some((m.dest_module_raw(), m.data_type_raw(), m.data_raw())),
        dbc::Messages::FrameP3d(m) => Some((m.dest_module_raw(), m.data_type_raw(), m.data_raw())),
        dbc::Messages::FrameP4d(m) => Some((m.dest_module_raw(), m.data_type_raw(), m.data_raw())),
        dbc::Messages::FrameP5d(m) => Some((m.dest_module_raw(), m.data_type_raw(), m.data_raw())),
        dbc::Messages::FrameP6d(m) => Some((m.dest_module_raw(), m.data_type_raw(), m.data_raw())),
        dbc::Messages::FrameP7d(m) => Some((m.dest_module_raw(), m.data_type_raw(), m.data_raw())),
        dbc::Messages::FrameP8d(m) => Some((m.dest_module_raw(), m.data_type_raw(), m.data_raw())),
        _ => None,
    }
}

// Apply a decoded d-frame to the shared BoatData.
// dest_module is 1-indexed; module 0 carries global boat data.
// TODO: adjust the f64/f32 bit reinterpretation if your protocol uses a different encoding.
fn apply_d_frame(state: &mut BoatData, dest_module: u8, data_type: u8, raw: u64) {
    match (dest_module, data_type) {
        (0, dtype::SPEED) => state.speed = f64::from_bits(raw),
        (0, dtype::HYDROGEN_LEVEL) => state.hydrogen_level = raw as i32,
        (module, dtype::VOLTAGE) | (module, dtype::CURRENT) | (module, dtype::TEMPERATURE) => {
            let idx = module as usize;
            if idx == 0 { return; }
            // Grow the Vec so slot `idx` exists; each slot's id equals its Module enum value
            while state.modules.len() <= idx {
                let id = state.modules.len() as i32;
                state.modules.push(ModuleData { id, ..ModuleData::default() });
            }
            let m = &mut state.modules[idx];
            match data_type {
                dtype::VOLTAGE     => m.voltage = raw as i32,
                dtype::CURRENT     => m.current = raw as i32,
                dtype::TEMPERATURE => m.temperature = f32::from_bits(raw as u32),
                _ => {}
            }
        }
        _ => {}
    }
}

// --- Task 1 (CAN receiver thread): reads d-frames and updates shared state ---
fn spawn_can_receiver(state: SharedState) {
    std::thread::spawn(move || {
        let socket = CanSocket::open("can0").expect("Failed to open can0");
        println!("CAN receiver: listening on can0 for d-frames...");

        loop {
            match socket.read_frame() {
                Ok(CanFrame::Data(frame)) => {
                    let id = frame.raw_id();
                    if let Ok(msg) = dbc::Messages::from_can_message(id, frame.data()) {
                        if let Some((dest_module, data_type, raw)) = extract_d_frame(msg) {
                            let mut s = state.lock().unwrap();
                            apply_d_frame(&mut s, dest_module, data_type, raw);
                        }
                    }
                }
                Ok(_) => {}
                Err(e) => eprintln!("CAN read error: {e}"),
            }
        }
    });
}

// --- Task 2 (WebSocket sender): pushes a JSON snapshot every SEND_INTERVAL_MS ---
#[get("/")]
fn stream(ws: ws::WebSocket, state: &rocket::State<SharedState>) -> ws::Channel<'static> {
    let state = Arc::clone(state);
    ws.channel(move |mut stream| {
        Box::pin(async move {
            loop {
                let json = {
                    let s = state.lock().unwrap();
                    serde_json::to_string(&*s).unwrap()
                };
                if stream.send(ws::Message::Text(json.into())).await.is_err() {
                    break;
                }
                rocket::tokio::time::sleep(std::time::Duration::from_millis(SEND_INTERVAL_MS)).await;
            }
            Ok(())
        })
    })
}

#[launch]
fn rocket() -> _ {
    let state: SharedState = Arc::new(Mutex::new(BoatData::default()));


    // inject test data
    {
        let mut s = state.lock().unwrap();
        apply_d_frame(&mut s, 0, dtype::SPEED, f64::to_bits(42.5));
        apply_d_frame(&mut s, 1, dtype::VOLTAGE, 3700);
        apply_d_frame(&mut s, 1, dtype::TEMPERATURE, f32::to_bits(25.3) as u64);
    }

    spawn_can_receiver(Arc::clone(&state));

    rocket::build()
        .mount("/", routes![stream])
        .manage(state)
}


enum Module {
    Broadcast = 0b1111,
    Cockpit = 0b0000,
    Hydrogen = 0b0001,
    HighPower = 0b0010,
    Sensors = 0b0011,
    CoolingSystem = 0b0100,
    IsolationControl = 0b0101,
    Dashboard = 0b0110,
    Telemetry = 0b0111,
} 
