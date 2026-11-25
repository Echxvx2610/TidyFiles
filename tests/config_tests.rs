use tidyfiles::categories::CategoryRegistry;
use tidyfiles::config::{CategoryConfig, Config};
use tidyfiles::organizer::{Organizer, OrganizerOptions};
use std::fs;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_custom_config_categories() {
    let mut config = Config::default();

    // Agregar categoría personalizada 'Modelos3D'
    config.categories.insert(
        "Modelos3D".to_string(),
        CategoryConfig {
            icon: "3D".to_string(),
            extensions: vec!["blend".to_string(), "fbx".to_string(), "obj".to_string()],
        },
    );

    let registry = CategoryRegistry::from_config(&config);
    let matched = registry
        .get_category_for_path(Path::new("personaje.blend"))
        .unwrap();
    assert_eq!(matched.name, "Modelos3D");
    assert_eq!(matched.icon, "3D");
}

#[test]
fn test_organize_with_custom_config() {
    let temp = tempdir().unwrap();
    let root = temp.path();

    let mut config = Config::default();
    config.categories.insert(
        "Torrents".to_string(),
        CategoryConfig {
            icon: "🧲".to_string(),
            extensions: vec!["torrent".to_string()],
        },
    );

    let torrent_file = root.join("ubuntu.torrent");
    fs::write(&torrent_file, b"torrent metadata").unwrap();

    let organizer = Organizer::with_config(&config);
    let options = OrganizerOptions {
        include_hidden: false,
        include_unknown: false,
        dry_run: false,
    };

    let plan = organizer.generate_plan(root, &options).unwrap();
    assert_eq!(plan.items.len(), 1);
    assert_eq!(plan.items[0].category_name, "Torrents");

    organizer.execute_plan(&plan).unwrap();
    assert!(root.join("Torrents").join("ubuntu.torrent").exists());
}

#[test]
fn test_local_config_override() {
    let temp = tempdir().unwrap();
    let root = temp.path();

    // Crear un archivo local tidyfiles.toml con una regla local
    let local_toml = r#"
[categories.MusicaLocal]
icon = "MUS"
extensions = ["mp3", "wav"]
"#;
    fs::write(root.join("tidyfiles.toml"), local_toml).unwrap();

    let resolution = Config::resolve(Some(root)).unwrap();
    assert!(resolution.source_path.is_some());
    assert_eq!(resolution.source_path.unwrap(), root.join("tidyfiles.toml"));
    assert!(resolution.config.categories.contains_key("MusicaLocal"));
}

#[test]
fn test_invalid_local_config_is_an_error() {
    let temp = tempdir().unwrap();
    let root = temp.path();

    let invalid_toml = r#"
[categories.Video Procesado]
icon = "VID"
extensions = ["mp4"]
"#;
    fs::write(root.join("tidyfiles.toml"), invalid_toml).unwrap();

    let error = Config::resolve(Some(root)).unwrap_err();
    let message = format!("{:#}", error);
    assert!(message.contains("Error de sintaxis"));
    assert!(message.contains("tidyfiles.toml"));
}

#[test]
fn test_local_category_name_with_spaces() {
    let temp = tempdir().unwrap();
    let root = temp.path();

    let local_toml = r#"
[categories."Video Procesado"]
icon = "VP"
extensions = ["mp4"]
"#;
    fs::write(root.join("tidyfiles.toml"), local_toml).unwrap();

    let resolution = Config::resolve(Some(root)).unwrap();
    let category = resolution.config.categories.get("Video Procesado").unwrap();
    assert_eq!(category.extensions, vec!["mp4"]);
}
