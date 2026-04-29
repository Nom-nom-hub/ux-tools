use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

pub struct Runner {
    cache_dir: PathBuf,
}

impl Runner {
    pub fn new(cache_dir: PathBuf) -> Self {
        Self { cache_dir }
    }

    pub fn venv_dir(&self, tool: &str) -> PathBuf {
        self.cache_dir.join("venvs").join(tool)
    }

    pub async fn find_python(&self) -> Result<String> {
        let output = Command::new("which")
            .arg("python3")
            .output()
            .await
            .context("Failed to find python3")?;

        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return Ok(path);
            }
        }

        let output = Command::new("which")
            .arg("python")
            .output()
            .await
            .context("Failed to find python")?;

        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if path.is_empty() {
            anyhow::bail!("No python found");
        }
        Ok(path)
    }

    pub async fn create_venv(&self, path: &PathBuf, python: &str) -> Result<()> {
        if path.exists() {
            return Ok(());
        }

        tokio::fs::create_dir_all(path).await?;

        let output = Command::new(python)
            .args(["-m", "venv", path.to_str().unwrap()])
            .output()
            .await
            .context("Failed to create venv")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to create venv: {}", stderr);
        }

        Ok(())
    }

    pub async fn install_package(&self, venv: &PathBuf, package: &str) -> Result<String> {
        let pip = venv.join("bin/pip");

        let output = Command::new(pip.to_str().unwrap())
            .args(["install", package, "--quiet"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .context("Failed to install package")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to install {}: {}", package, stderr);
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub async fn warm(&self, tool: &str) -> Result<CacheInfo> {
        let python = self.find_python().await?;
        let venv_dir = self.venv_dir(tool);

        if venv_dir.exists() {
            let bin_py = venv_dir.join("bin/python");
            if bin_py.exists() {
                return Ok(CacheInfo {
                    tool: tool.to_string(),
                    version: "cached".to_string(),
                });
            }
        }

        self.create_venv(&venv_dir, &python).await?;
        
        let output = self.install_package(&venv_dir, tool).await?;
        if output.is_empty() {
            eprintln!("Installed {}", tool);
        } else {
            eprintln!("{}", output);
        }

        Ok(CacheInfo {
            tool: tool.to_string(),
            version: "installed".to_string(),
        })
    }

    pub async fn run_tool(&self, tool: &str, args: &[String]) -> Result<i32> {
        let venv_dir = self.venv_dir(tool);
        let bin_dir = venv_dir.join("bin");

        let executable = self.find_executable(tool, &bin_dir).await?;
        let bin_path = bin_dir.join(&executable);

        if !bin_path.exists() {
            anyhow::bail!("Executable not found: {}", executable);
        }

        let status = Command::new(bin_path)
            .args(args)
            .current_dir(std::env::current_dir()?)
            .spawn()?
            .wait()
            .await?
            .code()
            .unwrap_or(1);

        Ok(status)
    }

    async fn get_entry_points(&self, tool: &str, bin_dir: &PathBuf) -> Result<Vec<String>> {
        let output = Command::new(bin_dir.join("python").to_str().unwrap())
            .args([
                "-c",
                &format!(
                    "import importlib.metadata; print('\\n'.join(importlib.metadata.entry_points().get('console_scripts', [])))"
                ),
            ])
            .output()
            .await?;

        if !output.status.success() {
            return Ok(vec![tool.to_string()]);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let entries: Vec<String> = stdout
            .lines()
            .filter_map(|line| {
                let mut parts = line.split('=');
                parts.next().map(|s| s.trim().to_string())
            })
            .collect();

        if entries.is_empty() {
            Ok(vec![tool.to_string()])
        } else {
            if entries.iter().any(|e| e == tool) {
                Ok(entries)
            } else {
                Ok(vec![entries.first().cloned().unwrap_or_else(|| tool.to_string())])
            }
        }
    }

    pub async fn find_executable(&self, tool: &str, bin_dir: &PathBuf) -> Result<String> {
        let bin_path = bin_dir.join(tool);
        if bin_path.exists() {
            return Ok(tool.to_string());
        }

        let entries = self.get_entry_points(tool, bin_dir).await?;

        entries
            .first()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No executables found"))
    }

    pub async fn run_with_uv(&self, package: &str, args: &[String]) -> Result<i32> {
        let mut cmd = tokio::process::Command::new("uv");
        cmd.arg("tool");
        cmd.arg("run");
        cmd.arg(package);
        cmd.args(args);

        let status = cmd
            .spawn()?
            .wait()
            .await?
            .code()
            .unwrap_or(1);

        Ok(status)
    }

    pub async fn update(&self, tool: &str) -> Result<String> {
        let venv_dir = self.venv_dir(tool);
        if !venv_dir.exists() {
            anyhow::bail!("Tool not installed: {}", tool);
        }

        let new_version = self.upgrade_package(&venv_dir, tool).await?;
        Ok(new_version)
    }

    pub async fn check_tool_updates(&self, tool: &str) -> Result<(String, String)> {
        let client = reqwest::Client::new();
        let url = format!("https://pypi.org/pypi/{}/json", tool);
        
        let response = client.get(&url).send().await?;
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            anyhow::bail!("Package not found: {}", tool);
        }

        let data: serde_json::Value = response.json().await?;
        let new_version = data["info"]["version"].as_str().unwrap_or("unknown");

        Ok(("current".to_string(), new_version.to_string()))
    }

    pub async fn check_all_updates(&self) -> Result<Vec<(String, String, String)>> {
        let mut updates = vec![];
        
        let venvs_dir = self.cache_dir.join("venvs");
        if !venvs_dir.exists() {
            return Ok(updates);
        }

        let client = reqwest::Client::new();

        if let Ok(entries) = std::fs::read_dir(&venvs_dir) {
            for entry in entries.flatten() {
                let tool = entry.file_name().to_string_lossy().to_string();
                
                let url = format!("https://pypi.org/pypi/{}/json", tool);
                if let Ok(response) = client.get(&url).send().await {
                    if let Ok(data) = response.json::<serde_json::Value>().await {
                        let new_version = data["info"]["version"].as_str().unwrap_or("unknown");
                        updates.push((tool, "current".to_string(), new_version.to_string()));
                    }
                }
            }
        }

        Ok(updates)
    }

    async fn upgrade_package(&self, venv: &PathBuf, package: &str) -> Result<String> {
        let pip = venv.join("bin/pip");

        let output = tokio::process::Command::new(pip.to_str().unwrap())
            .args(["install", "--upgrade", package, "--quiet"])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to upgrade {}: {}", package, stderr);
        }

        let client = reqwest::Client::new();
        let url = format!("https://pypi.org/pypi/{}/json", package);
        
        let response = client.get(&url).send().await?;
        let data: serde_json::Value = response.json().await?;
        Ok(data["info"]["version"].as_str().unwrap_or("unknown").to_string())
    }
}

#[derive(Debug, Clone)]
pub struct CacheInfo {
    pub tool: String,
    pub version: String,
}