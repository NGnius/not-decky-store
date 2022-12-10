mod cli;
mod consts;
mod not_decky;
mod storage;

use crate::storage::IStorageWrap;

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use simplelog::{LevelFilter, WriteLogger};

#[get("/version_info")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body(format!("{} v{}", consts::PACKAGE_NAME, consts::PACKAGE_VERSION))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = cli::CliArgs::get();
    let log_filepath = std::path::Path::new("/tmp").join(format!("{}.log", consts::PACKAGE_NAME));
    WriteLogger::init(
        LevelFilter::Debug,
        Default::default(),
        std::fs::File::create(&log_filepath).unwrap(),
    )
    .unwrap();

    println!("Logging to {}", log_filepath.display());

    HttpServer::new(move || {
        let cors = actix_cors::Cors::default()
            //.allowed_origin("https://steamloopback.host")
            .allow_any_origin()
            .send_wildcard()
            .allow_any_method()
            .allow_any_header()
            .expose_any_header();

        let storage_data: Box<dyn storage::IStorage> = match &args.storage {
            cli::StorageArgs::Default => storage::FileStorage::new(
                "./store".into(),
                "http://192.168.0.128:22252".into(),
                true,
            ).wrap(args.clone()),
            cli::StorageArgs::Filesystem(fs) => storage::FileStorage::new(
                fs.root.clone().into(),
                fs.domain_root.clone().into(),
                fs.enable_stats,
            ).wrap(args.clone()),
        };

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(storage_data))
            .service(hello)
            .service(not_decky::decky_index)
            .service(not_decky::decky_plugins)
            .service(not_decky::decky_artifact)
            .service(not_decky::decky_image)
            .service(not_decky::decky_statistics)
    })
    .bind(("0.0.0.0", 22252))?
    .run()
    .await
}
