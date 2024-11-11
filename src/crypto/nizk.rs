use super::*;
use blake3::Hash;
use merlin::Transcript;
use curve25519_dalek::ristretto::{RistrettoPoint, CompressedRistretto};
use curve25519_dalek::scalar::Scalar;

pub struct NIZKProof {
    quantum_state: QuantumState,
    classical_proof: SNARKProof,
    commitment: CompressedRistretto,
    response: Scalar,
    auxiliary_data: Vec<u8>,
}

pub struct NIZKVerifier {
    snark_verifier: SNARKVerifier,
    quantum_verifier: QuantumVerifier,
}

impl NIZKVerifier {
    pub fn new(
        public_parameters: PublicParameters,
        verification_key: VerificationKey,
        quantum_parameters: QuantumParameters,
    ) -> Self {
        Self {
            snark_verifier: SNARKVerifier::new(public_parameters, verification_key),
            quantum_verifier: QuantumVerifier::new(quantum_parameters),
        }
    }

    pub fn verify(
        &self,
        statement: &[u8],
        proof: &NIZKProof,
        quantum_channel: &QuantumChannel,
    ) -> Result<bool, CryptoError> {
        // Verify classical part
        let classical_valid = self.snark_verifier.verify(statement, &proof.classical_proof)?;
        if !classical_valid {
            return Ok(false);
        }

        // Verify quantum part
        let quantum_valid = self.quantum_verifier.verify_state(
            &proof.quantum_state,
            statement,
            quantum_channel,
        )?;
        if !quantum_valid {
            return Ok(false);
        }

        // Verify commitment consistency
        let commitment_valid = self.verify_commitment(
            &proof.commitment,
            &proof.response,
            statement,
            &proof.auxiliary_data,
        )?;

        Ok(commitment_valid)
    }

    fn verify_commitment(
        &self,
        commitment: &CompressedRistretto,
        response: &Scalar,
        statement: &[u8],
        auxiliary_data: &[u8],
    ) -> Result<bool, CryptoError> {
        let mut transcript = Transcript::new(b"nizk-commitment");
        transcript.append_message(b"statement", statement);
        transcript.append_message(b"auxiliary", auxiliary_data);

        let point = commitment.decompress()
            .ok_or(CryptoError::InvalidPoint)?;
        
        let challenge = self.derive_challenge(&mut transcript);
        let verification_point = (point * challenge + 
            self.quantum_verifier.get_base_point() * response) *
            self.quantum_verifier.get_blinding_factor();

        Ok(verification_point == self.quantum_verifier.get_verification_point())
    }

    fn derive_challenge(&self, transcript: &mut Transcript) -> Scalar {
        let mut scalar_bytes = [0u8; 64];
        transcript.challenge_bytes(b"commitment-challenge", &mut scalar_bytes);
        Scalar::from_bytes_mod_order_wide(&scalar_bytes)
    }
}