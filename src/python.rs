/**
 * Just a Helper to call python Qiskit stuff
 **/

use cpython::*;
use std::rc::Rc;
use std::cell::RefCell;

macro_rules! PyErr_to_string {
    ($x:expr) => {format!("{:?}",$x)}
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

 pub struct QiskitPython<'a> {
     gil: GILGuard,
     py: Rc<RefCell<Option<Python<'a>>>>,
     qiskit: Rc<RefCell<Option<PyModule>>>,
 }

impl<'a> QiskitPython<'a> {
    pub fn new() -> Result<QiskitPython<'a>, String> {
        let gil = Python::acquire_gil();
        Ok(QiskitPython {
            gil: gil,
            py: Rc::new(RefCell::new(None)),
            qiskit: Rc::new(RefCell::new(None)),
        })
    }


    pub fn get_qasm_circuit(&'a self, name: &str, file: &str) -> Result<String, String> {
        self.maybe_init_qiskit()?;
        let quantum_program = self.get_quantum_program(&self.py.borrow().unwrap())?;
        let qasm_text = self.load_qasm(&self.py.borrow().unwrap(), &quantum_program, name, file)?;
        Ok((*qasm_text.to_string(self.py.borrow().unwrap()).unwrap()).to_string())
    }

    // TODO Return a String representing the Json? (so avoiding strong depedencies on serde)
    pub fn get_backend_circuit(&'a self, circuit: String) -> Result<String, String> {
        self.maybe_init_qiskit()?;
        let qasm = self.get_qasm_object(&self.py.borrow().unwrap(), circuit)?;
        let program = self.parse(&self.py.borrow().unwrap(), &qasm)?;
        let unroller = self.get_unroller(&self.py.borrow().unwrap(), &program)?;

        if let Err(err) = unroller.call_method(self.py.borrow().unwrap(), "execute", NoArgs, None) {
            return Err(format!("Error: Calling Unroller::execute() method!!: {}", PyErr_to_string!(err)));
        }

        let backend_circuit = match unroller.getattr(self.py.borrow().unwrap(), "backend") {
            Ok(backend) => {
                match backend.getattr(self.py.borrow().unwrap(), "circuit") {
                    Ok(_circuit) => _circuit,
                    Err(err) => return Err(format!("Error: Gettging unroller.backend.circuit attribute: {}", PyErr_to_string!(err)))
                }
            }
            Err(err) => return Err(format!("Error: Getting unroller.backend attribute: {}", PyErr_to_string!(err))),
        };

        debug!("get_backend_circuit: backend_circuit: {:?}", backend_circuit);
        Ok((backend_circuit.to_string()))
    }

    fn maybe_init_qiskit(&'a self) -> Result<(),String> {
        if let Some(_) = *self.py.borrow() {
            info!("maybe_init_qiskit: already initialized");
            return Ok(());
        }

        info!("maybe_init_qiskit: initializing py");
        *self.py.borrow_mut() = Some(self.gil.python());

        if let Some(_) = *self.qiskit.borrow() {
            return Ok(());
        }

        match self.py.borrow().unwrap().import("qiskit") {
            Ok(qiskit) => Ok(*self.qiskit.borrow_mut() = Some(qiskit)),
            Err(err) => return Err(format!("Error: while importing qiskit pyhton module: {}", PyErr_to_string!(err))),
        }
    }

    fn get_qasm_object(&self, py: &Python, circuit: String) -> Result<PyObject, String> {
        let ref borrowed_qiskit = *self.qiskit.borrow();
        let qiskit = match *borrowed_qiskit {
            Some(ref _qiskit) => _qiskit,
            None => return Err(format!("Error: qiskit module has not been loaded!!")),
        };

        let qasm = match qiskit.get(*py, "qasm"){
            Ok(_qasm) => {
                match _qasm.getattr(*py, "Qasm") {
                    Ok(__qasm) => __qasm,
                    Err(err) => return Err(format!("Error: While getting Qasm object: {}", PyErr_to_string!(err))),
                }
            },
            Err(err) => return Err(format!("Error: While getting qasm attibute: {}", PyErr_to_string!(err))),
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

     fn get_unroller(&self, py: &Python, program: &PyObject) -> Result<PyObject, String> {
         let ref borrowed_qiskit = *self.qiskit.borrow();
         let qiskit = match *borrowed_qiskit {
             Some(ref _qiskit) => _qiskit,
             None => return Err(format!("Error: qiskit module has not been loaded!!")),
         };


         let unroll = match qiskit.get(*py, "unroll") {
             Ok(_unroll) => _unroll,
             Err(err) => return Err(format!("Error: While getting python unroll attribute: {}", PyErr_to_string!(err))),
         };

         let json_backend = match unroll.getattr(*py, "JsonBackend") {
             Ok(jb) => jb,
             Err(err) => return Err(format!("Error: While getting JsonBackend object: {}", PyErr_to_string!(err))),
         };

         let basis_gates = PyDict::new(*py);
         let jb_instance = match json_backend.call(*py, (basis_gates,), None) {
             Ok(_jb_instance) => _jb_instance,
             Err(err) => return Err(format!("Error: While calling python Uroller::JsonBackend(&args) method: {}", PyErr_to_string!(err))),
         };

         let unroller = match unroll.getattr(*py, "Unroller") {
             Ok(_unroller) => _unroller,
             Err(err) => return Err(format!("Error: Getting Unroller python type: {}", PyErr_to_string!(err))),
         };

         let unroller_instance = match unroller.call(*py, (program, &jb_instance), None) {
             Ok(_unroller_instance) => _unroller_instance,
             Err(err) =>{
                 return Err(format!("Error: Creating Unroller object: {}", PyErr_to_string!(err)));
             }
         };

         Ok(unroller_instance)
     }

     fn get_quantum_program(&self, py: &Python) -> Result<PyObject, String> {
         let ref borrowed_qiskit = *self.qiskit.borrow();
         let qiskit = match *borrowed_qiskit {
             Some(ref _qiskit) => _qiskit,
             None => return Err(format!("Error: qiskit module has not been loaded!!")),
         };


         match qiskit.get(*py, "QuantumProgram") {
             Ok(qp) => Ok(qp),
             Err(err) => return Err(format!("Error: While getting QuantumProgram object: {}", PyErr_to_string!(err))),
         }
     }

     fn load_qasm(&self, py: &Python, quantum_program: &PyObject, name: &str, file: &str) -> Result<PyString, String> {
         let quantum_program_instance = match quantum_program.call(*py, NoArgs, None) {
             Ok(qp_instance) => qp_instance,
             Err(err) => return Err(format!("Error: While instantiating QuantumProgram object!: {}", PyErr_to_string!(err))),
         };

         if let Err(err) = quantum_program_instance.call_method(*py, "load_qasm", (name, file), None) {
             return Err(format!("Error: While calling load_qasm method!: {}", PyErr_to_string!(err)));
         }

         match quantum_program_instance.call_method(*py, "get_qasm", (name,), None ) {
             Ok(qasm_text) => Ok(PyString::extract(*py, &qasm_text).unwrap()),
             Err(err) => return Err(format!("Error: While calling get_qasm method!: {}", PyErr_to_string!(err))),
         }
     }
 }
