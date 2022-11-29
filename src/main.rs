use std::{thread, time::Duration, net::SocketAddr, fs::{self, File}, env, io::Write};
use anyhow::Ok;
use axum::{Router, service, http::StatusCode};
use inquire::Text;
use tower_http::services::ServeDir;

use gongjeon::config::Config;

fn main() -> Result<(), anyhow::Error> {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => println!("Need to specify arg run or init"),
        2 => {
            match &args[1][..] {
                "init" => init_config(), // handle setup config
                "run" => run_dev_server().unwrap(), // handle run dev server 
                // if can find config, otherwise tell user to use init
                _ => println!("Unrecognized command"),
            }
        },
        _ => println!("too many args"),
    }
    Ok(())
}

pub fn init_config() {
    let input_content_dir = Text::new("What is the name of content dir?")
        .prompt();

    match &input_content_dir {
        core::result::Result::Ok(status) => println!("Content dir '{}' was choosen.", status),
        Err(_) => println!("error"),
    }

    let input_public_dir = Text::new("What is the name of content dir?")
        .prompt();

    match &input_public_dir {
        core::result::Result::Ok(_) => println!("Content dir was choosen."),
        Err(_) => println!("error"),
    }
    let config = Config { content_dir: input_content_dir.unwrap(), output_dir: input_public_dir.unwrap() };
    let mut f = File::create("config.json").unwrap();
    let a = serde_json::to_string(&config).unwrap();
    f.write_all(a.as_bytes()).unwrap();

    println!("Successfull init! Run gong dev to run dev server.")
}

#[tokio::main]
async fn run_dev_server() -> Result<(), anyhow::Error> {
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
