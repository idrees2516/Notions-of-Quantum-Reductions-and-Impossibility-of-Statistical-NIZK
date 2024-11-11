use quantum_cryptography::*;
use rand::rngs::OsRng;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let security_parameter = 256;
    
    let quantum_channel = QuantumChannel {
        noise_model: NoiseModel {
            decoherence_rate: 0.01,
            depolarization_probability: 0.001,
            thermal_noise: 0.0001,
        },
        error_correction: ErrorCorrection {
            code_distance: 7,
            syndrome_measurements: vec![],
            recovery_operations: vec![],
        },
        authentication: QuantumAuthentication {
            key: SecretKey(vec![0u8; 32]),
            tag: AuthenticationTag(vec![0u8; 32]),
            verification_scheme: VerificationScheme::Clifford,
        },
    };

    let protocol = SNIZKProtocol::new(security_parameter, quantum_channel.clone());
    let mut rng = OsRng;

    let crs = (protocol.crs_generator)(&mut rng)?;
    let statement = vec![0u8; security_parameter];
    let witness = vec![0u8; security_parameter];

    let proof = (protocol.prover)(&crs, &statement, &witness)?;
    let verification = (protocol.verifier)(&crs, &statement, &proof)?;

    println!("Protocol verification result: {}", verification);
    Ok(())
}