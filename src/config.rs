use base64::prelude::*;
use ini::Ini;
use once_cell::sync::OnceCell;

static CONFIG: OnceCell<Config> = OnceCell::new();

struct Config {
    epo_credentials: EpoOpsCredentials,
    cache_downloads: String,
}

#[derive(Clone)]
pub struct EpoOpsCredentials {
    consumer_key: String,
    secret_key: String,
}

impl EpoOpsCredentials {
    pub fn format_credentials(&self) -> String {
        BASE64_STANDARD.encode(format!("{}:{}", self.consumer_key, self.secret_key))
    }
}

pub fn load_config(file: &str) {
    let conf = Ini::load_from_file(file).expect("Couldn't open config file");
    let epo_credentials_conf = conf.section(Some("EPO OPS")).unwrap();
    let epo_credentials = EpoOpsCredentials {
        consumer_key: epo_credentials_conf
            .get("consumer_key")
            .expect("Error finding consumer_key")
            .to_string(),
        secret_key: epo_credentials_conf
            .get("secret_key")
            .expect("Error finding secret_key")
            .to_string(),
    };
    let cache_folders = conf.section(Some("Cache Folders")).unwrap();
    let cache_downloads = cache_folders
        .get("cache_downloads")
        .expect("Error finding epo_downloads")
        .to_string();
    let _ = CONFIG.set(Config {
        epo_credentials,
        cache_downloads,
    });
}

pub fn get_epo_credentials() -> EpoOpsCredentials {
    let config = CONFIG
        .get()
        .expect("Config not initialized - you need to call \"load_config\" first");
    config.epo_credentials.clone()
}

pub fn get_cache_folder() -> String {
    let config = CONFIG
        .get()
        .expect("Config not initialized - you need to call \"load_config\" first");
    config.cache_downloads.clone()
}
