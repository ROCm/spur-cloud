// Copyright (c) 2026 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use sqlx::PgPool;
use tonic::transport::Channel;

use spur_proto::proto::slurm_controller_client::SlurmControllerClient;

use crate::config::Config;

/// Shared application state, injected into all route handlers via axum State.
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub spur: SlurmControllerClient<Channel>,
    /// K8s client — None when running in native-host mode.
    pub kube: Option<kube::Client>,
    pub config: Arc<Config>,
}
