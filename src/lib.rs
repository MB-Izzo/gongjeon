mod templates;
mod site_building;

pub use site_building::rebuild_site;

pub mod config {
    use serde_derive::{Deserialize, Serialize};
    #[derive(Deserialize, Serialize)]
    pub struct Config {
        pub content_dir: String,
        pub output_dir: String,
        pub username: String,
        pub intro: String,
    }
}


