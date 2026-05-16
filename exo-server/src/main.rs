use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rocket::futures::SinkExt;
mod boat_data;

#[macro_use]
extern crate rocket;

#[get("/")]
fn send_data(ws: ws::WebSocket) -> ws::Channel<'static> {
    let _rng = StdRng::from_entropy();
    ws.channel(move |mut stream| {
        Box::pin(async move {
            let mut time: f64 = 0.0;

            loop {
                // Speed Simulation
                let mut rng = StdRng::from_entropy();
                let noise = rng.gen_range(-0.5..0.5);
                let gear_shift = (time * 0.5).sin().max(0.0) * (-(time * 0.1).powi(2)).exp() * 8.0;
                let speed =
                    ((time * 2.5).min(50.0) - gear_shift + 0.5 * (time * 0.2).sin() + noise)
                        .round()
                        .max(0.0);

                // Hydrogen Level Simulation
                let hydrogen_level = ((80.0 - time * 0.01).round() as i32).max(0);

                // Temperature Simulation
                let temp1 = (40.0 + (time * 0.05).sin() * 5.0) as f32;
                let temp2 = (30.0 + (time * 0.05 + 2.0).sin() * 4.0) as f32;
                let temp3 = (32.0 + (time * 0.05 + 4.0).sin() * 3.0) as f32;

                //Voltage Simulation
                let volt_base = 48.0 * (hydrogen_level as f64 / 80.0);
                let volt1 = (volt_base + (time * 0.04).sin() * 4.0) as i32;
                let volt2 = (volt_base + (time * 0.04 + 2.0).sin() * 4.0 - 2.0) as i32;
                let volt3 = (volt_base + (time * 0.04 + 4.0).sin() * 4.0 + 2.0) as i32;

                //Current Simulation
                let curr1 = (100.0 + (time * 0.03).sin() * 5.0) as i32;
                let curr2 = (102.0 + (time * 0.03 + 2.0).sin() * 5.0) as i32;
                let curr3 = (1.2 + (time * 0.03 + 4.0).sin() * 0.05) as i32;

                //TODO: Replace this with actual data from the boat
                let data = serde_json::to_string(&boat_data::BoatData {
                    latitude: 0.0,
                    longitude: 0.0,
                    speed: speed,
                    hydrogen_level: hydrogen_level,
                    modules: vec![
                        boat_data::ModuleData {
                            id: 1,
                            status: String::from("Active"),
                            estimated_life: 3,
                            voltage: volt1,
                            current: curr1,
                            temperature: temp1,
                        },
                        boat_data::ModuleData {
                            id: 2,
                            status: String::from("Active"),
                            estimated_life: 2,
                            voltage: volt2,
                            current: curr2,
                            temperature: temp2,
                        },
                        boat_data::ModuleData {
                            id: 3,
                            status: String::from("Active"),
                            estimated_life: 1,
                            voltage: volt3,
                            current: curr3,
                            temperature: temp3,
                        },
                    ],
                });

                let _ = stream.send(ws::Message::Text(data.unwrap().into())).await;
                time += 0.200;
                rocket::tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            }
        })
    })
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![send_data])
}
