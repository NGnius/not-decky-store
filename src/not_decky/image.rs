use actix_web::{get, web, Responder};

use crate::storage::IStorage;

#[get("/plugins/{name}.png")]
pub async fn decky_image(data: web::Data<Box<dyn IStorage>>, path: web::Path<String>) -> actix_web::Result<impl Responder> {
    let zip = web::block(move || data.get_image(&path)).await
        .map_err(|e| actix_web::error::ErrorNotFound(e.to_string()))?;
    Ok(zip)
}
