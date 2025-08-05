// zkp-service/zk_api/src/main.rs
use axum::{extract::{Path, State}, http::StatusCode, routing::{get, post}, Json, Router};
use redis::Commands;
use std::sync::Arc;
use uuid::Uuid;
use zk_common::{JobPayload, JobResult, VerificationRequest};
use std::env;

type AppState = Arc<redis::Client>;
#[tokio::main]
async fn main() {
    // Read the REDIS_URL from the environment, defaulting to localhost for local runs
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
    println!("Connecting to Redis at {}", redis_url);

    let redis_client = Arc::new(redis::Client::open(redis_url).unwrap());
    let app = Router::new()
        .route("/v1/verifications", post(create_verification))
        .route("/v1/verifications/:job_id", get(get_verification_status))
        .with_state(redis_client);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("API server listening on port 3000");
    axum::serve(listener, app).await.unwrap();
}

// The handler to create a new verification job
async fn create_verification(
    State(state): State<AppState>,
    Json(request): Json<VerificationRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let job_id = Uuid::new_v4().to_string();
    let job_to_queue = JobPayload {
        job_id: job_id.clone(),
        user_id: request.user_id,
        score: request.score,
        threshold: request.threshold,
    };

    let mut con = state.get_connection().unwrap();
    // Serialize the full job payload to a string
    let job_str = serde_json::to_string(&job_to_queue).unwrap();

    // Push the job to the Redis queue for the worker
    let _: () = con.lpush("zkp:jobs", job_str).unwrap();

    // Return only the job_id to the client.
    let response = serde_json::json!({ "job_id": job_id });
    (StatusCode::ACCEPTED, Json(response))
}

// The handler to get the status/result of a job
async fn get_verification_status(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    let mut con = state.get_connection().unwrap();
    let result_key = format!("zkp:result:{}", job_id);
    let result: redis::RedisResult<Option<String>> = con.get(result_key);

    match result {
        // Case 1: Redis command succeeded
        Ok(Some(result_str)) => {
            let job_result: JobResult = serde_json::from_str(&result_str).unwrap();
            (StatusCode::OK, Json(serde_json::to_value(job_result).unwrap()))
        }
        Ok(None) => {
            // Case 2: The key did not exist
            let response = serde_json::json!({
                "error": "Not Found",
                "message": "No result found for this job_id. It may still be processing or the ID is invalid."
            });
            (StatusCode::NOT_FOUND, Json(response))
        }
        // Case 3: A Redis error occurred
        Err(_) => {
            let response = serde_json::json!({ "error": "Internal Server Error" });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
        }
    }
}