use crate::functions::{self, ArgType};
use crate::value::Value;
use crate::vm::Vm;

use base64::{engine::general_purpose, Engine as _};
use uuid::Uuid;

pub fn base64_decode(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY_MIN: u8 = 1;
    const ARITY_MAX: u8 = 3;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::String, // input
                         // (encoding): String
                         // (stop if error): Boolean
                         // -> String
    ];
    let stack = &vm.stack;
    functions::check_arity_range(ARITY_MIN, ARITY_MAX, arg_count)?;
    functions::check_arg_types(ARG_TYPES, arg_start, vm)?;

    let input = stack[arg_start].to_string(vm);

    let encoding = if arg_count > ARITY_MIN {
        let offset = arg_start + 1;
        functions::check_arg_types(&[ArgType::String], offset, vm)?;
        char_encoding(stack[offset].to_lower(vm).as_str())?
    } else {
        CharEncoding::Utf8
    };

    let stop_if_err = if arg_count == ARITY_MAX {
        let offset = arg_start + 2;
        functions::check_arg_types(&[ArgType::Boolean], offset, vm)?;
        stack[offset].to_bool(vm)
    } else {
        true
    };

    let mut decoded;
    let Ok(bytes) = general_purpose::STANDARD.decode(input) else {
        return Err(Value::error("Unable to decode input.".to_owned()));
    };
    decoded = String::from_utf8_lossy(&bytes).into_owned();

    let u16_bytes;
    if encoding == CharEncoding::Utf16 {
        u16_bytes = bytemuck::pod_align_to::<u8, u16>(&bytes).1;
        decoded = String::from_utf16_lossy(u16_bytes);
    }

    if decoded.contains('\0') && stop_if_err {
        return Err(Value::error(
            "Decoded string contains an interior NUL character. Ensure the correct \
                character encoding scheme (e.g., `UTF-8` or `UTF-16`) is specified."
                .to_owned(),
        ));
    }
    let val = Value::string(decoded, vm);

    Ok(val)
}

pub fn base64_encode(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY_MIN: u8 = 1;
    const ARITY_MAX: u8 = 2;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::String, // input
                         // (encoding): String
                         // -> String
    ];
    let stack = &vm.stack;
    functions::check_arity_range(ARITY_MIN, ARITY_MAX, arg_count)?;
    functions::check_arg_types(ARG_TYPES, arg_start, vm)?;

    let input = stack[arg_start].to_string(vm);
    let mut bytes = input.as_bytes();

    let encoding = if arg_count > ARITY_MIN {
        let offset = arg_start + 1;
        functions::check_arg_types(&[ArgType::String], offset, vm)?;
        char_encoding(stack[offset].to_lower(vm).as_str())?
    } else {
        CharEncoding::Utf8
    };

    let input_u16;
    if encoding == CharEncoding::Utf16 {
        input_u16 = input.encode_utf16().collect::<Vec<_>>();
        bytes = bytemuck::pod_align_to::<u16, u8>(&input_u16).1;
    }

    let encoded = general_purpose::STANDARD.encode(bytes);
    let val = Value::string(encoded, vm);

    Ok(val)
}

pub fn guid(_arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY: u8 = 0; // -> String
    functions::check_arity(ARITY, arg_count)?;

    let val = Value::string(Uuid::new_v4().to_string(), vm);

    Ok(val)
}

//

#[derive(PartialEq)]
pub enum CharEncoding {
    Utf8,
    Utf16,
}

pub fn char_encoding(scheme: &str) -> Result<CharEncoding, Value> {
    let encoding =
        match scheme {
            "utf-8" => CharEncoding::Utf8,
            "utf-16" => CharEncoding::Utf16,
            _ => return Err(Value::error(
                "Accepted case-insensitive values for character encoding are `UTF-8` and `UTF-16`."
                    .to_owned(),
            )),
        };

    Ok(encoding)
}
