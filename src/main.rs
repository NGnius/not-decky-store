mod consts;
mod not_decky;

use actix_web::{get, App, HttpResponse, HttpServer, Responder};

#[get("/version_info")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body(format!("{} v{}", consts::PACKAGE_NAME, consts::PACKAGE_VERSION))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let cors = actix_cors::Cors::default()
            .allowed_origin("https://steamloopback.host")
            .allow_any_header()
            .expose_any_header();

        App::new()
            .wrap(cors)
            .service(hello)
            .service(not_decky::decky_index)
            .service(not_decky::decky_plugins)
    })
    .bind(("0.0.0.0", 22252))?
    .run()
    .await
}
