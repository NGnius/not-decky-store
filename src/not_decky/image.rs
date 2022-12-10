use actix_web::{get, web, Responder};

use crate::storage::IStorage;

#[get("/plugins/{name}.png")]
pub async fn decky_image(data: web::Data<Box<dyn IStorage>>, path: web::Path<String>) -> actix_web::Result<impl Responder> {
    let zip = data.get_image(&path).map_err(|e| actix_web::error::ErrorNotFound(e.to_string()))?;
    Ok(zip)
}
