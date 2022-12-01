use anyhow::Ok;
use axum::{http::StatusCode, service, Router};
use clap::{clap_derive, Parser};
use inquire::{Text, InquireError};
use std::{
    fs::{self, File},
    io::Write,
    net::SocketAddr,
    thread,
    time::Duration,
};
use tower_http::services::ServeDir;

use gongjeon::config::Config;

#[derive(clap_derive::Parser)]
#[clap(version, about)]
struct Opt {
    #[clap(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(PartialEq, clap_derive::Subcommand)]
enum Cmd {
    Init,
    Dev,
}

fn main() -> Result<(), anyhow::Error> {
    let opt: Opt = Parser::parse();
    match opt.cmd {
        Some(Cmd::Init) => init_config(),
        Some(Cmd::Dev) => run_dev_server().unwrap(), // if cant find conf tell user to init
        None => println!("You need to use Init or Dev arg."),
    }

    Ok(())
}

pub fn init_config() {
    let content_dir = ask("Content dir name", "content")
        .expect("Error content dir name");

    let public_dir = ask("Content dir name", "public")
        .expect("Error public dir name");

    let username = ask("What is your username", "John Doe")
        .expect("Error entering username");

    let intro = ask("What is your website about", "This is an intro...")
        .expect("Error entering description");

    let config = Config {
        content_dir,
        output_dir: public_dir,
        username: username.to_string(),
        intro,
    };

    let mut conf_file = File::create("config.json").unwrap();
    let conf_string = serde_json::to_string(&config).unwrap();
    conf_file.write_all(conf_string.as_bytes()).unwrap();

    println!("Successfull init! Run gong dev to run dev server.")
}

#[tokio::main]
async fn run_dev_server() -> Result<(), anyhow::Error> {

    fn read_config_file() -> Result<Config, anyhow::Error> {
        let conf_text = fs::read_to_string("config.json").expect("Could not read file");
        let config: Config = serde_json::from_str(&conf_text)?;
        Ok(config)
    }

    let config = read_config_file().expect("Could not read config file");
    
    // because use for hotwatch (closure)
    // may be a better way to avoid clone but I don't know how
    let content_dir = config.content_dir.clone();
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

    let app = Router::new().nest(
        "/",
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

fn ask<'a>(question: &str, default_value: &'a str) -> Result<String, InquireError>{
    let input_public_dir = Text::new(question)
        .with_default(default_value)
        .prompt();

    match input_public_dir {
        core::result::Result::Ok(status) => core::result::Result::Ok(status),
        Err(e) => Err(e),
    }
}
