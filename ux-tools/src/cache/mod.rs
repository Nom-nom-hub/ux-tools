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
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size_gb: 10.0,
            ttl_days: 30,
        }
    }
}

pub struct ToolCache {
    cache_dir: PathBuf,
    index: HashMap<String, CacheEntry>,
    config: CacheConfig,
}

impl ToolCache {
    pub fn new() -> Result<Self> {
        let cache_dir = Self::get_cache_dir()?;
        let index_path = cache_dir.join("index.json");

        let index = if index_path.exists() {
            let content = fs::read_to_string(&index_path)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            HashMap::new()
        };

        let config = Self::load_config(&cache_dir)?;

        Ok(Self {
            cache_dir,
            index,
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

    pub fn get_venv_path(&self, tool: &str, version: &str) -> PathBuf {
        self.cache_dir.join("venvs").join(format!("{}-{}", tool, version))
    }

    pub fn get_entry(&self, tool: &str) -> Option<&CacheEntry> {
        self.index.get(tool)
    }

    pub fn has_cached(&self, tool: &str) -> bool {
        self.index.contains_key(tool)
    }

    pub fn save_entry(&mut self, entry: CacheEntry) -> Result<()> {
        self.index.insert(entry.tool.clone(), entry);
        self.save_index()?;
        Ok(())
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
            let venv_path = self.get_venv_path(&entry.tool, &entry.version);
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

    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }
}

impl Default for ToolCache {
    fn default() -> Self {
        Self::new().unwrap_or(Self {
            cache_dir: PathBuf::new(),
            index: HashMap::new(),
            config: CacheConfig::default(),
        })
    }
}