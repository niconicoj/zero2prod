use config::Environment;
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use sqlx::postgres::PgConnectOptions;

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
    pub fn without_db(&self) -> PgConnectOptions {
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(&self.password.expose_secret())
            .port(self.port)
    }

    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db().database(&self.name)
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
