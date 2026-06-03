// Copyright (c) 2026 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use spur_cloud_common::session_types::SessionState;

/// Fields for inserting a new session row.
pub struct NewSession<'a> {
    pub user_id: Uuid,
    pub name: &'a str,
    pub gpu_type: &'a str,
    pub gpu_count: i32,
    pub container_image: &'a str,
    pub partition: Option<&'a str>,
    pub ssh_enabled: bool,
    pub time_limit_min: i32,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub spur_job_id: Option<i32>,
    pub state: String,
    pub gpu_type: String,
    pub gpu_count: i32,
    pub container_image: String,
    pub partition: Option<String>,
    pub ssh_enabled: bool,
    pub ssh_port: Option<i32>,
    pub ssh_host: Option<String>,
    pub time_limit_min: i32,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub node_name: Option<String>,
    pub pod_name: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionDetail {
    pub id: Uuid,
    pub name: String,
    pub state: SessionState,
    pub gpu_type: String,
    pub gpu_count: i32,
    pub container_image: String,
    pub partition: Option<String>,
    pub ssh_enabled: bool,
    pub ssh_host: Option<String>,
    pub ssh_port: Option<i32>,
    pub spur_job_id: Option<i32>,
    pub time_limit_min: i32,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub node_name: Option<String>,
    pub pod_name: Option<String>,
    pub error_message: Option<String>,
}

impl From<Session> for SessionDetail {
    fn from(s: Session) -> Self {
        Self {
            id: s.id,
            name: s.name,
            state: SessionState::parse(&s.state),
            gpu_type: s.gpu_type,
            gpu_count: s.gpu_count,
            container_image: s.container_image,
            partition: s.partition,
            ssh_enabled: s.ssh_enabled,
            ssh_host: s.ssh_host,
            ssh_port: s.ssh_port,
            spur_job_id: s.spur_job_id,
            time_limit_min: s.time_limit_min,
            created_at: s.created_at,
            started_at: s.started_at,
            ended_at: s.ended_at,
            node_name: s.node_name,
            pod_name: s.pod_name,
            error_message: s.error_message,
        }
    }
}
