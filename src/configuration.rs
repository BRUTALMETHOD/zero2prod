use config::{self, Config, ConfigBuilder, ConfigError, File};

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

pub fn get_configuration() -> Result<Settings, ConfigError> {
    let mut builder = Config::builder();
    builder.add_source(File::with_name("configuration"));
    match builder.build() {
        Ok(config) => config.try_deserialize(),
        Err(e) => e,
    }
}
