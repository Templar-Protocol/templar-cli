//! Configuration profiles, network settings, and file management.
//!
//! The configuration system uses TOML files stored at `~/.templar/config.toml`
//! (or the path specified by `$TEMPLAR_CONFIG`). Multiple named profiles
//! (e.g., `mainnet`, `testnet`) can coexist in a single config file.

pub mod network;
pub mod profile;

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::CliError;

/// Default directory name under the user's home.
const TEMPLAR_DIR: &str = ".templar";
/// Default config filename.
const CONFIG_FILE: &str = "config.toml";
/// Environment variable to override config path.
const CONFIG_ENV_VAR: &str = "TEMPLAR_CONFIG";

/// Top-level configuration containing all profiles and theme settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Name of the active profile.
    #[serde(default = "default_active_profile")]
    pub active_profile: String,

    /// Named profiles keyed by name (e.g., "mainnet", "testnet").
    #[serde(default)]
    pub profiles: std::collections::HashMap<String, profile::Profile>,

    /// Theme configuration.
    #[serde(default)]
    pub theme: ThemeConfig,
}

/// Theme settings loaded from the `[theme]` section of config.toml.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// Show banner on startup.
    #[serde(default = "default_true")]
    pub banner: bool,

    /// Color mode: "auto", "always", or "never".
    #[serde(default = "default_color_mode")]
    pub color: String,

    /// Enable text scramble, binary spinner animations.
    #[serde(default = "default_true")]
    pub animations: bool,

    /// Use Unicode box-drawing characters.
    #[serde(default = "default_true")]
    pub unicode: bool,

    /// Voice mode: "cypherpunk" or "standard".
    #[serde(default = "default_voice")]
    pub voice: String,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            banner: true,
            color: "auto".into(),
            animations: true,
            unicode: true,
            voice: "cypherpunk".into(),
        }
    }
}

fn default_active_profile() -> String {
    "mainnet".into()
}

fn default_true() -> bool {
    true
}

fn default_color_mode() -> String {
    "auto".into()
}

fn default_voice() -> String {
    "cypherpunk".into()
}

impl Default for Config {
    fn default() -> Self {
        let mut profiles = std::collections::HashMap::new();
        profiles.insert("mainnet".into(), profile::Profile::mainnet());
        profiles.insert("testnet".into(), profile::Profile::testnet());
        Self { active_profile: "mainnet".into(), profiles, theme: ThemeConfig::default() }
    }
}

impl Config {
    /// Resolve the config file path.
    ///
    /// Priority: `$TEMPLAR_CONFIG` env var > `~/.templar/config.toml`
    pub fn config_path() -> Result<PathBuf, CliError> {
        if let Ok(path) = std::env::var(CONFIG_ENV_VAR) {
            return Ok(PathBuf::from(path));
        }
        let home = dirs::home_dir()
            .ok_or_else(|| CliError::Config("cannot determine home directory".into()))?;
        Ok(home.join(TEMPLAR_DIR).join(CONFIG_FILE))
    }

    /// Resolve the templar data directory (`~/.templar/`).
    pub fn templar_dir() -> Result<PathBuf, CliError> {
        let home = dirs::home_dir()
            .ok_or_else(|| CliError::Config("cannot determine home directory".into()))?;
        Ok(home.join(TEMPLAR_DIR))
    }

    /// Load config from the resolved path, or return defaults if not found.
    pub fn load() -> Result<Self, CliError> {
        let path = Self::config_path()?;
        if path.exists() {
            #[cfg(unix)]
            validate_config_permissions(&path);
            let contents = std::fs::read_to_string(&path)?;
            let config: Config = toml::from_str(&contents)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    /// Save config to the resolved path.
    pub fn save(&self) -> Result<(), CliError> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                create_dir_secure(parent)?;
            }
        }
        let toml_str = toml::to_string_pretty(self)?;
        write_file_secure(&path, toml_str.as_bytes())?;
        Ok(())
    }

    /// Get the active profile.
    pub fn active_profile(&self) -> Result<&profile::Profile, CliError> {
        self.profiles
            .get(&self.active_profile)
            .ok_or_else(|| CliError::Config(format!("profile '{}' not found", self.active_profile)))
    }

    /// Returns `true` if the config file exists on disk.
    pub fn exists() -> Result<bool, CliError> {
        Ok(Self::config_path()?.exists())
    }
}

/// Create a directory with mode 0o700 (Unix) or default (other platforms).
pub fn create_dir_secure(path: &Path) -> Result<(), CliError> {
    #[cfg(unix)]
    {
        use std::fs::DirBuilder;
        use std::os::unix::fs::DirBuilderExt;
        DirBuilder::new().mode(0o700).recursive(true).create(path)?;
    }
    #[cfg(not(unix))]
    {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

/// Write a file with mode 0o600 (Unix) or default (other platforms).
pub fn write_file_secure(path: &Path, data: &[u8]) -> Result<(), CliError> {
    #[cfg(unix)]
    {
        use std::fs::OpenOptions;
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let mut file =
            OpenOptions::new().write(true).create(true).truncate(true).mode(0o600).open(path)?;
        file.write_all(data)?;
    }
    #[cfg(not(unix))]
    {
        std::fs::write(path, data)?;
    }
    Ok(())
}

/// Validate config file permissions on Unix. Warns if group/other readable.
#[cfg(unix)]
pub fn validate_config_permissions(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(metadata) = std::fs::metadata(path) {
        let mode = metadata.permissions().mode() & 0o777;
        if mode & 0o077 != 0 {
            eprintln!(
                "WARNING: Config file {} has overly permissive permissions ({:04o}). \
                 Run 'chmod 600 {}' to fix.",
                path.display(),
                mode,
                path.display(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn default_config_has_mainnet_profile() {
        let config = Config::default();
        assert_eq!(config.active_profile, "mainnet");
        assert!(config.profiles.contains_key("mainnet"));
        assert!(config.profiles.contains_key("testnet"));
    }

    #[test]
    fn default_theme_config() {
        let theme = ThemeConfig::default();
        assert!(theme.banner);
        assert_eq!(theme.color, "auto");
        assert!(theme.animations);
        assert!(theme.unicode);
        assert_eq!(theme.voice, "cypherpunk");
    }

    #[test]
    fn config_round_trips_through_toml() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.active_profile, config.active_profile);
        assert_eq!(parsed.profiles.len(), config.profiles.len());
    }

    #[test]
    fn config_save_and_load() {
        let tmp = TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");
        std::env::set_var(CONFIG_ENV_VAR, config_path.to_str().unwrap());

        let config = Config::default();
        config.save().unwrap();
        assert!(config_path.exists());

        let loaded = Config::load().unwrap();
        assert_eq!(loaded.active_profile, "mainnet");

        std::env::remove_var(CONFIG_ENV_VAR);
    }

    #[test]
    fn missing_config_returns_defaults() {
        let tmp = TempDir::new().unwrap();
        let config_path = tmp.path().join("nonexistent.toml");
        std::env::set_var(CONFIG_ENV_VAR, config_path.to_str().unwrap());

        let config = Config::load().unwrap();
        assert_eq!(config.active_profile, "mainnet");

        std::env::remove_var(CONFIG_ENV_VAR);
    }

    #[test]
    fn invalid_toml_produces_error() {
        let tmp = TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");
        std::fs::write(&config_path, "= invalid toml").unwrap();
        std::env::set_var(CONFIG_ENV_VAR, config_path.to_str().unwrap());

        let result = Config::load();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, CliError::Config(_)));

        std::env::remove_var(CONFIG_ENV_VAR);
    }

    #[test]
    fn active_profile_not_found() {
        let config = Config { active_profile: "nonexistent".into(), ..Config::default() };
        let result = config.active_profile();
        assert!(result.is_err());
    }

    #[cfg(unix)]
    #[test]
    fn secure_file_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = TempDir::new().unwrap();
        let file_path = tmp.path().join("secure.toml");
        write_file_secure(&file_path, b"test data").unwrap();

        let metadata = std::fs::metadata(&file_path).unwrap();
        let mode = metadata.permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }

    #[cfg(unix)]
    #[test]
    fn secure_dir_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = TempDir::new().unwrap();
        let dir_path = tmp.path().join("secure_dir");
        create_dir_secure(&dir_path).unwrap();

        let metadata = std::fs::metadata(&dir_path).unwrap();
        let mode = metadata.permissions().mode() & 0o777;
        assert_eq!(mode, 0o700);
    }
}
