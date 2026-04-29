use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct PypiClient {
    client: Client,
}

#[derive(Debug, Deserialize)]
struct PypiResponse {
    info: PackageInfo,
}

#[derive(Debug, Deserialize)]
struct PackageInfo {
    version: String,
}

impl PypiClient {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent("ux/0.1.0")
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_else(|_| Client::new()),
        }
    }

    pub async fn get_package(&self, name: &str) -> Result<(String, String)> {
        let url = format!("https://pypi.org/pypi/{}/json", name);
        
        let response = self.client.get(&url).send().await.context("Network error")?;
        
        if response.status() == 404 {
            anyhow::bail!("Package not found: {}", name);
        }
        
        if !response.status().is_success() {
            anyhow::bail!("HTTP error: {}", response.status());
        }
        
        let data: PypiResponse = response.json().await.context("Failed to parse response")?;
        
        Ok((name.to_string(), data.info.version))
    }
}

impl Default for PypiClient {
    fn default() -> Self {
        Self::new()
    }
}