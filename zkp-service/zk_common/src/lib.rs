// zkp-service/zk_common/src/lib.rs
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct VerificationRequest {
    pub user_id: String,
    pub score: u64,
    pub threshold: u64,
}

#[derive(Serialize, Deserialize)]
pub struct JobPayload {
    pub job_id: String,
    pub user_id: String,
    pub score: u64,
    pub threshold: u64,
}

#[derive(Serialize, Deserialize)]
pub struct JobResult {
    pub status: String,
    pub error_message: Option<String>,
    // The proof and commitment, encoded as Base64 strings
    pub proof_b64: Option<String>,
    pub commitment_b64: Option<String>,
}
