use config::Environment;
use email_address::EmailAddress;
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use sqlx::postgres::{PgConnectOptions, PgSslMode};

const DB_DEFAULT_TIMEOUT: u64 = 5000;
const EMAIL_CLIENT_DEFAULT_TIMEOUT: u64 = 10000;

#[derive(Deserialize, Clone)]
pub struct Configuration {
    pub profile: String,
    pub app: AppConfig,
    pub db: DbConfig,
    pub email_client: EmailClientConfig,
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
    pub timeout: u64,
    pub ssl: bool,
}

#[derive(Deserialize, Clone)]
pub struct EmailClientConfig {
    pub base_url: String,
    pub sender_email: EmailAddress,
    pub auth_token: SecretString,
    pub timeout: u64,
}

#[derive(PartialEq, Eq)]
pub enum WithDb {
    Yes,
    No,
}

impl DbConfig {
    pub fn connection_options(&self, with_db: WithDb) -> PgConnectOptions {
        let mut options = PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .ssl_mode(match self.ssl {
                true => PgSslMode::Require,
                false => PgSslMode::Prefer,
            });

        if with_db == WithDb::Yes {
            options = options.database(&self.name);
        }
        options
    }
}

pub fn get_test_configuration() -> Result<Configuration, config::ConfigError> {
    let mut configuration = get_configuration_with_profile("test".into(), "../configuration")?;
    configuration.db.name = format!("{}-{}", configuration.db.name, uuid::Uuid::new_v4());
    Ok(configuration)
}

pub fn get_configuration() -> Result<Configuration, config::ConfigError> {
    // Detect profile to use
    let app_profile = std::env::var("Z2P_PROFILE").unwrap_or_else(|_| "local".into());
    get_configuration_with_profile(app_profile, "configuration")
}

fn get_configuration_with_profile(
    app_profile: String,
    config_dir: &str,
) -> Result<Configuration, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine current directory");
    let config_dir = base_path.join(config_dir);

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
    builder = builder
        .set_default("db.timeout", DB_DEFAULT_TIMEOUT)?
        .set_default("email_client.timeout", EMAIL_CLIENT_DEFAULT_TIMEOUT)?
        .set_override("profile", app_profile)?;

    let configuration = builder.build()?;
    configuration.try_deserialize::<Configuration>()
}
