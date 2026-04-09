use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub role: String,
    pub model: String,
    pub capabilities: Vec<String>,
    pub current_task: Option<String>,
    pub status: AgentStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentStatus {
    Available,
    Busy,
    Offline,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub name: String,
    pub agents: HashMap<String, Agent>,
    pub workflow: Workflow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub steps: Vec<WorkflowStep>,
    pub current_step: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub name: String,
    pub required_roles: Vec<String>,
    pub timeout_secs: u64,
}

#[async_trait]
pub trait TeamCoordinator {
    async fn assign_task(&mut self, task: &str, required_roles: Vec<String>) -> Result<()>;
    async fn get_agent_status(&self, agent_id: &str) -> Result<AgentStatus>;
    async fn broadcast_message(&self, message: &str) -> Result<()>;
    async fn resolve_conflicts(&mut self) -> Result<()>;
}

pub struct DefaultTeamCoordinator {
    team: Team,
}

impl DefaultTeamCoordinator {
    pub fn new(team: Team) -> Self {
        Self { team }
    }
}

#[async_trait]
impl TeamCoordinator for DefaultTeamCoordinator {
    async fn assign_task(&mut self, task: &str, required_roles: Vec<String>) -> Result<()> {
        // Implémentation de l'assignation de tâches
        Ok(())
    }
    
    async fn get_agent_status(&self, agent_id: &str) -> Result<AgentStatus> {
        self.team.agents.get(agent_id)
            .map(|a| a.status.clone())
            .ok_or_else(|| anyhow::anyhow!("Agent not found"))
    }
    
    async fn broadcast_message(&self, message: &str) -> Result<()> {
        // Implémentation broadcast
        Ok(())
    }
    
    async fn resolve_conflicts(&mut self) -> Result<()> {
        // Résolution conflits entre agents
        Ok(())
    }
}
