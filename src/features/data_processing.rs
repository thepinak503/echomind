use crate::error::{EchomindError, Result};
use calamine::Reader;
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataAnalysis {
    pub file_type: String,
    pub total_rows: usize,
    pub total_columns: usize,
    pub column_names: Vec<String>,
    pub column_types: HashMap<String, String>,
    pub summary_stats: HashMap<String, ColumnStats>,
    pub sample_data: Vec<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnStats {
    pub null_count: usize,
    pub unique_count: usize,
    pub min_value: Option<serde_json::Value>,
    pub max_value: Option<serde_json::Value>,
    pub mean_value: Option<f64>,
    pub median_value: Option<f64>,
    pub most_common: Option<(serde_json::Value, usize)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationConfig {
    pub chart_type: ChartType,
    pub x_column: String,
    pub y_column: Option<String>,
    pub title: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub color_scheme: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChartType {
    Bar,
    Line,
    Scatter,
    Pie,
    Histogram,
    Heatmap,
}

pub struct DataProcessor {
    cache: HashMap<String, DataAnalysis>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn process_csv(&mut self, file_path: &str) -> Result<DataAnalysis> {
        if let Some(cached) = self.cache.get(file_path) {
            return Ok(cached.clone());
        }

        let file = fs::File::open(file_path)
            .map_err(|e| EchomindError::FileError(format!("Failed to open CSV file: {}", e)))?;

        let mut rdr = ReaderBuilder::new().from_reader(file);
        let headers = rdr.headers()
            .map_err(|e| EchomindError::Other(format!("Failed to read CSV headers: {}", e)))?;

        let column_names: Vec<String> = headers.iter().map(|s| s.to_string()).collect();
        let mut records: Vec<HashMap<String, serde_json::Value>> = Vec::new();
        let mut column_data: HashMap<String, Vec<serde_json::Value>> = HashMap::new();

        // Initialize column data
        for name in &column_names {
            column_data.insert(name.clone(), Vec::new());
        }

        for result in rdr.records() {
            let record = result
                .map_err(|e| EchomindError::Other(format!("Failed to read CSV record: {}", e)))?;

            let mut record_map: HashMap<String, serde_json::Value> = HashMap::new();

            for (i, field) in record.iter().enumerate() {
                let column_name = &column_names[i];
                let value = self.parse_value(field);
                record_map.insert(column_name.clone(), value.clone());
                column_data.get_mut(column_name).unwrap().push(value);
            }

            records.push(record_map);
        }

        let total_rows = records.len();
        let total_columns = column_names.len();

        // Calculate column statistics
        let mut summary_stats = HashMap::new();
        let mut column_types = HashMap::new();

        for (column_name, values) in &column_data {
            let stats = self.calculate_column_stats(values);
            let data_type = self.infer_column_type(values);

            summary_stats.insert(column_name.clone(), stats);
            column_types.insert(column_name.clone(), data_type);
        }

        let analysis = DataAnalysis {
            file_type: "CSV".to_string(),
            total_rows,
            total_columns,
            column_names: column_names.clone(),
            column_types,
            summary_stats,
            sample_data: records.into_iter().take(10).collect(),
        };

        self.cache.insert(file_path.to_string(), analysis.clone());
        Ok(analysis)
    }

    pub fn process_json(&mut self, file_path: &str) -> Result<DataAnalysis> {
        if let Some(cached) = self.cache.get(file_path) {
            return Ok(cached.clone());
        }

        let contents = fs::read_to_string(file_path)
            .map_err(|e| EchomindError::FileError(format!("Failed to read JSON file: {}", e)))?;
        
        let json_value: serde_json::Value = serde_json::from_str(&contents)
            .map_err(|e| EchomindError::ParseError(format!("Failed to parse JSON: {}", e)))?;
        
        let (records, column_names) = match json_value {
            serde_json::Value::Array(arr) => {
                let mut all_keys = std::collections::HashSet::new();
                for item in &arr {
                    if let serde_json::Value::Object(obj) = item {
                        for key in obj.keys() {
                            all_keys.insert(key.clone());
                        }
                    }
                }
                let keys: Vec<String> = all_keys.into_iter().collect();
                
                let records: Vec<HashMap<String, serde_json::Value>> = arr.into_iter()
                    .filter_map(|item| {
                        if let serde_json::Value::Object(obj) = item {
                            Some(obj.into_iter().collect())
                        } else {
                            None
                        }
                    })
                    .collect();
                
                (records, keys)
            }
            serde_json::Value::Object(obj) => {
                let record: std::collections::HashMap<String, serde_json::Value> = obj.into_iter().collect();
                let keys = record.keys().cloned().collect();
                (vec![record], keys)
            }
            _ => {
                return Err(EchomindError::Other("JSON must be an object or array of objects".to_string()));
            }
        };
        
        let total_rows = records.len();
        let total_columns = column_names.len();
        
        // Calculate column statistics
        let mut summary_stats = HashMap::new();
        let mut column_types = HashMap::new();
        let mut column_data: HashMap<String, Vec<serde_json::Value>> = HashMap::new();
        
        // Initialize column data
        for name in &column_names {
            column_data.insert(name.clone(), Vec::new());
        }
        
        for record in &records {
            for column_name in &column_names {
                let value = record.get(column_name).cloned().unwrap_or(serde_json::Value::Null);
                column_data.get_mut(column_name).unwrap().push(value);
            }
        }
        
        for (column_name, values) in &column_data {
            let stats = self.calculate_column_stats(values);
            let data_type = self.infer_column_type(values);
            
            summary_stats.insert(column_name.clone(), stats);
            column_types.insert(column_name.clone(), data_type);
        }
        
        let analysis = DataAnalysis {
            file_type: "JSON".to_string(),
            total_rows,
            total_columns,
            column_names: column_names.clone(),
            column_types,
            summary_stats,
            sample_data: records.into_iter().take(10).collect(),
        };
        
        self.cache.insert(file_path.to_string(), analysis.clone());
        Ok(analysis)
    }

    pub fn process_excel(&mut self, file_path: &str) -> Result<DataAnalysis> {
        if let Some(cached) = self.cache.get(file_path) {
            return Ok(cached.clone());
        }

        let mut excel_data: Vec<Vec<String>> = Vec::new();
        let mut workbook: calamine::Sheets<std::io::BufReader<std::fs::File>> = calamine::open_workbook(file_path)
            .map_err(|e| EchomindError::Other(format!("Failed to open Excel file: {}", e)))?;

        if let Some(Ok(range)) = workbook.worksheet_range_at(0) {
            for row in range.rows() {
                let row_data: Vec<String> = row.iter().map(|cell| cell.to_string()).collect();
                excel_data.push(row_data);
            }
        }

        if excel_data.is_empty() {
            return Err(EchomindError::Other("Excel file is empty or could not be read".to_string()));
        }

        let column_names = excel_data.first().unwrap().clone();
        let data_rows = &excel_data[1..];

        let mut records: Vec<HashMap<String, serde_json::Value>> = Vec::new();
        let mut column_data: HashMap<String, Vec<serde_json::Value>> = HashMap::new();

        // Initialize column data
        for name in &column_names {
            column_data.insert(name.clone(), Vec::new());
        }

        for row in data_rows {
            let mut record_map: HashMap<String, serde_json::Value> = HashMap::new();

            for (i, cell) in row.iter().enumerate() {
                if i < column_names.len() {
                    let column_name = &column_names[i];
                    let value = self.parse_value(cell);
                    record_map.insert(column_name.clone(), value.clone());
                    column_data.get_mut(column_name).unwrap().push(value);
                }
            }

            records.push(record_map);
        }

        let total_rows = records.len();
        let total_columns = column_names.len();

        // Calculate column statistics
        let mut summary_stats = HashMap::new();
        let mut column_types = HashMap::new();

        for (column_name, values) in &column_data {
            let stats = self.calculate_column_stats(values);
            let data_type = self.infer_column_type(values);

            summary_stats.insert(column_name.clone(), stats);
            column_types.insert(column_name.clone(), data_type);
        }

        let analysis = DataAnalysis {
            file_type: "Excel".to_string(),
            total_rows,
            total_columns,
            column_names: column_names.clone(),
            column_types,
            summary_stats,
            sample_data: records.into_iter().take(10).collect(),
        };

        self.cache.insert(file_path.to_string(), analysis.clone());
        Ok(analysis)
    }

    pub fn generate_visualization(&self, analysis: &DataAnalysis, config: &VisualizationConfig) -> Result<String> {
        // This would generate actual visualization data (e.g., SVG, HTML with Chart.js, etc.)
        // For now, return a simple representation
        
        let chart_data = match config.chart_type {
            ChartType::Bar => self.generate_bar_chart(analysis, config)?,
            ChartType::Line => self.generate_line_chart(analysis, config)?,
            ChartType::Scatter => self.generate_scatter_chart(analysis, config)?,
            ChartType::Pie => self.generate_pie_chart(analysis, config)?,
            ChartType::Histogram => self.generate_histogram(analysis, config)?,
            ChartType::Heatmap => self.generate_heatmap(analysis, config)?,
        };
        
        Ok(chart_data)
    }

    pub fn query_data(&self, analysis: &DataAnalysis, query: &str) -> Result<Vec<HashMap<String, serde_json::Value>>> {
        // Simple query language implementation
        // For now, just support basic filtering
        
        if query.starts_with("SELECT") {
            // Very basic SQL-like parsing
            if query.contains("WHERE") {
                let parts: Vec<&str> = query.split("WHERE").collect();
                if parts.len() == 2 {
                    let where_clause = parts[1].trim();
                    return self.filter_data(&analysis.sample_data, where_clause);
                }
            }
            return Ok(analysis.sample_data.clone());
        }
        
        // Default: return all sample data
        Ok(analysis.sample_data.clone())
    }

    pub fn export_data(&self, data: &[HashMap<String, serde_json::Value>], format: &str, output_path: &str) -> Result<()> {
        match format {
            "csv" => {
                if data.is_empty() {
                    return Err(EchomindError::Other("No data to export".to_string()));
                }

                let file = fs::File::create(output_path)
                    .map_err(|e| EchomindError::FileError(format!("Failed to create output file: {}", e)))?;

                let mut wtr = WriterBuilder::new().from_writer(file);

                // Write headers
                let headers: Vec<String> = data[0].keys().cloned().collect();
                wtr.write_record(&headers)
                    .map_err(|e| EchomindError::Other(format!("Failed to write CSV headers: {}", e)))?;

                // Write data
                for record in data {
                    let row: Vec<String> = headers.iter()
                        .map(|header| {
                            record.get(header)
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string()
                        })
                        .collect();
                    wtr.write_record(&row)
                        .map_err(|e| EchomindError::Other(format!("Failed to write CSV row: {}", e)))?;
                }

                wtr.flush()
                    .map_err(|e| EchomindError::Other(format!("Failed to flush CSV: {}", e)))?;
            }
            "json" => {
                let json = serde_json::to_string_pretty(data)
                    .map_err(|e| EchomindError::ParseError(format!("Failed to serialize data: {}", e)))?;

                fs::write(output_path, json)
                    .map_err(|e| EchomindError::FileError(format!("Failed to write JSON file: {}", e)))?;
            }
            _ => {
                return Err(EchomindError::Other(format!("Unsupported export format: {}", format)));
            }
        }

        Ok(())
    }

    #[allow(dead_code)]
    fn parse_value(&self, value: &str) -> serde_json::Value {
        // Try to parse as different types
        if value.trim().is_empty() {
            return serde_json::Value::Null;
        }
        
        // Try to parse as number
        if let Ok(int_val) = value.parse::<i64>() {
            return serde_json::Value::Number(serde_json::Number::from(int_val));
        }
        
        if let Ok(float_val) = value.parse::<f64>() {
            return serde_json::Value::Number(serde_json::Number::from_f64(float_val).unwrap());
        }
        
        // Try to parse as boolean
        let lower = value.to_lowercase();
        if lower == "true" || lower == "false" {
            return serde_json::Value::Bool(lower == "true");
        }
        
        // Default to string
        serde_json::Value::String(value.to_string())
    }

    fn calculate_column_stats(&self, values: &[serde_json::Value]) -> ColumnStats {
        let mut null_count = 0;
        let mut unique_values = std::collections::HashSet::new();
        let mut numeric_values = Vec::new();
        let mut string_values = Vec::new();
        
        for value in values {
            if value.is_null() {
                null_count += 1;
            } else {
                unique_values.insert(value.clone());
                
                match value {
                    serde_json::Value::Number(n) => {
                        if let Some(f) = n.as_f64() {
                            numeric_values.push(f);
                        }
                    }
                    serde_json::Value::String(s) => {
                        string_values.push(s.clone());
                    }
                    _ => {}
                }
            }
        }
        
        let unique_count = unique_values.len();
        
        let (min_value, max_value) = if !numeric_values.is_empty() {
            let min = numeric_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max = numeric_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            (
                Some(serde_json::Value::Number(serde_json::Number::from_f64(min).unwrap())),
                Some(serde_json::Value::Number(serde_json::Number::from_f64(max).unwrap())),
            )
        } else if !string_values.is_empty() {
            let min = string_values.iter().min();
            let max = string_values.iter().max();
            (
                min.map(|s| serde_json::Value::String(s.clone())),
                max.map(|s| serde_json::Value::String(s.clone())),
            )
        } else {
            (None, None)
        };
        
        let mean_value = if !numeric_values.is_empty() {
            Some(numeric_values.iter().sum::<f64>() / numeric_values.len() as f64)
        } else {
            None
        };
        
        let median_value = if !numeric_values.is_empty() {
            let mut sorted = numeric_values.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let len = sorted.len();
            if len % 2 == 0 {
                Some((sorted[len/2 - 1] + sorted[len/2]) / 2.0)
            } else {
                Some(sorted[len/2])
            }
        } else {
            None
        };
        
        let most_common = if !unique_values.is_empty() {
            let mut counts: HashMap<&serde_json::Value, usize> = HashMap::new();
            for value in &unique_values {
                let count = values.iter().filter(|v| *v == value).count();
                counts.insert(value, count);
            }
            
            counts.into_iter()
                .max_by_key(|(_, count)| *count)
                .map(|(value, count)| (value.clone(), count))
        } else {
            None
        };
        
        ColumnStats {
            null_count,
            unique_count,
            min_value,
            max_value,
            mean_value,
            median_value,
            most_common,
        }
    }

    fn infer_column_type(&self, values: &[serde_json::Value]) -> String {
        let mut type_counts = HashMap::new();
        
        for value in values {
            if value.is_null() {
                continue;
            }
            
            let type_name = match value {
                serde_json::Value::Number(_) => "number",
                serde_json::Value::String(_) => "string",
                serde_json::Value::Bool(_) => "boolean",
                serde_json::Value::Array(_) => "array",
                serde_json::Value::Object(_) => "object",
                _ => "unknown",
            };
            
            *type_counts.entry(type_name).or_insert(0) += 1;
        }
        
        type_counts.into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(type_name, _)| type_name.to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }

    fn filter_data(&self, data: &[HashMap<String, serde_json::Value>], where_clause: &str) -> Result<Vec<HashMap<String, serde_json::Value>>> {
        // Very basic WHERE clause parsing
        // For now, just support simple equality checks
        
        let parts: Vec<&str> = where_clause.split('=').collect();
        if parts.len() != 2 {
            return Ok(data.to_vec());
        }
        
        let column = parts[0].trim();
        let value = parts[1].trim().trim_matches('\'').trim_matches('"');
        
        let filtered: Vec<HashMap<String, serde_json::Value>> = data.iter()
            .filter(|record| {
                if let Some(field_value) = record.get(column) {
                    match field_value {
                        serde_json::Value::String(s) => s == value,
                        serde_json::Value::Number(n) => {
                            if let Ok(num_val) = value.parse::<f64>() {
                                n.as_f64() == Some(num_val)
                            } else {
                                false
                            }
                        }
                        serde_json::Value::Bool(b) => {
                            if let Ok(bool_val) = value.parse::<bool>() {
                                *b == bool_val
                            } else {
                                false
                            }
                        }
                        _ => false,
                    }
                } else {
                    false
                }
            })
            .cloned()
            .collect();
        
        Ok(filtered)
    }

    fn generate_bar_chart(&self, _analysis: &DataAnalysis, config: &VisualizationConfig) -> Result<String> {
        // Simplified bar chart generation
        Ok(format!(
            r#"<div style="width: {}px; height: {}px;">
                <h3>{}</h3>
                <p>Bar chart for column: {}</p>
                <div style="border: 1px solid #ccc; padding: 10px;">
                    Chart data would be rendered here
                </div>
            </div>"#,
            config.width.unwrap_or(600),
            config.height.unwrap_or(400),
            config.title.as_ref().unwrap_or(&"Bar Chart".to_string()),
            config.x_column
        ))
    }

    fn generate_line_chart(&self, _analysis: &DataAnalysis, config: &VisualizationConfig) -> Result<String> {
        Ok(format!(
            r#"<div style="width: {}px; height: {}px;">
                <h3>{}</h3>
                <p>Line chart for column: {}</p>
                <div style="border: 1px solid #ccc; padding: 10px;">
                    Chart data would be rendered here
                </div>
            </div>"#,
            config.width.unwrap_or(600),
            config.height.unwrap_or(400),
            config.title.as_ref().unwrap_or(&"Line Chart".to_string()),
            config.x_column
        ))
    }

    fn generate_scatter_chart(&self, _analysis: &DataAnalysis, config: &VisualizationConfig) -> Result<String> {
        Ok(format!(
            r#"<div style="width: {}px; height: {}px;">
                <h3>{}</h3>
                <p>Scatter chart: {} vs {}</p>
                <div style="border: 1px solid #ccc; padding: 10px;">
                    Chart data would be rendered here
                </div>
            </div>"#,
            config.width.unwrap_or(600),
            config.height.unwrap_or(400),
            config.title.as_ref().unwrap_or(&"Scatter Chart".to_string()),
            config.x_column,
            config.y_column.as_ref().unwrap_or(&"Y".to_string())
        ))
    }

    fn generate_pie_chart(&self, _analysis: &DataAnalysis, config: &VisualizationConfig) -> Result<String> {
        Ok(format!(
            r#"<div style="width: {}px; height: {}px;">
                <h3>{}</h3>
                <p>Pie chart for column: {}</p>
                <div style="border: 1px solid #ccc; padding: 10px;">
                    Chart data would be rendered here
                </div>
            </div>"#,
            config.width.unwrap_or(600),
            config.height.unwrap_or(400),
            config.title.as_ref().unwrap_or(&"Pie Chart".to_string()),
            config.x_column
        ))
    }

    fn generate_histogram(&self, _analysis: &DataAnalysis, config: &VisualizationConfig) -> Result<String> {
        Ok(format!(
            r#"<div style="width: {}px; height: {}px;">
                <h3>{}</h3>
                <p>Histogram for column: {}</p>
                <div style="border: 1px solid #ccc; padding: 10px;">
                    Chart data would be rendered here
                </div>
            </div>"#,
            config.width.unwrap_or(600),
            config.height.unwrap_or(400),
            config.title.as_ref().unwrap_or(&"Histogram".to_string()),
            config.x_column
        ))
    }

    fn generate_heatmap(&self, _analysis: &DataAnalysis, config: &VisualizationConfig) -> Result<String> {
        Ok(format!(
            r#"<div style="width: {}px; height: {}px;">
                <h3>{}</h3>
                <p>Heatmap for column: {}</p>
                <div style="border: 1px solid #ccc; padding: 10px;">
                    Chart data would be rendered here
                </div>
            </div>"#,
            config.width.unwrap_or(600),
            config.height.unwrap_or(400),
            config.title.as_ref().unwrap_or(&"Heatmap".to_string()),
            config.x_column
        ))
    }
}

impl Default for DataProcessor {
    fn default() -> Self {
        Self::new()
    }
}