mod bytecode;
mod compiler;
mod db;
mod debug;
mod functions;
mod lexer;
mod object;
mod value;
mod vm;

use std::{
    collections::HashMap,
    ffi::{c_char, CStr, CString},
    {cell::RefCell, rc::Rc},
};
use vm::{Status, Vm};

#[repr(C)]
pub struct InterpretResult {
    pub value: *mut c_char,
    pub status: i32,
}

/// # Safety
///
/// We can dance if we want to.
#[no_mangle]
pub unsafe extern "C" fn interpret(input: *const c_char) -> *mut InterpretResult {
    let strings = Rc::new(RefCell::new(HashMap::new()));
    let output = Rc::new(RefCell::new(String::new()));
    let mut vm = Vm::new(Rc::clone(&strings), Rc::clone(&output));
    let status;
    let c_string; // `vm` writes to `output` regardless of status
    let err_string = CString::new("Error: The result contains an interior NUL character.").unwrap();

    match CStr::from_ptr(input).to_str() {
        Ok(source) => {
            match db::init() {
                Ok(_) => {
                    status = match vm.run(source) {
                        Ok(_) => Status::Ok,
                        Err(err_status) => err_status,
                    };
                    c_string = CString::new(output.take()).unwrap_or(err_string);
                }
                Err(err) => {
                    status = Status::DatabaseError;
                    c_string = CString::new(err.to_string()).unwrap_or(err_string);
                }
            };
        }
        Err(err) => {
            status = Status::InputError;
            c_string = CString::new(err.to_string()).unwrap_or(err_string);
        }
    }

    let c_string_ptr = c_string.into_raw();
    let result = InterpretResult {
        value: c_string_ptr,
        status: status as i32,
    };

    Box::into_raw(Box::new(result))
}

/// # Safety
///
/// We can leave your friends behind.
#[no_mangle]
pub unsafe extern "C" fn free_result(ptr: *mut InterpretResult) {
    let _result = Box::from_raw(ptr); // automatically drops when this scope ends
}
