use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "orderfiles",
    author = "OrderFiles Team",
    version = "0.2.0",
    about = "⚡ Organizador ultrarrápido y seguro de archivos para directorios desordenados en Rust",
    long_about = "OrderFiles clasifica automáticamente archivos por tipo (imágenes, documentos, videos, comprimidos, etc.),\ncon soporte para interfaz visual interactiva (TUI), simulación (dry-run), historial y reversión (undo)."
)]
pub struct Cli {
    #[arg(
        help = "Directorio a organizar (por defecto: directorio actual)",
        default_value = "."
    )]
    pub path: PathBuf,

    #[arg(
        long = "tui",
        help = "Forzar el modo interactivo visual (TUI)",
        default_value_t = false
    )]
    pub tui: bool,

    #[arg(
        long = "cli",
        help = "Forzar el modo línea de comandos (sin interfaz visual)",
        default_value_t = false
    )]
    pub cli: bool,

    #[arg(
        short = 'n',
        long = "dry-run",
        help = "Simular la operación sin mover ningún archivo (vista previa en CLI)"
    )]
    pub dry_run: bool,

    #[arg(
        short = 'u',
        long = "undo",
        help = "Deshacer la última organización realizada en este directorio"
    )]
    pub undo: bool,

    #[arg(
        long = "hidden",
        help = "Incluir también archivos ocultos (que inician con punto)"
    )]
    pub include_hidden: bool,

    #[arg(
        long = "include-unknown",
        help = "Mover archivos con extensiones no reconocidas a una carpeta 'Otros'"
    )]
    pub include_unknown: bool,

    #[arg(
        short = 'v',
        long = "verbose",
        help = "Mostrar detalle de cada archivo individual a mover"
    )]
    pub verbose: bool,

    #[arg(
        long = "init-config",
        help = "Generar un archivo de configuración 'orderfiles.toml' de ejemplo con todas las reglas"
    )]
    pub init_config: bool,
}
