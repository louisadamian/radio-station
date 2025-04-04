mod metar;
mod web;
use self::web::web;
async fn get_metar() {}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    web().await
}
