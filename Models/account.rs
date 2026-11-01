// Copyright (C) 2026 slatkisasa
// SPDX-License-Identifier: AGPL-3.0-only
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Account {
    pub id: Uuid,
    pub display_name: String,
    pub account_identifier: String,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}
impl Account {
    pub fn new(identifier: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            display_name: identifier.split('@').next().unwrap_or(&identifier).into(),
            account_identifier: identifier,
            created_at: Utc::now(),
            last_used_at: None,
        }
    }
}
