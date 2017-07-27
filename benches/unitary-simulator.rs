#[macro_use] extern crate bencher;
extern crate unitary_simulator;

use unitary_simulator::UnitarySimulator;
use unitary_simulator::python::QiskitPython;

use bencher::Bencher;

fn bench_circuit1(b: &mut Bencher){
    let qiskit = QiskitPython::new().unwrap();
    let circuit = qiskit.get_qasm_circuit("example", "example/example.qasm").unwrap();
    let backend_circuit = qiskit.get_backend_circuit(circuit).unwrap();
    let mut us = UnitarySimulator::new(backend_circuit.to_string()).unwrap();
    b.iter(|| us.run());
}

benchmark_group!(benches, bench_circuit1);
benchmark_main!(benches);
