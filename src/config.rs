use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub provider: ProviderConfig,
    #[serde(default)]
    pub generation: GenerationConfig,
    #[serde(default)]
    pub context: ContextConfig,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ProviderConfig {
    pub name: Option<String>,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub default_model: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct GenerationConfig {
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContextConfig {
    pub max_tokens: Option<usize>,
    #[serde(default = "default_reserve")]
    pub reserve_for_response: usize,
}

fn default_reserve() -> usize {
    1024
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            max_tokens: None,
            reserve_for_response: default_reserve(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let mut config = Self::load_from_file().unwrap_or_default();
        config.apply_env_overrides();
        config
    }

    fn config_path() -> PathBuf {
        if let Ok(path) = std::env::var("LLM_TUI_CONFIG") {
            return PathBuf::from(path);
        }

        if let Some(config_dir) = dirs::config_dir() {
            return config_dir.join("llm-tui").join("config.toml");
        }

        PathBuf::from("config.toml")
    }

    fn load_from_file() -> Option<Self> {
        let path = Self::config_path();
        let content = std::fs::read_to_string(&path).ok()?;
        toml::from_str(&content).ok()
    }

    pub fn apply_env_overrides(&mut self) {
        if let Ok(url) = std::env::var("LLM_TUI_BASE_URL") {
            self.provider.base_url = Some(url);
        }
        if let Ok(key) = std::env::var("LLM_TUI_API_KEY") {
            self.provider.api_key = Some(key);
        }
        if let Ok(model) = std::env::var("LLM_TUI_MODEL") {
            self.provider.default_model = Some(model);
        }
    }

    pub fn is_provider_configured(&self) -> bool {
        self.provider.base_url.is_some()
    }
}
