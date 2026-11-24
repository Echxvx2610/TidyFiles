use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "orderfiles",
    author = "OrderFiles Team",
    version = "0.2.0",
    about = "⚡ Organizador ultrarrápido y seguro de archivos para directorios desordenados en Rust",
    long_about = "OrderFiles clasifica automáticamente archivos por tipo (imágenes, documentos, videos, comprimidos, etc.),\ncon soporte para interfaz visual interactiva (TUI), simulación (dry-run), historial y reversión (undo).\n\n⚙️ SISTEMA DE CONFIGURACIÓN:\n  • Global (Predeterminado): Se almacena en '~/.config/orderfiles/config.toml'.\n    Cualquier regla o categoría que definas aquí aplicará a todas las carpetas del sistema.\n  • Local (Opcional): Si colocas un archivo 'orderfiles.toml' dentro de una carpeta específica,\n    tendrá prioridad sobre la configuración global únicamente para ese directorio.\n\nEjemplos:\n  orderfiles ~/Downloads             # Abre la interfaz visual TUI en Descargas\n  orderfiles ~/Downloads --dry-run   # Muestra qué se movería sin tocar el disco\n  orderfiles ~/Downloads --undo      # Revierte la última organización realizada\n  orderfiles --config-path           # Muestra qué configuración se está utilizando\n  orderfiles --init-config           # Crea/abre el archivo global ~/.config/orderfiles/config.toml"
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
        help = "Generar o consultar el archivo de configuración global (~/.config/orderfiles/config.toml)"
    )]
    pub init_config: bool,

    #[arg(
        long = "config-path",
        help = "Mostrar si se está usando la configuración global (~/.config) o una local (orderfiles.toml)"
    )]
    pub config_path: bool,
}
