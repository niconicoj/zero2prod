use serde::Deserialize;

#[derive(Deserialize)]
pub struct Configuration {
    pub database: DatabaseConfig,
    pub port: u16,
}

#[derive(Deserialize)]
pub struct DatabaseConfig {
    pub username: String,
    pub password: String,
    pub host: String,
    pub port: u16,
    pub database_name: String,
}

impl DatabaseConfig {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }
}

pub fn get_configuration() -> Result<Configuration, config::ConfigError> {
    let configuration = config::Config::builder()
        .add_source(config::File::new(
            "configuration.yaml",
            config::FileFormat::Yaml,
        ))
        .build()?;

    configuration.try_deserialize::<Configuration>()
}
