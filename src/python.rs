/**
 * Just a Helper to call python Qiskit stuff
 **/

use cpython::*;
use serde_json;

macro_rules! PyErr_to_string {
    ($x:expr) => ({""})
}
/* TODO Implement this macro properly, so we can extract the information of a PyErr as a string
macro_rules! PyErr_to_string {
    ($x:expr) => ({
        let pvalue_str = match $x.pvalue {
            Some(val) => {
                match val.str(self.py) {
                    Ok(text) => text.to_string_lossy(),
                    Err(_) => "".to_string()
                };
            },
            None => "".to_string()
        };

        let ptraceback_str = match $x.ptraceback {
            Some(val) => {
                match val.str(self.py) {
                    Ok(text) => text,
                    Err(_) => "".to_string()
                };
            },
            None => "".to_string()
        };

        format!("{} {} {}", $x.ptype.str(self.py), pvalue_str, ptraceback_str)
    })
}
*/

 pub struct QiskitPython {
     gil: GILGuard,
     qiskit: Option<PyModule>,
 }

impl QiskitPython {
    pub fn new() -> Result<QiskitPython, String> {
        let gil = Python::acquire_gil();
        Ok(QiskitPython {
            gil: gil,
            qiskit: None,
        })
    }


    pub fn get_backend_circuit(mut self, circuit: String) -> Result<serde_json::Value, String> {
        let py = self.gil.python();
        let qiskit = match py.import("qiskit") {
            Ok(_qiskit) => _qiskit,
            Err(err) => return Err(format!("Error: while importing qiskit pyhton module: {}", PyErr_to_string!(err))),
        };

        self.qiskit = Some(qiskit);

        let qasm = self.init_qasm(&py, circuit)?;
        let program = self.parse(&py, &qasm)?;
        let (unroller, sb) = self.unroller(&py)?;
        // TODO sb is being passed as a ref to the call (check marshalling is correct)
        let unroller_result = match unroller.call(py, (program, &sb), None) {
            Ok(_unroller_result) => _unroller_result,
            Err(err) =>{
                return Err(format!("Error: Creating Unroller object: {}", PyErr_to_string!(err)));
            }
        };

        if let Err(err) = sb.call_method(py, "set_trace", (py.False(),), None) {
            return Err(format!("Error: Calling Unroller::set_trace() method!!: {}", PyErr_to_string!(err)));
        }

        if let Err(err) = unroller_result.call_method(py, "execute", NoArgs, None) {
            return Err(format!("Error: Calling Unroller::execute() method!!: {}", PyErr_to_string!(err)));
        }

        // TODO We need to extract the circuit from any of the structures used in Python
        Ok(json!({
            "qasm": [{
                "gate_size": 1,
                "name": "U",
                "theta": 1.5707963267948966,
                "phi": 0.0,
                "lambda": 3.141592653589793,
                "qubit_indices": [3]
                },
                {
                "gate_size": 2,
                "name": "CX",
                "qubit_indices": [3, 4]
                }],
                "number_of_qubits": 8,
                "qubit_order": {
                    "0": ("a", 0),
                    "1": ("a", 1),
                    "2": ("a", 2),
                    "3": ("a", 3),
                    "4": ("b", 0),
                    "5": ("b", 1),
                    "6": ("b", 2),
                    "7": ("b", 3)
                },
                "number_of_cbits": 5,
                "cbit_order": {
                    "0": ("ans", 0),
                    "1": ("ans", 1),
                    "2": ("ans", 2),
                    "3": ("ans", 3),
                    "4": ("ans", 4)
                },
                "number_of_operations": 2
        }))

    }

    fn init_qasm(&self, py: &Python, circuit: String) -> Result<PyObject, String> {
         let qasm = match self.qiskit {
             Some(ref _qiskit) => {
                 match _qiskit.get(*py, "qasm"){
                     Ok(_qasm) => {
                         match _qasm.getattr(*py, "Qasm") {
                             Ok(__qasm) => __qasm,
                             Err(err) => return Err(format!("Error: While getting Qasm object: {}", PyErr_to_string!(err))),
                         }
                     },
                     Err(err) => return Err(format!("Error: While getting qasm attibute: {}", PyErr_to_string!(err))),
                 }
             },
             None => return Err(format!("Error: qiskit module is not initialized!")),
         };

         let qasm_obj = match qasm.call(*py, (py.None(), circuit), None) {
             Ok(_qasm_obj) => _qasm_obj,
             Err(err) => return Err(format!("Couldn't initialize Qasm python object: {}", PyErr_to_string!(err))),
         };

         Ok(qasm_obj)
     }

     fn parse(&self, py: &Python, qasm: &PyObject) -> Result<PyObject, String> {
         match qasm.call_method(*py, "parse", NoArgs, None) {
             Ok(program) => Ok(program),
             Err(err) => return Err(format!("Error: While calling python Qasm::parse() method: {:?}", err)),
         }
     }

     fn unroller(&self, py: &Python) -> Result<(PyObject, PyObject), String> {

         let unroll = match self.qiskit {
             Some(ref _qiskit) => {
                 match _qiskit.get(*py, "unroll") {
                     Ok(_unroll) => _unroll,
                     Err(err) => return Err(format!("Error: While getting python unroll attribute: {}", PyErr_to_string!(err))),
                 }
             },
             None => return Err(format!("Error: qiskit module is not initialized!")),
         };

         let simulator_backend = match unroll.getattr(*py, "SimulatorBackend") {
             Ok(sb) => sb,
             Err(err) => return Err(format!("Error: While getting SimulatorBackend object: {}", PyErr_to_string!(err))),
         };

         let basis_gates = PyDict::new(*py);
         let sb = match simulator_backend.call(*py, (basis_gates,), None) {
             Ok(_sb) => _sb,
             Err(err) => return Err(format!("Error: While calling python Uroller::SimulatorBackend(&args) method: {}", PyErr_to_string!(err))),
         };

         let unroller = match unroll.getattr(*py, "Unroller") {
             Ok(_unroller) => _unroller,
             Err(err) => return Err(format!("Error: Getting Unroller python type: {}", PyErr_to_string!(err))),
         };

         Ok((unroller, sb))
     }

 }
