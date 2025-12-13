mod metar;
mod web;

use self::web::web;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    web().await
}
