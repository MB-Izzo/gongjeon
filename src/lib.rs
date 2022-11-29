mod templates;
mod site_building;

pub mod config {
    use serde_derive::{Deserialize, Serialize};
    #[derive(Deserialize, Serialize)]
    pub struct Config {
        pub content_dir: String,
        pub output_dir: String,
    }
}

pub fn rebuild_site(config: &config::Config) -> Result<(), anyhow::Error> {
    site_building::rebuild_site(&config).expect("Error while rebuild");
    Ok(())
}
