/*
# -*- coding: utf-8 -*-

# Copyright 2017 IBM RESEARCH. All Rights Reserved.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
# =============================================================================
*/
/**
Contains a (slow) Python simulator that returns the unitary of the circuit.

Author: Jay Gambetta and John Smolin

It simulates a unitary of a quantum circuit that has been compiled to run on
the simulator. It is exponential in the number of qubits.

The input is the circuit object and the output is the same circuit object with
a result field added results['data']['unitary'] where the unitary is
a 2**n x 2**n complex numpy array representing the unitary matrix.


The input is
    compiled_circuit object
and the output is the results object

The simulator is run using

    UnitarySimulator(compiled_circuit).run().

In the qasm, key operations with type 'measure' and 'reset' are dropped.

Internal circuit_object

circuit =
    {
    'number_of_qubits': 2,
    'number_of_cbits': 2,
    'number_of_operations': 4,
    'qubit_order': {('q', 0): 0, ('v', 0): 1}
    'cbit_order': {('c', 1): 1, ('c', 0): 0},
    'qasm':
        [{
        'type': 'gate',
        'name': 'U',
        'theta': 1.570796326794897
        'phi': 1.570796326794897
        'lambda': 1.570796326794897
        'qubit_indices': [0],
        'gate_size': 1,
        },
        {
        'type': 'gate',
        'name': 'CX',
        'qubit_indices': [0, 1],
        'gate_size': 2,
        },
        {
        'type': 'reset', //TODO: Validate if reset uses name "reset"
        'name': 'reset',
        'qubit_indices': [1]
        }
        {
        'type': 'measure', //TODO: Validate if measure uses name "measure"
        'name': 'measure',
        'cbit_indices': [0],
        'qubit_indices': [0]self.circuit
        }],
    }

returned results object
 //{Python, PyDict, NoArgs, ObjectProtocol, PyResult, PyString};
result =
        {
        'data':
            {
            'unitary': np.array([[ 0.70710678 +0.00000000e+00j
                                 0.70710678 -8.65956056e-17j
                                 0.00000000 +0.00000000e+00j
                                 0.00000000 +0.00000000e+00j]
                               [ 0.00000000 +0.00000000e+00j
                                 0self.circuit.00000000 +0.00000000e+00j
                                 0.70710678 +0.00000000e+00j
                                 -0.70710678 +8.65956056e-17j]
                               [ 0.00000000 +0.00000000e+00j
                                 0.00000000 +0.00000000e+00j
                                 0.70710678 +0.00000000e+00j
                                 0.70710678 -8.65956056e-17j]
                               [ 0.70710678 +0.00000000e+00j
                                -0.70710678 +8.65956056e-17j
                                 0.00000000 +0.00000000e+00j
                                 0.00000000 +0.00000000e+00j]
            }
        'state': 'DONE'
        }
*/

extern crate serde;
#[macro_use] extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate nalgebra as na;
extern crate num;
extern crate cpython;
#[macro_use] extern crate log;
extern crate env_logger;


pub mod macros;
pub mod matrix;
pub mod complex;
pub mod simulatortools;
pub mod gate;
pub mod python;

use std::collections::HashMap;
use complex::Complex;
use gate::Gate;
use simulatortools::*;
use python::QiskitPython;
use matrix::*;

pub struct UnitarySimulator {
    circuit: serde_json::Value,
    number_of_qubits: usize,
    result: HashMap<&'static str, serde_json::Value>,
    unitary_state: Matrix,
    number_of_operations: usize
}


impl UnitarySimulator {
    pub fn new(compiled_circuit: String) -> Result<UnitarySimulator, String> {
        let qiskit_python = QiskitPython::new()?;
        let circuit = qiskit_python.get_backend_circuit(compiled_circuit)?;

        let mut result = HashMap::new();
        result.insert("data",json!({"unitary":{}}));
        result.insert("result", json!({}));
        result.insert("status", json!({}));

        let number_of_qubits = match circuit["number_of_qubits"].as_u64() {
            Some(val) => val,
            None => return Err("No number_of_qubits field in the circuit!!".to_string()),
        };

        let number_of_operations = match circuit["number_of_operations"].as_u64(){
            Some(val) => val,
            None => return Err("No number_of_operations field in the circuit!!".to_string()),
        };

        let possible_states = 2usize.pow(number_of_qubits as u32);
        let unitary_state = Matrix::identity(possible_states);

        debug!("new: number_of_qubits={} number_of_operations={} unitary_state.size={}",
                number_of_qubits, number_of_operations, unitary_state.size());

        Ok(UnitarySimulator {
            circuit : circuit,
            number_of_qubits: number_of_qubits as usize,
            result: result,
            unitary_state: unitary_state,
            number_of_operations: number_of_operations as usize
        })
    }

    fn add_unitary_single(&mut self, gate: &Gate<Complex>, qubit: usize){
        let unitary_add = enlarge_single_opt(gate, qubit, self.number_of_qubits);
        debug!("add_unitary_single: unitary_add: {}", unitary_add);
        // dot() from numpy does a matrix mult when the input is a 2D Matrix,
        // We don't use 2D Arrays so we should probably be using a simple Matrix mult.
        self.unitary_state = &unitary_add * &self.unitary_state; //dot product
        debug!("add_unitary_single: unitary_state: {}", self.unitary_state);
    }

    fn add_unitary_two(&mut self, gate: &Gate<f64>, qubit0: usize , qubit1: usize){
        let unitary_add = enlarge_two_opt(&gate, qubit0, qubit1, self.number_of_qubits);
        debug!("add_unitary_two: unitary_add: {}", unitary_add);
        // dot() from numpy does a matrix mult when the input is a 2D Matrix,
        // We don't use 2D Arrays so we should probably be using a simple Matrix mult.
        self.unitary_state = &unitary_add * &self.unitary_state; //dot product
        debug!("add_unitary_two: unitary_state: {}",  self.unitary_state);
    }

    pub fn run(&mut self) -> Result<HashMap<&'static str, serde_json::Value>, String> {
        for j in 0..self.number_of_operations {
            let c_qasm = self.circuit["qasm"][j].clone();
            debug!("Gate: {}", c_qasm["name"].to_string().as_str());
            match c_qasm["name"].to_string().as_str() {
                "\"U\"" => {
                    let qubit = c_qasm["qubit_indices"][0].as_i64().unwrap() as usize;
                    let theta  = c_qasm["theta"].as_f64().unwrap();
                    let phi = c_qasm["phi"].as_f64().unwrap();
                    let lam = c_qasm["lambda"].as_f64().unwrap();

                    let gate = Gate::<Complex>::from_slice(&[
                        Complex::new(f64::cos(theta/2.0f64),0.0f64),
                        -(Complex::i() * lam).exp() * f64::sin(theta / 2.0f64),
                        (Complex::i() * phi).exp() * Complex::new(f64::sin(theta / 2.0f64),0.0f64),
                        (Complex::i() * phi + Complex::i() * lam).exp() * Complex::new(f64::cos(theta / 2.0f64), 0.0f64)]);
                    debug!("run: U match: qubit:'{}' theta:'{}' phi:'{}' lam:'{}' gate:'{}'", qubit, theta, phi, lam, gate);
                    self.add_unitary_single(&gate, qubit);
                },
                "\"CX\"" => {
                    let qubit0 = c_qasm["qubit_indices"][0].as_i64().unwrap() as usize;
                    let qubit1 = c_qasm["qubit_indices"][1].as_i64().unwrap() as usize;
                    let gate = Gate::<f64>::from_slice(&[1.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64,
                                                         0.0f64, 1.0f64, 0.0f64, 0.0f64, 1.0f64, 0.0f64,
                                                         0.0f64, 1.0f64, 0.0f64, 0.0f64]);
                    debug!("run: CX match: qubit0:'{}' qubit1:'{}' gate:'{}'", qubit0, qubit1, gate);
                    self.add_unitary_two(&gate, qubit0, qubit1);
                },
                "\"measure\"" => {
                    warn!("Warning: Measure has been dropped from unitary simulator");
                },
                "\"reset\"" => {
                    warn!("Warning: Reset has been dropped from unitary simulator");
                },
                _ => {
                    error!("Error: Unknown gate type!!");
                    *self.result.get_mut("status").unwrap() = json!("ERROR");
                    return Ok(self.result.clone());
                }
            }
        }

        *self.result.get_mut("data").unwrap().get_mut("unitary").unwrap() = json!(self.unitary_state.as_slice());
        *self.result.get_mut("status").unwrap() = json!("DONE");
        Ok(self.result.clone())
    }
}



#[cfg(test)]
mod tests {
    use UnitarySimulator;
    use env_logger;
    #[test]
    fn circuit1() {
        env_logger::init().ok().expect("Error initializing loggger");
        let qasm = r#"OPENQASM 2.0;
            include "qelib1.inc";

            qreg a[4];
            qreg b[4];
            creg ans[5];
            h a[3];
            cx a[3],b[0];"#;

        let mut us = UnitarySimulator::new(qasm.to_string()).unwrap();
        let result = us.run().unwrap();
        debug!("test: circuit1: result['data']['unitary']={}", result["data"]["unitary"]);
        assert_eq!(result["status"], json!("DONE"));
    }
}
