use std::{thread, time::Duration, fs, path::Path, net::SocketAddr};

use anyhow::Ok;
use askama::Template;
use axum::{Router, service, http::StatusCode};
use templates::Post;
use tower_http::services::ServeDir;

mod templates;

const CONTENT_DIR: &str = "content";
const OUTPUT_DIR: &str = "public";

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    rebuild_site(CONTENT_DIR, OUTPUT_DIR).expect("Rebuilding site");
    
    tokio::task::spawn_blocking(move || {
        println!("listening for changes: {}", CONTENT_DIR);
        let mut hotwatch = hotwatch::Hotwatch::new().expect("hotwatch failed to init");
        hotwatch
            .watch(CONTENT_DIR, |_| {
                println!("Rebuilding site");
                rebuild_site(CONTENT_DIR, OUTPUT_DIR).expect("Rebuilding site");
            })
            .expect("failed to watch content folder");
        loop {
            thread::sleep(Duration::from_secs(1));
        }
    });

    let app = Router::new().nest("/", 
        service::get(ServeDir::new(OUTPUT_DIR)).handle_error(|error: std::io::Error| {
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

fn rebuild_site(content_dir: &str, output_dir: &str) -> Result<(), anyhow::Error> {
    let _ = fs::remove_dir_all(output_dir);

    // Get all markdown filepath from content dir
    let markdown_files: Vec<String> = walkdir::WalkDir::new(content_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().display().to_string().ends_with(".md"))
        .map(|e| e.path().display().to_string())
        .collect();
    
    // Create a vector of the lenght of the md files
    let mut html_files: Vec<String> = Vec::with_capacity(markdown_files.len());
    for file in &markdown_files {
        let mut html = templates::HEADER.to_owned(); // fill html string with header
        let markdown = fs::read_to_string(&file)?; // get markdown as string
        let parser = pulldown_cmark::Parser::new_ext(&markdown, pulldown_cmark::Options::all());

        let mut body = String::new();
        pulldown_cmark::html::push_html(&mut body, parser); // put md to html in body
        
        let post_template = templates::PostTemplate { content: body.as_str()};
        html.push_str(post_template.render().unwrap().as_str());
        
        // create the path for html file from the original md file
        let html_file = file
            .replace(content_dir, output_dir)
            .replace(".md", ".html");
        
        // Create folder
        let folder = Path::new(&html_file).parent().unwrap();
        let _ = fs::create_dir_all(folder);

        // Write file content (from template)
        fs::write(&html_file, html)?;
        html_files.push(html_file);
    }
    write_index(html_files, output_dir)?;
    Ok(())
}

fn write_index(files: Vec<String>, output_dir: &str) -> Result<(), anyhow::Error> {
    // Add header
    // TODO: remove
    let mut html =  templates::HEADER.to_owned();

    let mut posts: Vec<Post> = Vec::with_capacity(files.len());
    for file in &files {
        let file_name = file.trim_start_matches(output_dir);
        let title = file_name.trim_start_matches("/").trim_end_matches(".html");
        posts.push(Post { name: title, url: file_name })
    } 

    let index_template = templates::IndexTemplate { posts };
    html.push_str(index_template.render().unwrap().as_str());
    
    let index_path = Path::new(&output_dir).join("index.html");
    fs::write(index_path, html)?;
    Ok(())
}

