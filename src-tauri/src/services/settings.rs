use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_true", alias = "startWithWindows")]
    pub start_with_windows: bool,
    #[serde(default = "default_model_id", alias = "modelId")]
    pub model_id: String,
}

fn default_true() -> bool {
    true
}

fn default_model_id() -> String {
    crate::services::cleanup::DEFAULT_MODEL.to_string()
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            start_with_windows: true,
            model_id: default_model_id(),
        }
    }
}

impl Settings {
    fn path() -> PathBuf {
        let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        base.join("com.dikte.app").join("settings.json")
    }

    pub fn load() -> anyhow::Result<Self> {
        let p = Self::path();
        if !p.exists() {
            return Ok(Self::default());
        }
        let data = std::fs::read_to_string(&p)?;
        let s: Self = serde_json::from_str(&data)?;
        Ok(s)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let p = Self::path();
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(&p, data)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_json_includes_model_id_excludes_api_key() {
        let s = Settings {
            start_with_windows: true,
            model_id: "deepseek-v4-flash-free".into(),
        };
        let json = serde_json::to_string(&s).unwrap();
        assert!(json.contains("model_id"));
        assert!(json.contains("deepseek-v4-flash-free"));
        assert!(json.contains("start_with_windows"));
        assert!(!json.contains("api_key"));
        assert!(!json.contains("apiKey"));
    }

    #[test]
    fn settings_load_defaults_model_id_for_old_file() {
        // Old settings JSON without model_id should default correctly
        let old = r#"{"start_with_windows": false}"#;
        let s: Settings = serde_json::from_str(old).unwrap();
        assert_eq!(s.start_with_windows, false);
        assert_eq!(s.model_id, crate::services::cleanup::DEFAULT_MODEL);
    }

    #[test]
    fn settings_alias_camel_case() {
        let camel = r#"{"startWithWindows": true, "modelId": "custom-model"}"#;
        let s: Settings = serde_json::from_str(camel).unwrap();
        assert_eq!(s.start_with_windows, true);
        assert_eq!(s.model_id, "custom-model");
    }

    #[test]
    fn settings_default_has_expected_values() {
        let d = Settings::default();
        assert_eq!(d.start_with_windows, true);
        assert_eq!(d.model_id, crate::services::cleanup::DEFAULT_MODEL);
    }
}
