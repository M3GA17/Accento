use crate::i18n::{Text, UI_CODES, tr};
use accento_core::Language;
use serde::{Deserialize, Serialize};
use std::{io, path::PathBuf};

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)]
pub struct Preferences {
    pub schema_version: u32,
    pub delay_ms: u64,
    pub languages: Vec<String>,
    pub theme: Theme,
    pub ui_language: String,
    pub excluded_apps: Vec<String>,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            schema_version: 1,
            delay_ms: 300,
            languages: vec!["it".into()],
            theme: Theme::System,
            ui_language: "it".into(),
            excluded_apps: vec![],
        }
    }
}

impl Preferences {
    pub fn languages(&self) -> Vec<Language> {
        self.languages
            .iter()
            .filter_map(|s| Language::from_code(s))
            .collect()
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err(tr(&self.ui_language, Text::ConfigError).into());
        }
        if !(100..=1500).contains(&self.delay_ms) {
            return Err(tr(&self.ui_language, Text::DelayError).into());
        }
        if self.languages.is_empty()
            || self
                .languages
                .iter()
                .any(|s| Language::from_code(s).is_none())
        {
            return Err(tr(&self.ui_language, Text::AtLeastOne).into());
        }
        if !UI_CODES.contains(&self.ui_language.as_str()) {
            return Err(tr(&self.ui_language, Text::LanguageError).into());
        }
        if self
            .excluded_apps
            .iter()
            .any(|s| normalize_executable(s).is_none() || s.trim() != s)
        {
            return Err(tr(&self.ui_language, Text::AppError).into());
        }
        Ok(())
    }
    pub fn path() -> PathBuf {
        std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
            .join("Accento")
            .join("settings.json")
    }
    /// An invalid file is preserved for recovery; the UI reports that defaults are in use.
    pub fn load() -> (Self, Option<String>) {
        match std::fs::read(Self::path()) {
            Ok(data) => match serde_json::from_slice::<Self>(&data)
                .map_err(|e| e.to_string())
                .and_then(|p| {
                    p.validate()?;
                    Ok(p)
                }) {
                Ok(p) => (p, None),
                Err(e) => (
                    Self::default(),
                    Some(format!(
                        "Configurazione non valida: {e} Il file originale è conservato. Salva per sostituirlo."
                    )),
                ),
            },
            Err(e) if e.kind() == io::ErrorKind::NotFound => (Self::default(), None),
            Err(e) => (
                Self::default(),
                Some(format!("Impossibile leggere le preferenze: {e}")),
            ),
        }
    }
    pub fn save(&self) -> Result<(), String> {
        self.save_to(&Self::path())
    }
    fn save_to(&self, path: &std::path::Path) -> Result<(), String> {
        self.validate()?;
        std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
        let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
        let data = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        use std::io::Write;
        let mut file = std::fs::File::create(&temporary).map_err(|e| e.to_string())?;
        file.write_all(&data)
            .and_then(|_| file.sync_all())
            .map_err(|e| e.to_string())?;
        drop(file);
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStrExt;
            use windows::{Win32::Storage::FileSystem::*, core::PCWSTR};
            let from: Vec<_> = temporary.as_os_str().encode_wide().chain(Some(0)).collect();
            let to: Vec<_> = path.as_os_str().encode_wide().chain(Some(0)).collect();
            unsafe {
                MoveFileExW(
                    PCWSTR(from.as_ptr()),
                    PCWSTR(to.as_ptr()),
                    MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
                )
            }
            .map_err(|e| e.to_string())?;
        }
        #[cfg(not(windows))]
        std::fs::rename(temporary, path).map_err(|e| e.to_string())?;
        Ok(())
    }
}

/// One executable per entry. Never accept separators, paths or shell wildcards.
pub fn normalize_executable(value: &str) -> Option<String> {
    let value = value.trim();
    (value.len() > 4
        && value.len() <= 260
        && value.to_ascii_lowercase().ends_with(".exe")
        && !value
            .chars()
            .any(|c| c.is_control() || r#"/\,:;*?<>|""#.contains(c)))
    .then(|| value.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn interface_locale_and_extended_sets_are_independent() {
        for locale in ["en", "sq"] {
            let preferences = Preferences {
                ui_language: locale.into(),
                languages: vec![
                    "sq".into(),
                    "pt".into(),
                    "ro".into(),
                    "pl".into(),
                    "tr".into(),
                ],
                ..Preferences::default()
            };
            assert!(preferences.validate().is_ok());
            assert_eq!(preferences.languages().len(), 5);
        }
    }
    #[test]
    fn old_glass_setting_is_ignored_and_ui_language_is_independent() {
        let p: Preferences =
            serde_json::from_str(r#"{"transparency":false,"languages":["fr"],"delay_ms":450}"#)
                .unwrap();
        assert_eq!(p.ui_language, "it");
        assert_eq!(p.languages, ["fr"]);
        assert_eq!(p.delay_ms, 450);
        assert!(p.validate().is_ok());
        assert!(!serde_json::to_string(&p).unwrap().contains("transparency"));
        let mut p = p;
        p.ui_language = "de".into();
        assert!(p.validate().is_ok());
        assert_eq!(p.languages, ["fr"]);
        p.ui_language = "zz".into();
        assert!(p.validate().is_err());
    }
    #[test]
    fn blacklist_accepts_only_one_executable_per_item() {
        assert_eq!(
            normalize_executable(" Notepad.EXE ").as_deref(),
            Some("notepad.exe")
        );
        for invalid in [
            "a.exe,b.exe",
            "a.exe;b.exe",
            "C:\\a.exe",
            "*.exe",
            ".exe",
            "a.exe\nb.exe",
        ] {
            assert!(normalize_executable(invalid).is_none(), "{invalid}");
        }
    }
    #[test]
    fn invalid_or_future_configuration_is_rejected() {
        let mut p = Preferences {
            schema_version: 2,
            ..Preferences::default()
        };
        assert!(p.validate().is_err());
        p.schema_version = 1;
        p.delay_ms = 20;
        assert!(p.validate().is_err());
        p.delay_ms = 300;
        p.excluded_apps = vec!["note".into()];
        assert!(p.validate().is_err());
        p.excluded_apps = vec!["notepad.exe".into()];
        assert!(p.validate().is_ok());
    }
    #[test]
    fn atomic_save_replaces_only_a_valid_configuration() {
        let root = std::env::temp_dir().join(format!(
            "Accento-config-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&root).unwrap();
        let path = root.join("settings.json");
        std::fs::write(&path, b"original").unwrap();
        let invalid = Preferences {
            delay_ms: 2,
            ..Preferences::default()
        };
        assert!(invalid.save_to(&path).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"original");
        let valid = Preferences {
            languages: vec!["it".into(), "fr".into()],
            ui_language: "de".into(),
            excluded_apps: vec!["notepad.exe".into(), "code.exe".into()],
            ..Preferences::default()
        };
        valid.save_to(&path).unwrap();
        let loaded: Preferences = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(loaded, valid);
        assert!(loaded.validate().is_ok());
        assert_eq!(std::fs::read_dir(&root).unwrap().count(), 1);
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(root).unwrap();
    }
}
