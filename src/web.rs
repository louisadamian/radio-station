use actix_files::Files;
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, web};
use actix_ws::AggregatedMessage;
use bytestring::ByteString;
use futures_util::StreamExt as _;
use std::sync::Arc;
use std::thread;
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
        let mut wait = interval(Duration::from_secs(5));
        loop {
            wait.tick().await;
            weather_brief_sender.send(messages[i].clone());
            i = (i + 1) % messages.len();
        }
    });
    let mut session2 = session.clone();
    let alive2 = alive.clone();
    actix_web::rt::spawn(async move {
        let mut wait = actix_web::rt::time::interval(Duration::from_secs(5));
        let mut w = weather_brief_receiver.clone();
        session2.text(ByteString::from(w.borrow_and_update().clone())).await;
        loop {
            if w.has_changed().unwrap() {
                let s = w.borrow_and_update().clone();
                session2.text(ByteString::from(s)).await.expect("TODO: panic message"); //.unwrap();
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
    println!("starting server at http://localhost:8080");
    HttpServer::new(|| {
        App::new()
            .route("/ws", web::get().to(ws))
            .service(Files::new("/", "static").index_file("index.html"))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
