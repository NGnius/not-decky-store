use actix_web::{get, web, Responder};

use crate::storage::IStorage;

#[get("/plugins/{name}/{version}/{hash}.zip")]
pub async fn decky_artifact(data: web::Data<Box<dyn IStorage>>, path: web::Path<(String, String, String)>) -> actix_web::Result<impl Responder> {
    let zip = web::block(move || data.get_artifact(&path.0, &path.1, &path.2)).await
        .map_err(|e| actix_web::error::ErrorNotFound(e.to_string()))?;
    Ok(zip)
}
