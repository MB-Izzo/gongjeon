use crate::{templates::{self, Post}, config::Config};
use askama::Template;
use std::{fs, path::Path};

pub fn rebuild_site(config: &Config) -> Result<(), anyhow::Error> {
    let _ = fs::remove_dir_all(config.output_dir.as_str());

    let markdown_files: Vec<String> = get_markdown_files_from(config.content_dir.as_str());

    let html_files: Vec<String> = convert_md_to_html(markdown_files, config.content_dir.as_str(), config.output_dir.as_str());

    write_index(html_files, config.output_dir.as_str())?;
    Ok(())
}

fn convert_md_to_html(md_files: Vec<String>, content_dir: &str, output_dir: &str) -> Vec<String> {
    let mut html_files: Vec<String> = Vec::with_capacity(md_files.len());
    for file in &md_files {
        let mut html_content = String::new();
        let markdown = fs::read_to_string(&file).unwrap();
        let parser = pulldown_cmark::Parser::new_ext(&markdown, pulldown_cmark::Options::all());

        let mut body = String::new();
        pulldown_cmark::html::push_html(&mut body, parser);
        let post_template = templates::PostTemplate {
            content: body.as_str(),
        };
        html_content.push_str(post_template.render().unwrap().as_str());

        let created_html_file = write_html_file(&file, &html_content, &content_dir, &output_dir);
        html_files.push(created_html_file);
    }
    html_files
}

fn write_html_file(file: &str, content: &str, content_dir: &str, output_dir: &str) -> String {
    let html_file = file.replace(content_dir, output_dir).replace(".md", ".html");
    let folder = Path::new(&html_file)
        .parent()
        .expect("Could not create path from origin folder");
    let _ = fs::create_dir_all(folder);
    println!("Building: {}...", html_file);
    // Write file content (from template)
    fs::write(&html_file, content).expect("could not write");
    html_file
}

fn get_markdown_files_from(content_dir: &str) -> Vec<String> {
    let markdown_files: Vec<String> = walkdir::WalkDir::new(content_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().display().to_string().ends_with(".md"))
        .map(|e| e.path().display().to_string())
        .collect();
    markdown_files
}

fn write_index(files: Vec<String>, output_dir: &str) -> Result<(), anyhow::Error> {
    let mut html = String::new();

    let mut posts: Vec<Post> = Vec::with_capacity(files.len());
    for file in &files {
        let file_name = file.trim_start_matches(output_dir);
        let title = file_name.trim_start_matches("/").trim_end_matches(".html");
        posts.push(Post {
            name: title,
            url: file_name,
        })
    }

    let index_template = templates::IndexTemplate { posts };
    html.push_str(index_template.render().unwrap().as_str());

    let index_path = Path::new(&output_dir).join("index.html");
    fs::write(index_path, html)?;
    Ok(())
}
