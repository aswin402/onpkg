use crate::config::Config;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RegistryTemplate {
    pub name: String,
    pub category: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub source: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RegistrySkill {
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub source: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RegistryPackage {
    pub name: String,
    pub runtime: String,
    pub description: String,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchResults {
    pub templates: Vec<RegistryTemplate>,
    pub skills: Vec<RegistrySkill>,
    pub packages: Vec<RegistryPackage>,
}

pub struct Registry {
    config: Config,
    client: reqwest::Client,
}

impl Registry {
    pub fn new(config: Config) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.network.timeout_secs))
            .build()
            .unwrap_or_default();
        Self { config, client }
    }

    pub async fn search(&self, query: &str, r#type: Option<&str>) -> Result<SearchResults> {
        let base = self.config.registry_url();
        let mut url = format!("{}/api/search?q={}", base, urlencoding(query));
        if let Some(t) = r#type {
            url.push_str(&format!("&type={}", t));
        }

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow!("Registry search failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(anyhow!("Registry returned HTTP {}", resp.status()));
        }

        let results: SearchResults = resp
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse search results: {}", e))?;
        Ok(results)
    }

    pub async fn get_template(&self, name: &str) -> Result<RegistryTemplate> {
        let url = format!("{}/api/templates/{}", self.config.registry_url(), name);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to fetch template '{}': {}", name, e))?;
        if !resp.status().is_success() {
            return Err(anyhow!("Template '{}' not found (HTTP {})", name, resp.status()));
        }
        let tmpl: RegistryTemplate = resp.json().await?;
        Ok(tmpl)
    }

    pub async fn get_skill(&self, name: &str) -> Result<RegistrySkill> {
        let url = format!("{}/api/skills/{}", self.config.registry_url(), name);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to fetch skill '{}': {}", name, e))?;
        if !resp.status().is_success() {
            return Err(anyhow!("Skill '{}' not found (HTTP {})", name, resp.status()));
        }
        let skill: RegistrySkill = resp.json().await?;
        Ok(skill)
    }

    pub async fn download_template_source(&self, _source: &str, _dest: &std::path::Path) -> Result<()> {
        Err(anyhow!("Template download not yet implemented"))
    }

    pub async fn publish_template(&self, name: &str, _dir: &std::path::Path) -> Result<()> {
        let url = format!("{}/api/templates", self.config.registry_url());
        Err(anyhow!(
            "Publishing to {} is not yet implemented. The template '{}' is saved locally.",
            url, name
        ))
    }

    pub async fn publish_skill(&self, _name: &str, _path: &std::path::Path) -> Result<()> {
        let url = format!("{}/api/skills", self.config.registry_url());
        Err(anyhow!(
            "Publishing to {} is not yet implemented. The skill is saved locally.",
            url
        ))
    }

    pub async fn check_health(&self) -> Result<HashMap<String, String>> {
        let url = format!("{}/api/health", self.config.registry_url());
        match self.client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                let status: HashMap<String, String> = resp.json().await?;
                Ok(status)
            }
            Ok(resp) => {
                let mut map = HashMap::new();
                map.insert("status".to_string(), format!("HTTP {}", resp.status()));
                Ok(map)
            }
            Err(e) => {
                let mut map = HashMap::new();
                map.insert("status".to_string(), format!("offline ({})", e));
                Ok(map)
            }
        }
    }
}

fn urlencoding(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}
