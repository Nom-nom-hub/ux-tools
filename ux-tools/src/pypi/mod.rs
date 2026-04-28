use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct PypiClient {
    client: Client,
    base_url: String,
}

#[derive(Debug, Deserialize, Default)]
struct PackageResponse {
    info: Option<PackageInfo>,
}

#[derive(Debug, Deserialize, Default)]
struct PackageInfo {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
}

impl PypiClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: "https://pypi.org/pypi".to_string(),
        }
    }

    pub async fn get_package(&self, name: &str) -> Result<(String, String)> {
        let url = format!("{}/{}/json", self.base_url, name);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch package")?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            anyhow::bail!("Package not found: {}", name);
        }

        let data: PackageResponse = response
            .json()
            .await
            .context("Failed to parse package metadata")?;

        let info = data.info.context("Missing package info")?;
        let name = info.name.unwrap_or_else(|| name.to_string());
        let version = info.version.unwrap_or_else(|| "latest".to_string());

        Ok((name, version))
    }

    pub async fn get_latest_version(&self, name: &str) -> Result<String> {
        let (_, version) = self.get_package(name).await?;
        Ok(version)
    }
}

impl Default for PypiClient {
    fn default() -> Self {
        Self::new()
    }
}