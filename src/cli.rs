use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "tidyfiles",
    author = "TidyFiles Team",
    version = "0.2.0",
    about = "⚡ Organizador ultrarrápido y seguro de archivos para directorios desordenados en Rust",
    long_about = "TidyFiles clasifica automáticamente archivos por tipo (imágenes, documentos, videos, comprimidos, etc.),\ncon soporte para interfaz visual interactiva (TUI), simulación (dry-run), historial y reversión (undo).\n\n⚙️ SISTEMA DE CONFIGURACIÓN:\n  • Global (Predeterminado): Se almacena en '~/.config/tidyfiles/config.toml'.\n    Cualquier regla o categoría que definas aquí aplicará a todas las carpetas del sistema.\n  • Local (Opcional): Si colocas un archivo 'tidyfiles.toml' dentro de una carpeta específica,\n    tendrá prioridad sobre la configuración global únicamente para ese directorio.\n\nEjemplos:\n  tidyfiles ~/Downloads             # Abre la interfaz visual TUI en Descargas\n  tidyfiles ~/Downloads --dry-run   # Muestra qué se movería sin tocar el disco\n  tidyfiles ~/Downloads --undo      # Revierte la última organización realizada\n  tidyfiles --config-path           # Muestra qué archivo de configuración está activo\n  tidyfiles --edit-config           # Abre tu configuración en tu editor ($EDITOR, code, nano)\n  tidyfiles --init-config           # Crea/muestra la configuración global (~/.config/tidyfiles/config.toml)\n  tidyfiles --init-local            # Crea una plantilla 'tidyfiles.toml' en la carpeta actual"
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
        help = "Generar o consultar el archivo de configuración global (~/.config/tidyfiles/config.toml)"
    )]
    pub init_config: bool,

    #[arg(
        long = "init-local",
        help = "Crear un archivo de configuración local ('tidyfiles.toml') para la carpeta actual"
    )]
    pub init_local: bool,

    #[arg(
        long = "edit-config",
        help = "Abrir directamente el archivo de configuración activo en tu editor de texto"
    )]
    pub edit_config: bool,

    #[arg(
        long = "config-path",
        help = "Mostrar si se está usando la configuración global (~/.config) o una local (tidyfiles.toml)"
    )]
    pub config_path: bool,
}
