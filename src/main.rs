use anyhow::Result;
use clap::Parser;
use colored::*;
use orderfiles::cli::Cli;
use orderfiles::config::Config;
use orderfiles::organizer::{Organizer, OrganizerOptions};
use orderfiles::tui::TuiApp;
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
        "{}        {}          {}",
        "║".cyan().bold(),
        "📁 OrderFiles (Rust Engine)".bright_white().bold(),
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
        "🔄".cyan(),
        target_dir.display().to_string().yellow()
    );

    match organizer.undo_last(target_dir)? {
        Some(tx) => {
            println!(
                "\n{} Se revirtieron con éxito {} archivo(s):",
                "✅".green(),
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
                    "↩".yellow(),
                    to_dir.green()
                );
            }

            println!("\n{}", "🎉 Directorio restaurado con éxito.".green().bold());
        }
        None => {
            println!(
                "{} No se encontró ningún historial previo para deshacer en este directorio.",
                "ℹ️".yellow()
            );
        }
    }
    Ok(())
}

fn handle_init_config(target_dir: &Path) -> Result<()> {
    let config_path = target_dir.join("orderfiles.toml");
    if config_path.exists() {
        println!(
            "{} Ya existe un archivo de configuración en {}",
            "⚠️".yellow(),
            config_path.display()
        );
        return Ok(());
    }

    let default_config = Config::default();
    default_config.save_to_file(&config_path)?;

    println!(
        "{} Se ha generado la configuración inicial en: {}",
        "✅".green().bold(),
        config_path.display().to_string().cyan()
    );
    println!(
        "\n{} Puedes abrir '{}' con tu editor para:",
        "💡".yellow(),
        config_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
    );
    println!("   - Añadir nuevas extensiones a las categorías existentes.");
    println!("   - Crear nuevas categorías personalizadas (ej. Modelos 3D, Torrents, etc.).");
    println!("   - Cambiar los iconos de cada categoría.");

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
        "🔍".cyan(),
        target.display().to_string().bold().yellow()
    );

    let plan = organizer.generate_plan(target, &options)?;

    if plan.items.is_empty() {
        println!(
            "\n{} No hay archivos sueltos para organizar en este directorio.",
            "✨".green()
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

    println!("\n{}", "📊 Resumen de organización propuesto:".bold());
    println!("{}", "─".repeat(50).dimmed());

    let mut sorted_categories: Vec<_> = plan.summary.iter().collect();
    sorted_categories.sort_by(|a, b| b.1.count.cmp(&a.1.count));

    for (cat_name, summary) in sorted_categories {
        println!(
            "  {} {:<16} : {:>3} archivo(s) ({})",
            summary.icon,
            cat_name.bold(),
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
            "ℹ️".dimmed(),
            plan.skipped_unrecognized
        );
    }

    if args.verbose || args.dry_run {
        println!("\n{}", "📝 Detalle de movimientos:".bold());
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
                "➜".cyan(),
                target_subpath.yellow(),
                format!("({})", format_bytes(item.file_size)).dimmed()
            );
        }
    }

    if args.dry_run {
        println!(
            "\n{}",
            "💡 Modo simulación (--dry-run). No se modificó ningún archivo en el disco."
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
    println!("\n{} Moviendo archivos...", "🚀".cyan().bold());
    let transaction = organizer.execute_plan(&plan)?;

    println!(
        "\n{} ¡Organización completada con éxito! Se movieron {} archivo(s).",
        "🎉".green().bold(),
        transaction.moves.len().to_string().bold().green()
    );

    println!(
        "\n{} Si necesitas revertir esta operación, corre:",
        "💡".yellow()
    );
    println!(
        "   {}",
        format!("orderfiles {} --undo", target.display())
            .bold()
            .cyan()
    );

    Ok(())
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let target = &args.path;

    if args.init_config {
        return handle_init_config(target);
    }

    let config = Config::load_or_default(Some(target));
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
