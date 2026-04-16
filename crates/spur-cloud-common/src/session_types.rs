use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    Creating,
    Pending,
    Running,
    Stopping,
    Completed,
    Failed,
    Cancelled,
}

impl SessionState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Creating => "creating",
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Stopping => "stopping",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "creating" => Self::Creating,
            "pending" => Self::Pending,
            "running" => Self::Running,
            "stopping" => Self::Stopping,
            "completed" => Self::Completed,
            "failed" => Self::Failed,
            "cancelled" => Self::Cancelled,
            _ => Self::Failed,
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub id: Uuid,
    pub name: String,
    pub state: SessionState,
    pub gpu_type: String,
    pub gpu_count: i32,
    pub container_image: String,
    pub ssh_enabled: bool,
    pub ssh_host: Option<String>,
    pub ssh_port: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub node_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    pub name: String,
    pub gpu_type: String,
    #[serde(default = "default_gpu_count")]
    pub gpu_count: i32,
    pub container_image: String,
    #[serde(default)]
    pub ssh_enabled: bool,
    #[serde(default = "default_time_limit")]
    pub time_limit_min: i32,
    pub partition: Option<String>,
}

fn default_gpu_count() -> i32 {
    1
}

fn default_time_limit() -> i32 {
    240
}

#[cfg(test)]
mod tests {
    use super::SessionState;

    #[test]
    fn session_state_round_trips_through_strings() {
        let cases = [
            (SessionState::Creating, "creating"),
            (SessionState::Pending, "pending"),
            (SessionState::Running, "running"),
            (SessionState::Stopping, "stopping"),
            (SessionState::Completed, "completed"),
            (SessionState::Failed, "failed"),
            (SessionState::Cancelled, "cancelled"),
        ];

        for (state, raw) in cases {
            assert_eq!(state.as_str(), raw);
            assert_eq!(SessionState::from_str(raw), state);
        }
    }

    #[test]
    fn session_state_unknown_string_maps_to_failed() {
        assert_eq!(SessionState::from_str("unknown"), SessionState::Failed);
    }

    #[test]
    fn session_state_terminal_flag_matches_terminal_states() {
        assert!(!SessionState::Creating.is_terminal());
        assert!(!SessionState::Pending.is_terminal());
        assert!(!SessionState::Running.is_terminal());
        assert!(!SessionState::Stopping.is_terminal());
        assert!(SessionState::Completed.is_terminal());
        assert!(SessionState::Failed.is_terminal());
        assert!(SessionState::Cancelled.is_terminal());
    }
}
