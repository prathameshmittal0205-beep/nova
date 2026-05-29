use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SessionBuffer {
    pub path:        PathBuf,
    pub cursor_line: usize,
    pub cursor_col:  usize,
    pub scroll_top:  usize,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Session {
    pub buffers:       Vec<SessionBuffer>,
    pub active_buffer: usize,
    pub working_dir:   PathBuf,
}

impl Session {
    pub fn session_path() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("ted")
            .join("session.toml")
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::session_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating session dir {}", parent.display()))?;
        }
        let content = toml::to_string(self).context("serializing session")?;
        std::fs::write(&path, content)
            .with_context(|| format!("writing session to {}", path.display()))?;
        Ok(())
    }

    pub fn load() -> Result<Self> {
        let path = Self::session_path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("reading session from {}", path.display()))?;
        let session = toml::from_str(&content).context("parsing session")?;
        Ok(session)
    }

    pub fn clear() -> Result<()> {
        let path = Self::session_path();
        if path.exists() {
            std::fs::remove_file(&path)
                .with_context(|| format!("removing session file {}", path.display()))?;
        }
        Ok(())
    }
}

// ── Nova Tauri GUI Workspace Session ──────────────────────────────────────────

use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSession {
    pub version: u32,
    pub workspace_path: String,
    pub last_updated: u64,
    pub left_pane: SessionPane,
    pub right_pane: Option<SessionPane>,
    pub focused_pane: String, // "left" | "right"
    pub terminals: Vec<SessionTerminal>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SessionPane {
    pub tabs: Vec<SessionTab>,
    pub active_idx: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionTab {
    pub path: String,
    pub name: String,
    pub dirty: bool,
    pub language: String,
    pub kind: Option<String>,
    pub ai_provider: Option<String>,
    pub cursor_line: Option<usize>,
    pub cursor_col: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionTerminal {
    pub id: String,
    pub shell: String,
    pub cwd: Option<String>,
    pub title: Option<String>,
    pub split_id: Option<String>,
}

impl WorkspaceSession {
    fn session_dir() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("nova")
            .join("sessions")
    }

    fn session_path(workspace_path: &str) -> PathBuf {
        let mut hasher = DefaultHasher::new();
        workspace_path.hash(&mut hasher);
        let hash = hasher.finish();
        Self::session_dir().join(format!("{:016x}.json", hash))
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::session_path(&self.workspace_path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating session dir {}", parent.display()))?;
        }
        let content = serde_json::to_string(self).context("serializing workspace session")?;
        
        // Write to temp file then rename for atomic write
        let tmp_path = path.with_extension("json.tmp");
        std::fs::write(&tmp_path, content)
            .with_context(|| format!("writing session to {}", tmp_path.display()))?;
        std::fs::rename(&tmp_path, &path)
            .with_context(|| format!("renaming session file to {}", path.display()))?;
            
        Ok(())
    }

    pub fn load(workspace_path: &str) -> Result<Self> {
        let path = Self::session_path(workspace_path);
        if !path.exists() {
            return Ok(Self {
                version: 1,
                workspace_path: workspace_path.to_string(),
                ..Default::default()
            });
        }
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("reading session from {}", path.display()))?;
        let session = serde_json::from_str(&content).context("parsing workspace session")?;
        Ok(session)
    }
}
