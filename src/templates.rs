use askama::Template;

#[derive(Template)]
#[template(path = "post_template.html", escape = "none")]
pub struct PostTemplate<'a> {
    pub content: &'a str,
    pub username: &'a str,
}

#[derive(Template)]
#[template(path = "index_template.html", escape = "none")]
pub struct IndexTemplate<'a> {
    pub posts: Vec<Post<'a>>,
    pub username: &'a str,
    pub intro: &'a str,
}

pub struct Post<'a> {
    pub name: &'a str,
    pub url: &'a str,
    pub title: &'a str,
    pub description: &'a str,
    pub date: &'a str,
}

