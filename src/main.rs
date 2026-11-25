use anyhow::{Context, Result};
use clap::FromArgMatches;
use colored::*;
use tidyfiles::cli::Cli;
use tidyfiles::config::Config;
use tidyfiles::icons::display_icon;
use tidyfiles::organizer::{Organizer, OrganizerOptions};
use tidyfiles::tui::TuiApp;
use std::io::IsTerminal;
use std::path::Path;

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

fn print_banner() {
    println!(
        "{}",
        "╔══════════════════════════════════════════════╗"
            .cyan()
            .bold()
    );
    println!(
        "{}        {}               {}",
        "║".cyan().bold(),
        "TidyFiles (Rust Engine)".bright_white().bold(),
        "║".cyan().bold()
    );
    println!(
        "{}",
        "╚══════════════════════════════════════════════╝"
            .cyan()
            .bold()
    );
}

fn handle_undo(target_dir: &Path, organizer: &Organizer) -> Result<()> {
    println!(
        "{} Buscando historial de cambios en {}...",
        "[UNDO]".cyan(),
        target_dir.display().to_string().yellow()
    );

    match organizer.undo_last(target_dir)? {
        Some(tx) => {
            println!(
                "\n{} Se revirtieron con éxito {} archivo(s):",
                "[OK]".green(),
                tx.moves.len().to_string().bold().green()
            );

            for (i, mv) in tx.moves.iter().enumerate() {
                let from_name = mv
                    .current_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy();
                let to_dir = mv
                    .original_path
                    .parent()
                    .unwrap_or_else(|| Path::new("."))
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy();

                println!(
                    "   {:>3}. {} {} {}",
                    i + 1,
                    from_name.cyan(),
                    "->".yellow(),
                    to_dir.green()
                );
            }

            println!(
                "\n{}",
                "[OK] Directorio restaurado con éxito.".green().bold()
            );
        }
        None => {
            println!(
                "{} No se encontró ningún historial previo para deshacer en este directorio.",
                "[INFO]".yellow()
            );
        }
    }
    Ok(())
}

fn handle_init_config() -> Result<()> {
    let global_path = Config::global_config_path()
        .context("No se pudo determinar el directorio HOME del usuario")?;

    if global_path.exists() {
        println!(
            "{} Tu archivo de configuración global está en:",
            "[INFO]".cyan().bold()
        );
        println!("   {}", global_path.display().to_string().cyan().bold());
        println!(
            "\n{} Para abrirlo y editarlo ahora mismo, corre:",
            "[TIP]".yellow()
        );
        println!("   tidyfiles --edit-config");
        println!("   # o manualmente: code {}", global_path.display());
        return Ok(());
    }

    let default_config = Config::default();
    default_config.save_to_file(&global_path)?;

    println!(
        "{} Configuración global generada exitosamente en:",
        "[OK]".green().bold()
    );
    println!("   {}", global_path.display().to_string().cyan().bold());
    println!(
        "\n{} Todas las reglas que definas aquí aplicarán automáticamente a todas tus carpetas.",
        "[INFO]".green()
    );
    println!("   - Añade extensiones a las categorías existentes (ej. 'ipynb' en Codigo).");
    println!("   - Crea tus propias categorías (ej. Modelos3D, Torrents, Libros).");
    println!(
        "\n{} Puedes editarlo ejecutando: 'tidyfiles --edit-config'",
        "[TIP]".yellow()
    );

    Ok(())
}

fn handle_init_local(target_dir: &Path) -> Result<()> {
    let local_path = target_dir.join("tidyfiles.toml");
    if local_path.exists() {
        println!(
            "{} Ya existe una configuración local en esta carpeta:",
            "[WARN]".yellow().bold()
        );
        println!("   {}", local_path.display().to_string().yellow());
        return Ok(());
    }

    let default_config = Config::default();
    default_config.save_to_file(&local_path)?;

    println!(
        "{} Configuración local creada con éxito en:",
        "[OK]".green().bold()
    );
    println!("   {}", local_path.display().to_string().cyan().bold());
    println!(
        "\n{} Esta configuración sobrescribirá la global únicamente cuando organices esta carpeta.",
        "[INFO]".cyan()
    );

    Ok(())
}

fn handle_edit_config(target_dir: &Path) -> Result<()> {
    let local_path = ["tidyfiles.toml", ".tidyfiles.toml"]
        .iter()
        .map(|filename| target_dir.join(filename))
        .find(|path| path.exists());

    let path_to_edit = match local_path {
        Some(path) => path,
        None => {
            let global =
                Config::global_config_path().context("No se pudo obtener la ruta global")?;
            if !global.exists() {
                Config::default().save_to_file(&global)?;
            }
            global
        }
    };

    println!(
        "{} Abriendo archivo de configuración: {}",
        "[EDIT]".cyan(),
        path_to_edit.display().to_string().bold().yellow()
    );

    let editor = std::env::var("EDITOR").unwrap_or_else(|_| {
        if which_command_exists("code") {
            "code".to_string()
        } else if which_command_exists("nano") {
            "nano".to_string()
        } else if which_command_exists("vim") {
            "vim".to_string()
        } else {
            "xdg-open".to_string()
        }
    });

    let status = std::process::Command::new(&editor)
        .arg(&path_to_edit)
        .status()
        .with_context(|| format!("No se pudo ejecutar el editor '{}'", editor))?;

    if !status.success() {
        println!(
            "{} El editor finalizó con código: {}",
            "[WARN]".yellow(),
            status
        );
    }

    Ok(())
}

fn which_command_exists(cmd: &str) -> bool {
    std::process::Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn handle_show_config_path(target_dir: &Path) -> Result<()> {
    let resolution = Config::resolve(Some(target_dir))?;
    println!("\n{}", "CONFIGURACION ACTIVA DE TIDYFILES".bold());
    println!("{}", "─".repeat(50).dimmed());

    match resolution.source_path {
        Some(path) => {
            let is_global = Config::global_config_path().map_or(false, |g| g == path);
            let tipo = if is_global {
                "Global (Aplica a todo el sistema)".green()
            } else {
                "Local (Específico para esta carpeta)".yellow()
            };

            println!("  Archivo : {}", path.display().to_string().cyan().bold());
            println!("  Ámbito  : {}", tipo);
            println!(
                "  Reglas  : {} categorías cargadas",
                resolution.config.categories.len()
            );
        }
        None => {
            println!(
                "  Archivo : {}",
                "Ninguno (usando reglas de fábrica)".dimmed()
            );
            println!(
                "  Reglas  : {} categorías predeterminadas",
                resolution.config.categories.len()
            );
        }
    }
    println!("{}", "─".repeat(50).dimmed());
    Ok(())
}

fn run_cli_mode(args: &Cli, organizer: &Organizer) -> Result<()> {
    let target = &args.path;
    print_banner();

    let options = OrganizerOptions {
        include_hidden: args.include_hidden,
        include_unknown: args.include_unknown,
        dry_run: args.dry_run,
    };

    println!(
        "{} Analizando directorio: {}",
        "[SCAN]".cyan(),
        target.display().to_string().bold().yellow()
    );

    let plan = organizer.generate_plan(target, &options)?;

    if plan.items.is_empty() {
        println!(
            "\n{} No hay archivos sueltos para organizar en este directorio.",
            "[INFO]".green()
        );
        if plan.skipped_dirs > 0 || plan.skipped_unrecognized > 0 {
            println!(
                "   ({} carpetas ya existentes, {} archivos no reconocidos ignorados)",
                plan.skipped_dirs, plan.skipped_unrecognized
            );
        }
        return Ok(());
    }

    let total_bytes: u64 = plan.items.iter().map(|item| item.file_size).sum();

    println!("\n{}", "RESUMEN DE ORGANIZACION PROPUESTO".bold());
    println!("{}", "─".repeat(50).dimmed());

    let mut sorted_categories: Vec<_> = plan.summary.iter().collect();
    sorted_categories.sort_by(|a, b| b.1.count.cmp(&a.1.count));

    for (cat_name, summary) in sorted_categories {
        println!(
            "  {} {} : {:>3} archivo(s) ({})",
            display_icon(&summary.icon, cat_name),
            format!("{:<16}", cat_name).bold(),
            summary.count.to_string().cyan().bold(),
            format_bytes(summary.total_bytes).dimmed()
        );
    }

    println!("{}", "─".repeat(50).dimmed());
    println!(
        "  Total: {} archivo(s) | Espacio: {}",
        plan.items.len().to_string().bold().green(),
        format_bytes(total_bytes).bold().yellow()
    );

    if plan.skipped_unrecognized > 0 {
        println!(
            "  {} {} archivo(s) con extensiones no reconocidas fueron omitidos (usa --include-unknown si deseas moverlos a 'Otros')",
            "[INFO]".dimmed(),
            plan.skipped_unrecognized
        );
    }

    if args.verbose || args.dry_run {
        println!("\n{}", "DETALLE DE MOVIMIENTOS".bold());
        for (i, item) in plan.items.iter().enumerate() {
            let src_name = item
                .source
                .file_name()
                .unwrap_or_default()
                .to_string_lossy();
            let dest_name = item
                .destination
                .file_name()
                .unwrap_or_default()
                .to_string_lossy();

            let target_subpath = format!("{}/{}", item.category_name, dest_name);

            println!(
                "   {:>3}. {} {} {} {}",
                i + 1,
                src_name.bright_white(),
                "->".cyan(),
                target_subpath.yellow(),
                format!("({})", format_bytes(item.file_size)).dimmed()
            );
        }
    }

    if args.dry_run {
        println!(
            "\n{}",
            "[DRY-RUN] No se modificó ningún archivo en el disco."
                .yellow()
                .bold()
        );
        println!(
            "   Para ejecutar los movimientos reales, corre el mismo comando sin '{}'.",
            "--dry-run".bold()
        );
        return Ok(());
    }

    // Ejecutar movimientos reales
    println!("\n{} Moviendo archivos...", "[MOVE]".cyan().bold());
    let transaction = organizer.execute_plan(&plan)?;

    println!(
        "\n{} Organización completada con éxito. Se movieron {} archivo(s).",
        "[OK]".green().bold(),
        transaction.moves.len().to_string().bold().green()
    );

    println!(
        "\n{} Si necesitas revertir esta operación, corre:",
        "[TIP]".yellow()
    );
    println!(
        "   {}",
        format!("tidyfiles {} --undo", target.display())
            .bold()
            .cyan()
    );

    Ok(())
}

fn main() -> Result<()> {
    let matches = tidyfiles::cli::build_command().get_matches();
    let args = match Cli::from_arg_matches(&matches) {
        Ok(cli) => cli,
        Err(err) => err.exit(),
    };
    let target = &args.path;

    if args.init_config {
        return handle_init_config();
    }

    if args.init_local {
        return handle_init_local(target);
    }

    if args.edit_config {
        return handle_edit_config(target);
    }

    if args.config_path {
        return handle_show_config_path(target);
    }

    let config = Config::load_or_default(Some(target))?;
    let organizer = Organizer::with_config(&config);

    if args.undo {
        print_banner();
        return handle_undo(target, &organizer);
    }

    let is_terminal = std::io::stdout().is_terminal() && std::io::stdin().is_terminal();
    let should_launch_tui = (args.tui || is_terminal) && !args.cli && !args.dry_run;

    if should_launch_tui {
        let mut app = TuiApp::new(target)?;
        app.run()?;
        Ok(())
    } else {
        run_cli_mode(&args, &organizer)
    }
}
