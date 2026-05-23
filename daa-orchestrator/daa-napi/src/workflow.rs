//! Workflow engine bindings for NAPI
//!
//! Provides Node.js access to the DAA workflow engine for creating and executing
//! complex multi-step workflows.

use daa_orchestrator::workflow::{
    Workflow, WorkflowEngine, WorkflowResult, WorkflowStatus, WorkflowStep,
};
use daa_orchestrator::WorkflowConfig;
use napi::bindgen_prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Workflow configuration for Node.js
#[napi(object)]
#[derive(Debug, Clone)]
pub struct WorkflowConfigJs {
    /// Maximum workflow execution time in milliseconds
    pub max_execution_time_ms: Option<u32>,
    /// Maximum steps per workflow
    pub max_steps: Option<u32>,
    /// Whether parallel execution is enabled
    pub parallel_execution: Option<bool>,
}

/// Workflow definition for Node.js
#[napi(object)]
#[derive(Debug, Clone)]
pub struct WorkflowJs {
    /// Unique workflow identifier
    pub id: String,
    /// Workflow name
    pub name: String,
    /// Workflow steps
    pub steps: Vec<WorkflowStepJs>,
}

/// Workflow step definition for Node.js
#[napi(object)]
#[derive(Debug, Clone)]
pub struct WorkflowStepJs {
    /// Step identifier
    pub id: String,
    /// Step type
    pub step_type: String,
    /// Step parameters (JSON string)
    pub parameters: String,
}

/// Workflow execution result for Node.js
#[napi(object)]
pub struct WorkflowResultJs {
    /// Workflow identifier
    pub workflow_id: String,
    /// Execution status
    pub status: String,
    /// Step results
    pub results: Vec<StepResultJs>,
}

/// Step execution result for Node.js
#[napi(object)]
pub struct StepResultJs {
    /// Step identifier
    pub step_id: String,
    /// Step status
    pub status: String,
    /// Step output (JSON string)
    pub output: String,
}

impl WorkflowConfigJs {
    fn to_rust_config(&self) -> WorkflowConfig {
        WorkflowConfig {
            max_execution_time: self.max_execution_time_ms.unwrap_or(3600000) as u64 / 1000,
            max_steps: self.max_steps.unwrap_or(100) as usize,
            parallel_execution: self.parallel_execution.unwrap_or(true),
        }
    }
}

impl WorkflowJs {
    fn to_rust_workflow(&self) -> Result<Workflow> {
        let steps: Result<Vec<WorkflowStep>> = self
            .steps
            .iter()
            .map(|step| {
                let parameters: serde_json::Value = serde_json::from_str(&step.parameters)
                    .map_err(|e| Error::from_reason(format!("Invalid parameters JSON: {}", e)))?;

                Ok(WorkflowStep {
                    id: step.id.clone(),
                    step_type: step.step_type.clone(),
                    parameters,
                })
            })
            .collect();

        Ok(Workflow {
            id: self.id.clone(),
            name: self.name.clone(),
            steps: steps?,
        })
    }
}

impl WorkflowResultJs {
    fn from_rust_result(result: WorkflowResult) -> Self {
        let status = match result.status {
            WorkflowStatus::Running => "running",
            WorkflowStatus::Completed => "completed",
            WorkflowStatus::Failed => "failed",
        };

        let results: Vec<StepResultJs> = result
            .results
            .into_iter()
            .map(|r| StepResultJs {
                step_id: r.step_id,
                status: r.status,
                output: r.output.to_string(),
            })
            .collect();

        Self {
            workflow_id: result.workflow_id,
            status: status.to_string(),
            results,
        }
    }
}

/// Workflow engine for creating and executing workflows
#[napi]
pub struct WorkflowEngineWrapper {
    engine: Arc<Mutex<WorkflowEngine>>,
}

#[napi]
impl WorkflowEngineWrapper {
    /// Create a new workflow engine instance.
    ///
    /// # Arguments
    ///
    /// * `config` - Workflow engine configuration
    ///
    /// # Returns
    ///
    /// A new WorkflowEngine instance
    ///
    /// # Example
    ///
    /// ```javascript
    /// const workflowEngine = new WorkflowEngine({
    ///   maxExecutionTimeMs: 3600000,  // 1 hour
    ///   maxSteps: 100,
    ///   parallelExecution: true
    /// });
    /// ```
    #[napi(constructor)]
    pub fn new(config: Option<WorkflowConfigJs>) -> Result<Self> {
        let rust_config = config.map(|c| c.to_rust_config()).unwrap_or_default();

        let engine = WorkflowEngine::new(rust_config);

        Ok(Self {
            engine: Arc::new(Mutex::new(engine)),
        })
    }

    /// Create a new workflow definition.
    ///
    /// # Arguments
    ///
    /// * `workflow` - Workflow definition with id, name, and steps
    ///
    /// # Returns
    ///
    /// The workflow ID
    ///
    /// # Example
    ///
    /// ```javascript
    /// const workflowId = await workflowEngine.createWorkflow({
    ///   id: 'workflow-1',
    ///   name: 'Data Processing Pipeline',
    ///   steps: [
    ///     {
    ///       id: 'step-1',
    ///       stepType: 'fetch_data',
    ///       parameters: JSON.stringify({ source: 'api' })
    ///     },
    ///     {
    ///       id: 'step-2',
    ///       stepType: 'transform',
    ///       parameters: JSON.stringify({ operation: 'normalize' })
    ///     },
    ///     {
    ///       id: 'step-3',
    ///       stepType: 'store',
    ///       parameters: JSON.stringify({ destination: 'database' })
    ///     }
    ///   ]
    /// });
    /// ```
    #[napi]
    pub async fn create_workflow(&self, workflow: WorkflowJs) -> Result<String> {
        let rust_workflow = workflow.to_rust_workflow()?;
        let workflow_id = rust_workflow.id.clone();

        // In a real implementation, we would store this workflow
        // For now, just validate and return the ID

        Ok(workflow_id)
    }

    /// Execute a workflow.
    ///
    /// # Arguments
    ///
    /// * `workflow` - Workflow to execute
    ///
    /// # Returns
    ///
    /// Workflow execution result with status and step results
    ///
    /// # Example
    ///
    /// ```javascript
    /// const result = await workflowEngine.executeWorkflow({
    ///   id: 'workflow-1',
    ///   name: 'Data Processing',
    ///   steps: [...]
    /// });
    ///
    /// console.log('Status:', result.status);
    /// result.results.forEach(stepResult => {
    ///   console.log(`Step ${stepResult.stepId}:`, stepResult.status);
    /// });
    /// ```
    #[napi]
    pub async fn execute_workflow(&self, workflow: WorkflowJs) -> Result<WorkflowResultJs> {
        let rust_workflow = workflow.to_rust_workflow()?;
        let engine = self.engine.lock().await;

        let result = engine
            .execute(rust_workflow)
            .await
            .map_err(|e| Error::from_reason(format!("Workflow execution failed: {}", e)))?;

        Ok(WorkflowResultJs::from_rust_result(result))
    }

    /// Get the number of active workflows.
    ///
    /// # Returns
    ///
    /// Count of currently executing workflows
    ///
    /// # Example
    ///
    /// ```javascript
    /// const activeCount = await workflowEngine.getActiveCount();
    /// console.log('Active workflows:', activeCount);
    /// ```
    #[napi]
    pub async fn get_active_count(&self) -> Result<u32> {
        let engine = self.engine.lock().await;
        let count = engine.get_active_count().await;
        Ok(count as u32)
    }

    /// Start the workflow engine.
    ///
    /// # Returns
    ///
    /// Promise that resolves when the engine is started
    ///
    /// # Example
    ///
    /// ```javascript
    /// await workflowEngine.start();
    /// console.log('Workflow engine started');
    /// ```
    #[napi]
    pub async fn start(&self) -> Result<()> {
        let mut engine = self.engine.lock().await;
        engine
            .start()
            .await
            .map_err(|e| Error::from_reason(format!("Failed to start workflow engine: {}", e)))?;
        Ok(())
    }

    /// Validate a workflow definition.
    ///
    /// # Arguments
    ///
    /// * `workflow` - Workflow to validate
    ///
    /// # Returns
    ///
    /// true if the workflow is valid, throws error otherwise
    ///
    /// # Example
    ///
    /// ```javascript
    /// try {
    ///   const isValid = await workflowEngine.validateWorkflow({
    ///     id: 'workflow-1',
    ///     name: 'Test Workflow',
    ///     steps: [...]
    ///   });
    ///   console.log('Workflow is valid');
    /// } catch (error) {
    ///   console.error('Invalid workflow:', error.message);
    /// }
    /// ```
    #[napi]
    pub async fn validate_workflow(&self, workflow: WorkflowJs) -> Result<bool> {
        // Basic validation
        if workflow.id.is_empty() {
            return Err(Error::from_reason("Workflow ID cannot be empty"));
        }

        if workflow.name.is_empty() {
            return Err(Error::from_reason("Workflow name cannot be empty"));
        }

        if workflow.steps.is_empty() {
            return Err(Error::from_reason("Workflow must have at least one step"));
        }

        // Validate each step
        for step in &workflow.steps {
            if step.id.is_empty() {
                return Err(Error::from_reason("Step ID cannot be empty"));
            }

            if step.step_type.is_empty() {
                return Err(Error::from_reason("Step type cannot be empty"));
            }

            // Validate parameters JSON
            serde_json::from_str::<serde_json::Value>(&step.parameters)
                .map_err(|e| Error::from_reason(format!("Invalid step parameters JSON: {}", e)))?;
        }

        Ok(true)
    }
}
