use crate::utils::logger::LOGGER;
use chrono::{DateTime, Duration, Utc};
use serde::Serialize;
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Serialize, serde::Deserialize)]
pub struct TimeBasedMetrics {
    pub period: String,
    pub anonymous_statistics: AnonymousStatistics,
    pub trends: TrendAnalysis,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, serde::Deserialize)]
pub struct AnonymousStatistics {
    pub total_job_postings_analyzed: i64,
    pub unique_companies: i64,
    pub application_success_rate_percent: f64,
    pub average_response_time_days: f64,
    pub popular_job_domains: Vec<JobDomain>,
    pub geographical_distribution: HashMap<String, i64>, // City -> Count (anonymized)
    pub industry_breakdown: HashMap<String, i64>,
    pub temporal_patterns: TemporalPatterns,
}

#[derive(Debug, Serialize, serde::Deserialize)]
pub struct JobDomain {
    pub domain: String, // e.g., "tech", "finance", "healthcare" (anonymized)
    pub application_count: i64,
    pub success_rate: f64,
}

#[derive(Debug, Serialize, serde::Deserialize)]
pub struct TemporalPatterns {
    pub best_application_days: Vec<String>,    // Day of week
    pub seasonal_trends: HashMap<String, f64>, // Month -> relative activity
    pub peak_hours: Vec<i32>,                  // Hours when most applications are submitted
}

#[derive(Debug, Serialize, serde::Deserialize)]
pub struct TrendAnalysis {
    pub weekly_change_percent: f64,
    pub monthly_change_percent: f64,
    pub prediction_next_week: i64,
    pub anomalies_detected: Vec<String>,
}

#[derive(Debug)]
pub struct MetricsService {
    pool: PgPool,
}

#[derive(Debug)]
pub enum MetricsError {
    DatabaseError(String),
    CalculationError(String),
}

impl MetricsService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Generate anonymized time-based metrics for the specified period
    pub async fn generate_anonymous_metrics(
        &self,
        days_back: i32,
    ) -> Result<TimeBasedMetrics, MetricsError> {
        let start_time = Instant::now();

        LOGGER.log_business_event(
            "anonymous_metrics_generation_started",
            None,
            [(
                "period_days".to_string(),
                serde_json::Value::Number(serde_json::Number::from(days_back)),
            )]
            .iter()
            .cloned()
            .collect(),
        );

        let period = format!("last_{}_days", days_back);

        let (anonymous_stats, trends) = tokio::try_join!(
            self.calculate_anonymous_statistics(days_back),
            self.calculate_trend_analysis(days_back)
        )
        .map_err(|e| MetricsError::DatabaseError(e.to_string()))?;

        let metrics = TimeBasedMetrics {
            period,
            anonymous_statistics: anonymous_stats,
            trends,
            generated_at: Utc::now(),
        };

        let duration = start_time.elapsed();
        LOGGER.log_performance_metric(
            "anonymous_metrics_generation",
            duration.as_millis() as f64,
            HashMap::new(),
        );

        Ok(metrics)
    }

    async fn calculate_anonymous_statistics(
        &self,
        days_back: i32,
    ) -> Result<AnonymousStatistics, sqlx::Error> {
        let cutoff_date = Utc::now().naive_utc().date() - Duration::days(days_back as i64);

        // Basic counts with privacy protection
        let basic_stats = sqlx::query(
            "SELECT 
                COUNT(DISTINCT a.job_url) as total_postings,
                COUNT(DISTINCT a.company) as unique_companies,
                COUNT(DISTINCT a.id) as total_applications,
                COUNT(CASE WHEN i.result = 'passed' THEN 1 END) as successful_applications,
                AVG(EXTRACT(EPOCH FROM (s.screening_date - a.applied_date))/86400.0) as avg_response_days
             FROM applications a
             LEFT JOIN screenings s ON a.id = s.application_id
             LEFT JOIN interviews i ON a.id = i.application_id  
             WHERE a.applied_date >= $1"
        )
        .bind(cutoff_date)
        .fetch_one(&self.pool)
        .await?;

        let total_postings: i64 = basic_stats.get(0);
        let unique_companies: i64 = basic_stats.get(1);
        let total_applications: i64 = basic_stats.get(2);
        let successful_applications: i64 = basic_stats.get(3);
        let avg_response_days: Option<f64> = basic_stats.get(4);

        let success_rate = if total_applications > 0 {
            (successful_applications as f64 / total_applications as f64) * 100.0
        } else {
            0.0
        };

        // Get anonymized job domains based on company patterns
        let job_domains = self.calculate_job_domains(days_back).await?;

        // Calculate temporal patterns
        let temporal_patterns = self.calculate_temporal_patterns(days_back).await?;

        // Anonymized geographical distribution (use first letter of city + size bucket)
        let geo_distribution = self.calculate_anonymous_geography(days_back).await?;

        // Industry breakdown based on company keywords (anonymized)
        let industry_breakdown = self.calculate_industry_breakdown(days_back).await?;

        Ok(AnonymousStatistics {
            total_job_postings_analyzed: total_postings,
            unique_companies,
            application_success_rate_percent: success_rate,
            average_response_time_days: avg_response_days.unwrap_or(0.0),
            popular_job_domains: job_domains,
            geographical_distribution: geo_distribution,
            industry_breakdown,
            temporal_patterns,
        })
    }

    async fn calculate_job_domains(&self, days_back: i32) -> Result<Vec<JobDomain>, sqlx::Error> {
        let cutoff_date = Utc::now().naive_utc().date() - Duration::days(days_back as i64);

        // Analyze company names and job URLs to categorize into domains (anonymized)
        let domains = sqlx::query(
            "SELECT 
                CASE 
                    WHEN LOWER(company) LIKE '%tech%' OR LOWER(company) LIKE '%software%' 
                         OR LOWER(job_url) LIKE '%developer%' OR LOWER(job_url) LIKE '%engineer%'
                         THEN 'technology'
                    WHEN LOWER(company) LIKE '%bank%' OR LOWER(company) LIKE '%finance%' 
                         OR LOWER(job_url) LIKE '%financial%'
                         THEN 'finance'
                    WHEN LOWER(company) LIKE '%health%' OR LOWER(company) LIKE '%medical%'
                         OR LOWER(job_url) LIKE '%healthcare%'
                         THEN 'healthcare'
                    WHEN LOWER(company) LIKE '%retail%' OR LOWER(company) LIKE '%shop%'
                         OR LOWER(job_url) LIKE '%sales%'
                         THEN 'retail'
                    WHEN LOWER(company) LIKE '%consult%' OR LOWER(job_url) LIKE '%consult%'
                         THEN 'consulting'
                    ELSE 'other'
                END as domain,
                COUNT(*) as application_count,
                AVG(CASE WHEN i.result = 'passed' THEN 1.0 ELSE 0.0 END) * 100 as success_rate
             FROM applications a
             LEFT JOIN interviews i ON a.id = i.application_id
             WHERE a.applied_date >= $1
             GROUP BY domain
             ORDER BY application_count DESC
             LIMIT 10",
        )
        .bind(cutoff_date)
        .fetch_all(&self.pool)
        .await?;

        let mut job_domains = Vec::new();
        for row in domains {
            job_domains.push(JobDomain {
                domain: row.get(0),
                application_count: row.get(1),
                success_rate: row.get::<Option<f64>, _>(2).unwrap_or(0.0),
            });
        }

        Ok(job_domains)
    }

    async fn calculate_temporal_patterns(
        &self,
        days_back: i32,
    ) -> Result<TemporalPatterns, sqlx::Error> {
        let cutoff_date = Utc::now().naive_utc().date() - Duration::days(days_back as i64);

        // Best application days (day of week analysis)
        let day_analysis = sqlx::query(
            "SELECT 
                TO_CHAR(applied_date, 'Day') as day_name,
                COUNT(*) as applications,
                AVG(CASE WHEN i.result = 'passed' THEN 1.0 ELSE 0.0 END) as success_rate
             FROM applications a
             LEFT JOIN interviews i ON a.id = i.application_id
             WHERE a.applied_date >= $1
             GROUP BY EXTRACT(DOW FROM applied_date), TO_CHAR(applied_date, 'Day')
             ORDER BY success_rate DESC, applications DESC",
        )
        .bind(cutoff_date)
        .fetch_all(&self.pool)
        .await?;

        let best_days: Vec<String> = day_analysis
            .into_iter()
            .take(3)
            .map(|row| row.get::<String, _>(0).trim().to_string())
            .collect();

        // Seasonal trends (monthly analysis)
        let seasonal_data = sqlx::query(
            "SELECT 
                TO_CHAR(applied_date, 'Month') as month_name,
                COUNT(*) as applications
             FROM applications a
             WHERE a.applied_date >= $1
             GROUP BY EXTRACT(MONTH FROM applied_date), TO_CHAR(applied_date, 'Month')
             ORDER BY EXTRACT(MONTH FROM applied_date)",
        )
        .bind(cutoff_date)
        .fetch_all(&self.pool)
        .await?;

        let mut seasonal_trends = HashMap::new();
        let total_apps: i64 = seasonal_data.iter().map(|row| row.get::<i64, _>(1)).sum();
        for row in seasonal_data {
            let month: String = row.get::<String, _>(0).trim().to_string();
            let apps: i64 = row.get(1);
            let relative = if total_apps > 0 {
                apps as f64 / total_apps as f64
            } else {
                0.0
            };
            seasonal_trends.insert(month, relative);
        }

        // Peak hours (based on created_at timestamps)
        let hour_data = sqlx::query(
            "SELECT 
                EXTRACT(HOUR FROM created_at) as hour,
                COUNT(*) as applications
             FROM applications a  
             WHERE a.applied_date >= $1
             GROUP BY EXTRACT(HOUR FROM created_at)
             ORDER BY applications DESC
             LIMIT 3",
        )
        .bind(cutoff_date)
        .fetch_all(&self.pool)
        .await?;

        let peak_hours: Vec<i32> = hour_data
            .into_iter()
            .map(|row| row.get::<f64, _>(0) as i32)
            .collect();

        Ok(TemporalPatterns {
            best_application_days: best_days,
            seasonal_trends,
            peak_hours,
        })
    }

    async fn calculate_anonymous_geography(
        &self,
        days_back: i32,
    ) -> Result<HashMap<String, i64>, sqlx::Error> {
        let cutoff_date = Utc::now().naive_utc().date() - Duration::days(days_back as i32 as i64);

        // Mock geographical data based on company patterns (in real app, would use user location)
        let geo_data = sqlx::query(
            "SELECT 
                CASE 
                    WHEN LOWER(company) LIKE '%moscow%' OR LOWER(company) LIKE '%мск%' THEN 'Central-Large'
                    WHEN LOWER(company) LIKE '%spb%' OR LOWER(company) LIKE '%peter%' THEN 'Northwest-Large'  
                    WHEN LOWER(company) LIKE '%kazan%' OR LOWER(company) LIKE '%екб%' THEN 'Volga-Medium'
                    WHEN LOWER(company) LIKE '%remote%' OR LOWER(company) LIKE '%удален%' THEN 'Remote-All'
                    ELSE 'Other-Mixed'
                END as geo_region,
                COUNT(*) as count
             FROM applications a
             WHERE a.applied_date >= $1
             GROUP BY geo_region
             HAVING COUNT(*) >= 3" // Only show regions with sufficient data for anonymity
        )
        .bind(cutoff_date)
        .fetch_all(&self.pool)
        .await?;

        let mut geography = HashMap::new();
        for row in geo_data {
            geography.insert(row.get(0), row.get(1));
        }

        Ok(geography)
    }

    async fn calculate_industry_breakdown(
        &self,
        days_back: i32,
    ) -> Result<HashMap<String, i64>, sqlx::Error> {
        let cutoff_date = Utc::now().naive_utc().date() - Duration::days(days_back as i32 as i64);

        let industry_data = sqlx::query(
            "SELECT 
                CASE 
                    WHEN LOWER(company) LIKE '%tech%' OR LOWER(company) LIKE '%software%' 
                         OR LOWER(company) LIKE '%it%' THEN 'Technology'
                    WHEN LOWER(company) LIKE '%bank%' OR LOWER(company) LIKE '%financial%'
                         OR LOWER(company) LIKE '%insurance%' THEN 'Financial Services'  
                    WHEN LOWER(company) LIKE '%retail%' OR LOWER(company) LIKE '%ecommerce%'
                         OR LOWER(company) LIKE '%shop%' THEN 'Retail & E-commerce'
                    WHEN LOWER(company) LIKE '%health%' OR LOWER(company) LIKE '%medical%'
                         OR LOWER(company) LIKE '%pharma%' THEN 'Healthcare'
                    WHEN LOWER(company) LIKE '%consult%' OR LOWER(company) LIKE '%advisory%'
                         THEN 'Consulting'
                    WHEN LOWER(company) LIKE '%media%' OR LOWER(company) LIKE '%marketing%'
                         OR LOWER(company) LIKE '%advertising%' THEN 'Media & Marketing'
                    ELSE 'Other Industries'
                END as industry,
                COUNT(*) as count
             FROM applications a
             WHERE a.applied_date >= $1  
             GROUP BY industry
             HAVING COUNT(*) >= 2", // Minimum for anonymization
        )
        .bind(cutoff_date)
        .fetch_all(&self.pool)
        .await?;

        let mut industries = HashMap::new();
        for row in industry_data {
            industries.insert(row.get(0), row.get(1));
        }

        Ok(industries)
    }

    async fn calculate_trend_analysis(&self, days_back: i32) -> Result<TrendAnalysis, sqlx::Error> {
        let current_period = Utc::now().naive_utc().date() - Duration::days(days_back as i64);
        let prev_week = current_period - Duration::weeks(1);
        let prev_month = current_period - Duration::weeks(4);

        // Weekly comparison
        let weekly_data = sqlx::query(
            "SELECT 
                COUNT(CASE WHEN applied_date >= $1 THEN 1 END) as current_week,
                COUNT(CASE WHEN applied_date >= $2 AND applied_date < $1 THEN 1 END) as prev_week
             FROM applications WHERE applied_date >= $2",
        )
        .bind(current_period)
        .bind(prev_week)
        .fetch_one(&self.pool)
        .await?;

        let current_week: i64 = weekly_data.get(0);
        let previous_week: i64 = weekly_data.get(1);

        let weekly_change = if previous_week > 0 {
            ((current_week - previous_week) as f64 / previous_week as f64) * 100.0
        } else {
            0.0
        };

        // Monthly comparison
        let monthly_data = sqlx::query(
            "SELECT 
                COUNT(CASE WHEN applied_date >= $1 THEN 1 END) as current_month,
                COUNT(CASE WHEN applied_date >= $2 AND applied_date < $1 THEN 1 END) as prev_month
             FROM applications WHERE applied_date >= $2",
        )
        .bind(current_period)
        .bind(prev_month)
        .fetch_one(&self.pool)
        .await?;

        let current_month: i64 = monthly_data.get(0);
        let previous_month: i64 = monthly_data.get(1);

        let monthly_change = if previous_month > 0 {
            ((current_month - previous_month) as f64 / previous_month as f64) * 100.0
        } else {
            0.0
        };

        // Simple prediction based on trend
        let prediction = if weekly_change > 0.0 {
            (current_week as f64 * (1.0 + weekly_change / 100.0)) as i64
        } else {
            current_week
        };

        // Detect anomalies
        let mut anomalies = Vec::new();
        if weekly_change.abs() > 50.0 {
            anomalies.push(format!("Unusual weekly change: {:.1}%", weekly_change));
        }
        if monthly_change.abs() > 30.0 {
            anomalies.push(format!("Unusual monthly change: {:.1}%", monthly_change));
        }

        Ok(TrendAnalysis {
            weekly_change_percent: weekly_change,
            monthly_change_percent: monthly_change,
            prediction_next_week: prediction,
            anomalies_detected: anomalies,
        })
    }

    /// Get cached metrics or generate new ones
    pub async fn get_cached_metrics(
        &self,
        days_back: i32,
        cache_duration_minutes: i32,
    ) -> Result<TimeBasedMetrics, MetricsError> {
        let cache_key = format!("metrics_{}d", days_back);

        // Try to get from cache first
        if let Ok(cached) = self.get_from_cache(&cache_key).await {
            LOGGER.log_business_event(
                "metrics_cache_hit",
                None,
                [(
                    "cache_key".to_string(),
                    serde_json::Value::String(cache_key.clone()),
                )]
                .iter()
                .cloned()
                .collect(),
            );
            return Ok(cached);
        }

        // Generate new metrics
        let metrics = self.generate_anonymous_metrics(days_back).await?;

        // Cache the result
        self.cache_metrics(&cache_key, &metrics, cache_duration_minutes)
            .await?;

        Ok(metrics)
    }

    async fn get_from_cache(&self, key: &str) -> Result<TimeBasedMetrics, MetricsError> {
        let row =
            sqlx::query("SELECT value FROM cache_store WHERE key = $1 AND expires_at > NOW()")
                .bind(key)
                .fetch_one(&self.pool)
                .await
                .map_err(|_| MetricsError::DatabaseError("Cache miss".to_string()))?;

        let json_value: serde_json::Value = row.get(0);
        serde_json::from_value(json_value).map_err(|e| {
            MetricsError::CalculationError(format!("Cache deserialization error: {}", e))
        })
    }

    async fn cache_metrics(
        &self,
        key: &str,
        metrics: &TimeBasedMetrics,
        duration_minutes: i32,
    ) -> Result<(), MetricsError> {
        let expires_at = Utc::now() + Duration::minutes(duration_minutes as i64);
        let json_value = serde_json::to_value(metrics)
            .map_err(|e| MetricsError::CalculationError(format!("Serialization error: {}", e)))?;

        sqlx::query(
            "INSERT INTO cache_store (key, value, expires_at) 
             VALUES ($1, $2, $3) 
             ON CONFLICT (key) DO UPDATE SET value = $2, expires_at = $3",
        )
        .bind(key)
        .bind(json_value)
        .bind(expires_at)
        .execute(&self.pool)
        .await
        .map_err(|e| MetricsError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}
