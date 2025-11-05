use crate::api::{ApiClient, ChatRequest, Message};
use crate::error::{EchomindError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub model: String,
    pub provider: String,
    pub prompt: String,
    pub response: String,
    pub response_time_ms: u64,
    pub tokens_per_second: f64,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cost_estimate: f64,
    pub quality_score: Option<f64>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time: f64,
    pub total_tokens_processed: u64,
    pub total_cost: f64,
    pub requests_per_minute: f64,
    pub error_rate: f64,
    pub uptime_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelComparison {
    pub models: Vec<String>,
    pub prompt: String,
    pub results: Vec<BenchmarkResult>,
    pub winner: Option<String>,
    pub comparison_metrics: HashMap<String, f64>,
}

pub struct PerformanceMonitor {
    metrics: PerformanceMetrics,
    benchmark_results: Vec<BenchmarkResult>,
    start_time: Instant,
    request_times: Vec<Duration>,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            metrics: PerformanceMetrics {
                total_requests: 0,
                successful_requests: 0,
                failed_requests: 0,
                average_response_time: 0.0,
                total_tokens_processed: 0,
                total_cost: 0.0,
                requests_per_minute: 0.0,
                error_rate: 0.0,
                uptime_percentage: 100.0,
            },
            benchmark_results: Vec::new(),
            start_time: Instant::now(),
            request_times: Vec::new(),
        }
    }

    pub async fn benchmark_model(
        &mut self,
        api_client: &ApiClient,
        model: &str,
        provider: &str,
        prompt: &str,
    ) -> Result<BenchmarkResult> {
        let start_time = Instant::now();
        
        let messages = vec![Message::text("user".to_string(), prompt.to_string())];
        let request = ChatRequest {
            messages,
            model: Some(model.to_string()),
            temperature: Some(0.7),
            max_tokens: Some(1000),
            stream: None,
        };
        
        let response = api_client.send_message(request).await;
        let response_time = start_time.elapsed();
        
        match response {
            Ok(response_text) => {
                let input_tokens = self.estimate_token_count(prompt);
                let output_tokens = self.estimate_token_count(&response_text);
                let total_tokens = input_tokens + output_tokens;
                let tokens_per_second = total_tokens as f64 / response_time.as_secs_f64();
                
                let cost_estimate = self.calculate_cost(provider, model, input_tokens, output_tokens);
                let quality_score = self.calculate_quality_score(&response_text);
                
                let result = BenchmarkResult {
                    model: model.to_string(),
                    provider: provider.to_string(),
                    prompt: prompt.to_string(),
                    response: response_text.clone(),
                    response_time_ms: response_time.as_millis() as u64,
                    tokens_per_second,
                    input_tokens,
                    output_tokens,
                    cost_estimate,
                    quality_score,
                    timestamp: chrono::Utc::now(),
                };
                
                self.update_metrics(true, response_time, total_tokens, cost_estimate);
                self.benchmark_results.push(result.clone());
                
                Ok(result)
            }
            Err(e) => {
                self.update_metrics(false, response_time, 0, 0.0);
                Err(e)
            }
        }
    }

    pub async fn compare_models(
        &mut self,
        api_clients: &HashMap<String, ApiClient>,
        models: &[String],
        prompt: &str,
    ) -> Result<ModelComparison> {
        let mut results = Vec::new();
        
        for model in models {
            if let Some((provider, client)) = self.get_provider_for_model(model, api_clients) {
                match self.benchmark_model(client, model, &provider, prompt).await {
                    Ok(result) => results.push(result),
                    Err(e) => eprintln!("Failed to benchmark {}: {}", model, e),
                }
            }
        }
        
        let winner = self.select_best_model(&results);
        let comparison_metrics = self.calculate_comparison_metrics(&results);
        
        Ok(ModelComparison {
            models: models.to_vec(),
            prompt: prompt.to_string(),
            results,
            winner,
            comparison_metrics,
        })
    }

    pub async fn run_stress_test(
        &mut self,
        api_client: &ApiClient,
        model: &str,
        provider: &str,
        prompt: &str,
        concurrent_requests: usize,
        duration_seconds: u64,
    ) -> Result<Vec<BenchmarkResult>> {
        let api_client = api_client.clone();
        let mut handles = Vec::new();
        let start_time = Instant::now();
        let end_time = start_time + Duration::from_secs(duration_seconds);
        
        while Instant::now() < end_time {
            for _ in 0..concurrent_requests {
                if Instant::now() >= end_time {
                    break;
                }
                
                let client = api_client.clone();
                let model = model.to_string();
                let provider = provider.to_string();
                let prompt = prompt.to_string();
                
                let handle = tokio::spawn(async move {
                    let mut monitor = PerformanceMonitor::new();
                    monitor.benchmark_model(&client, &model, &provider, &prompt).await
                });
                
                handles.push(handle);
                
                // Small delay between requests
                sleep(Duration::from_millis(100)).await;
            }
            
            // Wait for current batch to complete before starting new one
            for handle in handles.drain(..) {
                if let Ok(result) = handle.await {
                    if let Ok(benchmark) = result {
                        self.benchmark_results.push(benchmark);
                    }
                }
            }
        }
        
        Ok(self.benchmark_results.clone())
    }

    pub fn get_performance_report(&self) -> PerformanceReport {
        let uptime = self.start_time.elapsed();
        let uptime_hours = uptime.as_secs_f64() / 3600.0;
        
        PerformanceReport {
            metrics: self.metrics.clone(),
            uptime_hours,
            top_performing_models: self.get_top_performing_models(),
            recommendations: self.generate_recommendations(),
            last_updated: chrono::Utc::now(),
        }
    }

    pub fn export_benchmark_data(&self, format: &str) -> Result<String> {
        match format {
            "json" => {
                serde_json::to_string_pretty(&self.benchmark_results)
                    .map_err(|e| EchomindError::ParseError(format!("Failed to export JSON: {}", e)))
            }
            "csv" => {
                let mut csv = String::new();
                csv.push_str("Model,Provider,Prompt,Response,ResponseTimeMS,TokensPerSecond,InputTokens,OutputTokens,CostEstimate,QualityScore,Timestamp\n");
                
                for result in &self.benchmark_results {
                    csv.push_str(&format!(
                        "{},{},{},{},{},{},{},{},{},{},{}\n",
                        result.model,
                        result.provider,
                        result.prompt.replace(',', ";"),
                        result.response.replace(',', ";").replace('\n', " "),
                        result.response_time_ms,
                        result.tokens_per_second,
                        result.input_tokens,
                        result.output_tokens,
                        result.cost_estimate,
                        result.quality_score.unwrap_or(0.0),
                        result.timestamp.to_rfc3339()
                    ));
                }
                
                Ok(csv)
            }
            _ => Err(EchomindError::Other(format!("Unsupported export format: {}", format))),
        }
    }

    fn update_metrics(&mut self, success: bool, response_time: Duration, tokens: u32, cost: f64) {
        self.metrics.total_requests += 1;
        self.request_times.push(response_time);
        
        if success {
            self.metrics.successful_requests += 1;
            self.metrics.total_tokens_processed += tokens as u64;
            self.metrics.total_cost += cost;
        } else {
            self.metrics.failed_requests += 1;
        }
        
        // Update average response time
        let total_time: Duration = self.request_times.iter().sum();
        self.metrics.average_response_time = total_time.as_secs_f64() / self.request_times.len() as f64;
        
        // Update error rate
        self.metrics.error_rate = self.metrics.failed_requests as f64 / self.metrics.total_requests as f64;
        
        // Update requests per minute
        let elapsed_minutes = self.start_time.elapsed().as_secs_f64() / 60.0;
        if elapsed_minutes > 0.0 {
            self.metrics.requests_per_minute = self.metrics.total_requests as f64 / elapsed_minutes;
        }
    }

    fn estimate_token_count(&self, text: &str) -> u32 {
        // Simple token estimation (rough approximation)
        // In reality, you'd use the tokenizer for the specific model
        let word_count = text.split_whitespace().count() as u32;
        let char_count = text.chars().count() as u32;
        
        // Average token is about 4 characters or 0.75 words
        let tokens_by_chars = char_count / 4;
        let tokens_by_words = (word_count as f32 * 1.3) as u32;
        
        tokens_by_chars.max(tokens_by_words)
    }

    fn calculate_cost(&self, provider: &str, model: &str, input_tokens: u32, output_tokens: u32) -> f64 {
        // Simplified cost calculation - in reality, you'd have a pricing table
        let (input_cost_per_1k, output_cost_per_1k) = match provider {
            "openai" => match model {
                "gpt-4" => (0.03, 0.06),
                "gpt-3.5-turbo" => (0.0015, 0.002),
                _ => (0.001, 0.002),
            },
            "claude" => match model {
                "claude-3-opus" => (0.015, 0.075),
                "claude-3-sonnet" => (0.003, 0.015),
                _ => (0.002, 0.01),
            },
            _ => (0.001, 0.002),
        };
        
        let input_cost = (input_tokens as f64 / 1000.0) * input_cost_per_1k;
        let output_cost = (output_tokens as f64 / 1000.0) * output_cost_per_1k;
        
        input_cost + output_cost
    }

    fn calculate_quality_score(&self, response: &str) -> Option<f64> {
        // Simple quality scoring based on various factors
        let mut score = 50.0; // Base score
        
        // Length factor (not too short, not too long)
        let length = response.len();
        if length > 50 && length < 2000 {
            score += 10.0;
        } else if length < 20 {
            score -= 20.0;
        } else if length > 5000 {
            score -= 10.0;
        }
        
        // Sentence structure (basic check)
        let sentence_count = response.matches(&['.', '!', '?'][..]).count();
        if sentence_count > 0 {
            score += 5.0;
        }
        
        // No repetitive content
        let words: Vec<&str> = response.split_whitespace().collect();
        if words.len() > 10 {
            let unique_words: std::collections::HashSet<_> = words.iter().collect();
            let uniqueness_ratio = unique_words.len() as f64 / words.len() as f64;
            score += uniqueness_ratio * 20.0;
        }
        
        Some(score.clamp(0.0, 100.0))
    }

    fn get_provider_for_model<'a>(
        &self,
        model: &str,
        api_clients: &'a HashMap<String, ApiClient>,
    ) -> Option<(String, &'a ApiClient)> {
        for (provider, client) in api_clients {
            // This is a simplified check - in reality, you'd have a more sophisticated mapping
            if model.starts_with("gpt") && provider == "openai" {
                return Some((provider.clone(), client));
            } else if model.starts_with("claude") && provider == "claude" {
                return Some((provider.clone(), client));
            } else if provider == "ollama" {
                return Some((provider.clone(), client));
            }
        }
        None
    }

    fn select_best_model(&self, results: &[BenchmarkResult]) -> Option<String> {
        if results.is_empty() {
            return None;
        }
        
        // Find model with best combined score (speed + quality + cost)
        let best_result = results.iter().min_by(|a, b| {
            let score_a = self.calculate_combined_score(a);
            let score_b = self.calculate_combined_score(b);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        best_result.map(|r| r.model.clone())
    }

    fn calculate_combined_score(&self, result: &BenchmarkResult) -> f64 {
        let speed_score = result.tokens_per_second;
        let quality_score = result.quality_score.unwrap_or(50.0);
        let cost_score = 100.0 / (result.cost_estimate + 1.0); // Lower cost = higher score
        
        // Weighted combination
        (speed_score * 0.3) + (quality_score * 0.5) + (cost_score * 0.2)
    }

    fn calculate_comparison_metrics(&self, results: &[BenchmarkResult]) -> HashMap<String, f64> {
        let mut metrics = HashMap::new();
        
        if results.is_empty() {
            return metrics;
        }
        
        let avg_response_time: f64 = results.iter().map(|r| r.response_time_ms as f64).sum::<f64>() / results.len() as f64;
        let avg_tokens_per_second: f64 = results.iter().map(|r| r.tokens_per_second).sum::<f64>() / results.len() as f64;
        let avg_cost: f64 = results.iter().map(|r| r.cost_estimate).sum::<f64>() / results.len() as f64;
        let avg_quality: f64 = results.iter().map(|r| r.quality_score.unwrap_or(0.0)).sum::<f64>() / results.len() as f64;
        
        metrics.insert("average_response_time_ms".to_string(), avg_response_time);
        metrics.insert("average_tokens_per_second".to_string(), avg_tokens_per_second);
        metrics.insert("average_cost".to_string(), avg_cost);
        metrics.insert("average_quality_score".to_string(), avg_quality);
        
        metrics
    }

    fn get_top_performing_models(&self) -> Vec<(String, f64)> {
        let mut model_scores: HashMap<String, Vec<f64>> = HashMap::new();
        
        for result in &self.benchmark_results {
            let score = self.calculate_combined_score(result);
            model_scores.entry(result.model.clone()).or_insert_with(Vec::new).push(score);
        }
        
        let mut avg_scores: Vec<(String, f64)> = model_scores
            .into_iter()
            .map(|(model, scores)| {
                let avg_score = scores.iter().sum::<f64>() / scores.len() as f64;
                (model, avg_score)
            })
            .collect();
        
        avg_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        avg_scores.truncate(5);
        
        avg_scores
    }

    fn generate_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if self.metrics.error_rate > 0.1 {
            recommendations.push("High error rate detected. Consider checking API configuration and network connectivity.".to_string());
        }
        
        if self.metrics.average_response_time > 5000.0 {
            recommendations.push("Slow response times. Consider using faster models or optimizing prompts.".to_string());
        }
        
        if self.metrics.total_cost > 100.0 {
            recommendations.push("High costs detected. Consider using more cost-effective models or implementing caching.".to_string());
        }
        
        if self.metrics.requests_per_minute > 60.0 {
            recommendations.push("High request rate. Consider implementing rate limiting or caching to avoid API limits.".to_string());
        }
        
        recommendations
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub metrics: PerformanceMetrics,
    pub uptime_hours: f64,
    pub top_performing_models: Vec<(String, f64)>,
    pub recommendations: Vec<String>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}