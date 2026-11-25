use clap::{ArgAction, Command, CommandFactory, Parser};
use std::path::PathBuf;

/// Construye el comando de Clap con las secciones de ayuda en español.
pub fn build_command() -> Command {
    <Cli as CommandFactory>::command()
}

#[derive(Parser, Debug)]
#[command(
    name = "tidyfiles",
    author = "TidyFiles Team",
    version,
    about = "Organizador ultrarrápido y seguro de archivos para directorios desordenados en Rust",
    long_about = "TidyFiles clasifica automáticamente archivos por tipo (imágenes, documentos, videos, comprimidos, etc.),\ncon soporte para interfaz visual interactiva (TUI), simulación (dry-run), historial y reversión (undo).\n\nSISTEMA DE CONFIGURACIÓN:\n  - Global (Predeterminado): Se almacena en '~/.config/tidyfiles/config.toml'.\n    Cualquier regla o categoría que definas aquí aplicará a todas las carpetas del sistema.\n  - Local (Opcional): Si colocas un archivo 'tidyfiles.toml' dentro de una carpeta específica,\n    tendrá prioridad sobre la configuración global únicamente para ese directorio.\n\nEjemplos:\n  tidyfiles ~/Downloads             # Abre la interfaz visual TUI en Descargas\n  tidyfiles ~/Downloads --dry-run   # Muestra qué se movería sin tocar el disco\n  tidyfiles ~/Downloads --undo      # Revierte la última organización realizada\n  tidyfiles --config-path           # Muestra qué archivo de configuración está activo\n  tidyfiles --edit-config           # Abre tu configuración en tu editor ($EDITOR, code, nano)\n  tidyfiles --init-config           # Crea/muestra la configuración global (~/.config/tidyfiles/config.toml)\n  tidyfiles --init-local            # Crea una plantilla 'tidyfiles.toml' en la carpeta actual",
    help_template = "{before-help}{about-with-newline}\nUSO: {usage}\n\nARGUMENTOS:\n{positionals}\n\nOPCIONES:\n{options}{after-help}",
    disable_help_flag = true,
    disable_version_flag = true
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

    #[arg(
        short = 'h',
        long = "help",
        action = ArgAction::Help,
        help = "Muestra esta ayuda"
    )]
    pub help: Option<bool>,

    #[arg(
        short = 'V',
        long = "version",
        action = ArgAction::Version,
        help = "Muestra la versión"
    )]
    pub version: Option<bool>,
}

#[cfg(test)]
mod tests {
    fn forbidden_symbols(text: &str) -> String {
        text.chars()
            .filter(|c| {
                let code = *c as u32;
                (0x1F000..=0x1FAFF).contains(&code) // emojis
                    || (0x2600..=0x27BF).contains(&code) // símbolos y dingbats
                    || (0x2B00..=0x2BFF).contains(&code) // flechas decorativas
                    || (0x2190..=0x21FF).contains(&code) // flechas
                    || code == 0x2022 // viñeta
                    || code == 0xFE0F // selector de variación emoji
            })
            .collect()
    }

    #[test]
    fn help_has_no_emoji_or_decorative_symbols() {
        let short = super::build_command().render_help().to_string();
        let long = super::build_command().render_long_help().to_string();

        for (name, text) in [("corta", short), ("larga", long)] {
            let found = forbidden_symbols(&text);
            assert!(found.is_empty(), "la ayuda {name} contiene: {found:?}");
        }
    }
}
