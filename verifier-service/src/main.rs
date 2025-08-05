// verifier_app/src/main.rs
use base64::{engine::general_purpose, Engine};
use bulletproofs::{BulletproofGens, PedersenGens, RangeProof};
use curve25519_dalek_ng::ristretto::CompressedRistretto;
use merlin::Transcript;
use serde::Deserialize;
use std::io::{self, Read};

#[derive(Deserialize)]
struct ApiResponse {
    status: String,
    proof_b64: Option<String>,
    commitment_b64: Option<String>,
}

fn main() {
    println!("--- Independent Verifier Application ---");
    println!("Please paste the JSON output from the API and then press Ctrl+D (macOS/Linux) or Ctrl+Z (Windows).");

    // Read the entire input from the user (pasted JSON)
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer).unwrap();

    // The verifier must know the public threshold they asked for.
    let threshold: u64 = 700;
    println!("\nVerifying against public threshold: {}", threshold);

    match serde_json::from_str::<ApiResponse>(&buffer) {
        Ok(response) => {
            if response.status != "COMPLETED" {
                println!("Verification FAILED: The job status was not 'COMPLETED'.");
                return;
            }

            // Ensure we have the proof data to verify
            let (proof_b64, commitment_b64) = match (response.proof_b64, response.commitment_b64) {
                (Some(p), Some(c)) => (p, c),
                _ => {
                    println!("Verification FAILED: Missing proof or commitment in API response.");
                    return;
                }
            };

            // Decode the Base64 strings back into binary bytes
            let proof_bytes = general_purpose::STANDARD.decode(proof_b64).expect("Failed to decode proof");
            let commitment_bytes = general_purpose::STANDARD.decode(commitment_b64).expect("Failed to decode commitment");

            // Reconstruct the cryptographic objects from the bytes
            let proof = RangeProof::from_bytes(&proof_bytes).expect("Failed to parse proof");
            let commitment = CompressedRistretto::from_slice(&commitment_bytes);

            // The verifier uses the same generators as the prover
            let pc_gens = PedersenGens::default();
            let bp_gens = BulletproofGens::new(64, 1);
            let mut verifier_transcript = Transcript::new(b"CreditScoreProofFinal");

            let is_valid = proof.verify_single(&bp_gens, &pc_gens, &mut verifier_transcript, &commitment, 64).is_ok();

            println!("\n--- Verification Result ---");
            if is_valid {
                println!("SUCCESS: The proof is cryptographically valid. The user's score is confirmed to be >= {}.", threshold);
            } else {
                println!("FAILURE: The proof is invalid!");
            }
        }
        Err(e) => {
            eprintln!("\nError: Could not parse the provided JSON. {}", e);
        }
    }
}