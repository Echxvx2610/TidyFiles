use crate::config::Config;
use crate::organizer::{Organizer, OrganizerOptions, PlanItem};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug, PartialEq, Eq)]
enum ActiveView {
    FilesList,
    CategoriesHelp,
}

pub struct TuiApp {
    target_dir: PathBuf,
    organizer: Organizer,
    config: Config,
    items: Vec<PlanItem>,
    selected_indices: HashSet<usize>,
    cursor: usize,
    categories_scroll: usize,
    status_message: String,
    active_view: ActiveView,
    should_quit: bool,
}

impl TuiApp {
    pub fn new(target_dir: &Path) -> Result<Self> {
        let abs_target = target_dir.canonicalize()?;
        let config = Config::load_or_default(Some(&abs_target));
        let organizer = Organizer::with_config(&config);

        let options = OrganizerOptions {
            include_hidden: false,
            include_unknown: false,
            dry_run: false,
        };

        let plan = organizer.generate_plan(&abs_target, &options)?;
        let items_len = plan.items.len();
        let selected_indices: HashSet<usize> = (0..items_len).collect();

        let status_message = if items_len == 0 {
            "No se encontraron archivos sueltos para organizar.".to_string()
        } else {
            format!(
                "Se detectaron {} archivo(s) listos para ordenar.",
                items_len
            )
        };

        Ok(Self {
            target_dir: abs_target,
            organizer,
            config,
            items: plan.items,
            selected_indices,
            cursor: 0,
            categories_scroll: 0,
            status_message,
            active_view: ActiveView::FilesList,
            should_quit: false,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        let mut terminal = ratatui::init();

        let run_result = self.main_loop(&mut terminal);

        ratatui::restore();
        run_result
    }

    fn main_loop(&mut self, terminal: &mut ratatui::DefaultTerminal) -> Result<()> {
        while !self.should_quit {
            terminal.draw(|frame| self.render_ui(frame))?;

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        self.handle_key(key.code)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, code: KeyCode) -> Result<()> {
        match self.active_view {
            ActiveView::CategoriesHelp => match code {
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('c') | KeyCode::Enter => {
                    self.active_view = ActiveView::FilesList;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.categories_scroll > 0 {
                        self.categories_scroll -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.categories_scroll += 1;
                }
                KeyCode::PageUp => {
                    self.categories_scroll = self.categories_scroll.saturating_sub(5);
                }
                KeyCode::PageDown => {
                    self.categories_scroll += 5;
                }
                _ => {}
            },
            ActiveView::FilesList => match code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    self.should_quit = true;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.cursor > 0 {
                        self.cursor -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if !self.items.is_empty() && self.cursor + 1 < self.items.len() {
                        self.cursor += 1;
                    }
                }
                KeyCode::Char(' ') => {
                    if !self.items.is_empty() {
                        if self.selected_indices.contains(&self.cursor) {
                            self.selected_indices.remove(&self.cursor);
                        } else {
                            self.selected_indices.insert(self.cursor);
                        }
                    }
                }
                KeyCode::Char('a') => {
                    if self.selected_indices.len() == self.items.len() {
                        self.selected_indices.clear();
                        self.status_message = "Todos los archivos deseleccionados.".to_string();
                    } else {
                        self.selected_indices = (0..self.items.len()).collect();
                        self.status_message = "Todos los archivos seleccionados.".to_string();
                    }
                }
                KeyCode::Enter | KeyCode::Char('o') => {
                    self.execute_organization()?;
                }
                KeyCode::Char('u') => {
                    self.execute_undo()?;
                }
                KeyCode::Char('c') => {
                    self.categories_scroll = 0;
                    self.active_view = ActiveView::CategoriesHelp;
                }
                KeyCode::Char('r') => {
                    self.refresh_plan()?;
                    self.status_message = "Directorio recargado.".to_string();
                }
                _ => {}
            },
        }
        Ok(())
    }

    fn execute_organization(&mut self) -> Result<()> {
        if self.items.is_empty() {
            self.status_message = "No hay archivos para organizar.".to_string();
            return Ok(());
        }

        let selected_items: Vec<PlanItem> = self
            .items
            .iter()
            .enumerate()
            .filter(|(idx, _)| self.selected_indices.contains(idx))
            .map(|(_, item)| item.clone())
            .collect();

        if selected_items.is_empty() {
            self.status_message =
                "Selecciona al menos un archivo con [Espacio] antes de organizar.".to_string();
            return Ok(());
        }

        let total = selected_items.len();
        match self
            .organizer
            .execute_items(&self.target_dir, &selected_items)
        {
            Ok(_) => {
                self.status_message = format!("🎉 ¡Éxito! Se movieron {} archivo(s).", total);
                self.refresh_plan()?;
            }
            Err(e) => {
                self.status_message = format!("❌ Error: {}", e);
            }
        }

        Ok(())
    }

    fn execute_undo(&mut self) -> Result<()> {
        match self.organizer.undo_last(&self.target_dir) {
            Ok(Some(tx)) => {
                self.status_message = format!(
                    "🔄 Reversión exitosa: {} archivo(s) restaurados.",
                    tx.moves.len()
                );
                self.refresh_plan()?;
            }
            Ok(None) => {
                self.status_message = "No hay historial previo para revertir.".to_string();
            }
            Err(e) => {
                self.status_message = format!("❌ Error al deshacer: {}", e);
            }
        }
        Ok(())
    }

    fn refresh_plan(&mut self) -> Result<()> {
        let options = OrganizerOptions {
            include_hidden: false,
            include_unknown: false,
            dry_run: false,
        };
        let plan = self.organizer.generate_plan(&self.target_dir, &options)?;
        self.items = plan.items;
        self.selected_indices = (0..self.items.len()).collect();
        if self.cursor >= self.items.len() && !self.items.is_empty() {
            self.cursor = self.items.len() - 1;
        }
        Ok(())
    }

    fn render_ui(&self, frame: &mut Frame) {
        let size = frame.area();

        // Layout vertical principal: Encabezado (3), Cuerpo (Min 10), Barra de estado / ayuda (4)
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(8),
                Constraint::Length(4),
            ])
            .split(size);

        self.render_header(frame, main_chunks[0]);
        self.render_body(frame, main_chunks[1]);
        self.render_footer(frame, main_chunks[2]);

        if self.active_view == ActiveView::CategoriesHelp {
            self.render_categories_modal(frame, size);
        }
    }

    fn render_header(&self, frame: &mut Frame, area: Rect) {
        let title_spans = vec![
            Span::styled(
                " ⚡ TidyFiles ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("│ "),
            Span::styled(
                self.target_dir.display().to_string(),
                Style::default().fg(Color::Yellow),
            ),
        ];

        let header = Paragraph::new(Line::from(title_spans)).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan)),
        );

        frame.render_widget(header, area);
    }

    fn render_body(&self, frame: &mut Frame, area: Rect) {
        // Dividir cuerpo en dos columnas: Categorías (32%), Lista de archivos (68%)
        let body_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(32), Constraint::Percentage(68)])
            .split(area);

        self.render_categories_panel(frame, body_chunks[0]);
        self.render_files_panel(frame, body_chunks[1]);
    }

    fn render_categories_panel(&self, frame: &mut Frame, area: Rect) {
        let mut category_counts: std::collections::BTreeMap<&str, (usize, &str)> =
            std::collections::BTreeMap::new();

        for item in &self.items {
            let entry = category_counts
                .entry(&item.category_name)
                .or_insert((0, &item.category_icon));
            entry.0 += 1;
        }

        let mut lines = Vec::new();
        lines.push(Line::from(vec![
            Span::styled(
                "Categoría          ",
                Style::default().add_modifier(Modifier::UNDERLINED),
            ),
            Span::styled(
                "Archivos",
                Style::default().add_modifier(Modifier::UNDERLINED),
            ),
        ]));

        if category_counts.is_empty() {
            lines.push(Line::from(" (Ningún archivo detectado)"));
        } else {
            for (cat_name, (count, icon)) in &category_counts {
                lines.push(Line::from(vec![
                    Span::raw(format!("{} {:<16}", icon, cat_name)),
                    Span::styled(
                        format!("{:>4}", count),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]));
            }
        }

        let block = Block::default()
            .title(" 📊 Resumen por Tipos ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Blue));

        let paragraph = Paragraph::new(lines).block(block);
        frame.render_widget(paragraph, area);
    }

    fn render_files_panel(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(format!(
                " 📁 Archivos detectados ({}/{}) ",
                self.selected_indices.len(),
                self.items.len()
            ))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan));

        if self.items.is_empty() {
            let empty_text = Paragraph::new(
                "\n  ✨ No hay archivos para organizar en esta carpeta.\n\n  Presiona [r] para recargar o [u] para deshacer la última operación.",
            )
            .block(block);
            frame.render_widget(empty_text, area);
            return;
        }

        // Calcular scrolling simple
        let inner_height = area.height.saturating_sub(2) as usize;
        let start_index = if self.cursor >= inner_height {
            self.cursor - inner_height + 1
        } else {
            0
        };
        let visible_items = self
            .items
            .iter()
            .enumerate()
            .skip(start_index)
            .take(inner_height);

        let list_items: Vec<ListItem> = visible_items
            .map(|(i, item)| {
                let is_selected = self.selected_indices.contains(&i);
                let is_cursor = i == self.cursor;

                let checkbox = if is_selected {
                    Span::styled(
                        "[✓] ",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    Span::styled("[ ] ", Style::default().fg(Color::DarkGray))
                };

                let filename = item
                    .source
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy();
                let dest_folder = format!("{}/", item.category_name);

                let line_spans = vec![
                    checkbox,
                    Span::raw(format!("{} ", item.category_icon)),
                    Span::styled(
                        format!("{:<28} ", filename),
                        if is_cursor {
                            Style::default()
                                .fg(Color::Black)
                                .bg(Color::Cyan)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::White)
                        },
                    ),
                    Span::styled("➜ ", Style::default().fg(Color::DarkGray)),
                    Span::styled(dest_folder, Style::default().fg(Color::Yellow)),
                ];

                ListItem::new(Line::from(line_spans))
            })
            .collect();

        let list = List::new(list_items).block(block);
        frame.render_widget(list, area);
    }

    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let footer_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(3)])
            .split(area);

        // Mensaje de estado dinámico
        let status = Paragraph::new(Span::styled(
            format!(" {}", self.status_message),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
        frame.render_widget(status, footer_chunks[0]);

        // Atajos de teclado
        let shortcuts = Paragraph::new(Line::from(vec![
            Span::styled(" [↑/↓/j/k] ", Style::default().fg(Color::Cyan)),
            Span::raw("Navegar │"),
            Span::styled(" [Espacio] ", Style::default().fg(Color::Cyan)),
            Span::raw("Seleccionar │"),
            Span::styled(" [a] ", Style::default().fg(Color::Cyan)),
            Span::raw("Todos │"),
            Span::styled(
                " [o/Enter] ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Organizar │"),
            Span::styled(" [u] ", Style::default().fg(Color::Yellow)),
            Span::raw("Deshacer │"),
            Span::styled(" [c] ", Style::default().fg(Color::Magenta)),
            Span::raw("Formatos │"),
            Span::styled(" [q/Esc] ", Style::default().fg(Color::Red)),
            Span::raw("Salir"),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray)),
        );

        frame.render_widget(shortcuts, footer_chunks[1]);
    }

    fn render_categories_modal(&self, frame: &mut Frame, area: Rect) {
        // Modal amplio, centrado y proporcionado
        let popup_width = (area.width * 85 / 100).clamp(70, 115).min(area.width);
        let popup_height = (area.height * 80 / 100).clamp(18, 28).min(area.height);

        let popup_x = (area.width.saturating_sub(popup_width)) / 2;
        let popup_y = (area.height.saturating_sub(popup_height)) / 2;

        let popup_rect = Rect::new(popup_x, popup_y, popup_width, popup_height);

        frame.render_widget(Clear, popup_rect);

        let outer_block = Block::default()
            .title(" ⚙️ Formatos y Categorías Configuradas ")
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(Color::Magenta));

        let inner_area = outer_block.inner(popup_rect);
        frame.render_widget(outer_block, popup_rect);

        // División vertical interna: Cabecera columnas (2 líneas), Lista de dos columnas (Min 6), Footer (4 líneas)
        let modal_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Min(6),
                Constraint::Length(4),
            ])
            .split(inner_area);

        // Cabecera clara de columnas
        let header_p = Paragraph::new(vec![Line::from(vec![
            Span::styled(
                " Categoría        ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            ),
            Span::styled("│ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "Formatos y Extensiones Soportadas",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            ),
            Span::styled(
                format!(" ({} tipos)", self.config.categories.len()),
                Style::default().fg(Color::DarkGray),
            ),
        ])]);
        frame.render_widget(header_p, modal_chunks[0]);

        // Cálculo exacto del ancho de la columna derecha para envolver sin desbordar
        // Columna izquierda: 18 caracteres. Separador "│ ": 2 caracteres.
        let right_col_width = (inner_area.width.saturating_sub(21) as usize).max(20);

        let mut lines = Vec::new();
        for (name, cat) in &self.config.categories {
            // Dividir las extensiones en líneas que quepan en right_col_width
            let mut ext_lines: Vec<String> = Vec::new();
            let mut current_line = String::new();

            for (i, ext) in cat.extensions.iter().enumerate() {
                let token = if i == 0 {
                    ext.clone()
                } else {
                    format!(", {}", ext)
                };

                if !current_line.is_empty() && current_line.len() + token.len() > right_col_width {
                    ext_lines.push(current_line);
                    current_line = ext.clone();
                } else {
                    current_line.push_str(&token);
                }
            }
            if !current_line.is_empty() {
                ext_lines.push(current_line);
            }

            // Renderizar cada línea: la primera con el nombre; las siguientes con sangría en la columna derecha
            for (line_idx, ext_line) in ext_lines.into_iter().enumerate() {
                if line_idx == 0 {
                    lines.push(Line::from(vec![
                        Span::styled(
                            format!(" {} {:<14}", cat.icon, name),
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                        Span::styled(ext_line, Style::default().fg(Color::White)),
                    ]));
                } else {
                    lines.push(Line::from(vec![
                        Span::raw("                  "), // 18 espacios exactos: nunca invade la columna de nombre
                        Span::styled("│ ", Style::default().fg(Color::DarkGray)),
                        Span::styled(ext_line, Style::default().fg(Color::White)),
                    ]));
                }
            }
        }

        let cat_paragraph = Paragraph::new(lines).scroll((self.categories_scroll as u16, 0));
        frame.render_widget(cat_paragraph, modal_chunks[1]);

        // Caja de feedback y ayuda protegida contra desbordamientos con Wrap
        let footer_block = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::DarkGray));

        let footer_lines = vec![
            Line::from(vec![
                Span::styled(
                    "💡 Reglas: ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("edita "),
                Span::styled(
                    "~/.config/tidyfiles/config.toml",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" (aplica a todo el sistema)."),
            ]),
            Line::from(vec![
                Span::styled("   Atajos: ", Style::default().fg(Color::DarkGray)),
                Span::styled("[Esc] / [q] / [c] ", Style::default().fg(Color::Red)),
                Span::raw("Cerrar  │  "),
                Span::styled("[↑/↓/j/k] ", Style::default().fg(Color::Cyan)),
                Span::raw("Desplazar"),
            ]),
        ];

        let footer_p = Paragraph::new(footer_lines)
            .block(footer_block)
            .wrap(Wrap { trim: true });
        frame.render_widget(footer_p, modal_chunks[2]);
    }
}
