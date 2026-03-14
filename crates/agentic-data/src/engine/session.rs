//! Session management — isolate data operations per user/agent session.
//!
//! Parity with AgenticMemory's session lifecycle management.

use std::collections::HashMap;
use crate::types::*;

/// A data session — tracks operations, provides isolation.
#[derive(Debug, Clone)]
pub struct DataSession {
    pub id: String,
    pub user: String,
    pub started_at: u64,
    pub ended_at: Option<u64>,
    pub operations: Vec<SessionOp>,
    pub sources_touched: Vec<String>,
    pub records_created: u64,
    pub records_modified: u64,
    pub queries_executed: u64,
}

/// A recorded operation within a session.
#[derive(Debug, Clone)]
pub struct SessionOp {
    pub op_type: OpType,
    pub target: String,
    pub timestamp: u64,
    pub details: String,
}

/// Types of session operations.
#[derive(Debug, Clone, PartialEq)]
pub enum OpType {
    Ingest,
    Query,
    Transform,
    QualityCheck,
    PiiScan,
    Export,
    SchemaInfer,
}

/// Session manager — tracks active and completed sessions.
#[derive(Debug, Default)]
pub struct SessionManager {
    active: HashMap<String, DataSession>,
    completed: Vec<DataSession>,
}

impl SessionManager {
    pub fn new() -> Self { Self::default() }

    /// Start a new session.
    pub fn start(&mut self, user: &str) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let session = DataSession {
            id: id.clone(),
            user: user.to_string(),
            started_at: now_micros(),
            ended_at: None,
            operations: Vec::new(),
            sources_touched: Vec::new(),
            records_created: 0,
            records_modified: 0,
            queries_executed: 0,
        };
        self.active.insert(id.clone(), session);
        id
    }

    /// Record an operation in a session.
    pub fn record_op(&mut self, session_id: &str, op_type: OpType, target: &str, details: &str) {
        if let Some(session) = self.active.get_mut(session_id) {
            session.operations.push(SessionOp {
                op_type: op_type.clone(),
                target: target.to_string(),
                timestamp: now_micros(),
                details: details.to_string(),
            });
            match op_type {
                OpType::Ingest => session.records_created += 1,
                OpType::Query => session.queries_executed += 1,
                _ => {}
            }
            if !session.sources_touched.contains(&target.to_string()) {
                session.sources_touched.push(target.to_string());
            }
        }
    }

    /// End a session.
    pub fn end(&mut self, session_id: &str) -> Option<&DataSession> {
        if let Some(mut session) = self.active.remove(session_id) {
            session.ended_at = Some(now_micros());
            self.completed.push(session);
            self.completed.last()
        } else {
            None
        }
    }

    /// Get an active session.
    pub fn get_active(&self, session_id: &str) -> Option<&DataSession> {
        self.active.get(session_id)
    }

    /// List all active session IDs.
    pub fn active_sessions(&self) -> Vec<&str> {
        self.active.keys().map(|s| s.as_str()).collect()
    }

    /// Count of active sessions.
    pub fn active_count(&self) -> usize { self.active.len() }

    /// Count of completed sessions.
    pub fn completed_count(&self) -> usize { self.completed.len() }

    /// Get session history for a user.
    pub fn user_history(&self, user: &str) -> Vec<&DataSession> {
        self.completed.iter().filter(|s| s.user == user).collect()
    }

    /// Resume context: what was the user doing last?
    pub fn resume_context(&self, user: &str) -> Option<String> {
        let history = self.user_history(user);
        let last = history.last()?;
        let last_ops: Vec<String> = last.operations.iter().rev().take(3)
            .map(|op| format!("{:?} on {}", op.op_type, op.target))
            .collect();
        Some(format!("Last session: {} operations ({} sources). Recent: {}",
            last.operations.len(), last.sources_touched.len(), last_ops.join(", ")))
    }
}

impl DataSession {
    /// Duration in milliseconds (0 if still active).
    pub fn duration_ms(&self) -> u64 {
        let end = self.ended_at.unwrap_or_else(now_micros);
        (end - self.started_at) / 1000
    }

    /// Summary string.
    pub fn summary(&self) -> String {
        format!("{}: {} ops, {} sources, {} records created, {} queries",
            self.user, self.operations.len(), self.sources_touched.len(),
            self.records_created, self.queries_executed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_lifecycle() {
        let mut mgr = SessionManager::new();
        let sid = mgr.start("alice");
        assert_eq!(mgr.active_count(), 1);

        mgr.record_op(&sid, OpType::Ingest, "data.csv", "Ingested 100 records");
        mgr.record_op(&sid, OpType::Query, "data.csv", "Filtered by name");
        mgr.record_op(&sid, OpType::QualityCheck, "data.csv", "Score: 85");

        let session = mgr.get_active(&sid).unwrap();
        assert_eq!(session.operations.len(), 3);
        assert_eq!(session.records_created, 1);
        assert_eq!(session.queries_executed, 1);

        mgr.end(&sid);
        assert_eq!(mgr.active_count(), 0);
        assert_eq!(mgr.completed_count(), 1);
    }

    #[test]
    fn test_resume_context() {
        let mut mgr = SessionManager::new();
        let sid = mgr.start("bob");
        mgr.record_op(&sid, OpType::Ingest, "sales.csv", "Loaded");
        mgr.record_op(&sid, OpType::PiiScan, "sales.csv", "3 PII found");
        mgr.end(&sid);

        let ctx = mgr.resume_context("bob").unwrap();
        assert!(ctx.contains("2 operations"));
        assert!(ctx.contains("sales.csv"));
    }

    #[test]
    fn test_user_history() {
        let mut mgr = SessionManager::new();
        let s1 = mgr.start("alice");
        mgr.end(&s1);
        let s2 = mgr.start("alice");
        mgr.end(&s2);
        let s3 = mgr.start("bob");
        mgr.end(&s3);

        assert_eq!(mgr.user_history("alice").len(), 2);
        assert_eq!(mgr.user_history("bob").len(), 1);
    }

    #[test]
    fn test_session_summary() {
        let mut mgr = SessionManager::new();
        let sid = mgr.start("test");
        mgr.record_op(&sid, OpType::Ingest, "x", "");
        let session = mgr.get_active(&sid).unwrap();
        assert!(session.summary().contains("test"));
    }
}
