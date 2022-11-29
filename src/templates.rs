use askama::Template;

#[derive(Template)]
#[template(path = "post_template.html", escape = "none")]
pub struct PostTemplate<'a> {
    pub content: &'a str,
}

#[derive(Template)]
#[template(path = "index_template.html", escape = "none")]
pub struct IndexTemplate<'a> {
    pub posts: Vec<Post<'a>>,
}

pub struct Post<'a> {
    pub name: &'a str,
    pub url: &'a str,
}

