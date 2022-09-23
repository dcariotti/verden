use crate::{config::CONFIG, db::get_client, errors::AppError};
use chrono::{Local, NaiveDateTime};
use serde::{Deserialize, Serialize};
use sqlx::Row;

/// Model for warnings.
#[derive(Deserialize, Serialize, sqlx::FromRow)]
pub struct Warning {
    id: i32,
    user_id: Option<i32>,
    model_id: Option<i32>,
    resolved_by: Option<i32>,
    note: String,
    admin_note: String,
    created: NaiveDateTime,
    updated: NaiveDateTime,
}

/// Payload used to create a new warning
#[derive(Deserialize)]
pub struct WarningCreate {
    pub model_id: i32,
    pub note: String,
}

impl Warning {
    /// Create a warning means create an object which has an `user_id` (creator of the warning), a
    /// `model_id` (suspect model) and a `note`
    pub fn new(user_id: i32, model_id: i32, note: String) -> Self {
        let now = Local::now().naive_utc();
        Self {
            id: 0,
            user_id: Some(user_id),
            model_id: Some(model_id),
            resolved_by: None,
            note,
            admin_note: String::new(),
            created: now,
            updated: now,
        }
    }

    /// List all warnings. A staffer can see all the warnings, a user cannot
    pub async fn list(page: i64, user_id: Option<i32>) -> Result<Vec<Warning>, AppError> {
        let pool = unsafe { get_client() };
        let rows: Vec<Warning> = match user_id {
            Some(id) => {
                sqlx::query_as(
                    r#"
                    SELECT * FROM warnings WHERE user_id = $1
                    LIMIT $2 OFFSET $3
                    "#,
                )
                .bind(id)
                .bind(CONFIG.page_limit)
                .bind(CONFIG.page_limit * page)
                .fetch_all(pool)
                .await?
            }
            None => {
                sqlx::query_as(
                    r#"
                    SELECT * FROM warnings
                    LIMIT $1 OFFSET $2
                    "#,
                )
                .bind(CONFIG.page_limit)
                .bind(CONFIG.page_limit * page)
                .fetch_all(pool)
                .await?
            }
        };

        Ok(rows)
    }

    /// Return the number of warnings.
    pub async fn count(user_id: Option<i32>) -> Result<i64, AppError> {
        let pool = unsafe { get_client() };

        let cursor = match user_id {
            Some(id) => {
                sqlx::query(r#"SELECT COUNT(id) as count FROM warnings WHERE user_id = $1"#)
                    .bind(id)
                    .fetch_one(pool)
                    .await?
            }
            None => {
                sqlx::query(r#"SELECT COUNT(id) as count FROM warnings"#)
                    .fetch_one(pool)
                    .await?
            }
        };

        let count: i64 = cursor.try_get(0).unwrap();
        Ok(count)
    }

    /// Create a new upload for model
    pub async fn create(warning: Warning) -> Result<Warning, AppError> {
        let pool = unsafe { get_client() };

        let rec: Warning = sqlx::query_as(
            r#"
                INSERT INTO warnings (user_id, model_id, resolved_by, note, admin_note, created, updated)
                VALUES ( $1, $2, $3, $4, $5, $6, $7)
                RETURNING *
            "#,
        )
        .bind(warning.user_id)
        .bind(warning.model_id)
        .bind(warning.resolved_by)
        .bind(warning.note)
        .bind(warning.admin_note)
        .bind(warning.created)
        .bind(warning.updated)
        .fetch_one(pool)
        .await?;

        Ok(rec)
    }
}