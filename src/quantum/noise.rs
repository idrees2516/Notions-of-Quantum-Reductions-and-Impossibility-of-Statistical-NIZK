use super::*;
use rand_distr::{Distribution, Normal, Uniform};

#[derive(Clone, Debug)]
pub struct NoiseModel {
    decoherence_rate: f64,
    depolarizing_probability: f64,
    thermal_noise_strength: f64,
    correlation_length: f64,
    spatial_correlations: HashMap<(usize, usize), f64>,
}

impl NoiseModel {
    pub fn new(
        decoherence_rate: f64,
        depolarizing_probability: f64,
        thermal_noise_strength: f64,
        correlation_length: f64,
    ) -> Self {
        Self {
            decoherence_rate,
            depolarizing_probability,
            thermal_noise_strength,
            correlation_length,
            spatial_correlations: HashMap::new(),
        }
    }

    pub fn apply_noise(&self, state: &mut QuantumState) -> Result<(), QuantumError> {
        self.apply_decoherence(state)?;
        self.apply_depolarizing_noise(state)?;
        self.apply_thermal_noise(state)?;
        self.apply_correlated_noise(state)?;
        Ok(())
    }

    fn apply_decoherence(&self, state: &mut QuantumState) -> Result<(), QuantumError> {
        let mut rng = rand::thread_rng();
        let uniform = Uniform::new(0.0, 1.0);

        for i in 0..state.num_qubits {
            if uniform.sample(&mut rng) < self.decoherence_rate {
                state.apply_gate(QuantumGate::PauliZ, i)?;
            }
        }

        Ok(())
    }

    fn apply_depolarizing_noise(&self, state: &mut QuantumState) -> Result<(), QuantumError> {
        let mut rng = rand::thread_rng();
        let uniform = Uniform::new(0.0, 1.0);

        for i in 0..state.num_qubits {
            if uniform.sample(&mut rng) < self.depolarizing_probability {
                match uniform.sample(&mut rng) {
                    x if x < 1.0/3.0 => state.apply_gate(QuantumGate::PauliX, i)?,
                    x if x < 2.0/3.0 => state.apply_gate(QuantumGate::PauliY, i)?,
                    _ => state.apply_gate(QuantumGate::PauliZ, i)?,
                }
            }
        }

        Ok(())
    }

    fn apply_thermal_noise(&self, state: &mut QuantumState) -> Result<(), QuantumError> {
        let mut rng = rand::thread_rng();
        let normal = Normal::new(0.0, self.thermal_noise_strength).unwrap();

        for i in 0..state.amplitudes.len() {
            let noise = Complex64::new(
                normal.sample(&mut rng),
                normal.sample(&mut rng)
            );
            state.amplitudes[i] += noise;
        }

        // Renormalize the state
        let norm = state.amplitudes.iter()
            .map(|x| x.norm_sqr())
            .sum::<f64>()
            .sqrt();
        
        for amplitude in &mut state.amplitudes {
            *amplitude /= norm;
        }

        Ok(())
    }

    fn apply_correlated_noise(&self, state: &mut QuantumState) -> Result<(), QuantumError> {
        let mut rng = rand::thread_rng();
        let uniform = Uniform::new(0.0, 1.0);

        for i in 0..state.num_qubits {
            for j in (i+1)..state.num_qubits {
                let correlation = self.get_spatial_correlation(i, j);
                if uniform.sample(&mut rng) < correlation {
                    // Apply correlated errors
                    match uniform.sample(&mut rng) {
                        x if x < 0.5 => {
                            state.apply_gate(QuantumGate::PauliX, i)?;
                            state.apply_gate(QuantumGate::PauliX, j)?;
                        },
                        _ => {
                            state.apply_gate(QuantumGate::PauliZ, i)?;
                            state.apply_gate(QuantumGate::PauliZ, j)?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn get_spatial_correlation(&self, i: usize, j: usize) -> f64 {
        let key = if i < j { (i, j) } else { (j, i) };
        *self.spatial_correlations.get(&key).unwrap_or_else(|| {
            let distance = (i as f64 - j as f64).abs();
            let correlation = (-distance / self.correlation_length).exp();
            &correlation
        })
    }
}