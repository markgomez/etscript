use crate::functions::{self, ArgType};
use crate::object;
use crate::value::Value;
use crate::vm::Vm;
use std::collections::HashMap;

pub fn build_rowset_from_string(
    arg_start: usize,
    arg_count: u8,
    vm: &mut Vm,
) -> Result<Value, Value> {
    const ARITY: u8 = 2;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::String, // delimited string
        ArgType::String, // delimiter
                         // -> Rowset
    ];
    let stack = &vm.stack;
    functions::check_arity(ARITY, arg_count)?;
    functions::check_arg_types(ARG_TYPES, arg_start, vm)?;

    let string = stack[arg_start].to_string(vm);
    let delimiter = stack[arg_start + 1].to_string(vm);
    let parts = string.split(&delimiter).collect::<Vec<_>>();
    let rows = parts
        .iter()
        .map(|&part| {
            let mut map = HashMap::new();
            map.insert(
                object::intern("1".to_owned(), vm),
                Value::string(part.to_owned(), vm),
            );
            Value::row(map)
        })
        .collect::<Vec<_>>();

    Ok(Value::rowset(rows))
}
