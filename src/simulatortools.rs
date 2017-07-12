
use complex::Complex;
use matrix::*;


use gate::Gate;

/// Magic index1 function.
/// Takes a bitstring k and inserts bit b as the ith bit,
/// shifting bits >= i over to make room.
fn index1(b: usize, i: usize, k: usize) -> usize{
    let lowbits = k & ((1 << i) - 1); // Get the low i basis_gates
    let retval = ((((k >> i) << 1) | b) << i) | lowbits;
    retval
}

/// Magic index1 function.
///
/// Takes a bitstring k and inserts bits b1 as the i1th bit
/// and b2 as the i2th bit
fn index2(b1: usize, i1: usize, b2: usize, i2: usize, k: usize) -> usize {
    assert!(i1 != i2);
    if i1 > i2 {
        index1(b2, i2, index1(b1, i1 - 1, k))
    } else {
        index1(b1, i1, index1(b2, i2 - 1, k))
    }
}

/// Enlarge single operator to n qubits.
///
/// It is exponential in the number of qubits.
/// opt is the single-qubit opt.
/// qubit is the qubit to apply it on counts from 0 and order
/// is q_{n-1} ... otimes q_1 otimes q_0.
/// number_of_qubits is the number of qubits in the system.
pub fn enlarge_single_opt(gate: &Gate<Complex>, qubit: usize, number_of_qubits: usize) -> Matrix{
    let dim = 2usize.pow(number_of_qubits as u32 - qubit as u32 - 1u32);
    let dim2 = 2usize.pow(qubit as u32);
    let temp1 = Matrix::identity(dim);
    let temp2 = Matrix::identity(dim2);
    temp1.kronecker(&gate.matrix.kronecker(&temp2))
}

/// Enlarge two-qubit operator to n qubits.
///
/// It is exponential in the number of qubits.
/// opt is the two-qubit gate
/// q0 is the first qubit (control) counts from 0
/// q1 is the second qubit (target)
/// returns a complex numpy array
/// number_of_qubits is the number of qubits in the system.
pub fn enlarge_two_opt(gate: &Gate<f64>, qubit0: usize, qubit1: usize, num: usize) -> Matrix<f64> {
    let mut enlarge_gate = Matrix::<f64>::new_from_value(1 << num, 0.0f64);

    for i in 0..1 << (num-2) {
        for j in 0..2 {
            for k in 0..2 {
                for jj in 0..2{
                    for kk in 0..2{
                        enlarge_gate[(index2(j, qubit0, k, qubit1, i), index2(jj, qubit0, kk, qubit1, i))] = gate[(j + 2 * k, jj + 2 * kk)];
                    }
                }
            }
        }
    }
    enlarge_gate
}
