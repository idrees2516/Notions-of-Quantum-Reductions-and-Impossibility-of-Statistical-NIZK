use super::*;
use std::ops::{Add, Mul};
use num_complex::Complex64;
use rand_distr::{Distribution, Normal};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuantumState {
    pub(crate) amplitudes: Vec<Complex64>,
    pub(crate) num_qubits: usize,
    pub(crate) basis_states: Vec<BasisState>,
    pub(crate) entanglement_map: HashMap<usize, Vec<usize>>,
    pub(crate) measurement_history: Vec<Measurement>,
    pub(crate) error_syndrome: Option<ErrorSyndrome>,
}

impl QuantumState {
    pub fn new(num_qubits: usize) -> Self {
        let mut amplitudes = vec![Complex64::new(0.0, 0.0); 1 << num_qubits];
        amplitudes[0] = Complex64::new(1.0, 0.0);
        
        Self {
            amplitudes,
            num_qubits,
            basis_states: vec![BasisState::new(num_qubits)],
            entanglement_map: HashMap::new(),
            measurement_history: Vec::new(),
            error_syndrome: None,
        }
    }

    pub fn apply_gate(&mut self, gate: QuantumGate, target: usize) -> Result<(), QuantumError> {
        if target >= self.num_qubits {
            return Err(QuantumError::InvalidQubitIndex);
        }

        match gate {
            QuantumGate::Hadamard => self.apply_hadamard(target),
            QuantumGate::PauliX => self.apply_pauli_x(target),
            QuantumGate::PauliY => self.apply_pauli_y(target),
            QuantumGate::PauliZ => self.apply_pauli_z(target),
            QuantumGate::Phase(phi) => self.apply_phase(target, phi),
            QuantumGate::CNOT(control) => self.apply_cnot(control, target),
        }
    }

    pub fn measure(&mut self, basis: MeasurementBasis) -> Result<Measurement, QuantumError> {
        let mut rng = rand::thread_rng();
        let distribution = Normal::new(0.0, 1.0).unwrap();
        
        let measurement = match basis {
            MeasurementBasis::Computational => self.measure_computational(&mut rng),
            MeasurementBasis::Bell => self.measure_bell(&mut rng),
            MeasurementBasis::Magic => self.measure_magic(&mut rng, &distribution),
        }?;

        self.measurement_history.push(measurement.clone());
        Ok(measurement)
    }

    pub fn apply_error_correction(&mut self, code: ErrorCorrectionCode) -> Result<(), QuantumError> {
        let syndrome = self.compute_error_syndrome(&code)?;
        let correction = code.compute_recovery_operation(&syndrome)?;
        self.apply_recovery_operation(correction)
    }

    fn compute_error_syndrome(&self, code: &ErrorCorrectionCode) -> Result<ErrorSyndrome, QuantumError> {
        let stabilizers = code.get_stabilizers();
        let mut syndrome = ErrorSyndrome::new(stabilizers.len());

        for (i, stabilizer) in stabilizers.iter().enumerate() {
            let measurement = self.measure_stabilizer(stabilizer)?;
            syndrome.set_bit(i, measurement);
        }

        Ok(syndrome)
    }

    fn measure_stabilizer(&self, stabilizer: &Stabilizer) -> Result<bool, QuantumError> {
        let mut state = self.clone();
        for (qubit, pauli) in stabilizer.iter() {
            match pauli {
                PauliOperator::X => state.apply_gate(QuantumGate::PauliX, *qubit)?,
                PauliOperator::Y => state.apply_gate(QuantumGate::PauliY, *qubit)?,
                PauliOperator::Z => state.apply_gate(QuantumGate::PauliZ, *qubit)?,
            }
        }
        
        let overlap = state.compute_overlap(self)?;
        Ok(overlap.re > 0.0)
    }

    fn compute_overlap(&self, other: &Self) -> Result<Complex64, QuantumError> {
        if self.num_qubits != other.num_qubits {
            return Err(QuantumError::DimensionMismatch);
        }

        let mut overlap = Complex64::new(0.0, 0.0);
        for (a1, a2) in self.amplitudes.iter().zip(other.amplitudes.iter()) {
            overlap = overlap + a1.conj() * a2;
        }

        Ok(overlap)
    }
}

#[derive(Clone, Debug)]
pub struct ErrorSyndrome {
    bits: BitVec,
    size: usize,
}

impl ErrorSyndrome {
    pub fn new(size: usize) -> Self {
        Self {
            bits: BitVec::from_elem(size, false),
            size,
        }
    }

    pub fn set_bit(&mut self, index: usize, value: bool) {
        if index < self.size {
            self.bits.set(index, value);
        }
    }

    pub fn get_bit(&self, index: usize) -> Option<bool> {
        if index < self.size {
            Some(self.bits[index])
        } else {
            None
        }
    }

    pub fn to_vec(&self) -> Vec<bool> {
        self.bits.to_vec()
    }
}

#[derive(Clone, Debug)]
pub struct Stabilizer {
    operators: Vec<(usize, PauliOperator)>,
}

impl Stabilizer {
    pub fn new(operators: Vec<(usize, PauliOperator)>) -> Self {
        Self { operators }
    }

    pub fn iter(&self) -> impl Iterator<Item = &(usize, PauliOperator)> {
        self.operators.iter()
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PauliOperator {
    X,
    Y,
    Z,
}

#[derive(Clone, Debug)]
pub enum QuantumGate {
    Hadamard,
    PauliX,
    PauliY,
    PauliZ,
    Phase(f64),
    CNOT(usize),
}