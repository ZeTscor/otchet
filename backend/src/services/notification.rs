use anyhow::Result;
use chrono::{Duration, Utc};
use sqlx::PgPool;

use crate::models::{application::Application, user::User};

pub struct NotificationService {
    pub db: PgPool,
}

impl NotificationService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn find_stale_applications(&self, days: i32) -> Result<Vec<Application>> {
        let cutoff_date = Utc::now() - Duration::days(days as i64);

        let results = sqlx::query_as::<_, Application>(
            r#"
            SELECT * FROM applications
            WHERE updated_at < $1 
            AND status IN ('waiting', 'next_stage')
            ORDER BY updated_at ASC
            "#,
        )
        .bind(cutoff_date)
        .fetch_all(&self.db)
        .await?;

        Ok(results)
    }

    pub async fn send_notification(
        &self,
        user_email: &str,
        applications: &[Application],
    ) -> Result<()> {
        // In a real implementation, this would send emails or push notifications
        // For now, we'll just log the notification
        tracing::info!(
            "Notification: User {} has {} stale applications: {:?}",
            user_email,
            applications.len(),
            applications.iter().map(|a| &a.company).collect::<Vec<_>>()
        );

        // TODO: Implement actual email/notification sending
        // This could use services like SendGrid, AWS SES, or a notification service

        Ok(())
    }

    pub async fn process_stale_notifications(&self) -> Result<()> {
        self.process_stale_notifications_with_days(7).await
    }

    pub async fn process_stale_notifications_with_days(&self, days: i32) -> Result<()> {
        let stale_applications = self.find_stale_applications(days).await?;

        // Group applications by user_id
        let mut user_applications: std::collections::HashMap<i32, Vec<Application>> =
            std::collections::HashMap::new();

        for application in stale_applications {
            user_applications
                .entry(application.user_id)
                .or_insert_with(Vec::new)
                .push(application);
        }

        // Send notifications to each user
        for (user_id, applications) in user_applications {
            let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
                .bind(user_id)
                .fetch_one(&self.db)
                .await?;

            if let Err(e) = self.send_notification(&user.email, &applications).await {
                tracing::error!("Failed to send notification to {}: {}", user.email, e);
            }
        }

        Ok(())
    }

    pub async fn find_user_stale_applications(
        &self,
        user_id: i32,
        days: i32,
    ) -> Result<Vec<Application>> {
        let cutoff_date = Utc::now() - Duration::days(days as i64);

        let results = sqlx::query_as::<_, Application>(
            r#"
            SELECT * FROM applications
            WHERE user_id = $1 
            AND updated_at < $2 
            AND status IN ('waiting', 'next_stage')
            ORDER BY updated_at ASC
            "#,
        )
        .bind(user_id)
        .bind(cutoff_date)
        .fetch_all(&self.db)
        .await?;

        Ok(results)
    }
}
