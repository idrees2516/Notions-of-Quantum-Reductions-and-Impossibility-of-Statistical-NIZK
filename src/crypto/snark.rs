use super::*;
use merlin::Transcript;
use curve25519_dalek::scalar::Scalar;
use curve25519_dalek::ristretto::{RistrettoPoint, CompressedRistretto};
use sha3::{Sha3_512, Digest};
use rand::rngs::OsRng;

pub struct SNARKProof {
    commitment: CompressedRistretto,
    response: Scalar,
    challenge: Scalar,
    auxiliary_points: Vec<CompressedRistretto>,
}

pub struct SNARKVerifier {
    public_parameters: PublicParameters,
    verification_key: VerificationKey,
}

impl SNARKVerifier {
    pub fn new(public_parameters: PublicParameters, verification_key: VerificationKey) -> Self {
        Self {
            public_parameters,
            verification_key,
        }
    }

    pub fn verify(&self, statement: &[u8], proof: &SNARKProof) -> Result<bool, CryptoError> {
        let mut transcript = Transcript::new(b"snark-verification");
        transcript.append_message(b"statement", statement);
        transcript.append_message(b"commitment", proof.commitment.as_bytes());

        for point in &proof.auxiliary_points {
            transcript.append_message(b"auxiliary", point.as_bytes());
        }

        let challenge = self.derive_challenge(&mut transcript);
        if challenge != proof.challenge {
            return Ok(false);
        }

        let verification_equation = self.verify_proof_equation(
            statement,
            &proof.commitment,
            &proof.response,
            &proof.auxiliary_points,
        )?;

        Ok(verification_equation)
    }

    fn derive_challenge(&self, transcript: &mut Transcript) -> Scalar {
        let mut scalar_bytes = [0u8; 64];
        transcript.challenge_bytes(b"challenge", &mut scalar_bytes);
        Scalar::from_bytes_mod_order_wide(&scalar_bytes)
    }

    fn verify_proof_equation(
        &self,
        statement: &[u8],
        commitment: &CompressedRistretto,
        response: &Scalar,
        auxiliary_points: &[CompressedRistretto],
    ) -> Result<bool, CryptoError> {
        let commitment_point = commitment.decompress()
            .ok_or(CryptoError::InvalidPoint)?;

        let mut combined_point = RistrettoPoint::identity();
        for (point, base) in auxiliary_points.iter()
            .zip(self.verification_key.bases.iter())
        {
            let aux_point = point.decompress()
                .ok_or(CryptoError::InvalidPoint)?;
            combined_point += aux_point * base;
        }

        let statement_point = self.hash_to_curve(statement)?;
        let verification_point = (commitment_point + 
            (statement_point * self.verification_key.statement_scalar) +
            (combined_point * response)) * self.verification_key.blinding_factor;

        Ok(verification_point == self.verification_key.verification_point)
    }

    fn hash_to_curve(&self, input: &[u8]) -> Result<RistrettoPoint, CryptoError> {
        let mut hasher = Sha3_512::new();
        hasher.update(input);
        let hash = hasher.finalize();
        
        let point = CompressedRistretto::from_slice(&hash[..32])
            .decompress()
            .ok_or(CryptoError::InvalidPoint)?;
            
        Ok(point)
    }
}
