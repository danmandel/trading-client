/// Immutable configuration object.
pub struct Config {
    pub alpaca_api_key: String,
    pub alpaca_secret_key: String,
    pub enable_real_trading: bool,
}

impl Config {
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }
}

/// Creates the final config object.
#[derive(Default)]
pub struct ConfigBuilder {
    alpaca_api_key: Option<String>,
    alpaca_secret_key: Option<String>,
    enable_real_trading: bool,
}

impl ConfigBuilder {
    pub fn alpaca_api_key(mut self, alpaca_api_key: String) -> Self {
        self.alpaca_api_key = Some(alpaca_api_key);
        self
    }

    pub fn alpaca_secret_key(mut self, alpaca_secret_key: String) -> Self {
        self.alpaca_secret_key = Some(alpaca_secret_key);
        self
    }

    pub fn enable_real_trading(mut self, enable_real_trading: bool) -> Self {
        self.enable_real_trading = enable_real_trading;
        self
    }

    pub fn build(self) -> Result<Config, &'static str> {
        Ok(Config {
            alpaca_api_key: self.alpaca_api_key.ok_or("API key must be set")?,
            alpaca_secret_key: self.alpaca_secret_key.ok_or("Secret key must be set")?,
            enable_real_trading: self.enable_real_trading,
        })
    }
}
