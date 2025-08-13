use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Interview {
    pub id: i32,
    pub application_id: i32,
    pub file_path: Option<String>,
    pub interview_date: Option<NaiveDate>,
    pub result: Option<InterviewResult>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "interview_result", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum InterviewResult {
    Passed,
    Failed,
}

#[derive(Debug, Deserialize)]
pub struct UpdateInterviewRequest {
    pub interview_date: Option<NaiveDate>,
    #[serde(rename = "interview_status")]
    pub result: Option<InterviewResult>,
}

#[derive(Debug, Serialize)]
pub struct InterviewResponse {
    pub id: i32,
    pub application_id: i32,
    pub file_path: Option<String>,
    pub interview_date: Option<NaiveDate>,
    pub result: Option<InterviewResult>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Interview> for InterviewResponse {
    fn from(interview: Interview) -> Self {
        Self {
            id: interview.id,
            application_id: interview.application_id,
            file_path: interview.file_path,
            interview_date: interview.interview_date,
            result: interview.result,
            created_at: interview.created_at,
            updated_at: interview.updated_at,
        }
    }
}
