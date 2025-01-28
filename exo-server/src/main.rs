#[macro_use] extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/ws")]
fn echo_stream(ws: ws::WebSocket)-> ws::Stream!['static]{
    ws::Stream! { ws =>
        loop {
            rocket::tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            yield ws::Message::Text("Hello, world!".to_string());
        }
    }
}


#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, echo_stream])
}
