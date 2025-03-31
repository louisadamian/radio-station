use actix_files::Files;
use actix_web::{App, Error, HttpRequest, HttpResponse, HttpServer, rt, web};
use actix_ws::AggregatedMessage;
use futures_util::StreamExt as _;
async fn socket(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let (res, mut session, stream) = actix_ws::handle(&req, stream)?;

    let mut stream = stream
        .aggregate_continuations()
        .max_continuation_size(2_usize.pow(20));

    rt::spawn(async move {
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(AggregatedMessage::Text(text)) => {
                    session.text(text).await.unwrap();
                }
                Ok(AggregatedMessage::Binary(bin)) => {
                    session.binary(bin).await.unwrap();
                }
                Ok(AggregatedMessage::Ping(msg)) => {
                    session.pong(&msg).await.unwrap();
                }
                _ => {}
            }
        }
    });
    Ok(res)
}

pub async fn web() -> std::io::Result<()> {
    println!("starting server at http://localhost:8080");
    HttpServer::new(|| {
        App::new()
            .service(Files::new("/", "static").index_file("index.html"))
            .route("/socket", web::get().to(socket))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
