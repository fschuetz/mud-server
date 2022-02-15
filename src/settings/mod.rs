use config::{ConfigError, Config, File};

#[derive(Debug, Deserialize)]
pub struct General {
    pub debug: bool,
}

#[derive(Debug, Deserialize)]
pub struct SSHServer {
    pub start_ssh: bool,
    pub port: u32,
    pub host: String,
}

#[derive(Debug, Deserialize)]
pub struct Security {
    pub allowed_keys: Vec<Vec<String>>
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub general: General,
    pub ssh_server: SSHServer,
    pub security: Security,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();

        // TODO set default settings to ensure success if no settings provided at all.

        // Parse the configuration
        // Add in './DefaultSettings.tom' to set the defaults
        s.merge(File::with_name("DefaultSettings"))?;
        // Add in `./Settings.toml`
        s.merge(File::with_name("Settings"))?;
        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        s.merge(config::Environment::with_prefix("UBBS"))?;

        // Deserialize (and thus freeze) the entire configuration
        s.try_into()
    }
}
