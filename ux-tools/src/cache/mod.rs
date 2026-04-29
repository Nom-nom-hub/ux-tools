use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub tool: String,
    pub version: String,
    pub created_at: u64,
    pub last_used: u64,
    pub python_version: String,
    pub executable: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub max_size_gb: f64,
    pub ttl_days: u32,
    pub max_tools: u32,
    pub auto_update: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size_gb: 10.0,
            ttl_days: 30,
            max_tools: 50,
            auto_update: true,
        }
    }
}

pub struct ToolCache {
    cache_dir: PathBuf,
    index: HashMap<String, CacheEntry>,
    aliases: HashMap<String, String>,
    config: CacheConfig,
}

impl ToolCache {
    pub fn new() -> Result<Self> {
        let cache_dir = Self::get_cache_dir()?;
        let index_path = cache_dir.join("index.json");
        let aliases_path = cache_dir.join("aliases.json");

        let index: HashMap<String, CacheEntry> = if index_path.exists() {
            let content = fs::read_to_string(&index_path)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            HashMap::new()
        };

        let aliases: HashMap<String, String> = if aliases_path.exists() {
            let content = fs::read_to_string(&aliases_path)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            HashMap::new()
        };

        let config = Self::load_config(&cache_dir)?;

        Ok(Self {
            cache_dir,
            index,
            aliases,
            config: config.unwrap_or_default(),
        })
    }

    fn get_cache_dir() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "ux", "ux")
            .context("Could not determine cache directory")?;
        Ok(proj_dirs.cache_dir().to_path_buf())
    }

    fn load_config(cache_dir: &Path) -> Result<Option<CacheConfig>> {
        let config_path = cache_dir.join("config.json");
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let config: CacheConfig = serde_json::from_str(&content)?;
            Ok(Some(config))
        } else {
            Ok(None)
        }
    }

    pub fn has_cached(&self, tool: &str) -> bool {
        self.index.contains_key(tool) || self.aliases.contains_key(tool)
    }

    pub fn get_venv_path(&self, tool: &str) -> PathBuf {
        self.cache_dir.join("venvs").join(tool)
    }

    pub fn get_entry(&self, tool: &str) -> Option<&CacheEntry> {
        self.index.get(tool)
    }

    pub fn add_alias(&mut self, alias: &str, tool: &str) -> Result<()> {
        self.aliases.insert(alias.to_string(), tool.to_string());
        self.save_aliases()
    }

    pub fn remove_alias(&mut self, alias: &str) -> Result<Option<String>> {
        let removed = self.aliases.remove(alias);
        if removed.is_some() {
            self.save_aliases()?;
        }
        Ok(removed)
    }

    pub fn resolve_alias(&self, name: &str) -> Option<String> {
        self.aliases.get(name).cloned()
    }

    pub fn list_aliases(&self) {
        if self.aliases.is_empty() {
            println!("No aliases");
        } else {
            println!("Aliases:");
            for (alias, tool) in &self.aliases {
                println!("  {} -> {}", alias, tool);
            }
        }
    }

    pub fn prune_old(&mut self) -> Result<usize> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let ttl = self.config.ttl_days as u64 * 86400;
        let mut removed = 0;

        let to_remove: Vec<String> = self.index
            .iter()
            .filter(|(_, e)| now.saturating_sub(e.last_used) > ttl)
            .map(|(k, _)| k.clone())
            .collect();

        for tool in to_remove {
            let venv_path = self.cache_dir.join("venvs").join(&tool);
            if venv_path.exists() {
                fs::remove_dir_all(&venv_path).ok();
            }
            self.index.remove(&tool);
            removed += 1;
        }

        if removed > 0 {
            self.save_index()?;
        }

        Ok(removed)
    }

    pub fn print_stats(&self) -> Result<()> {
        let count = self.index.len();
        let total_size = self.calculate_size()?;

        println!("Cache Statistics:");
        println!("  Tools: {}", count);
        println!("  Max tools: {}", self.config.max_tools);
        println!("  Max size: {} GB", self.config.max_size_gb);
        println!("  TTL: {} days", self.config.ttl_days);
        println!("  Cache size: {:.2} MB", total_size as f64 / 1024.0 / 1024.0);
        println!("  Aliases: {}", self.aliases.len());

        Ok(())
    }

    fn calculate_size(&self) -> Result<u64> {
        let mut total = 0u64;
        let venvs_dir = self.cache_dir.join("venvs");
        if venvs_dir.exists() {
            for entry in fs::read_dir(venvs_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_dir() {
                    total += Self::dir_size(entry.path());
                }
            }
        }
        Ok(total)
    }

    fn dir_size(path: std::path::PathBuf) -> u64 {
        let mut size = 0u64;
        if let Ok(entries) = fs::read_dir(&path) {
            for entry in entries.flatten() {
                if let Ok(meta) = entry.metadata() {
                    size += meta.len();
                    if meta.is_dir() {
                        size += Self::dir_size(entry.path());
                    }
                }
            }
        }
        size
    }

    pub fn remove_entry(&mut self, tool: &str) -> Result<Option<CacheEntry>> {
        let removed = self.index.remove(tool);
        if removed.is_some() {
            self.save_index()?;
        }
        Ok(removed)
    }

    pub fn list_entries(&self) -> Vec<&CacheEntry> {
        self.index.values().collect()
    }

    pub fn touch(&mut self, tool: &str) -> Result<()> {
        if let Some(entry) = self.index.get_mut(tool) {
            entry.last_used = SystemTime::now()
                .duration_since(UNIX_EPOCH)?
                .as_secs();
            self.save_index()?;
        }
        Ok(())
    }

    pub fn clear(&mut self) -> Result<()> {
        for entry in self.index.values() {
            let venv_path = self.cache_dir.join("venvs").join(&entry.tool);
            if venv_path.exists() {
                fs::remove_dir_all(venv_path)?;
            }
        }
        self.index.clear();
        self.save_index()?;
        Ok(())
    }

    fn save_index(&self) -> Result<()> {
        fs::create_dir_all(&self.cache_dir)?;
        let index_path = self.cache_dir.join("index.json");
        let content = serde_json::to_string_pretty(&self.index)?;
        fs::write(index_path, content)?;
        Ok(())
    }

    fn save_aliases(&self) -> Result<()> {
        fs::create_dir_all(&self.cache_dir)?;
        let aliases_path = self.cache_dir.join("aliases.json");
        let content = serde_json::to_string_pretty(&self.aliases)?;
        fs::write(aliases_path, content)?;
        Ok(())
    }

    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }
}

impl Default for ToolCache {
    fn default() -> Self {
        Self::new().unwrap_or(Self {
            cache_dir: PathBuf::new(),
            index: HashMap::new(),
            aliases: HashMap::new(),
            config: CacheConfig::default(),
        })
    }
}