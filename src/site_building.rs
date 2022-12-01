use std::{fs, path::Path};
use askama::Template;
use serde::Deserialize;
use yaml_front_matter::{Document, YamlFrontMatter};

use crate::{templates::{self, Post}, config::Config};

#[derive(Deserialize)]
struct Metadata {
    title: String,
    description: String,
    date: String,
}

struct MyPost {
    title: String,
    file_name: String,
    description: String,
    date: String,
}

pub fn rebuild_site(config: &Config) -> Result<(), anyhow::Error> {
    let _ = fs::remove_dir_all(config.output_dir.as_str());
    let markdown_files: Vec<String> = get_markdown_files_from(config.content_dir.as_str());
    let html_files: Vec<MyPost> = convert_md_to_html(markdown_files, config.content_dir.as_str(), config.output_dir.as_str(), config.username.as_str());
    write_index(html_files, config)?;
    Ok(())
}

fn convert_md_to_html(md_files: Vec<String>, content_dir: &str, output_dir: &str, username: &str) -> Vec<MyPost> {
    // let mut html_files: Vec<String> = Vec::with_capacity(md_files.len());
    let mut html_posts: Vec<MyPost> = Vec::with_capacity(md_files.len());

    for file in &md_files {
        let mut html_content = String::new();
        let markdown = fs::read_to_string(&file).unwrap();
        let document: Document<Metadata> = YamlFrontMatter::parse::<Metadata>(&markdown).unwrap();
        let Metadata { title, description, date } = document.metadata;
        let parser = pulldown_cmark::Parser::new_ext(&document.content, pulldown_cmark::Options::all());

        let mut body = String::new();
        pulldown_cmark::html::push_html(&mut body, parser);
        let post_template = templates::PostTemplate {
            content: body.as_str(),
            username,
        };
        html_content.push_str(post_template.render().unwrap().as_str());

        let created_html_file = write_html_file(&file, &html_content, &content_dir, &output_dir);
        html_posts.push(MyPost { title, file_name: created_html_file, description, date });
        //html_posts.push(created_html_file);
    }
    html_posts
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

fn write_index(my_posts: Vec<MyPost>, config: &Config) -> Result<(), anyhow::Error> {
    let mut html = String::new();

    let mut posts: Vec<Post> = Vec::with_capacity(my_posts.len());
    for my_post in &my_posts {
        let file_name = my_post.file_name.trim_start_matches(config.output_dir.as_str());
        let title = file_name.trim_start_matches("/").trim_end_matches(".html");
        posts.push(Post {
            name: title,
            url: file_name,
            title: my_post.title.as_str(),
            description: my_post.description.as_str(),
            date: my_post.date.as_str(),
        })
    }

    let index_template = templates::IndexTemplate { posts, intro: config.intro.as_str(), username: config.username.as_str()};
    html.push_str(index_template.render().unwrap().as_str());

    let index_path = Path::new(config.output_dir.as_str()).join("index.html");
    println!("{}", index_path.as_path().display().to_string());
    fs::write(index_path, html)?;
    Ok(())
}
