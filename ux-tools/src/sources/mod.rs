use anyhow::{bail, Result};
use reqwest::Client;
use serde::Deserialize;
use std::path::PathBuf;

pub async fn fetch_github(owner: &str, repo: &str, path: &str, r#ref: Option<&str>) -> Result<String> {
    let client = Client::new();
    let r#ref = r#ref.unwrap_or("main");
    let url = format!(
        "https://raw.githubusercontent.com/{}/{}/{}/{}",
        owner, repo, r#ref, path
    );

    let response = client.get(&url).send().await?;
    if !response.status().is_success() {
        bail!("Failed to fetch {}: status {}", url, response.status());
    }

    let content = response.text().await?;
    Ok(content)
}

pub async fn fetch_gist(gist_id: &str, filename: Option<&str>) -> Result<String> {
    let client = Client::new();
    let url = format!("https://api.github.com/gists/{}", gist_id);

    let response = client.get(&url).send().await?;
    if !response.status().is_success() {
        bail!("Failed to fetch gist: status {}", response.status());
    }

    #[derive(Deserialize)]
    struct GistResponse {
        files: std::collections::HashMap<String, GistFile>,
    }

    #[derive(Deserialize)]
    struct GistFile {
        content: String,
    }

    let gist: GistResponse = response.json().await?;

    if let Some(name) = filename {
        gist.files
            .get(name)
            .map(|f| f.content.clone())
            .ok_or_else(|| anyhow::anyhow!("File not found: {}", name))
    } else {
        gist.files
            .values()
            .next()
            .map(|f| f.content.clone())
            .ok_or_else(|| anyhow::anyhow!("Empty gist"))
    }
}

pub async fn fetch_url(url: &str) -> Result<String> {
    let client = Client::new();
    let response = client.get(url).send().await?;
    if !response.status().is_success() {
        bail!("Failed to fetch {}: status {}", url, response.status());
    }
    let content = response.text().await?;
    Ok(content)
}

pub fn parse_source(source: &str) -> Option<Source> {
    if source.starts_with("github:") {
        let rest = &source[7..];
        if let Some((repo_path, rest)) = rest.split_once('/') {
            if let Some((owner, repo)) = repo_path.split_once('/') {
                let (path, r#ref) = if let Some((p, r)) = rest.split_once('@') {
                    (p, Some(r))
                } else {
                    (rest, None)
                };
                return Some(Source::GitHub {
                    owner: owner.to_string(),
                    repo: repo.to_string(),
                    path: path.to_string(),
                    r#ref: r#ref.map(String::from),
                });
            }
        }
    } else if source.starts_with("gist:") {
        let rest = &source[5..];
        let (gist_id, filename) = if let Some((id, file)) = rest.split_once(':') {
            (id, Some(file))
        } else {
            (rest, None)
        };
        return Some(Source::Gist {
            gist_id: gist_id.to_string(),
            filename: filename.map(String::from),
        });
    } else if source.starts_with("http://") || source.starts_with("https://") {
        return Some(Source::Url(source.to_string()));
    } else {
        let p = PathBuf::from(source);
        if p.exists() {
            return Some(Source::Local(p));
        }
    }

    None
}

#[derive(Debug, Clone)]
pub enum Source {
    GitHub {
        owner: String,
        repo: String,
        path: String,
        r#ref: Option<String>,
    },
    Gist {
        gist_id: String,
        filename: Option<String>,
    },
    Url(String),
    Local(PathBuf),
}