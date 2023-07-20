use crate::functions::{self, ArgType, NResult};
use crate::value::Value;
use crate::vm::Vm;
use std::ffi::CString;

use libc::{c_char, c_double, c_int};
use rand::Rng;

extern "C" {
    fn n_format_currency(
        number: c_double,
        culture: *const c_char,
        precision: c_int,
        symbol: *const c_char,
    ) -> *const NResult;

    fn n_format_number(
        number: c_double,
        format: *const c_char,
        culture: *const c_char,
    ) -> *const NResult;
}

//

pub fn add(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY: u8 = 2;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::Number, // augend
        ArgType::Number, // addend
                         // -> Number
    ];
    let stack = &vm.stack;
    functions::check_arity(ARITY, arg_count)?;
    functions::check_arg_types(ARG_TYPES, arg_start, vm)?;

    let a = stack[arg_start].to_num(vm);
    let b = stack[arg_start + 1].to_num(vm);

    Ok(Value::num(a + b))
}

pub fn divide(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY: u8 = 2;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::Number, // dividend
        ArgType::Number, // divisor
                         // -> Number
    ];
    let stack = &vm.stack;
    functions::check_arity(ARITY, arg_count)?;
    functions::check_arg_types(ARG_TYPES, arg_start, vm)?;

    let a = stack[arg_start].to_num(vm);
    let b = stack[arg_start + 1].to_num(vm);

    if b == 0f64 {
        return Err(Value::error("Division by zero.".to_owned()));
    }

    Ok(Value::num(a / b))
}

pub fn format_currency(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY_MIN: u8 = 2;
    const ARITY_MAX: u8 = 4;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::Number, // input
        ArgType::String, // culture
                         // (precision): Number
                         // (symbol): String
                         // -> String
    ];
    let stack = &vm.stack;
    functions::check_arity_range(ARITY_MIN, ARITY_MAX, arg_count)?;
    functions::check_arg_types(ARG_TYPES, arg_start, vm)?;

    let num = stack[arg_start].to_num(vm);
    let culture = stack[arg_start + 1].to_c_string(vm)?;

    let precision = if arg_count > ARITY_MIN {
        let offset = arg_start + 2;
        functions::check_arg_types(&[ArgType::Number], offset, vm)?;
        stack[offset].to_num(vm)
    } else {
        -1f64
    };

    let symbol = if arg_count == ARITY_MAX {
        let offset = arg_start + 3;
        functions::check_arg_types(&[ArgType::String], offset, vm)?;
        stack[offset].to_c_string(vm)?
    } else {
        CString::default()
    };

    unsafe {
        let string = NResult::consume(n_format_currency(
            num,
            culture.as_ptr(),
            precision as c_int,
            symbol.as_ptr(),
        ))?;

        Ok(Value::string(string.0, vm))
    }
}

pub fn format_number(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY_MIN: u8 = 2;
    const ARITY_MAX: u8 = 3;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::Number, // input
        ArgType::String, // format
                         // (culture): String
                         // -> String
    ];
    let stack = &vm.stack;
    functions::check_arity_range(ARITY_MIN, ARITY_MAX, arg_count)?;
    functions::check_arg_types(ARG_TYPES, arg_start, vm)?;

    let num = stack[arg_start].to_num(vm);
    let format = stack[arg_start + 1].to_c_string(vm)?;

    let culture = if arg_count == ARITY_MAX {
        let offset = arg_start + 2;
        functions::check_arg_types(&[ArgType::String], offset, vm)?;
        stack[offset].to_c_string(vm)?
    } else {
        CString::default()
    };

    unsafe {
        let string = NResult::consume(n_format_number(num, format.as_ptr(), culture.as_ptr()))?;

        Ok(Value::string(string.0, vm))
    }
}

pub fn mod_(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY: u8 = 2;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::Number, // dividend
        ArgType::Number, // divisor
                         // -> Number
    ];
    let stack = &vm.stack;
    functions::check_arity(ARITY, arg_count)?;
    functions::check_arg_types(ARG_TYPES, arg_start, vm)?;

    let a = stack[arg_start].to_num(vm);
    let b = stack[arg_start + 1].to_num(vm);

    if b == 0f64 {
        return Err(Value::error("Modulo by zero.".to_owned()));
    }

    Ok(Value::num(a % b))
}

pub fn multiply(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY: u8 = 2;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::Number, // multiplicand
        ArgType::Number, // multiplier
                         // -> Number
    ];
    let stack = &vm.stack;
    functions::check_arity(ARITY, arg_count)?;
    functions::check_arg_types(ARG_TYPES, arg_start, vm)?;

    let a = stack[arg_start].to_num(vm);
    let b = stack[arg_start + 1].to_num(vm);

    Ok(Value::num(a * b))
}

pub fn random(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY: u8 = 2;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::Number, // min
        ArgType::Number, // max
                         // -> Number
    ];
    let stack = &vm.stack;
    functions::check_arity(ARITY, arg_count)?;
    functions::check_arg_types(ARG_TYPES, arg_start, vm)?;

    let a = stack[arg_start].to_num(vm);
    let b = stack[arg_start + 1].to_num(vm);

    let result = f64::trunc(rand::thread_rng().gen_range(a..=b));

    Ok(Value::num(result))
}

pub fn subtract(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY: u8 = 2;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::Number, // minuend
        ArgType::Number, // subtrahend
                         // -> Number
    ];
    let stack = &vm.stack;
    functions::check_arity(ARITY, arg_count)?;
    functions::check_arg_types(ARG_TYPES, arg_start, vm)?;

    let a = stack[arg_start].to_num(vm);
    let b = stack[arg_start + 1].to_num(vm);

    Ok(Value::num(a - b))
}
