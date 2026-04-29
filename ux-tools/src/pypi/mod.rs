use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct PypiClient {
    client: Client,
    base_url: String,
}

#[derive(Debug, Deserialize)]
struct SimpleIndex {
    #[serde(default)]
    files: Vec<SimpleFile>,
}

#[derive(Debug, Deserialize)]
struct SimpleFile {
    filename: String,
    #[serde(default)]
    python_version: String,
    #[serde(default)]
    requires_python: Option<String>,
}

impl PypiClient {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent("ux/0.1.0")
                .build()
                .unwrap_or_else(|_| Client::new()),
            base_url: "https://pypi.org/simple".to_string(),
        }
    }

    pub async fn get_package(&self, name: &str) -> Result<(String, String)> {
        let url = format!("{}/{}/", self.base_url, name);
        
        let response = self.client.get(&url).send().await.context("Network error")?;
        
        if response.status() == 404 {
            anyhow::bail!("Package not found: {}", name);
        }
        
        if response.status() == 301 {
            let new_url = response.headers().get("location")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());
            
            if let Some(new_url) = new_url {
                return self.get_package_from_url(&new_url, name).await;
            }
        }

        let html = response.text().await.context("Failed to read response")?;
        
        let version = self.parse_simple_index(&html)?;
        
        Ok((name.to_string(), version))
    }

    async fn get_package_from_url(&self, url: &str, name: &str) -> Result<(String, String)> {
        let response = self.client.get(url).send().await.context("Network error")?;
        
        if response.status() == 404 {
            anyhow::bail!("Package not found: {}", name);
        }
        
        let html = response.text().await.context("Failed to read response")?;
        let version = self.parse_simple_index(&html)?;
        
        Ok((name.to_string(), version))
    }

    fn parse_simple_index(&self, html: &str) -> Result<String> {
        let mut latest = String::new();
        let mut latest_version = 0u64;
        
        for line in html.lines() {
            if line.contains(".whl") || line.contains(".tar.gz") || line.contains(".pyz") {
                if let Some(version) = self.extract_version(line) {
                    let parsed = self.parse_version(&version);
                    if parsed > latest_version {
                        latest_version = parsed;
                        latest = version;
                    }
                }
            }
        }
        
        if latest.is_empty() {
            latest = "0.0.0".to_string();
        }
        
        Ok(latest)
    }

    fn extract_version(&self, line: &str) -> Option<String> {
        let start = line.find(">")?;
        let rest = &line[start+1..];
        let end = rest.find('<')?;
        let filename = &rest[..end];
        
        for part in filename.split('-') {
            if part.len() > 1 && part.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                let clean = part.trim_start_matches(".tar");
                return Some(clean.to_string());
            }
        }
        None
    }

    fn parse_version(&self, v: &str) -> u64 {
        let mut result = 0u64;
        for (i, part) in v.split('.').take(3).enumerate() {
            let num: u64 = part.chars().filter(|c| c.is_ascii_digit()).collect::<String>().parse().unwrap_or(0);
            result += num * 100_0000u64.saturating_pow(i as u32);
        }
        result
    }
}

impl Default for PypiClient {
    fn default() -> Self {
        Self::new()
    }
}