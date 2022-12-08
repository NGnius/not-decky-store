use decky_api::StorePluginList;

use actix_web::{get, web::Json, Responder};

#[get("/plugins")]
pub async fn decky_plugins() -> impl Responder {
    let plugins: Vec<StorePluginList> = Vec::new();
    Json(plugins)
}
