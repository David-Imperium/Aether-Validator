//! Orchestrator — Main coordination point for Synward validation

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::session::{Session, SessionId};
use crate::pipeline::Pipeline;
use crate::config::Config;
use crate::error::Result;

/// Main orchestrator for Synward validation system.
///
/// The Orchestrator manages:
/// - Session lifecycle (creation, tracking, cleanup)
/// - Pipeline execution coordination
/// - Result aggregation
pub struct Orchestrator {
    sessions: Arc<RwLock<HashMap<SessionId, Session>>>,
    pipeline: Pipeline,
    #[allow(dead_code)]
    config: Config,
}

impl Orchestrator {
    /// Create a new Orchestrator with the given configuration.
    pub fn new(config: Config) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            pipeline: Pipeline::new(&config),
            config,
        }
    }

    /// Create a new validation session.
    pub async fn create_session(&self) -> SessionId {
        let session = Session::new();
        let id = session.id();
        
        let mut sessions = self.sessions.write().await;
        sessions.insert(id, session);
        
        id
    }

    /// Get a session by ID.
    pub async fn get_session(&self, id: SessionId) -> Option<Session> {
        let sessions = self.sessions.read().await;
        sessions.get(&id).cloned()
    }

    /// Execute validation pipeline for a session.
    pub async fn validate(&self, source: &str) -> Result<SessionId> {
        let session_id = self.create_session().await;
        self.pipeline.execute(source).await?;
        Ok(session_id)
    }
}

impl Default for Orchestrator {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_session() {
        let orchestrator = Orchestrator::default();
        let id = orchestrator.create_session().await;
        assert!(orchestrator.get_session(id).await.is_some());
    }
}
