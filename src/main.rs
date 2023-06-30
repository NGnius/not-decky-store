mod cli;
mod consts;
mod not_decky;
mod storage;

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use simplelog::{LevelFilter, WriteLogger};

#[get("/version_info")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body(format!("{} v{}", consts::PACKAGE_NAME, consts::PACKAGE_VERSION))
}

fn build_storage_box(storage: &cli::StorageArgs) -> Box<dyn storage::IStorage> {
    log::debug!("storage args {:?}", storage);
    match storage {
        cli::StorageArgs::Default => Box::new(storage::FileStorage::new(
            "./store".into(),
            "http://192.168.0.128:22252".into(),
            true,
        )),
        cli::StorageArgs::Filesystem(fs) => Box::new(storage::FileStorage::new(
            fs.root.clone().into(),
            fs.domain_root.clone().into(),
            fs.enable_stats,
        )),
        cli::StorageArgs::Proxy(px) => Box::new(storage::ProxiedStorage::new(
            px.proxy_store.clone(),
        )),
        cli::StorageArgs::Empty => Box::new(storage::EmptyStorage),
        cli::StorageArgs::Merge(ls) => Box::new(storage::MergedStorage::new(
            ls.generate_args()
                .expect("Bad descriptor")
                .drain(..)
                .map(|args| build_storage_box(&args))
                .collect()
        ))
    }
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

        let storage_data: Box<dyn storage::IStorage> = build_storage_box(&args.storage);

        let storage_data = if let Some(cache_duration) = args.cache_duration {
            Box::new(storage::CachedStorage::new(cache_duration, storage_data))
        } else {
            storage_data
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
    .bind(("0.0.0.0", args.server_port.unwrap_or(22252)))?
    .run()
    .await
}
