use orderfiles::categories::CategoryRegistry;
use orderfiles::config::{CategoryConfig, Config};
use orderfiles::organizer::{Organizer, OrganizerOptions};
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
            icon: "🧊".to_string(),
            extensions: vec!["blend".to_string(), "fbx".to_string(), "obj".to_string()],
        },
    );

    let registry = CategoryRegistry::from_config(&config);
    let matched = registry
        .get_category_for_path(Path::new("personaje.blend"))
        .unwrap();
    assert_eq!(matched.name, "Modelos3D");
    assert_eq!(matched.icon, "🧊");
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
