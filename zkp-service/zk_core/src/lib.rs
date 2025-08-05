// src/lib.rs
use bulletproofs::{BulletproofGens, PedersenGens, RangeProof};
use curve25519_dalek_ng::ristretto::CompressedRistretto;
use curve25519_dalek_ng::scalar::Scalar;
use merlin::Transcript;
use rand::thread_rng;
use serde::Deserialize;

pub struct ZkProof {
    pub proof: RangeProof,
    pub commitment: CompressedRistretto,
}

#[derive(Deserialize)]
pub struct JobPayload {
    pub user_id: String,
    pub score: u64,
    pub threshold: u64,
}

pub fn generate_credit_score_proof(score: u64, threshold: u64) -> Result<ZkProof, String> {
    if score < threshold {
        return Err(format!(
            "Condition not met: Score {} is less than threshold {}.",
            score, threshold
        ));
    }

    let pc_gens = PedersenGens::default();
    let bp_gens = BulletproofGens::new(64, 1);
    let secret_value = score - threshold;

    let mut rng = thread_rng();
    let blinding = Scalar::random(&mut rng);

    let mut prover_transcript = Transcript::new(b"CreditScoreProofFinal");

    let (proof, commitment) = RangeProof::prove_single(
        &bp_gens, &pc_gens, &mut prover_transcript, secret_value, &blinding, 64,
    )
        .map_err(|e| e.to_string())?;

    Ok(ZkProof { proof, commitment })
}