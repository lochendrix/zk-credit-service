// src/main.rs
use redis::AsyncCommands;
//use redis::Commands;
use zk_common::JobResult;
use zk_common::JobPayload;
use zk_core::generate_credit_score_proof;
use std::env;
use base64::Engine;
#[tokio::main]
async fn main() -> redis::RedisResult<()> {
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
    println!("Connecting to Redis at {}", redis_url);
    let redis_client = redis::Client::open(redis_url)?;
    let mut con = redis_client.get_multiplexed_tokio_connection().await?;
    println!("Worker connected to Redis. Waiting for jobs on queue 'zkp:jobs'...");

    loop {
        // Wait for and pop a job from the queue
        let job_data: Vec<String> = redis::cmd("BRPOP").arg("zkp:jobs").arg(0).query_async(&mut con).await?;
        let job_payload_str = &job_data[1];

        println!("\nReceived job: {}", job_payload_str);

        let parsed_job: Result<JobPayload, _> = serde_json::from_str(job_payload_str);

        match parsed_job {
            Ok(job) => {
                println!("Processing proof for user '{}'...", job.user_id);

                let proof_result = generate_credit_score_proof(job.score, job.threshold);

                // Create a JobResult struct based on the outcome
                let final_result = match proof_result {
                    Ok(zk_proof) => {
                        println!("Proof generated successfully. Encoding to Base64...");
                        // The to_bytes() function converts the crypto objects to a byte array,
                        // base64 engine then encodes those bytes into a string
                        let proof_b64 = base64::engine::general_purpose::STANDARD.encode(zk_proof.proof.to_bytes());
                        let commitment_b64 = base64::engine::general_purpose::STANDARD.encode(zk_proof.commitment.to_bytes());

                        JobResult {
                            status: "COMPLETED".to_string(),
                            error_message: None,
                            proof_b64: Some(proof_b64),
                            commitment_b64: Some(commitment_b64),
                        }
                    }
                    Err(e) => {
                        eprintln!("Error generating proof: {}", e);
                        JobResult {
                            status: "FAILED".to_string(),
                            error_message: Some(e),
                            proof_b64: None,
                            commitment_b64: None,
                        }
                    }
                };

                // Serialize the result to a JSON string
                let result_str = serde_json::to_string(&final_result).unwrap();
                let result_key = format!("zkp:result:{}", job.job_id);

                // Store the result in Redis with a 24-hour expiration
                println!("Storing result in Redis at key: {}", result_key);
                let _: () = con.set_ex(result_key, result_str, 86400).await?;
            }
            Err(e) => {
                eprintln!("Failed to parse job payload: {}", e);
            }
        }
    }
}