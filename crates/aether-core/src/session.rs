//! Session — Validation session management

use std::time::Instant;
use uuid::Uuid;

/// Unique identifier for a validation session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SessionId(Uuid);

impl SessionId {
    /// Generate a new unique session ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A validation session.
///
/// Each session represents a single validation run and tracks:
/// - Timing information
/// - Files being validated
/// - Accumulated results
#[derive(Debug, Clone)]
pub struct Session {
    id: SessionId,
    created_at: Instant,
}

impl Session {
    /// Create a new session.
    pub fn new() -> Self {
        Self {
            id: SessionId::new(),
            created_at: Instant::now(),
        }
    }

    /// Get the session ID.
    pub fn id(&self) -> SessionId {
        self.id
    }

    /// Get the elapsed time since session creation.
    pub fn elapsed(&self) -> std::time::Duration {
        self.created_at.elapsed()
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_id_uniqueness() {
        let id1 = SessionId::new();
        let id2 = SessionId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_session_creation() {
        let session = Session::new();
        // Just verify the session was created and elapsed() works
        std::thread::sleep(std::time::Duration::from_micros(1));
        let _elapsed = session.elapsed();
    }
}
