use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryConfig {
    pub icon: String,
    pub extensions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub categories: BTreeMap<String, CategoryConfig>,
}

impl Default for Config {
    fn default() -> Self {
        let mut categories = BTreeMap::new();

        categories.insert(
            "Imagenes".to_string(),
            CategoryConfig {
                icon: "🖼️".to_string(),
                extensions: vec![
                    "jpg", "jpeg", "png", "gif", "webp", "svg", "bmp", "ico", "tiff", "tif",
                    "heic", "heif", "avif", "raw", "cr2", "nef", "psd", "ai",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
            },
        );

        categories.insert(
            "Documentos".to_string(),
            CategoryConfig {
                icon: "📄".to_string(),
                extensions: vec![
                    "pdf", "docx", "doc", "txt", "rtf", "odt", "epub", "mobi", "md", "pages",
                    "xlsx", "xls", "csv", "tsv", "ods", "pptx", "ppt", "odp",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
            },
        );

        categories.insert(
            "Videos".to_string(),
            CategoryConfig {
                icon: "🎬".to_string(),
                extensions: vec![
                    "mp4", "mkv", "avi", "mov", "wmv", "flv", "webm", "m4v", "mpg", "mpeg", "3gp",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
            },
        );

        categories.insert(
            "Audio".to_string(),
            CategoryConfig {
                icon: "🎵".to_string(),
                extensions: vec![
                    "mp3", "wav", "flac", "aac", "ogg", "m4a", "wma", "opus", "aiff", "mid", "midi",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
            },
        );

        categories.insert(
            "Comprimidos".to_string(),
            CategoryConfig {
                icon: "📦".to_string(),
                extensions: vec![
                    "zip", "tar", "gz", "bz2", "7z", "rar", "xz", "tgz", "tbz2", "zst", "iso",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
            },
        );

        categories.insert(
            "Instaladores".to_string(),
            CategoryConfig {
                icon: "⚙️".to_string(),
                extensions: vec![
                    "deb",
                    "rpm",
                    "AppImage",
                    "appimage",
                    "flatpakref",
                    "snap",
                    "exe",
                    "msi",
                    "apk",
                    "dmg",
                    "pkg",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
            },
        );

        categories.insert(
            "Codigo".to_string(),
            CategoryConfig {
                icon: "💻".to_string(),
                extensions: vec![
                    "rs", "py", "js", "ts", "tsx", "jsx", "c", "cpp", "h", "hpp", "cs", "java",
                    "go", "rb", "php", "sh", "bash", "zsh", "html", "css", "scss", "sql", "json",
                    "toml", "yaml", "yml", "xml",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
            },
        );

        categories.insert(
            "Fuentes".to_string(),
            CategoryConfig {
                icon: "🔤".to_string(),
                extensions: vec!["ttf", "otf", "woff", "woff2", "eot"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
            },
        );

        Config { categories }
    }
}

impl Config {
    pub fn load_or_default(target_dir: Option<&Path>) -> Self {
        // 1. Probar en el directorio destino: target_dir/orderfiles.toml
        if let Some(target) = target_dir {
            let local_path = target.join("orderfiles.toml");
            if local_path.exists() {
                if let Ok(content) = fs::read_to_string(&local_path) {
                    if let Ok(config) = toml::from_str::<Config>(&content) {
                        return config;
                    }
                }
            }
        }

        // 2. Probar en ~/.config/orderfiles/config.toml
        if let Some(home) = std::env::var_os("HOME") {
            let global_path = PathBuf::from(home)
                .join(".config")
                .join("orderfiles")
                .join("config.toml");
            if global_path.exists() {
                if let Ok(content) = fs::read_to_string(&global_path) {
                    if let Ok(config) = toml::from_str::<Config>(&content) {
                        return config;
                    }
                }
            }
        }

        Self::default()
    }

    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let serialized = toml::to_string_pretty(self)
            .context("Error serializando la configuración a formato TOML")?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, serialized)
            .with_context(|| format!("No se pudo escribir en {}", path.display()))?;
        Ok(())
    }
}
