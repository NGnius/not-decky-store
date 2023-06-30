use std::collections::HashMap;

use actix_web::{get, web, Responder};

use crate::storage::IStorage;

#[get("/stats")]
pub async fn decky_statistics(data: actix_web::web::Data<Box<dyn IStorage>>) -> impl Responder {
    println!("stats");
    let plugins: HashMap<String, u64> = data.get_statistics();
    web::Json(plugins)
}
