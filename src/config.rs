use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub registry: RegistryConfig,
    pub cache: CacheConfig,
    pub network: NetworkConfig,
    #[serde(skip)]
    pub home_override: Option<PathBuf>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RegistryConfig {
    pub url: String,
    #[serde(default)]
    pub token: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CacheConfig {
    pub path: String,
    pub max_size_gb: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NetworkConfig {
    pub timeout_secs: u64,
    pub retries: u64,
}

impl Default for Config {
    fn default() -> Self {
        let home = home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        Self {
            registry: RegistryConfig {
                url: "https://registry.onpkg.dev".to_string(),
                token: None,
            },
            cache: CacheConfig {
                path: home.join(".onpkg/cache").to_string_lossy().to_string(),
                max_size_gb: 10.0,
            },
            network: NetworkConfig {
                timeout_secs: 30,
                retries: 3,
            },
            home_override: None,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if path.exists() {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read config from {}", path.display()))?;
            toml::from_str(&content).map_err(|e| anyhow!("Failed to parse config.toml: {}", e))
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    fn config_path() -> Result<PathBuf> {
        let home = home_dir().ok_or_else(|| anyhow!("Could not determine home directory"))?;
        Ok(home.join(".onpkg/config.toml"))
    }

    pub fn cache_path(&self) -> PathBuf {
        if let Some(ref h) = self.home_override {
            return h.join(".onpkg/cache");
        }
        if let Ok(override_dir) = env::var("ONPKG_CACHE_DIR") {
            return PathBuf::from(override_dir);
        }
        PathBuf::from(&self.cache.path)
    }

    pub fn db_path(&self) -> PathBuf {
        self.cache_path().join("onpkg.db")
    }

    pub fn templates_dir(&self) -> PathBuf {
        let home = self.home_override.clone().unwrap_or_else(|| {
            home_dir().unwrap_or_else(|| PathBuf::from("/tmp"))
        });
        home.join(".onpkg/templates")
    }

    pub fn skills_dir(&self) -> PathBuf {
        let home = self.home_override.clone().unwrap_or_else(|| {
            home_dir().unwrap_or_else(|| PathBuf::from("/tmp"))
        });
        home.join(".onpkg/skills")
    }

    pub fn registry_url(&self) -> String {
        env::var("ONPKG_REGISTRY_URL").unwrap_or_else(|_| self.registry.url.clone())
    }
}

/// Cross-platform home directory
fn home_dir() -> Option<PathBuf> {
    env::var("HOME")
        .ok()
        .map(PathBuf::from)
        .or_else(|| env::var("USERPROFILE").ok().map(PathBuf::from))
}
