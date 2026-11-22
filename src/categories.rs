use crate::config::Config;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Category {
    pub name: String,
    pub icon: String,
    pub extensions: Vec<String>,
}

pub struct CategoryRegistry {
    categories: Vec<Category>,
    extension_map: HashMap<String, usize>,
}

impl CategoryRegistry {
    pub fn default_rules() -> Self {
        Self::from_config(&Config::default())
    }

    pub fn from_config(config: &Config) -> Self {
        let mut categories = Vec::new();
        let mut extension_map = HashMap::new();

        for (name, cat_cfg) in &config.categories {
            let idx = categories.len();
            for ext in &cat_cfg.extensions {
                extension_map.insert(ext.to_lowercase(), idx);
            }
            categories.push(Category {
                name: name.clone(),
                icon: cat_cfg.icon.clone(),
                extensions: cat_cfg.extensions.clone(),
            });
        }

        Self {
            categories,
            extension_map,
        }
    }

    pub fn get_category_for_path(&self, path: &Path) -> Option<&Category> {
        let ext = path.extension()?.to_str()?.to_lowercase();
        let idx = self.extension_map.get(&ext)?;
        Some(&self.categories[*idx])
    }

    #[allow(dead_code)]
    pub fn is_category_folder_name(&self, name: &str) -> bool {
        self.categories.iter().any(|c| c.name == name) || name == "Otros"
    }

    pub fn all_categories(&self) -> &[Category] {
        &self.categories
    }

    pub fn add_or_update_category(&mut self, name: String, icon: String, extensions: Vec<String>) {
        if let Some(pos) = self.categories.iter().position(|c| c.name == name) {
            self.categories[pos].icon = icon;
            self.categories[pos].extensions = extensions;
        } else {
            self.categories.push(Category {
                name,
                icon,
                extensions,
            });
        }

        // Rebuild extension_map
        self.extension_map.clear();
        for (idx, cat) in self.categories.iter().enumerate() {
            for ext in &cat.extensions {
                self.extension_map.insert(ext.to_lowercase(), idx);
            }
        }
    }
}
