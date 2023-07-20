use crate::functions::{self, ArgType};
use crate::value::Value;
use crate::vm::Vm;

pub fn empty(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY: u8 = 1;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::Value, // input
                        // -> Boolean
    ];
    let stack = &vm.stack;
    functions::check_arity(ARITY, arg_count)?;
    functions::check_arg_types(ARG_TYPES, arg_start, vm)?;

    let input = &stack[arg_start];

    if input.is_string() {
        let string = input.to_string(vm);
        return Ok(Value::boolean(string.is_empty()));
    }

    Ok(Value::boolean(input.is_null()))
}

pub fn iif(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY: u8 = 3;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::Value, // bool expression
        ArgType::Value, // truthy result
        ArgType::Value, // falsey result
                        // -> Value
    ];
    let stack = &vm.stack;
    functions::check_arity(ARITY, arg_count)?;
    functions::check_arg_types(ARG_TYPES, arg_start, vm)?;

    let bool_expr = &stack[arg_start];
    let truthy_result = &stack[arg_start + 1];
    let falsey_result = &stack[arg_start + 2];

    let val = if bool_expr.is_truthy(vm) {
        truthy_result
    } else {
        falsey_result
    };

    Ok(val.clone())
}

pub fn is_null(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY: u8 = 1;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::Value, // input
                        // -> Boolean
    ];
    let stack = &vm.stack;
    functions::check_arity(ARITY, arg_count)?;
    functions::check_arg_types(ARG_TYPES, arg_start, vm)?;

    let input = &stack[arg_start];

    Ok(Value::boolean(input.is_null()))
}

pub fn v(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY: u8 = 1;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::Value, // input
                        // -> Value
    ];
    let stack = &vm.stack;
    functions::check_arity(ARITY, arg_count)?;
    functions::check_arg_types(ARG_TYPES, arg_start, vm)?;

    Ok(stack[arg_start].clone())
}
