use std::{thread, time::Duration, net::SocketAddr, fs};
use anyhow::Ok;
use axum::{Router, service, http::StatusCode};
use tower_http::services::ServeDir;

use gongjeon::config::Config;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let config = read_config_file().expect("Could not read config file"); 

    let content_dir  = config.content_dir.clone();
    let output_dir = config.output_dir.clone();
    
    gongjeon::rebuild_site(&config).expect("Rebuilding site");
    
    tokio::task::spawn_blocking(move || {
        println!("listening for changes: {}", config.content_dir);
        let mut hotwatch = hotwatch::Hotwatch::new().expect("hotwatch failed to init");
        hotwatch
            .watch(content_dir, move |_| {
                println!("Rebuilding site");
                gongjeon::rebuild_site(&config).expect("Error rebuild");
            })
            .expect("failed to watch content folder");
        loop {
            thread::sleep(Duration::from_secs(1));
        }
    });

    let app = Router::new().nest("/", 
        service::get(ServeDir::new(output_dir)).handle_error(|error: std::io::Error| {
            Ok::<_>((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Unhandled internal error {}", error),
            ))
        }),
    );

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Serving on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

fn read_config_file() -> Result<Config, anyhow::Error> {
    let conf_text = fs::read_to_string("config.json").expect("Could not read file");
    let config: Config = serde_json::from_str(&conf_text)?;
    Ok(config)
}
