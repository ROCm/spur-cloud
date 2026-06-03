// Copyright (c) 2026 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

use uuid::Uuid;

/// Authenticated caller normalized at the application boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Principal {
    pub user_id: Uuid,
    pub email: String,
    pub username: String,
    pub is_admin: bool,
}
