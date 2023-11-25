use std::path::PathBuf;

use config::Environment;
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Configuration {
    pub profile: String,
    pub app: AppConfig,
    pub db: DbConfig,
}

#[derive(Deserialize, Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Deserialize, Clone)]
pub struct DbConfig {
    pub username: String,
    pub password: SecretString,
    pub host: String,
    pub port: u16,
    pub name: String,
}

pub enum WithDb {
    Yes,
    No,
}

impl DbConfig {
    pub fn connection_string(&self, with_db: WithDb) -> SecretString {
        match with_db {
            WithDb::Yes => SecretString::new(format!(
                "postgres://{}:{}@{}:{}/{}",
                self.username,
                self.password.expose_secret(),
                self.host,
                self.port,
                self.name
            )),
            WithDb::No => SecretString::new(format!(
                "postgres://{}:{}@{}:{}",
                self.username,
                self.password.expose_secret(),
                self.host,
                self.port
            )),
        }
    }
}

pub fn get_test_configuration() -> Result<Configuration, config::ConfigError> {
    let mut configuration = get_configuration_with_profile("test".into())?;
    configuration.db.name = format!("{}-{}", configuration.db.name, uuid::Uuid::new_v4());
    Ok(configuration)
}

pub fn get_configuration() -> Result<Configuration, config::ConfigError> {
    // Detect profile to use
    let app_profile = std::env::var("Z2P_PROFILE").unwrap_or_else(|_| "local".into());

    let mut configuration = get_configuration_with_profile(app_profile)?;
    configuration.db.name = format!("{}-{}", configuration.db.name, uuid::Uuid::new_v4());
    Ok(configuration)
}

fn get_configuration_with_profile(
    app_profile: String,
) -> Result<Configuration, config::ConfigError> {
    let config_dir: PathBuf = concat!(env!("CARGO_MANIFEST_DIR"), "/../configuration").into();

    // Start off by merging in the "base" configuration file
    let mut builder = config::Config::builder();
    builder = builder.add_source(config::File::from(config_dir.join("base.yaml")));

    let profiles: Vec<_> = app_profile
        .split(',')
        .map(|s| format!("{}.yaml", s))
        .collect();

    for profile in profiles {
        builder = builder.add_source(config::File::from(config_dir.join(profile)).required(false));
    }

    builder = builder.add_source(
        Environment::with_prefix("Z2P")
            .try_parsing(true)
            .separator("_"),
    );
    builder = builder.set_override("profile", app_profile)?;

    let configuration = builder.build()?;
    configuration.try_deserialize::<Configuration>()
}
