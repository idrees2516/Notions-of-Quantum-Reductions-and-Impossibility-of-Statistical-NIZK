use super::*;
use bitvec::prelude::*;
use itertools::Itertools;

#[derive(Clone, Debug)]
pub struct ErrorCorrectionCode {
    distance: usize,
    stabilizers: Vec<Stabilizer>,
    logical_operators: Vec<LogicalOperator>,
    recovery_lookup: HashMap<BitVec, RecoveryOperation>,
}

impl ErrorCorrectionCode {
    pub fn new_steane_code() -> Self {
        let stabilizers = vec![
            Stabilizer::new(vec![
                (0, PauliOperator::X),
                (2, PauliOperator::X),
                (4, PauliOperator::X),
                (6, PauliOperator::X),
            ]),
            Stabilizer::new(vec![
                (1, PauliOperator::X),
                (3, PauliOperator::X),
                (5, PauliOperator::X),
                (6, PauliOperator::X),
            ]),
            Stabilizer::new(vec![
                (0, PauliOperator::Z),
                (2, PauliOperator::Z),
                (4, PauliOperator::Z),
                (6, PauliOperator::Z),
            ]),
            Stabilizer::new(vec![
                (1, PauliOperator::Z),
                (3, PauliOperator::Z),
                (5, PauliOperator::Z),
                (6, PauliOperator::Z),
            ]),
        ];

        let logical_operators = vec![
            LogicalOperator::new(
                vec![
                    (0, PauliOperator::X),
                    (1, PauliOperator::X),
                    (2, PauliOperator::X),
                    (3, PauliOperator::X),
                ],
                OperatorType::X,
            ),
            LogicalOperator::new(
                vec![
                    (0, PauliOperator::Z),
                    (1, PauliOperator::Z),
                    (2, PauliOperator::Z),
                    (3, PauliOperator::Z),
                ],
                OperatorType::Z,
            ),
        ];

        let mut code = Self {
            distance: 3,
            stabilizers,
            logical_operators,
            recovery_lookup: HashMap::new(),
        };

        code.precompute_recovery_operations();
        code
    }

    pub fn get_stabilizers(&self) -> &[Stabilizer] {
        &self.stabilizers
    }

    pub fn compute_recovery_operation(&self, syndrome: &ErrorSyndrome) -> Result<RecoveryOperation, QuantumError> {
        let syndrome_bits = syndrome.to_bitvec();
        self.recovery_lookup
            .get(&syndrome_bits)
            .cloned()
            .ok_or(QuantumError::UnknownSyndrome)
    }

    fn precompute_recovery_operations(&mut self) {
        let n = self.stabilizers.len();
        let all_errors = self.enumerate_likely_errors();

        for error in all_errors {
            let syndrome = self.compute_syndrome_for_error(&error);
            if !self.recovery_lookup.contains_key(&syndrome) {
                self.recovery_lookup.insert(syndrome, RecoveryOperation::from_error(&error));
            }
        }
    }

    fn enumerate_likely_errors(&self) -> Vec<QuantumError> {
        let n = self.distance - 1;
        let mut errors = Vec::new();

        for t in 0..=n {
            for positions in (0..self.stabilizers.len()).combinations(t) {
                for error_types in (0..3).combinations_with_replacement(t) {
                    let mut error = QuantumError::new(self.stabilizers.len());
                    for (pos, err_type) in positions.iter().zip(error_types) {
                        error.add_pauli(*pos, match err_type {
                            0 => PauliOperator::X,
                            1 => PauliOperator::Y,
                            _ => PauliOperator::Z,
                        });
                    }
                    errors.push(error);
                }
            }
        }

        errors
    }

    fn compute_syndrome_for_error(&self, error: &QuantumError) -> BitVec {
        let mut syndrome = bitvec![0; self.stabilizers.len()];
        
        for (i, stabilizer) in self.stabilizers.iter().enumerate() {
            let mut parity = false;
            for (qubit, pauli) in stabilizer.iter() {
                if let Some(error_pauli) = error.get_pauli(*qubit) {
                    parity ^= Self::commutes(pauli, error_pauli);
                }
            }
            syndrome.set(i, parity);
        }

        syndrome
    }

    fn commutes(a: &PauliOperator, b: &PauliOperator) -> bool {
        match (a, b) {
            (PauliOperator::X, PauliOperator::Y) => false,
            (PauliOperator::X, PauliOperator::Z) => false,
            (PauliOperator::Y, PauliOperator::X) => false,
            (PauliOperator::Y, PauliOperator::Z) => false,
            (PauliOperator::Z, PauliOperator::X) => false,
            (PauliOperator::Z, PauliOperator::Y) => false,
            _ => true,
        }
    }
}