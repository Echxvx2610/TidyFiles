use tidyfiles::categories::CategoryRegistry;
use tidyfiles::organizer::{Organizer, OrganizerOptions};
use std::fs;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_category_matching() {
    let registry = CategoryRegistry::default_rules();

    let img = registry
        .get_category_for_path(Path::new("vacaciones.jpg"))
        .unwrap();
    assert_eq!(img.name, "Imagenes");

    let doc = registry
        .get_category_for_path(Path::new("factura.pdf"))
        .unwrap();
    assert_eq!(doc.name, "Documentos");

    let vid = registry
        .get_category_for_path(Path::new("pelicula.mp4"))
        .unwrap();
    assert_eq!(vid.name, "Videos");

    let aud = registry
        .get_category_for_path(Path::new("cancion.flac"))
        .unwrap();
    assert_eq!(aud.name, "Audio");

    let zip = registry
        .get_category_for_path(Path::new("backup.tar.gz"))
        .unwrap();
    assert_eq!(zip.name, "Comprimidos");

    let deb = registry
        .get_category_for_path(Path::new("discord.deb"))
        .unwrap();
    assert_eq!(deb.name, "Instaladores");

    let code = registry
        .get_category_for_path(Path::new("main.rs"))
        .unwrap();
    assert_eq!(code.name, "Codigo");

    assert!(
        registry
            .get_category_for_path(Path::new("desconocido.xyz123"))
            .is_none()
    );
}

#[test]
fn test_end_to_end_organize_and_undo() {
    let temp = tempdir().unwrap();
    let root = temp.path();

    // Crear archivos simulados en la carpeta
    let file1 = root.join("foto.png");
    let file2 = root.join("documento.pdf");
    let file3 = root.join("cancion.mp3");
    let file_unknown = root.join("archivo.customformat");

    fs::write(&file1, b"imagen simulada").unwrap();
    fs::write(&file2, b"pdf simulado").unwrap();
    fs::write(&file3, b"audio simulado").unwrap();
    fs::write(&file_unknown, b"desconocido").unwrap();

    let organizer = Organizer::new();
    let options = OrganizerOptions {
        include_hidden: false,
        include_unknown: false,
        dry_run: false,
    };

    // 1. Generar plan
    let plan = organizer.generate_plan(root, &options).unwrap();
    assert_eq!(plan.items.len(), 3);
    assert_eq!(plan.skipped_unrecognized, 1);

    // 2. Ejecutar movimientos
    let tx = organizer.execute_plan(&plan).unwrap();
    assert_eq!(tx.moves.len(), 3);

    // Verificar que los archivos ahora están en sus subcarpetas
    assert!(!file1.exists());
    assert!(!file2.exists());
    assert!(!file3.exists());
    assert!(file_unknown.exists()); // No se debió mover

    let dest_img = root.join("Imagenes").join("foto.png");
    let dest_doc = root.join("Documentos").join("documento.pdf");
    let dest_audio = root.join("Audio").join("cancion.mp3");

    assert!(dest_img.exists());
    assert!(dest_doc.exists());
    assert!(dest_audio.exists());

    // 3. Probar reversión (Undo)
    let undo_result = organizer.undo_last(root).unwrap();
    assert!(undo_result.is_some());

    // Verificar que los archivos volvieron a su posición original
    assert!(file1.exists());
    assert!(file2.exists());
    assert!(file3.exists());

    // Verificar que las subcarpetas vacías se eliminaron
    assert!(!root.join("Imagenes").exists());
    assert!(!root.join("Documentos").exists());
    assert!(!root.join("Audio").exists());
}

#[test]
fn test_collision_resolution() {
    let temp = tempdir().unwrap();
    let root = temp.path();

    // Crear carpeta 'Imagenes' con un archivo existente 'foto.png'
    let img_dir = root.join("Imagenes");
    fs::create_dir(&img_dir).unwrap();
    fs::write(img_dir.join("foto.png"), b"existente").unwrap();

    // Crear otro 'foto.png' en la raíz que se va a mover
    let new_file = root.join("foto.png");
    fs::write(&new_file, b"nuevo").unwrap();

    let organizer = Organizer::new();
    let options = OrganizerOptions {
        include_hidden: false,
        include_unknown: false,
        dry_run: false,
    };

    let plan = organizer.generate_plan(root, &options).unwrap();
    assert_eq!(plan.items.len(), 1);

    let dest_filename = plan.items[0]
        .destination
        .file_name()
        .unwrap()
        .to_str()
        .unwrap();
    assert_eq!(dest_filename, "foto (1).png");

    organizer.execute_plan(&plan).unwrap();
    assert!(img_dir.join("foto.png").exists());
    assert!(img_dir.join("foto (1).png").exists());
}
