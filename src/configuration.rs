use config::Environment;
use secrecy::{ExposeSecret, Secret, SecretString};
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub app: ApplicationSettings,
    pub env_filter: String,
    pub test_log: Option<bool>,
}

#[derive(Deserialize, Clone)]
pub struct ApplicationSettings {
    pub name: String,
    pub host: String,
    pub port: u16,
}

#[derive(Deserialize, Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: SecretString,
    pub port: u16,
    pub host: String,
    pub name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.name
        ))
    }

    pub fn connection_string_no_db(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port
        ))
    }
}

pub fn get_configuration(profiles: &Vec<String>) -> Result<Settings, config::ConfigError> {
    let mut builder = config::Config::builder()
        .add_source(config::File::new("config.toml", config::FileFormat::Toml).required(false));

    builder = profiles.iter().fold(builder, |b, profile| {
        b.add_source(
            config::File::new(
                &format!("config-{}.toml", profile),
                config::FileFormat::Toml,
            )
            .required(false),
        )
    });

    builder = builder.add_source(Environment::with_prefix("Z2P").separator("_"));

    let settings = builder.build()?;

    settings.try_deserialize::<Settings>()
}
