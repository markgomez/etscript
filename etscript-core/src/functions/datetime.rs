use crate::functions::{self, ArgType, NResult};
use crate::value::Value;
use crate::vm::Vm;
use std::ffi::CString;

use libc::{c_char, c_int};

extern "C" {
    fn n_date_add(date: *const c_char, addend: c_int, unit: c_int) -> *const NResult;

    fn n_date_diff(
        minuend: *const c_char,
        subtrahend: *const c_char,
        unit: c_int,
    ) -> *const NResult;

    fn n_date_parse(date: *const c_char, as_utc: c_int) -> *const NResult;

    fn n_date_part(date: *const c_char, part: c_int) -> *const NResult;

    fn n_format_date(
        date: *const c_char,
        date_format: *const c_char,
        time_format: *const c_char,
        culture: *const c_char,
    ) -> *const NResult;

    fn n_local_date_to_system_date(date: *const c_char) -> *const NResult;

    fn n_now() -> *const NResult; // parameter for send time preservation omitted

    fn n_system_date_to_local_date(date: *const c_char) -> *const NResult;
}

//

pub fn date_add(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY: u8 = 3;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::String, // date
        ArgType::Number, // addend
        ArgType::String, // unit
                         // -> String
    ];
    const ADDEND_MAX: isize = i32::MAX as isize;
    let stack = &vm.stack;
    functions::check_arity(ARITY, arg_count)?;
    functions::check_arg_types(ARG_TYPES, arg_start, vm)?;

    let date = stack[arg_start].to_c_string(vm)?;
    let addend = stack[arg_start + 1].to_num(vm) as c_int;
    if addend > ADDEND_MAX as c_int {
        return Err(Value::error("Addend limit exceeded.".to_owned()));
    }
    let unit = stack[arg_start + 2].to_lower(vm);

    unsafe {
        let dt = NResult::consume(n_date_add(date.as_ptr(), addend, date_unit_id(&unit)?))?;

        Ok(Value::datetime(dt, vm))
    }
}

pub fn date_diff(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY: u8 = 3;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::String, // date
        ArgType::String, // date
        ArgType::String, // unit
                         // -> Number
    ];
    let stack = &vm.stack;
    functions::check_arity(ARITY, arg_count)?;
    functions::check_arg_types(ARG_TYPES, arg_start, vm)?;

    let minuend = stack[arg_start].to_c_string(vm)?;
    let subtrahend = stack[arg_start + 1].to_c_string(vm)?;
    let unit = stack[arg_start + 2].to_lower(vm);

    unsafe {
        let string = NResult::consume(n_date_diff(
            minuend.as_ptr(),
            subtrahend.as_ptr(),
            date_unit_id(&unit)?,
        ))?;

        let Ok(num) = string.0.parse::<f64>() else {
            return Err(Value::error("Invalid string representation of a date or time.".to_owned()));
        };

        Ok(Value::num(num))
    }
}

pub fn date_parse(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY_MIN: u8 = 1;
    const ARITY_MAX: u8 = 2;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::String, // date
                         // (as utc): Boolean
                         // -> String
    ];
    let stack = &vm.stack;
    functions::check_arity_range(ARITY_MIN, ARITY_MAX, arg_count)?;
    functions::check_arg_types(ARG_TYPES, arg_start, vm)?;

    let date = stack[arg_start].to_c_string(vm)?;
    let mut as_utc = false;

    if arg_count == ARITY_MAX {
        let offset = arg_start + 1;
        functions::check_arg_types(&[ArgType::Boolean], offset, vm)?;
        as_utc = stack[offset].to_bool(vm);
    }

    unsafe {
        let dt = NResult::consume(n_date_parse(date.as_ptr(), as_utc as i32))?;

        Ok(Value::datetime(dt, vm))
    }
}

pub fn date_part(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY: u8 = 2;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::String, // date
        ArgType::String, // name or unit
                         // -> String
    ];
    let stack = &vm.stack;
    functions::check_arity(ARITY, arg_count)?;
    functions::check_arg_types(ARG_TYPES, arg_start, vm)?;

    let date = stack[arg_start].to_c_string(vm)?;
    let part = stack[arg_start + 1].to_lower(vm);

    unsafe {
        let string = NResult::consume(n_date_part(date.as_ptr(), date_part_id(&part)?))?;

        Ok(Value::string(string.0, vm))
    }
}

pub fn format_date(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY_MIN: u8 = 1;
    const ARITY_MAX: u8 = 4;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::String, // date
                         // (date format): String
                         // (time format): String
                         // (culture): String
                         // -> String
    ];
    let stack = &vm.stack;
    functions::check_arity_range(ARITY_MIN, ARITY_MAX, arg_count)?;
    functions::check_arg_types(ARG_TYPES, arg_start, vm)?;

    let date = stack[arg_start].to_c_string(vm)?;

    let date_format = if arg_count > ARITY_MIN {
        let offset = arg_start + 1;
        functions::check_arg_types(&[ArgType::String], offset, vm)?;
        stack[offset].to_c_string(vm)?
    } else {
        CString::default()
    };

    let time_format = if arg_count >= ARITY_MIN + 2 {
        let offset = arg_start + 2;
        functions::check_arg_types(&[ArgType::String], offset, vm)?;
        stack[offset].to_c_string(vm)?
    } else {
        CString::default()
    };

    let culture = if arg_count == ARITY_MAX {
        let offset = arg_start + 3;
        functions::check_arg_types(&[ArgType::String], offset, vm)?;
        stack[offset].to_c_string(vm)?
    } else {
        CString::default()
    };

    unsafe {
        let dt = NResult::consume(n_format_date(
            date.as_ptr(),
            date_format.as_ptr(),
            time_format.as_ptr(),
            culture.as_ptr(),
        ))?;

        Ok(Value::datetime(dt, vm))
    }
}

pub fn local_date_to_system_date(
    arg_start: usize,
    arg_count: u8,
    vm: &mut Vm,
) -> Result<Value, Value> {
    const ARITY: u8 = 1;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::String, // date
                         // -> String
    ];
    let stack = &vm.stack;
    functions::check_arity(ARITY, arg_count)?;
    functions::check_arg_types(ARG_TYPES, arg_start, vm)?;

    let date = stack[arg_start].to_c_string(vm)?;

    unsafe {
        let dt = NResult::consume(n_local_date_to_system_date(date.as_ptr()))?;

        Ok(Value::datetime(dt, vm))
    }
}

pub fn now(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    const ARITY_MAX: u8 = 1; // (preserve send time): Boolean ... -> String
    let _stack = &vm.stack;
    functions::check_arity_max(ARITY_MAX, arg_count)?;

    if arg_count == ARITY_MAX {
        let offset = arg_start;
        functions::check_arg_types(&[ArgType::Boolean], offset, vm)?;
        // preserving send time is not supported (out of scope)
    }

    unsafe {
        let dt = NResult::consume(n_now())?;

        Ok(Value::datetime(dt, vm))
    }
}

pub fn system_date_to_local_date(
    arg_start: usize,
    arg_count: u8,
    vm: &mut Vm,
) -> Result<Value, Value> {
    const ARITY: u8 = 1;
    const ARG_TYPES: &[ArgType] = &[
        ArgType::String, // date
                         // -> String
    ];
    let stack = &vm.stack;
    functions::check_arity(ARITY, arg_count)?;
    functions::check_arg_types(ARG_TYPES, arg_start, vm)?;

    let date = stack[arg_start].to_c_string(vm)?;

    unsafe {
        let dt = NResult::consume(n_system_date_to_local_date(date.as_ptr()))?;

        Ok(Value::datetime(dt, vm))
    }
}

//

fn date_part_id(date_part: &str) -> Result<c_int, Value> {
    let id = match date_part {
        "year" | "y" => 1,
        "month" | "m" => 2,
        "day" | "d" => 3,
        "hour" | "h" => 4,
        "minute" | "mi" => 5,
        _ => {
            return Err(Value::error(
                "Accepted case-insensitive values for the date part are `year` (or `y`), \
                    `month` (or `m`), `day` (or `d`), `hour` (or `h`), and `minute` (or `mi`)."
                    .to_owned(),
            ));
        }
    };

    Ok(id)
}

fn date_unit_id(date_unit: &str) -> Result<c_int, Value> {
    let id = match date_unit {
        "y" => 1,
        "m" => 2,
        "d" => 3,
        "h" => 4,
        "mi" => 5,
        _ => {
            return Err(Value::error(
                "Accepted case-insensitive values for the date-time unit are `y` (year), \
                    `m` (month), `d` (day), `h` (hour), and `mi` (minute)."
                    .to_owned(),
            ));
        }
    };

    Ok(id)
}
