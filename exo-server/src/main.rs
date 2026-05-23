#[macro_use]

extern crate rocket;

use std::sync::{Arc, Mutex};
use socketcan::{CanFrame, CanSocket, EmbeddedFrame, Frame, Socket};

mod boat_data;

mod dbc {
    #![allow(unused, non_snake_case, non_camel_case_types, clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/dbc_gen.rs"));
}

// frame_P1D — CockpitECU data frame, CAN ID 23
#[derive(Clone, Default, Debug)]
struct FrameP1dState {
    dest_module: u8,
    dest_sub_module: u8,
    data_type: u8,
    data: u64,
}

type SharedState = Arc<Mutex<FrameP1dState>>;

#[launch]
fn rocket() -> _ {
    let state: SharedState = Arc::new(Mutex::new(FrameP1dState::default()));

    let state_clone = Arc::clone(&state);
    std::thread::spawn(move || {
        let socket = CanSocket::open("can0").expect("Failed to open can0");
        println!("Listening on can0 for frame_P1D (ID 23)...");

        loop {
            match socket.read_frame() {
                Ok(CanFrame::Data(frame)) if frame.raw_id() == dbc::FrameP1d::MESSAGE_ID => {
                    if let Ok(dbc::Messages::FrameP1d(msg)) =
                        dbc::Messages::from_can_message(dbc::FrameP1d::MESSAGE_ID, frame.data())
                    {
                        let mut s = state_clone.lock().unwrap();
                        *s = FrameP1dState {
                            dest_module: msg.dest_module_raw(),
                            dest_sub_module: msg.dest_sub_module_raw(),
                            data_type: msg.data_type_raw(),
                            data: msg.data_raw(),
                        };
                        println!("{:?}", *s);
                    }
                }
                Ok(_) => {}
                Err(e) => eprintln!("CAN read error: {e}"),
            }
        }
    });

    rocket::build().manage(state)
}
