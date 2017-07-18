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
Contains a Rust simulator that returns the unitary of the circuit.

Authors: Juan Gómez Mosquera
Based on the Python Unitary Simulator created by: Jay Gambetta and John Smolin

It simulates a unitary of a quantum circuit that has been compiled to run on
the simulator. It is exponential in the number of qubits.

The input is the circuit object and the output is the same circuit object with
a result field added results["data"]["unitary"] where the unitary is
a 2**n x 2**n complex numpy array representing the unitary matrix.

The input is
    compiled_circuit object
and the output is the results object

In the qasm, key operations with type 'measure' and 'reset' are dropped.
*/

extern crate serde;
#[macro_use] extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate nalgebra as na;
extern crate num;
#[macro_use] extern crate log;
extern crate env_logger;
extern crate cpython;

pub mod python;
pub mod macros;
pub mod matrix;
pub mod complex;
pub mod simulatortools;
pub mod gate;

use std::collections::HashMap;
use complex::Complex;
use gate::Gate;
use simulatortools::*;
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
        let circuit : serde_json::Value = match serde_json::from_str(parsed_circuit.as_str()) {
            Ok(_circuit) => _circuit,
            Err(err) => return Err(format!("Error: parsing compiled circuit!!: {}", err)),
        };

        let mut result = HashMap::new();
        result.insert("data",json!({"unitary":{}}));
        result.insert("result", json!({}));
        result.insert("status", json!({}));

        let number_of_qubits = match circuit["header"]["number_of_qubits"].as_u64() {
            Some(val) => val,
            None => return Err("No number_of_qubits field in the circuit!!".to_string()),
        };

        let number_of_operations = match circuit["operations"].as_array() {
            Some(operations) => operations.len(),
            None => return Err("No operations field in the circuit!!".to_string())
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
            let c_qasm = self.circuit["operations"][j].clone();
            debug!("Gate: {}", c_qasm["name"].to_string().as_str());
            match c_qasm["name"].to_string().as_str() {
                "\"U\"" => {
                    let qubit = c_qasm["qubits"][0].as_i64().unwrap() as usize;
                    let theta  = c_qasm["params"][0].as_f64().unwrap();
                    let phi = c_qasm["params"][1].as_f64().unwrap();
                    let lam = c_qasm["params"][2].as_f64().unwrap();

                    let gate = Gate::<Complex>::from_slice(&[
                        Complex::new(f64::cos(theta/2.0f64),0.0f64),
                        -(Complex::i() * lam).exp() * f64::sin(theta / 2.0f64),
                        (Complex::i() * phi).exp() * Complex::new(f64::sin(theta / 2.0f64),0.0f64),
                        (Complex::i() * phi + Complex::i() * lam).exp() * Complex::new(f64::cos(theta / 2.0f64), 0.0f64)]);
                    debug!("run: U match: qubqiskit_pythonit:'{}' theta:'{}' phi:'{}' lam:'{}' gate:'{}'", qubit, theta, phi, lam, gate);
                    self.add_unitary_single(&gate, qubit);
                },
                "\"CX\"" => {
                    let qubit0 = c_qasm["qubits"][0].as_i64().unwrap() as usize;
                    let qubit1 = c_qasm["qubits"][1].as_i64().unwrap() as usize;
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
                "\"barrier\"" => {
                    () // Pass
                }
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

use super::python::QiskitPython;
use super::env_logger;
use super::UnitarySimulator;
use super::Matrix;
use super::Complex;

    #[test]
    fn circuit1() {
        env_logger::init().ok().expect("Error initializing loggger");
        let qiskit = QiskitPython::new().unwrap();
        let circuit = qiskit.get_qasm_circuit("example", "example/example.qasm").unwrap();
        let backend_circuit = qiskit.get_backend_circuit(circuit).unwrap();
        let mut us = UnitarySimulator::new(backend_circuit.to_string()).unwrap();
        let result = us.run().unwrap();

        let dim = ((result["data"]["unitary"].as_array().unwrap().len()) as f64).sqrt() as usize;
        info!("dim = {}", dim);
        let unitary_vec : Vec<Complex> = (*result["data"]["unitary"].as_array().unwrap()).iter()
                            .map(|val| {
                                let complex = val.as_object().unwrap();
                                Complex::new(complex["re"].as_f64().unwrap(),
                                             complex["im"].as_f64().unwrap())
                            })
                            .collect();

        let mut quantum_state = vec![0.0f64;dim];
        quantum_state[0] = 1.0f64;
        let unitary = Matrix::new_from_vector(dim, unitary_vec);
        let result = Matrix::<Complex>::dot(&unitary, &quantum_state);
        let expected = Matrix::new_from_row_slice(&[
            Complex::new(0.35355339059327384f64,0f64), Complex::new(0f64,0f64), Complex::new(0f64,0f64), Complex::new(0f64,0f64),
            Complex::new(0f64,0f64), Complex::new(0f64,0f64), Complex::new(0f64,0f64), Complex::new(0f64,0f64),
            Complex::new(0f64,0f64), Complex::new(0.3535533905932738f64,0f64), Complex::new(0f64,0f64), Complex::new(0f64,0f64),
            Complex::new(0f64,0f64), Complex::new(0f64,0f64), Complex::new(0f64,0f64), Complex::new(0f64,0f64),
            Complex::new(0f64,0f64), Complex::new(0f64,0f64), Complex::new(0.3535533905932738f64,0f64), Complex::new(0f64,0f64),
            Complex::new(0f64,0f64), Complex::new(0f64,0f64), Complex::new(0f64,0f64), Complex::new(0f64,0f64),
            Complex::new(0f64,0f64), Complex::new(0f64,0f64), Complex::new(0f64,0f64), Complex::new(0.35355339059327373f64,0f64),
            Complex::new(0f64,0f64), Complex::new(0f64,0f64), Complex::new(0f64,0f64), Complex::new(0f64,0f64),
            Complex::new(0f64,0f64), Complex::new(0f64,0f64), Complex::new(0f64,0f64), Complex::new(0f64,0f64),
            Complex::new(0.3535533905932738f64,0f64), Complex::new(0f64,0f64), Complex::new(0f64,0f64), Complex::new(0f64,0f64),
            Complex::new(0f64,0f64), Complex::new(0f64,0f64), Complex::new(0f64,0f64), Complex::new(0f64,0f64),
            Complex::new(0f64,0f64), Complex::new(0.35355339059327373f64,0f64), Complex::new(0f64,0f64), Complex::new(0f64,0f64),
            Complex::new(0f64,0f64), Complex::new(0f64,0f64), Complex::new(0f64,0f64), Complex::new(0f64,0f64),
            Complex::new(0f64,0f64), Complex::new(0f64,0f64), Complex::new(0.35355339059327373f64,0f64), Complex::new(0f64,0f64),
            Complex::new(0f64,0f64), Complex::new(0f64,0f64), Complex::new(0f64,0f64), Complex::new(0f64,0f64),
            Complex::new(0f64,0f64), Complex::new(0f64,0f64), Complex::new(0f64,0f64), Complex::new(0.3535533905932737f64,0f64),
        ]);

        assert_eq!(expected, result);
    }
}
