/* Immutable configuration object. */
pub struct Config {
    pub alpaca_base_url: String,
    pub alpaca_api_key: String,
    pub alpaca_secret_key: String,
}

impl Config {
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }
}

/* Creates the final config object.  */
#[derive(Default)]
pub struct ConfigBuilder {
    alpaca_base_url: String,
    alpaca_api_key: String,
    alpaca_secret_key: String,
}

impl ConfigBuilder {
    /// Determines whether the alpaca client is doing real or paper trading.
    pub fn alpaca_base_url(mut self, alpaca_base_url: String) -> Self {
        self.alpaca_base_url = alpaca_base_url;
        self
    }
    pub fn alpaca_api_key(mut self, alpaca_api_key: String) -> Self {
        self.alpaca_api_key = alpaca_api_key;
        self
    }

    pub fn alpaca_secret_key(mut self, alpaca_secret_key: String) -> Self {
        self.alpaca_secret_key = alpaca_secret_key;
        self
    }

    pub fn build(self) -> Config {
        Config {
            alpaca_base_url: self.alpaca_base_url,
            alpaca_api_key: self.alpaca_api_key,
            alpaca_secret_key: self.alpaca_secret_key,
        }
    }
}
