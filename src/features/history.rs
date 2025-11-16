use crate::api::Message;
use crate::error::{EchomindError, Result};
use chrono::{DateTime, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub role: String,
    pub content: String,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub has_image: bool,
    pub token_count: Option<u32>,
    pub cost_estimate: Option<f64>,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoryStats {
    pub total_conversations: usize,
    pub total_messages: usize,
    pub total_tokens: u32,
    pub total_cost: f64,
    pub average_response_time: f64,
    pub most_used_models: Vec<(String, usize)>,
    pub most_used_providers: Vec<(String, usize)>,
    pub activity_by_day: HashMap<String, usize>,
    pub activity_by_hour: HashMap<u32, usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: String,
    pub role: Option<String>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub tags: Vec<String>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
}

pub struct HistoryManager {
    history_file: String,
}

impl HistoryManager {
    pub fn new(history_file: &str) -> Self {
        Self {
            history_file: history_file.to_string(),
        }
    }

    pub fn add_entry(&mut self, entry: HistoryEntry) -> Result<()> {
        let mut entries = self.load_entries()?;
        entries.push(entry);
        self.save_entries(&entries)?;
        Ok(())
    }

    pub fn load_entries(&self) -> Result<Vec<HistoryEntry>> {
        if !Path::new(&self.history_file).exists() {
            return Ok(Vec::new());
        }

        let contents = fs::read_to_string(&self.history_file)
            .map_err(|e| EchomindError::FileError(format!("Failed to read history: {}", e)))?;

        serde_json::from_str(&contents)
            .map_err(|e| EchomindError::ParseError(format!("Failed to parse history: {}", e)))
    }

    pub fn save_entries(&self, entries: &[HistoryEntry]) -> Result<()> {
        let json = serde_json::to_string_pretty(entries)
            .map_err(|e| EchomindError::ParseError(format!("Failed to serialize history: {}", e)))?;

        fs::write(&self.history_file, json)
            .map_err(|e| EchomindError::FileError(format!("Failed to write history: {}", e)))?;

        Ok(())
    }

    pub fn search(&self, query: SearchQuery) -> Result<Vec<HistoryEntry>> {
        let entries = self.load_entries()?;
        let mut results = Vec::new();

        for entry in entries {
            let mut matches = true;

            // Text search
            if !query.query.is_empty() {
                let search_lower = query.query.to_lowercase();
                let content_lower = entry.content.to_lowercase();
                if !content_lower.contains(&search_lower) {
                    matches = false;
                }
            }

            // Role filter
            if let Some(role) = &query.role {
                if entry.role != *role {
                    matches = false;
                }
            }

            // Provider filter
            if let Some(provider) = &query.provider {
                if entry.provider.as_ref() != Some(provider) {
                    matches = false;
                }
            }

            // Model filter
            if let Some(model) = &query.model {
                if entry.model.as_ref() != Some(model) {
                    matches = false;
                }
            }

            // Tags filter
            if !query.tags.is_empty() {
                let has_all_tags = query.tags.iter().all(|tag| entry.tags.contains(tag));
                if !has_all_tags {
                    matches = false;
                }
            }

            // Date range filter
            if let Some(from) = query.date_from {
                if entry.timestamp < from {
                    matches = false;
                }
            }
            if let Some(to) = query.date_to {
                if entry.timestamp > to {
                    matches = false;
                }
            }

            if matches {
                results.push(entry);
            }
        }

        // Apply limit
        if let Some(limit) = query.limit {
            results.truncate(limit);
        }

        Ok(results)
    }

    pub fn export(&self, format: &str, query: Option<SearchQuery>) -> Result<String> {
        let entries = if let Some(q) = query {
            self.search(q)?
        } else {
            self.load_entries()?
        };

        match format {
            "json" => {
                serde_json::to_string_pretty(&entries)
                    .map_err(|e| EchomindError::ParseError(format!("Failed to export JSON: {}", e)))
            }
            "csv" => {
                let mut csv = String::new();
                csv.push_str("ID,Timestamp,Role,Content,Provider,Model,HasImage,TokenCount,CostEstimate,Tags\n");
                
                for entry in entries {
                    let tags_str = entry.tags.join(";");
                    csv.push_str(&format!(
                        "{},{},{},{},{},{},{},{},{},{}\n",
                        entry.id,
                        entry.timestamp.to_rfc3339(),
                        entry.role,
                        entry.content.replace(',', ";").replace('\n', " "),
                        entry.provider.unwrap_or_default(),
                        entry.model.unwrap_or_default(),
                        entry.has_image,
                        entry.token_count.unwrap_or(0),
                        entry.cost_estimate.unwrap_or(0.0),
                        tags_str
                    ));
                }
                
                Ok(csv)
            }
            "markdown" => {
                let mut md = String::new();
                md.push_str("# Conversation History\n\n");
                
                for entry in entries {
                    md.push_str(&format!("## {} - {}\n\n", entry.role.to_uppercase(), entry.timestamp.format("%Y-%m-%d %H:%M:%S")));
                    md.push_str(&format!("**Provider:** {}\n", entry.provider.unwrap_or_default()));
                    md.push_str(&format!("**Model:** {}\n", entry.model.unwrap_or_default()));
                    if !entry.tags.is_empty() {
                        md.push_str(&format!("**Tags:** {}\n", entry.tags.join(", ")));
                    }
                    md.push_str("\n```\n");
                    md.push_str(&entry.content);
                    md.push_str("\n```\n\n");
                }
                
                Ok(md)
            }
            _ => Err(EchomindError::Other(format!("Unsupported export format: {}", format))),
        }
    }

    pub fn get_stats(&self) -> Result<HistoryStats> {
        let entries = self.load_entries()?;
        let mut stats = HistoryStats {
            total_conversations: 0,
            total_messages: entries.len(),
            total_tokens: 0,
            total_cost: 0.0,
            average_response_time: 0.0,
            most_used_models: Vec::new(),
            most_used_providers: Vec::new(),
            activity_by_day: HashMap::new(),
            activity_by_hour: HashMap::new(),
        };

        let mut model_counts: HashMap<String, usize> = HashMap::new();
        let mut provider_counts: HashMap<String, usize> = HashMap::new();

        for entry in &entries {
            // Count tokens and cost
            if let Some(tokens) = entry.token_count {
                stats.total_tokens += tokens;
            }
            if let Some(cost) = entry.cost_estimate {
                stats.total_cost += cost;
            }

            // Count models and providers
            if let Some(model) = &entry.model {
                *model_counts.entry(model.clone()).or_insert(0) += 1;
            }
            if let Some(provider) = &entry.provider {
                *provider_counts.entry(provider.clone()).or_insert(0) += 1;
            }

            // Activity by day and hour
            let day = entry.timestamp.format("%Y-%m-%d").to_string();
            let hour = entry.timestamp.hour();
            *stats.activity_by_day.entry(day).or_insert(0) += 1;
            *stats.activity_by_hour.entry(hour).or_insert(0) += 1;
        }

        // Convert to sorted vectors
        stats.most_used_models = model_counts.into_iter()
            .collect::<Vec<_>>();
        stats.most_used_models.sort_by(|a, b| b.1.cmp(&a.1));

        stats.most_used_providers = provider_counts.into_iter()
            .collect::<Vec<_>>();
        stats.most_used_providers.sort_by(|a, b| b.1.cmp(&a.1));

        Ok(stats)
    }

    pub fn merge_histories(&self, other_history_files: &[&str]) -> Result<()> {
        let mut all_entries = self.load_entries()?;
        
        for file in other_history_files {
            let other_manager = HistoryManager::new(file);
            let other_entries = other_manager.load_entries()?;
            all_entries.extend(other_entries);
        }

        // Remove duplicates based on ID
        all_entries.sort_by(|a, b| a.id.cmp(&b.id));
        all_entries.dedup_by(|a, b| a.id == b.id);

        self.save_entries(&all_entries)?;
        Ok(())
    }

    pub fn add_tags(&mut self, entry_id: &str, tags: Vec<String>) -> Result<()> {
        let mut entries = self.load_entries()?;
        
        if let Some(entry) = entries.iter_mut().find(|e| e.id == entry_id) {
            for tag in tags {
                if !entry.tags.contains(&tag) {
                    entry.tags.push(tag);
                }
            }
            self.save_entries(&entries)?;
            Ok(())
        } else {
            Err(EchomindError::Other(format!("Entry with ID {} not found", entry_id)))
        }
    }

    pub fn delete_entry(&mut self, entry_id: &str) -> Result<()> {
        let mut entries = self.load_entries()?;
        entries.retain(|e| e.id != entry_id);
        self.save_entries(&entries)?;
        Ok(())
    }

    pub fn clear_history(&mut self) -> Result<()> {
        self.save_entries(&[])?;
        Ok(())
    }
}

impl From<&Message> for HistoryEntry {
    fn from(msg: &Message) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            role: msg.role.clone(),
            content: msg.get_text().unwrap_or("").to_string(),
            provider: None,
            model: None,
            // has_image: matches!(msg.content, crate::api::MessageContent::MultiModal(_)),
            has_image: false,
            token_count: None,
            cost_estimate: None,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }
}