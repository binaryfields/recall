use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    #[serde(skip)]
    path: Option<PathBuf>,
    plugins: BTreeMap<String, serde_json::Map<String, serde_json::Value>>,
}

impl Config {
    pub fn load() -> Self {
        let Some(home) = std::env::home_dir() else {
            return Self::default();
        };
        let path = home.join(".config/recall/config.json");
        let mut config = Self::read(&path);
        config.path = Some(path);
        config
    }

    fn read(path: &Path) -> Self {
        let Ok(json) = fs::read_to_string(path) else {
            return Self::default();
        };
        match serde_json::from_str(&json) {
            Ok(config) => config,
            Err(_) => {
                let _ = fs::rename(path, path.with_extension("json.bad"));
                Self::default()
            }
        }
    }

    pub fn flag(&self, plugin: &str, flag: &str) -> Option<bool> {
        self.plugins.get(plugin)?.get(flag)?.as_bool()
    }

    pub fn set_flag(&mut self, plugin: &str, flag: &str, on: bool) {
        let section = self.plugins.entry(plugin.to_string()).or_default();
        section.insert(flag.to_string(), on.into());
    }

    pub fn save(&self) {
        let Some(path) = &self.path else {
            return;
        };
        let Some(dir) = path.parent() else {
            return;
        };
        if fs::create_dir_all(dir).is_err() {
            return;
        }
        let Ok(json) = serde_json::to_string_pretty(self) else {
            return;
        };
        let tmp = path.with_extension("json.tmp");
        if fs::write(&tmp, json).is_ok() {
            let _ = fs::rename(&tmp, path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(tag: &str) -> Self {
            let dir = std::env::temp_dir().join(format!("recall-{tag}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn read_corrupt_json() {
        let dir = TempDir::new("read-corrupt-json");
        let target = dir.0.join("config.json");
        fs::write(&target, "{ not json").unwrap();
        let config = Config::read(&target);
        assert!(config.path.is_none());
        assert!(!target.exists());
        assert!(target.with_extension("json.bad").exists());
    }

    #[test]
    fn set_flag_keeps_unknown_keys() {
        let mut config: Config =
            serde_json::from_str(r#"{"plugins":{"memory":{"future_knob":42,"rss":false}}}"#)
                .unwrap();
        config.set_flag("memory", "rss", true);
        assert_eq!(
            serde_json::to_value(&config.plugins["memory"]).unwrap(),
            serde_json::json!({"future_knob": 42, "rss": true})
        );
        assert_eq!(config.flag("memory", "rss"), Some(true));
        assert_eq!(config.flag("cpu", "enabled"), None);
    }

    #[test]
    fn save_then_read() {
        let dir = TempDir::new("save-then-read");
        let target = dir.0.join("nested").join("config.json");
        let mut config = Config {
            path: Some(target.clone()),
            ..Default::default()
        };
        config.set_flag("memory", "enabled", false);
        config.save();
        assert!(target.exists());
        assert!(!target.with_extension("json.tmp").exists());
        let reloaded = Config::read(&target);
        assert!(reloaded.plugins == config.plugins);
    }
}
