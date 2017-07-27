extern crate unitary_simulator;

use unitary_simulator::UnitarySimulator;
use unitary_simulator::python::QiskitPython;
use std::env;
use std::time::{Duration, Instant};
use std::str::FromStr;
use std::process::exit;

fn bench_circuit1(num_iters: u64){
    let qiskit = QiskitPython::new().unwrap();
    let circuit = qiskit.get_qasm_circuit("example", "example/example.qasm").unwrap();
    let backend_circuit = qiskit.get_backend_circuit(circuit).unwrap();
    let mut us = UnitarySimulator::new(backend_circuit.to_string()).unwrap();
    let mut sum = Duration::new(0u64,0u32);
    for _ in 0..num_iters {
        let now = Instant::now();
        us.run();
        sum += now.elapsed();
    }
    println!("Circuit run: {} times. Total time: {}", num_iters,
               sum.as_secs() as f64 + sum.subsec_nanos() as f64 * 1e-9);
}


fn usage() {
        println!(
        r#"Error: No valid arguments given!
You need to pass the number of iterations to run the benchmark.
        "#);
        exit(1);
}

fn main() {
    let num_iters = match env::args().nth(1) {
            Some(n) => {
                match u64::from_str(n.as_str()) {
                    Ok(_num_iters) => _num_iters,
                    _ => { usage(); 0u64 }
                }
            },
            None =>{ usage(); 0u64 }
    };

    bench_circuit1(num_iters);
}
