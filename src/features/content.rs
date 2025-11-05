use crate::error::{EchomindError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub content: String,
    pub variables: Vec<TemplateVariable>,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub usage_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    pub name: String,
    pub description: Option<String>,
    pub default_value: Option<String>,
    pub required: bool,
    pub variable_type: VariableType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VariableType {
    String,
    Number,
    Boolean,
    Date,
    File,
    Select(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snippet {
    pub id: String,
    pub name: String,
    pub content: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub category: Option<String>,
    pub language: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub usage_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentLibrary {
    pub templates: HashMap<String, Template>,
    pub snippets: HashMap<String, Snippet>,
    pub categories: HashMap<String, Category>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub struct ContentManager {
    library: ContentLibrary,
    library_path: String,
}

impl ContentManager {
    pub fn new(library_path: &str) -> Result<Self> {
        let mut manager = Self {
            library: ContentLibrary {
                templates: HashMap::new(),
                snippets: HashMap::new(),
                categories: HashMap::new(),
            },
            library_path: library_path.to_string(),
        };
        
        manager.load_library()?;
        Ok(manager)
    }

    pub fn create_template(
        &mut self,
        name: &str,
        content: &str,
        description: Option<&str>,
        category: Option<&str>,
        tags: Vec<String>,
    ) -> Result<Template> {
        let template = Template {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            content: content.to_string(),
            variables: self.extract_variables(content),
            category: category.map(|s| s.to_string()),
            tags,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            usage_count: 0,
        };
        
        self.library.templates.insert(template.id.clone(), template.clone());
        self.save_library()?;
        Ok(template)
    }

    pub fn create_snippet(
        &mut self,
        name: &str,
        content: &str,
        description: Option<&str>,
        tags: Vec<String>,
        category: Option<&str>,
        language: Option<&str>,
    ) -> Result<Snippet> {
        let snippet = Snippet {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            content: content.to_string(),
            description: description.map(|s| s.to_string()),
            tags,
            category: category.map(|s| s.to_string()),
            language: language.map(|s| s.to_string()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            usage_count: 0,
        };
        
        self.library.snippets.insert(snippet.id.clone(), snippet.clone());
        self.save_library()?;
        Ok(snippet)
    }

    pub fn render_template(
        &mut self,
        template_id: &str,
        variables: &HashMap<String, String>,
    ) -> Result<String> {
        let template = self.library.templates.get(template_id)
            .ok_or_else(|| EchomindError::Other(format!("Template {} not found", template_id)))?
            .clone();
        
        // Check required variables
        for var in &template.variables {
            if var.required && !variables.contains_key(&var.name) {
                return Err(EchomindError::Other(format!("Required variable '{}' not provided", var.name)));
            }
        }
        
        let mut rendered = template.content;
        
        // Replace variables
        for var in &template.variables {
            let value = variables.get(&var.name)
                .or(var.default_value.as_ref())
                .cloned()
                .unwrap_or_default();
            rendered = rendered.replace(&format!("{{{}}}", var.name), &value);
        }
        
        // Update usage count
        if let Some(t) = self.library.templates.get_mut(template_id) {
            t.usage_count += 1;
            t.updated_at = chrono::Utc::now();
        }
        
        self.save_library()?;
        Ok(rendered)
    }

    pub fn search_templates(&self, query: &str, category: Option<&str>, tags: Vec<String>) -> Vec<&Template> {
        self.library.templates.values()
            .filter(|template| {
                let matches_query = query.is_empty() || 
                    template.name.to_lowercase().contains(&query.to_lowercase()) ||
                    template.description.as_ref().map_or(false, |d| d.to_lowercase().contains(&query.to_lowercase())) ||
                    template.content.to_lowercase().contains(&query.to_lowercase());
                
                let matches_category = category.is_none() ||
                    template.category.as_ref().map_or(false, |c| Some(c.as_str()) == category);
                
                let matches_tags = tags.is_empty() || 
                    tags.iter().all(|tag| template.tags.contains(tag));
                
                matches_query && matches_category && matches_tags
            })
            .collect()
    }

    pub fn search_snippets(&self, query: &str, category: Option<&str>, tags: Vec<String>) -> Vec<&Snippet> {
        self.library.snippets.values()
            .filter(|snippet| {
                let matches_query = query.is_empty() || 
                    snippet.name.to_lowercase().contains(&query.to_lowercase()) ||
                    snippet.description.as_ref().map_or(false, |d| d.to_lowercase().contains(&query.to_lowercase())) ||
                    snippet.content.to_lowercase().contains(&query.to_lowercase());
                
                let matches_category = category.is_none() ||
                    snippet.category.as_ref().map_or(false, |c| Some(c.as_str()) == category);
                
                let matches_tags = tags.is_empty() || 
                    tags.iter().all(|tag| snippet.tags.contains(tag));
                
                matches_query && matches_category && matches_tags
            })
            .collect()
    }

    pub fn get_template(&self, template_id: &str) -> Option<&Template> {
        self.library.templates.get(template_id)
    }

    pub fn get_snippet(&self, snippet_id: &str) -> Option<&Snippet> {
        self.library.snippets.get(snippet_id)
    }

    pub fn update_template(&mut self, template_id: &str, updates: TemplateUpdate) -> Result<()> {
        // Extract variables first if content is being updated
        let variables = if let Some(ref content) = updates.content {
            Some(self.extract_variables(content))
        } else {
            None
        };

        let template = self.library.templates.get_mut(template_id)
            .ok_or_else(|| EchomindError::Other(format!("Template {} not found", template_id)))?;

        if let Some(name) = updates.name {
            template.name = name;
        }
        if let Some(description) = updates.description {
            template.description = Some(description);
        }
        if let Some(content) = updates.content {
            template.content = content;
            template.variables = variables.unwrap();
        }
        if let Some(category) = updates.category {
            template.category = Some(category);
        }
        if let Some(tags) = updates.tags {
            template.tags = tags;
        }

        self.save_library()
    }

    pub fn update_snippet(&mut self, snippet_id: &str, updates: SnippetUpdate) -> Result<()> {
        let snippet = self.library.snippets.get_mut(snippet_id)
            .ok_or_else(|| EchomindError::Other(format!("Snippet {} not found", snippet_id)))?;
        
        if let Some(name) = updates.name {
            snippet.name = name;
        }
        if let Some(content) = updates.content {
            snippet.content = content;
        }
        if let Some(description) = updates.description {
            snippet.description = Some(description);
        }
        if let Some(category) = updates.category {
            snippet.category = Some(category);
        }
        if let Some(tags) = updates.tags {
            snippet.tags = tags;
        }
        if let Some(language) = updates.language {
            snippet.language = Some(language);
        }
        
        snippet.updated_at = chrono::Utc::now();
        self.save_library()?;
        Ok(())
    }

    pub fn delete_template(&mut self, template_id: &str) -> Result<()> {
        if self.library.templates.remove(template_id).is_none() {
            return Err(EchomindError::Other(format!("Template {} not found", template_id)));
        }
        self.save_library()?;
        Ok(())
    }

    pub fn delete_snippet(&mut self, snippet_id: &str) -> Result<()> {
        if self.library.snippets.remove(snippet_id).is_none() {
            return Err(EchomindError::Other(format!("Snippet {} not found", snippet_id)));
        }
        self.save_library()?;
        Ok(())
    }

    pub fn list_categories(&self) -> Vec<&Category> {
        self.library.categories.values().collect()
    }

    pub fn create_category(&mut self, name: &str, description: Option<&str>, parent_id: Option<&str>) -> Result<Category> {
        let category = Category {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            parent_id: parent_id.map(|s| s.to_string()),
            created_at: chrono::Utc::now(),
        };
        
        self.library.categories.insert(category.id.clone(), category.clone());
        self.save_library()?;
        Ok(category)
    }

    pub fn get_popular_templates(&self, limit: usize) -> Vec<&Template> {
        let mut templates: Vec<_> = self.library.templates.values().collect();
        templates.sort_by(|a, b| b.usage_count.cmp(&a.usage_count));
        templates.into_iter().take(limit).collect()
    }

    pub fn get_recent_snippets(&self, days: u32) -> Vec<&Snippet> {
        let cutoff = chrono::Utc::now() - chrono::Duration::days(days as i64);
        self.library.snippets.values()
            .filter(|snippet| snippet.updated_at > cutoff)
            .collect()
    }

    pub fn export_library(&self, format: &str) -> Result<String> {
        match format {
            "json" => {
                serde_json::to_string_pretty(&self.library)
                    .map_err(|e| EchomindError::ParseError(format!("Failed to export library: {}", e)))
            }
            "templates-only" => {
                serde_json::to_string_pretty(&self.library.templates)
                    .map_err(|e| EchomindError::ParseError(format!("Failed to export templates: {}", e)))
            }
            "snippets-only" => {
                serde_json::to_string_pretty(&self.library.snippets)
                    .map_err(|e| EchomindError::ParseError(format!("Failed to export snippets: {}", e)))
            }
            _ => Err(EchomindError::Other(format!("Unsupported export format: {}", format))),
        }
    }

    pub fn import_library(&mut self, data: &str, format: &str) -> Result<()> {
        match format {
            "json" => {
                let imported: ContentLibrary = serde_json::from_str(data)
                    .map_err(|e| EchomindError::ParseError(format!("Failed to import library: {}", e)))?;
                
                // Merge with existing library
                for (id, template) in imported.templates {
                    self.library.templates.insert(id, template);
                }
                for (id, snippet) in imported.snippets {
                    self.library.snippets.insert(id, snippet);
                }
                for (id, category) in imported.categories {
                    self.library.categories.insert(id, category);
                }
                
                self.save_library()?;
                Ok(())
            }
            _ => Err(EchomindError::Other(format!("Unsupported import format: {}", format))),
        }
    }

    fn extract_variables(&self, content: &str) -> Vec<TemplateVariable> {
        let mut variables = Vec::new();
        let re = regex::Regex::new(r"\{([^}]+)\}").unwrap();
        
        for cap in re.captures_iter(content) {
            let var_name = cap.get(1).unwrap().as_str();
            variables.push(TemplateVariable {
                name: var_name.to_string(),
                description: None,
                default_value: None,
                required: true,
                variable_type: VariableType::String,
            });
        }
        
        variables
    }

    fn load_library(&mut self) -> Result<()> {
        if Path::new(&self.library_path).exists() {
            let contents = fs::read_to_string(&self.library_path)
                .map_err(|e| EchomindError::FileError(format!("Failed to read library: {}", e)))?;
            
            self.library = serde_json::from_str(&contents)
                .map_err(|e| EchomindError::ParseError(format!("Failed to parse library: {}", e)))?;
        }
        Ok(())
    }

    fn save_library(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.library)
            .map_err(|e| EchomindError::ParseError(format!("Failed to serialize library: {}", e)))?;
        
        fs::write(&self.library_path, json)
            .map_err(|e| EchomindError::FileError(format!("Failed to write library: {}", e)))?;
        
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct TemplateUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct SnippetUpdate {
    pub name: Option<String>,
    pub content: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub language: Option<String>,
}

impl Default for ContentManager {
    fn default() -> Self {
        Self::new("~/.config/echomind/content_library.json").unwrap_or_else(|_| Self {
            library: ContentLibrary {
                templates: HashMap::new(),
                snippets: HashMap::new(),
                categories: HashMap::new(),
            },
            library_path: "~/.config/echomind/content_library.json".to_string(),
        })
    }
}