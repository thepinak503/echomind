use crate::api::{ApiClient, ChatRequest, Message};
use crate::error::{EchomindError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: String,
    pub name: String,
    pub step_type: StepType,
    pub prompt: Option<String>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub conditions: Vec<Condition>,
    pub next_step: Option<String>,
    pub error_step: Option<String>,
    pub retry_count: u32,
    pub timeout: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepType {
    AIRequest,
    Conditional,
    Delay,
    Transform,
    Output,
    Input,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub variable: String,
    pub operator: ConditionOperator,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionOperator {
    Equals,
    NotEquals,
    Contains,
    StartsWith,
    EndsWith,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub steps: Vec<WorkflowStep>,
    pub variables: HashMap<String, serde_json::Value>,
    pub start_step: String,
}

#[derive(Debug)]
pub struct WorkflowContext {
    pub variables: HashMap<String, serde_json::Value>,
    pub current_step: String,
    pub history: Vec<StepResult>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct StepResult {
    pub step_id: String,
    pub success: bool,
    pub output: Option<String>,
    pub duration_ms: u64,
    pub error: Option<String>,
}

pub struct WorkflowManager {
    workflows: HashMap<String, Workflow>,
}

impl WorkflowManager {
    pub fn new() -> Self {
        Self {
            workflows: HashMap::new(),
        }
    }

    pub fn load_workflow_from_file(&mut self, file_path: &str) -> Result<()> {
        let contents = fs::read_to_string(file_path)
            .map_err(|e| EchomindError::FileError(format!("Failed to read workflow file: {}", e)))?;
        
        let workflow: Workflow = serde_json::from_str(&contents)
            .map_err(|e| EchomindError::ParseError(format!("Failed to parse workflow: {}", e)))?;
        
        self.workflows.insert(workflow.id.clone(), workflow);
        Ok(())
    }

    pub fn save_workflow_to_file(&self, workflow_id: &str, file_path: &str) -> Result<()> {
        let workflow = self.workflows.get(workflow_id)
            .ok_or_else(|| EchomindError::Other(format!("Workflow {} not found", workflow_id)))?;
        
        let json = serde_json::to_string_pretty(workflow)
            .map_err(|e| EchomindError::ParseError(format!("Failed to serialize workflow: {}", e)))?;
        
        fs::write(file_path, json)
            .map_err(|e| EchomindError::FileError(format!("Failed to write workflow file: {}", e)))?;
        
        Ok(())
    }

    pub async fn execute_workflow(
        &mut self,
        workflow_id: &str,
        initial_variables: HashMap<String, serde_json::Value>,
        api_client: &ApiClient,
    ) -> Result<WorkflowContext> {
        let workflow = self.workflows.get(workflow_id)
            .ok_or_else(|| EchomindError::Other(format!("Workflow {} not found", workflow_id)))?
            .clone();
        
        let mut context = WorkflowContext {
            variables: workflow.variables.clone(),
            current_step: workflow.start_step.clone(),
            history: Vec::new(),
            errors: Vec::new(),
        };
        
        // Add initial variables
        for (key, value) in initial_variables {
            context.variables.insert(key, value);
        }
        
        let mut max_iterations = 100; // Prevent infinite loops
        let mut iteration = 0;
        
        while !context.current_step.is_empty() && iteration < max_iterations {
            iteration += 1;
            
            if let Some(step) = workflow.steps.iter().find(|s| s.id == context.current_step) {
                let result = self.execute_step(step, &mut context, api_client).await?;
                context.history.push(result.clone());
                
                if !result.success {
                    if let Some(error_step) = &step.error_step {
                        context.current_step = error_step.clone();
                    } else {
                        break;
                    }
                } else {
                    // Check conditions and determine next step
                    let next_step = self.evaluate_conditions(&step.conditions, &context)?;
                    context.current_step = next_step.or(step.next_step.clone()).unwrap_or_default();
                }
            } else {
                return Err(EchomindError::Other(format!("Step {} not found", context.current_step)));
            }
        }
        
        if iteration >= max_iterations {
            return Err(EchomindError::Other("Workflow execution exceeded maximum iterations".to_string()));
        }
        
        Ok(context)
    }

    async fn execute_step(
        &self,
        step: &WorkflowStep,
        context: &mut WorkflowContext,
        api_client: &ApiClient,
    ) -> Result<StepResult> {
        let start_time = std::time::Instant::now();
        
        match &step.step_type {
            StepType::AIRequest => {
                let prompt = self.replace_variables(&step.prompt.as_ref().unwrap_or(&String::new()), &context.variables);
                
                let messages = vec![Message::text("user".to_string(), prompt)];
                let request = ChatRequest {
                    messages,
                    model: step.model.clone(),
                    temperature: step.temperature,
                    max_tokens: step.max_tokens,
                    stream: None,
                };
                
                match api_client.send_message(request).await {
                    Ok(response) => {
                        context.variables.insert("last_response".to_string(), serde_json::Value::String(response.clone()));
                        context.variables.insert(format!("step_{}_output", step.id), serde_json::Value::String(response.clone()));
                        
                        Ok(StepResult {
                            step_id: step.id.clone(),
                            success: true,
                            output: Some(response),
                            duration_ms: start_time.elapsed().as_millis() as u64,
                            error: None,
                        })
                    }
                    Err(e) => {
                        context.errors.push(format!("Step {} failed: {}", step.id, e));
                        Ok(StepResult {
                            step_id: step.id.clone(),
                            success: false,
                            output: None,
                            duration_ms: start_time.elapsed().as_millis() as u64,
                            error: Some(e.to_string()),
                        })
                    }
                }
            }
            StepType::Conditional => {
                // Conditional steps are handled in the main execution loop
                Ok(StepResult {
                    step_id: step.id.clone(),
                    success: true,
                    output: Some("Conditional evaluation".to_string()),
                    duration_ms: start_time.elapsed().as_millis() as u64,
                    error: None,
                })
            }
            StepType::Delay => {
                let delay_ms = context.variables.get("delay_ms")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(1000);
                sleep(Duration::from_millis(delay_ms)).await;
                
                Ok(StepResult {
                    step_id: step.id.clone(),
                    success: true,
                    output: Some(format!("Delayed for {}ms", delay_ms)),
                    duration_ms: start_time.elapsed().as_millis() as u64,
                    error: None,
                })
            }
            StepType::Transform => {
                // Transform data based on transformation rules
                let input = context.variables.get("input")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                
                let transformed = self.apply_transformation(input, &context.variables);
                context.variables.insert("output".to_string(), serde_json::Value::String(transformed.clone()));
                
                Ok(StepResult {
                    step_id: step.id.clone(),
                    success: true,
                    output: Some(transformed),
                    duration_ms: start_time.elapsed().as_millis() as u64,
                    error: None,
                })
            }
            StepType::Output => {
                let output = context.variables.get("output")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                
                println!("{}", output);
                
                Ok(StepResult {
                    step_id: step.id.clone(),
                    success: true,
                    output: Some(output.to_string()),
                    duration_ms: start_time.elapsed().as_millis() as u64,
                    error: None,
                })
            }
            StepType::Input => {
                // For now, just read from stdin
                use std::io::{self, Write};
                print!("Input required for step {}: ", step.name);
                io::stdout().flush().unwrap();
                
                let mut input = String::new();
                io::stdin().read_line(&mut input)
                    .map_err(|e| EchomindError::Other(format!("Failed to read input: {}", e)))?;
                
                let input = input.trim().to_string();
                context.variables.insert("input".to_string(), serde_json::Value::String(input.clone()));
                
                Ok(StepResult {
                    step_id: step.id.clone(),
                    success: true,
                    output: Some(input),
                    duration_ms: start_time.elapsed().as_millis() as u64,
                    error: None,
                })
            }
        }
    }

    fn evaluate_conditions(
        &self,
        conditions: &[Condition],
        context: &WorkflowContext,
    ) -> Result<Option<String>> {
        for condition in conditions {
            let variable_value = context.variables.get(&condition.variable);
            
            let condition_met = match (&condition.operator, variable_value, &condition.value) {
                (ConditionOperator::Equals, Some(var), val) => {
                    serde_json::to_value(var).unwrap() == *val
                }
                (ConditionOperator::NotEquals, Some(var), val) => {
                    serde_json::to_value(var).unwrap() != *val
                }
                (ConditionOperator::Contains, Some(serde_json::Value::String(var)), serde_json::Value::String(val)) => {
                    var.contains(val)
                }
                (ConditionOperator::StartsWith, Some(serde_json::Value::String(var)), serde_json::Value::String(val)) => {
                    var.starts_with(val)
                }
                (ConditionOperator::EndsWith, Some(serde_json::Value::String(var)), serde_json::Value::String(val)) => {
                    var.ends_with(val)
                }
                (ConditionOperator::GreaterThan, Some(var), val) => {
                    // This would need more sophisticated comparison
                    false
                }
                (ConditionOperator::LessThan, Some(var), val) => {
                    // This would need more sophisticated comparison
                    false
                }
                _ => false,
            };
            
            if condition_met {
                // Return the next step ID if condition is met
                // For now, we'll use a simple approach where the condition value contains the next step
                if let Some(serde_json::Value::String(next_step)) = &condition.value {
                    return Ok(Some(next_step.clone()));
                }
            }
        }
        
        Ok(None)
    }

    fn replace_variables(&self, template: &str, variables: &HashMap<String, serde_json::Value>) -> String {
        let mut result = template.to_string();
        
        for (key, value) in variables {
            if let Some(str_value) = value.as_str() {
                result = result.replace(&format!("{{{}}}", key), str_value);
            }
        }
        
        result
    }

    fn apply_transformation(&self, input: &str, variables: &HashMap<String, serde_json::Value>) -> String {
        let transform_type = variables.get("transform_type")
            .and_then(|v| v.as_str())
            .unwrap_or("uppercase");
        
        match transform_type {
            "uppercase" => input.to_uppercase(),
            "lowercase" => input.to_lowercase(),
            "reverse" => input.chars().rev().collect(),
            "trim" => input.trim().to_string(),
            _ => input.to_string(),
        }
    }

    pub fn create_workflow(&mut self, workflow: Workflow) {
        self.workflows.insert(workflow.id.clone(), workflow);
    }

    pub fn list_workflows(&self) -> Vec<&Workflow> {
        self.workflows.values().collect()
    }

    pub fn get_workflow(&self, workflow_id: &str) -> Option<&Workflow> {
        self.workflows.get(workflow_id)
    }

    pub fn delete_workflow(&mut self, workflow_id: &str) -> Result<()> {
        if self.workflows.remove(workflow_id).is_none() {
            return Err(EchomindError::Other(format!("Workflow {} not found", workflow_id)));
        }
        Ok(())
    }
}

impl Default for WorkflowManager {
    fn default() -> Self {
        Self::new()
    }
}