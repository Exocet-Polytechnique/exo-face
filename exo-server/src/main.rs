use std::clone;

use rocket::futures::{SinkExt, StreamExt};

mod boat_data;

#[macro_use] extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/echo")]
fn echo(ws: ws::WebSocket) -> ws::Channel<'static> {
    ws.channel(move |mut stream| Box::pin(async move {
        loop{
            let _ = stream.send(ws::Message::Text((String::from("Hello, world!")).into())).await;
            rocket::tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }))
}


#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, echo])
}
