use actix_files::Files;
use actix_web::http::header::CacheControl;
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, Responder, middleware, web};
use actix_ws::AggregatedMessage;
use bytestring::ByteString;
use futures_util::StreamExt as _;
use std::sync::Arc;
use tokio::sync::{Mutex, watch};
use tokio::time::{Duration, Instant, interval};
async fn ws(req: HttpRequest, body: web::Payload) -> Result<HttpResponse, actix_web::Error> {
    let (response, mut session, stream) = actix_ws::handle(&req, body)?;
    let mut stream = stream.max_frame_size(128 * 1024).aggregate_continuations();
    tracing::info!("Inserted session");
    let alive = Arc::new(Mutex::new(Instant::now()));
    let (weather_brief_sender, mut weather_brief_receiver) = watch::channel(String::new());
    tokio::spawn(async move {
        let messages = [
            "24&deg;F light snow 🌨".to_string(),
            "45&deg;F Sunny ☀".to_string(),
            "80&deg;F light rain 🌦️".to_string(),
        ];
        let mut i = 0;
        let mut wait = interval(Duration::from_secs(30));
        loop {
            wait.tick().await;
            weather_brief_sender.send(messages[i].clone()).unwrap();
            i = (i + 1) % messages.len();
        }
    });
    let mut session2 = session.clone();
    let alive2 = alive.clone();
    actix_web::rt::spawn(async move {
        let mut wait = actix_web::rt::time::interval(Duration::from_secs(5));
        let mut w = weather_brief_receiver.clone();
        session2
            .text(ByteString::from(w.borrow_and_update().clone()))
            .await
            .unwrap();
        loop {
            if w.has_changed().unwrap() {
                let s = w.borrow_and_update().clone();
                session2
                    .text(ByteString::from(s))
                    .await
                    .expect("TODO: panic message");
            }
            if session2.ping(b"").await.is_err() {
                break;
            }
            if Instant::now().duration_since(*alive2.lock().await) > Duration::from_secs(10) {
                let _ = session2.close(None).await;
                break;
            }
            wait.tick().await;
        }
    });

    actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = stream.recv().await {
            match msg {
                AggregatedMessage::Ping(bytes) => {
                    if session.pong(&bytes).await.is_err() {
                        return;
                    }
                }
                AggregatedMessage::Pong(_) => {
                    *alive.lock().await = Instant::now();
                }
                _ => (),
            };
        }
        let _ = session.close(None).await;
    });
    tracing::info!("Spawned");
    Ok(response)
}

pub async fn web() -> std::io::Result<()> {
    let port: u16 = 9000;
    println!("starting server at http://0.0.0.0:{}", port);
    HttpServer::new(|| {
        App::new()
            .route("/ws", web::get().to(ws))
            // .route("/api", web::get().to(aprs::stations_api_bounded))
            // .route("/json", web::get().to(aprs::json))
            .service(Files::new("/", "../static").index_file("index.html"))
            .service(Files::new("/data", "../data"))
            .wrap(middleware::DefaultHeaders::new().add(("Cache-Control", "no-cache")))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
