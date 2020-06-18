use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::collections::HashMap;

/// Representation of an individual openId provider
#[derive(Debug, Deserialize)]
pub struct ProviderConfig {
    // /// Name used in the OpenID routes
    // pub name: String,
    /// Url used by the OpenID discover step to get all necessary fields
    pub issuer_url: String,
    /// Client ID, given by the provider
    pub client_id: String,
    /// Client Secret, given by the provider
    pub client_secret: String,
}

/// The representation of the OpenID config json
#[derive(Debug, Deserialize)]
pub struct OpenIdConfig {
    /// Url of the frontend
    pub host_url: String,
    /// List of each provider to configure
    pub providers: HashMap<String, ProviderConfig>,
}

#[derive(Debug, Deserialize)]
pub struct GdlkConfig {
    /// The hostname for the HTTP server to bind to.
    pub server_host: String,
    /// The secret key that's used to for authorization-related crypto stuff.
    pub secret_key: String,
    /// All configuration related to OpenID
    pub open_id: OpenIdConfig,
}

impl GdlkConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let mut s = Config::new();

        // Load "config/default.json", which has non-sensitive values
        s.merge(File::with_name("config/default.json"))?;
        // Optionally load a "config/dev.json" file
        s.merge(File::with_name("config/dev").required(false))?;
        // Load all env variables with the prefix "GDLK_"
        s.merge(Environment::new().prefix("gdlk").separator("__"))?;

        s.try_into()
    }
}
