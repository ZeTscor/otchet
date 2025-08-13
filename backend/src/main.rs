mod handlers;
mod middleware;
mod models;
mod services;
mod utils;

use axum::{
    extract::DefaultBodyLimit,
    http::{HeaderValue, Method},
    middleware::from_fn_with_state,
    routing::{get, post},
    Router,
};
use sqlx::PgPool;
use std::env;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    handlers::{admin, applications, auth, files, metrics, notifications},
    middleware::auth::auth_middleware,
    utils::database::create_pool,
};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub jwt_secret: String,
    pub upload_dir: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "job_tracker_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let upload_dir = env::var("UPLOAD_DIR").unwrap_or_else(|_| "./storage/uploads".to_string());

    // Create upload directory if it doesn't exist
    if let Err(e) = std::fs::create_dir_all(&upload_dir) {
        tracing::warn!("Failed to create upload directory {}: {}", upload_dir, e);
    }

    let db = create_pool(&database_url).await?;

    sqlx::migrate!("./migrations").run(&db).await?;

    let state = AppState {
        db,
        jwt_secret,
        upload_dir,
    };

    let cors_origin = env::var("CORS_ALLOWED_ORIGIN")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());
    
    let cors = if cors_origin == "*" {
        // Allow any origin for production flexibility
        CorsLayer::new()
            .allow_origin(axum::http::header::HeaderValue::from_static("*"))
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
            .allow_headers([
                axum::http::header::CONTENT_TYPE,
                axum::http::header::AUTHORIZATION,
            ])
    } else {
        // Specific origin
        CorsLayer::new()
            .allow_origin(cors_origin.parse::<HeaderValue>()?)
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
            .allow_headers([
                axum::http::header::CONTENT_TYPE,
                axum::http::header::AUTHORIZATION,
            ])
    };

    let protected_routes = Router::new()
        .route("/applications", get(applications::get_applications))
        .route("/applications", post(applications::create_application))
        .route("/applications/:id", get(applications::get_application))
        .route(
            "/applications/:id",
            axum::routing::put(applications::update_application),
        )
        .route(
            "/applications/:id",
            axum::routing::delete(applications::delete_application),
        )
        .route(
            "/applications/:id/screening",
            post(applications::upload_screening),
        )
        .route(
            "/applications/:id/interview",
            post(applications::upload_interview),
        )
        .route(
            "/applications/activity",
            get(applications::get_user_activity),
        )
        .route("/admin/analytics", get(admin::get_analytics))
        .route("/admin/students", get(admin::get_all_students))
        .route("/admin/applications", get(admin::get_all_applications))
        .route("/admin/activity", get(admin::get_admin_activity))
        .route(
            "/admin/users/:user_id/activity",
            get(admin::get_user_activity_admin),
        )
        .route("/admin/metrics", get(metrics::get_anonymous_metrics))
        .route("/admin/cache-stats", get(metrics::get_cache_stats))
        .route("/admin/cache-invalidate", post(metrics::invalidate_cache))
        .route("/admin/cache-warm", post(metrics::warm_cache))
        .route(
            "/admin/notifications/trigger",
            post(notifications::trigger_notifications),
        )
        .route("/admin/register", post(auth::register_admin))
        .route(
            "/notifications/stale",
            get(notifications::get_stale_applications),
        )
        .route("/files/:filename", get(files::serve_file))
        .layer(from_fn_with_state(state.clone(), auth_middleware));

    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/download/:filename", get(files::serve_file_with_token))
        .merge(protected_routes)
        .layer(cors)
        .layer(DefaultBodyLimit::max(
            env::var("MAX_REQUEST_BODY_MB")
                .unwrap_or_else(|_| "500".to_string())
                .parse::<usize>()
                .unwrap_or(500)
                * 1024
                * 1024,
        ))
        .with_state(state.clone());

    // Start background notification scheduler
    let notification_db = state.db.clone();
    tokio::spawn(async move {
        use crate::services::notification::NotificationService;
        use tokio_cron_scheduler::{Job, JobScheduler};

        let sched = JobScheduler::new()
            .await
            .expect("Failed to create scheduler");

        // Run notifications daily at 9 AM
        let job = Job::new_async("0 0 9 * * *", move |_uuid, _l| {
            let db = notification_db.clone();
            Box::pin(async move {
                let notification_service = NotificationService::new(db);
                if let Err(e) = notification_service.process_stale_notifications().await {
                    tracing::error!("Failed to process notifications: {}", e);
                } else {
                    tracing::info!("Daily notifications processed successfully");
                }
            })
        })
        .expect("Failed to create notification job");

        sched.add(job).await.expect("Failed to add job");
        sched.start().await.expect("Failed to start scheduler");

        tracing::info!("Notification scheduler started - running daily at 9 AM");

        // Keep the scheduler running
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
        }
    });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;
    tracing::info!("Server running on http://0.0.0.0:8000");

    axum::serve(listener, app).await?;

    Ok(())
}
