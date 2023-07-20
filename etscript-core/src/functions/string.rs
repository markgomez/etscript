use crate::functions::{
    self,
    encoding::{self, CharEncoding},
    ArgType, NResult,
};
use crate::value::Value;
use crate::vm::Vm;

use libc::{c_char, c_int};
use std::ffi::CString;
use unicode_segmentation::UnicodeSegmentation;

extern "C" {
    fn n_format(
        input: *const c_char,
        format: *const c_char,
        data_format: c_int, // 0 (``), 1 (`date`), or 2 (`number`)
        culture: *const c_char,
    ) -> *const NResult;

    fn n_proper_case(string: *const c_char) -> *const NResult;

    fn n_regex_match(
        input: *const c_char,
        regex: *const c_char,
        group: *const c_char,   // ordinal or name
        options: *const c_char, // csv format
    ) -> *const NResult;

    fn n_string_to_date(date: *const c_char) -> *const NResult;
}

//

pub fn char_(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY_MIN: u8 = 1;
    const ARITY_MAX: u8 = 2;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::Number, // ascii code
                         // (repeat count): Number
                         // -> String
    ];
    const ASCII_CODE_MAX: usize = i8::MAX as usize;
    const ASCII_REPEAT_MAX: usize = u16::MAX as usize + 1;
    let stack = &vm.stack;
    functions::check_arity_range(ARITY_MIN, ARITY_MAX, arg_count)?;
    functions::check_arg_types(ARG_TYPES, arg_start, vm)?;

    let ascii_code = stack[arg_start].to_num(vm) as usize;
    if !(0..=ASCII_CODE_MAX).contains(&ascii_code) {
        return Err(Value::error(format!(
            "Valid ASCII range is 0 to {ASCII_CODE_MAX}."
        )));
    }

    let mut buffer = [0u8; 1];
    let char_ = (ascii_code as u8) as char;
    let char_str = char_.encode_utf8(&mut buffer);
    let mut string = String::from(char_str);

    if arg_count == ARITY_MAX {
        let offset = arg_start + 1;
        functions::check_arg_types(&[ArgType::Number], offset, vm)?;

        let count = stack[offset].to_num(vm) as usize;
        if !(1..=ASCII_REPEAT_MAX).contains(&count) {
            return Err(Value::error(format!(
                "Range for repeating ASCII characters must be between 1 and {ASCII_REPEAT_MAX}."
            )));
        }
        string = string.repeat(count);
    }

    Ok(Value::string(string, vm))
}

pub fn concat(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY_MIN: u8 = 1;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::Value, // a
                        // (b ...): Value
                        // -> String
    ];
    let stack = &vm.stack;
    functions::check_arity_min(ARITY_MIN, arg_count)?;
    functions::check_arg_types(ARG_TYPES, arg_start, vm)?;

    let mut joined = String::new();
    let mut offset = arg_start;

    while offset < stack.len() {
        let item = &stack[offset];
        let num_arg = item.is_num_arg(vm);
        if !(num_arg.0 || item.is_string()) {
            return Err(Value::error(
                "Only numbers and strings can be concatenated.".to_owned(),
            ));
        }
        let string = if num_arg.1.is_some() {
            num_arg.1.unwrap()
        } else {
            item.to_string(vm)
        };

        joined.push_str(&string);

        offset += 1;
    }

    Ok(Value::string(joined, vm))
}

pub fn format(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY_MIN: u8 = 2;
    const ARITY_MAX: u8 = 4;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::String, // input
        ArgType::String, // format
                         // (`date` or `number`): String
                         // (culture): String
                         // -> String
    ];
    let stack = &vm.stack;
    functions::check_arity_range(ARITY_MIN, ARITY_MAX, arg_count)?;
    functions::check_arg_types(ARG_TYPES, arg_start, vm)?;

    let input = stack[arg_start].to_c_string(vm)?;
    let format = stack[arg_start + 1].to_c_string(vm)?;

    let data_format = if arg_count > ARITY_MIN {
        let offset = arg_start + 2;
        functions::check_arg_types(&[ArgType::String], offset, vm)?;
        stack[offset].to_lower(vm)
    } else {
        String::from("") // `0`
    };

    let culture = if arg_count == ARITY_MAX {
        let offset = arg_start + 3;
        functions::check_arg_types(&[ArgType::String], offset, vm)?;
        stack[offset].to_c_string(vm)?
    } else {
        CString::default()
    };

    unsafe {
        let string = NResult::consume(n_format(
            input.as_ptr(),
            format.as_ptr(),
            data_format_id(&data_format)?,
            culture.as_ptr(),
        ))?;

        Ok(Value::string(string.0, vm))
    }
}

pub fn index_of(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY: u8 = 2;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::String, // input
        ArgType::String, // position
                         // -> Number/Null
    ];
    let stack = &vm.stack;
    functions::check_arity(ARITY, arg_count)?;
    functions::check_arg_types(ARG_TYPES, arg_start, vm)?;

    let haystack = stack[arg_start].to_lower(vm);
    let needle = stack[arg_start + 1].to_lower(vm);
    if needle.is_empty() {
        return Ok(Value::null());
    }

    let val = if let Some(offset) = haystack.find(&needle) {
        let index = &haystack[..offset].graphemes(true).count();
        Value::num(*index as f64 + 1f64)
    } else {
        Value::null()
    };

    Ok(val)
}

pub fn length(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY: u8 = 1;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::String, // input
                         // -> Number
    ];
    let stack = &vm.stack;
    functions::check_arity(ARITY, arg_count)?;
    functions::check_arg_types(ARG_TYPES, arg_start, vm)?;

    let string = stack[arg_start].to_string(vm);
    let clstr_count = string.graphemes(true).count() as f64;

    Ok(Value::num(clstr_count))
}

pub fn lowercase(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY: u8 = 1;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::String, // input
                         // -> String
    ];
    let stack = &vm.stack;
    functions::check_arity(ARITY, arg_count)?;
    functions::check_arg_types(ARG_TYPES, arg_start, vm)?;

    let string = stack[arg_start].to_lower(vm);

    Ok(Value::string(string, vm))
}

pub fn proper_case(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY: u8 = 1;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::String, // input
                         // -> String
    ];
    let stack = &vm.stack;
    functions::check_arity(ARITY, arg_count)?;
    functions::check_arg_types(ARG_TYPES, arg_start, vm)?;

    let c_string = stack[arg_start].to_c_string(vm)?;

    unsafe {
        let string = NResult::consume(n_proper_case(c_string.as_ptr()))?;

        Ok(Value::string(string.0, vm))
    }
}

pub fn regex_match(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY_MIN: u8 = 3;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::String, // input
        ArgType::String, // regex
        ArgType::Value,  // group ordinal or name
                         // (options ...): String
                         // -> String
    ];
    let stack = &vm.stack;
    functions::check_arity_min(ARITY_MIN, arg_count)?;
    functions::check_arg_types(ARG_TYPES, arg_start, vm)?;

    let input = stack[arg_start].to_c_string(vm)?;
    let regex = stack[arg_start + 1].to_c_string(vm)?;
    let group_val = &stack[arg_start + 2];
    let num_arg = group_val.is_num_arg(vm);
    if !(num_arg.0 || group_val.is_string()) {
        return Err(Value::error(
            "Capturing groups must be a number (ordinal) or name.".to_owned(),
        ));
    }
    let group = if num_arg.1.is_some() {
        let string = num_arg.1.unwrap();
        let Ok(c_string) = CString::new(string) else {
            return Err(Value::error("Interior nul byte (`\0`) was found.".to_owned()));
        };
        c_string
    } else {
        group_val.to_c_string(vm)?
    };
    let mut opt_string = String::from("");

    if arg_count > ARITY_MIN {
        let mut offset = arg_start + 3;
        while offset < stack.len() {
            functions::check_arg_types(&[ArgType::String], offset, vm)?;
            let option = stack[offset].to_lower(vm);

            match option.as_str() {
                "ignorecase" => opt_string += "i,",
                "multiline" => opt_string += "m,",
                "explicitcapture" => opt_string += "n,",
                "singleline" => opt_string += "s,",
                "ignorepatternwhitespace" => opt_string += "x,",
                _ => opt_string += "",
            }
            offset += 1;
        }
        if !opt_string.is_empty() {
            opt_string.pop(); // extra trailing comma
        }
    }
    let options = CString::new(opt_string).unwrap();

    unsafe {
        let string = NResult::consume(n_regex_match(
            input.as_ptr(),
            regex.as_ptr(),
            group.as_ptr(),
            options.as_ptr(),
        ))?;

        Ok(Value::string(string.0, vm))
    }
}

pub fn replace(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY: u8 = 3;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::String, // input
        ArgType::String, // target
        ArgType::String, // replacement
                         // -> String
    ];
    let stack = &vm.stack;
    functions::check_arity(ARITY, arg_count)?;
    functions::check_arg_types(ARG_TYPES, arg_start, vm)?;

    let mut string = stack[arg_start].to_string(vm);
    let target = stack[arg_start + 1].to_string(vm);
    let replacement = stack[arg_start + 2].to_string(vm);

    string = string.replace(&target, &replacement);

    Ok(Value::string(string, vm))
}

pub fn replace_list(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY_MIN: u8 = 3;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::String, // input
        ArgType::String, // replacement
        ArgType::String, // target
                         // (targets ...): String
                         // -> String
    ];
    let stack = &vm.stack;
    functions::check_arity_min(ARITY_MIN, arg_count)?;
    functions::check_arg_types(ARG_TYPES, arg_start, vm)?;

    let mut string = stack[arg_start].to_string(vm);
    let replacement = stack[arg_start + 1].to_string(vm);
    let next_target = arg_start + 2;
    let target_count = stack.len() - next_target;

    let mut i = 0;
    while i < target_count {
        let offset = next_target + i;
        functions::check_arg_types(&[ArgType::String], offset, vm)?;

        let target = stack[offset].to_string(vm);

        string = string.replace(&target, &replacement);
        i += 1;
    }

    Ok(Value::string(string, vm))
}

pub fn string_to_date(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY: u8 = 1;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::String, // input
                         // -> String
    ];
    let stack = &vm.stack;
    functions::check_arity(ARITY, arg_count)?;
    functions::check_arg_types(ARG_TYPES, arg_start, vm)?;

    let date = stack[arg_start].to_c_string(vm)?;

    unsafe {
        let string = NResult::consume(n_string_to_date(date.as_ptr()))?;

        Ok(Value::string(string.0, vm))
    }
}

pub fn string_to_hex(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
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
        encoding::char_encoding(stack[offset].to_lower(vm).as_str())?
    } else {
        CharEncoding::Utf8
    };

    let input_u16;
    if encoding == CharEncoding::Utf16 {
        input_u16 = input.encode_utf16().collect::<Vec<_>>();
        bytes = bytemuck::pod_align_to::<u16, u8>(&input_u16).1;
    }

    let mut hex_string = String::new();

    use std::fmt::Write as _;
    for b in bytes {
        let Ok(_) = write!(hex_string, "{b:02x}") else {
            return Err(Value::error(format!("Unable to create hex string from `{input}`")));
        };
    }

    Ok(Value::string(hex_string, vm))
}

pub fn substring(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY_MIN: u8 = 2;
    const ARITY_MAX: u8 = 3;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::String, // input
        ArgType::Number, // start
                         // (length): Number
                         // -> String
    ];
    let stack = &vm.stack;
    functions::check_arity_range(ARITY_MIN, ARITY_MAX, arg_count)?;
    functions::check_arg_types(ARG_TYPES, arg_start, vm)?;

    let string = stack[arg_start].to_string(vm);
    let start = f64::trunc(stack[arg_start + 1].to_num(vm));
    if start < 1f64 {
        return Err(Value::error(
            "Starting position for substring must be greater than 0.".to_owned(),
        ));
    }

    let clstr_count = string.graphemes(true).count();
    if start as usize > clstr_count {
        return Ok(Value::string(String::from(""), vm));
    }

    let position = start as usize - 1;
    let end = if arg_count == ARITY_MAX {
        let arg_offset = arg_start + 2;
        functions::check_arg_types(&[ArgType::Number], arg_offset, vm)?;

        let length = f64::trunc(stack[arg_offset].to_num(vm));
        if length < 1f64 {
            return Err(Value::error(
                "Specified length for substring must be greater than 0.".to_owned(),
            ));
        }

        let offset = position + length as usize;
        if offset > clstr_count {
            clstr_count
        } else {
            offset
        }
    } else {
        clstr_count
    };

    let mut i = position;
    let mut substring = String::new();
    let graphemes = string.graphemes(true).collect::<Vec<_>>();

    while i < end {
        substring.push_str(graphemes[i]);
        i += 1;
    }

    Ok(Value::string(substring, vm))
}

pub fn trim(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY: u8 = 1;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::String, // input
                         // -> String
    ];
    let stack = &vm.stack;
    functions::check_arity(ARITY, arg_count)?;
    functions::check_arg_types(ARG_TYPES, arg_start, vm)?;

    let string = stack[arg_start].to_string(vm).trim().to_owned();

    Ok(Value::string(string, vm))
}

pub fn uppercase(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY: u8 = 1;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::String, // input
                         // -> String
    ];
    let stack = &vm.stack;
    functions::check_arity(ARITY, arg_count)?;
    functions::check_arg_types(ARG_TYPES, arg_start, vm)?;

    let string = stack[arg_start].to_upper(vm);

    Ok(Value::string(string, vm))
}

//

fn data_format_id(data_format: &str) -> Result<c_int, Value> {
    let id = match data_format {
        "" => 0,
        "date" => 1,
        "number" => 2,
        _ => {
            return Err(Value::error(
                "Accepted case-insensitive values for data format are `` (empty), `date`, and `number`."
                    .to_owned(),
            ));
        }
    };

    Ok(id)
}
