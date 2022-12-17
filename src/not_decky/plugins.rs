use decky_api::StorePluginList;

use actix_web::{get, web, Responder};

use crate::storage::IStorage;

#[get("/plugins")]
pub async fn decky_plugins(data: actix_web::web::Data<Box<dyn IStorage>>) -> impl Responder {
    let plugins: StorePluginList = web::block(move || data.plugins()).await.unwrap();
    web::Json(plugins)
}
