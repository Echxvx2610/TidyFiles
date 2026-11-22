use crate::categories::CategoryRegistry;
use crate::config::Config;
use crate::history::{HISTORY_FILE_NAME, HistoryManager, MoveRecord, Transaction};
use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct PlanItem {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub category_name: String,
    pub category_icon: String,
    pub file_size: u64,
}

#[derive(Debug, Default, Clone)]
pub struct CategorySummary {
    pub count: usize,
    pub total_bytes: u64,
    pub icon: String,
}

#[derive(Debug, Clone)]
pub struct OrganizationPlan {
    pub target_dir: PathBuf,
    pub items: Vec<PlanItem>,
    pub skipped_unrecognized: usize,
    pub skipped_dirs: usize,
    #[allow(dead_code)]
    pub skipped_hidden: usize,
    pub summary: HashMap<String, CategorySummary>,
}

#[derive(Debug, Clone)]
pub struct OrganizerOptions {
    pub include_hidden: bool,
    pub include_unknown: bool,
    #[allow(dead_code)]
    pub dry_run: bool,
}

pub struct Organizer {
    registry: CategoryRegistry,
}

impl Organizer {
    pub fn new() -> Self {
        Self {
            registry: CategoryRegistry::default_rules(),
        }
    }

    pub fn with_config(config: &Config) -> Self {
        Self {
            registry: CategoryRegistry::from_config(config),
        }
    }

    pub fn with_registry(registry: CategoryRegistry) -> Self {
        Self { registry }
    }

    pub fn registry(&self) -> &CategoryRegistry {
        &self.registry
    }

    pub fn registry_mut(&mut self) -> &mut CategoryRegistry {
        &mut self.registry
    }

    pub fn generate_plan(
        &self,
        target_dir: &Path,
        options: &OrganizerOptions,
    ) -> Result<OrganizationPlan> {
        let abs_target = target_dir.canonicalize().with_context(|| {
            format!("No se pudo acceder al directorio: {}", target_dir.display())
        })?;

        let mut items = Vec::new();
        let mut skipped_unrecognized = 0;
        let mut skipped_dirs = 0;
        let mut skipped_hidden = 0;
        let mut planned_destinations = HashSet::new();

        let entries = fs::read_dir(&abs_target)
            .with_context(|| format!("Error al leer directorio {}", abs_target.display()))?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            let file_name_os = entry.file_name();
            let file_name = file_name_os.to_string_lossy();

            // Ignorar el archivo de historial de orderfiles
            if file_name == HISTORY_FILE_NAME {
                continue;
            }

            // Ignorar archivos ocultos a menos que se indique
            if file_name.starts_with('.') {
                if !options.include_hidden {
                    skipped_hidden += 1;
                    continue;
                }
            }

            let file_type = entry.file_type()?;
            if file_type.is_dir() {
                // No movemos directorios de primer nivel
                skipped_dirs += 1;
                continue;
            }

            let file_size = entry.metadata().map(|m| m.len()).unwrap_or(0);

            let matched_category = self.registry.get_category_for_path(&path);

            let (cat_name, cat_icon) = match matched_category {
                Some(cat) => (cat.name.clone(), cat.icon.clone()),
                None => {
                    if options.include_unknown {
                        ("Otros".to_string(), "📁".to_string())
                    } else {
                        skipped_unrecognized += 1;
                        continue;
                    }
                }
            };

            let dest_dir = abs_target.join(&cat_name);
            let unique_dest = self.resolve_collision(&dest_dir, &file_name, &planned_destinations);
            planned_destinations.insert(unique_dest.clone());

            items.push(PlanItem {
                source: path,
                destination: unique_dest,
                category_name: cat_name,
                category_icon: cat_icon,
                file_size,
            });
        }

        let mut summary: HashMap<String, CategorySummary> = HashMap::new();
        for item in &items {
            let entry = summary
                .entry(item.category_name.clone())
                .or_insert_with(|| CategorySummary {
                    count: 0,
                    total_bytes: 0,
                    icon: item.category_icon.clone(),
                });
            entry.count += 1;
            entry.total_bytes += item.file_size;
        }

        Ok(OrganizationPlan {
            target_dir: abs_target,
            items,
            skipped_unrecognized,
            skipped_dirs,
            skipped_hidden,
            summary,
        })
    }

    fn resolve_collision(
        &self,
        dest_dir: &Path,
        file_name: &str,
        planned_destinations: &HashSet<PathBuf>,
    ) -> PathBuf {
        let initial_dest = dest_dir.join(file_name);
        if !initial_dest.exists() && !planned_destinations.contains(&initial_dest) {
            return initial_dest;
        }

        let path = Path::new(file_name);
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(file_name);
        let extension = path.extension().and_then(|e| e.to_str());

        let mut counter = 1;
        loop {
            let candidate_name = match extension {
                Some(ext) => format!("{} ({}).{}", stem, counter, ext),
                None => format!("{} ({})", stem, counter),
            };

            let candidate_path = dest_dir.join(candidate_name);
            if !candidate_path.exists() && !planned_destinations.contains(&candidate_path) {
                return candidate_path;
            }
            counter += 1;
        }
    }

    pub fn execute_plan(&self, plan: &OrganizationPlan) -> Result<Transaction> {
        self.execute_items(&plan.target_dir, &plan.items)
    }

    pub fn execute_items(&self, target_dir: &Path, items: &[PlanItem]) -> Result<Transaction> {
        let mut created_dirs = Vec::new();
        let mut created_dirs_set = HashSet::new();
        let mut executed_moves = Vec::new();

        for item in items {
            let parent_dir = item
                .destination
                .parent()
                .context("Destino no tiene directorio padre")?;

            if !parent_dir.exists() && !created_dirs_set.contains(parent_dir) {
                fs::create_dir_all(parent_dir).with_context(|| {
                    format!("No se pudo crear carpeta {}", parent_dir.display())
                })?;
                created_dirs.push(parent_dir.to_path_buf());
                created_dirs_set.insert(parent_dir.to_path_buf());
            }

            // Mover archivo (con fallback seguro en caso de particiones cruzadas)
            self.move_file_safely(&item.source, &item.destination)?;

            executed_moves.push(MoveRecord {
                original_path: item.source.clone(),
                current_path: item.destination.clone(),
            });
        }

        let transaction = HistoryManager::new_transaction(target_dir, created_dirs, executed_moves);

        if !transaction.moves.is_empty() {
            HistoryManager::save_transaction(target_dir, &transaction)?;
        }

        Ok(transaction)
    }

    fn move_file_safely(&self, from: &Path, to: &Path) -> Result<()> {
        if let Err(_rename_err) = fs::rename(from, to) {
            // Si rename falla (ej. entre sistemas de archivos diferentes), copiar y borrar origen
            fs::copy(from, to).with_context(|| {
                format!("Error al copiar de {} a {}", from.display(), to.display())
            })?;
            fs::remove_file(from)
                .with_context(|| format!("Error al borrar archivo original {}", from.display()))?;
        }
        Ok(())
    }

    pub fn undo_last(&self, target_dir: &Path) -> Result<Option<Transaction>> {
        let abs_target = target_dir.canonicalize()?;
        let last_tx = match HistoryManager::pop_last_transaction(&abs_target)? {
            Some(tx) => tx,
            None => return Ok(None),
        };

        // Revertir movimientos en orden inverso
        for record in last_tx.moves.iter().rev() {
            if record.current_path.exists() {
                if let Some(parent) = record.original_path.parent() {
                    if !parent.exists() {
                        let _ = fs::create_dir_all(parent);
                    }
                }
                self.move_file_safely(&record.current_path, &record.original_path)?;
            }
        }

        // Limpiar carpetas creadas si quedaron vacías
        for dir in &last_tx.created_dirs {
            if dir.exists() {
                if let Ok(mut read) = fs::read_dir(dir) {
                    if read.next().is_none() {
                        let _ = fs::remove_dir(dir);
                    }
                }
            }
        }

        Ok(Some(last_tx))
    }
}
