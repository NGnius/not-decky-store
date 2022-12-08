use actix_web::{get, HttpResponse, Responder};

#[get("/")]
pub async fn decky_index() -> impl Responder {
    HttpResponse::Ok().body("TODO")
}
