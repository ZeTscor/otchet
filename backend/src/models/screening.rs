use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Screening {
    pub id: i32,
    pub application_id: i32,
    pub file_path: Option<String>,
    pub screening_date: Option<NaiveDate>,
    pub result: Option<ScreeningResult>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "screening_result", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ScreeningResult {
    Passed,
    Failed,
}

#[derive(Debug, Deserialize)]
pub struct UpdateScreeningRequest {
    pub screening_date: Option<NaiveDate>,
    #[serde(rename = "screening_status")]
    pub result: Option<ScreeningResult>,
}

#[derive(Debug, Serialize)]
pub struct ScreeningResponse {
    pub id: i32,
    pub application_id: i32,
    pub file_path: Option<String>,
    pub screening_date: Option<NaiveDate>,
    pub result: Option<ScreeningResult>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Screening> for ScreeningResponse {
    fn from(screening: Screening) -> Self {
        Self {
            id: screening.id,
            application_id: screening.application_id,
            file_path: screening.file_path,
            screening_date: screening.screening_date,
            result: screening.result,
            created_at: screening.created_at,
            updated_at: screening.updated_at,
        }
    }
}
