use rocket::futures::SinkExt;
mod boat_data;

#[macro_use] extern crate rocket;

#[get("/")]
fn send_data(ws: ws::WebSocket) -> ws::Channel<'static> {
    ws.channel(move |mut stream| Box::pin(async move {
        loop{
            //TODO: Replace this with actual data from the boat
            let data = serde_json::to_string(&boat_data::BoatData{
                latitude: 14.0102020,
                longitude: 18.1203912,
                speed: 80.0,
                hydrogen_level: 80,
                modules: vec![
                    boat_data::ModuleData{
                        id: 1,
                        status: String::from("Active"),
                        estimated_life: 145,
                        voltage: 150,
                        current: 170,
                        temperature: 54.7,
                    },
                    boat_data::ModuleData{
                        id: 2,
                        status: String::from("Active"),
                        estimated_life: 60,
                        voltage: 130,
                        current: 170,
                        temperature: 20.7,
                    },
                    boat_data::ModuleData{
                        id: 3,
                        status: String::from("Active"),
                        estimated_life: 60,
                        voltage: 150,
                        current: 110,
                        temperature: 20.9,
                    }
                ]
            });
            let _ = stream.send(ws::Message::Text(data.unwrap().into())).await;
            rocket::tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }))
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![send_data])
}
