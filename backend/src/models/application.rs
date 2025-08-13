use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Application {
    pub id: i32,
    pub user_id: i32,
    #[serde(rename = "company_name")]
    pub company: String,
    pub job_url: Option<String>,
    #[serde(rename = "application_date")]
    pub applied_date: NaiveDate,
    pub status: ApplicationStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "application_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ApplicationStatus {
    Waiting,
    Rejected,
    NextStage,
    Ignored,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateApplicationRequest {
    #[validate(length(min = 1))]
    #[serde(rename = "company_name")]
    pub company: String,
    #[validate(url)]
    pub job_url: Option<String>,
    #[serde(rename = "application_date")]
    pub applied_date: NaiveDate,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateApplicationRequest {
    #[serde(rename = "company_name")]
    pub company: Option<String>,
    #[validate(url)]
    pub job_url: Option<String>,
    #[serde(rename = "application_date")]
    pub applied_date: Option<NaiveDate>,
    pub status: Option<ApplicationStatus>,
}

#[derive(Debug, Serialize)]
pub struct ApplicationResponse {
    pub id: i32,
    pub user_id: i32,
    #[serde(rename = "company_name")]
    pub company: String,
    pub job_url: Option<String>,
    #[serde(rename = "application_date")]
    pub applied_date: NaiveDate,
    pub status: ApplicationStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub screening: Option<crate::models::screening::ScreeningResponse>,
    pub interview: Option<crate::models::interview::InterviewResponse>,
}

impl From<Application> for ApplicationResponse {
    fn from(app: Application) -> Self {
        Self {
            id: app.id,
            user_id: app.user_id,
            company: app.company,
            job_url: app.job_url,
            applied_date: app.applied_date,
            status: app.status,
            created_at: app.created_at,
            updated_at: app.updated_at,
            screening: None,
            interview: None,
        }
    }
}
