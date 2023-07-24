pub mod content;
pub mod data_extension;
pub mod datetime;
pub mod encoding;
pub mod encryption;
pub mod math;
pub mod string;
pub mod utilities;

use crate::object;
use crate::value::Value;
use crate::vm::Vm;

use libc::{c_char, c_int, c_longlong};
use std::ffi::CStr;

#[repr(C)]
pub struct NResult {
    value: *const c_char,
    data: c_longlong,
    status: c_int,
}

extern "C" {
    fn free_n_result(ptr: *const NResult);
}

impl NResult {
    pub fn consume(ptr: *const NResult) -> Result<(String, i64), Value> {
        unsafe {
            let value = CStr::from_ptr((*ptr).value);
            let data = (*ptr).data;
            let status = (*ptr).status;

            let Ok(result) = value.to_str() else {
                return Err(Value::error("Invalid UTF-8 string.".to_owned()));
            };

            let string = result.to_owned();
            free_n_result(ptr);

            if status != 0 {
                return Err(Value::error(string));
            }

            Ok((string, data))
        }
    }
}

//

pub fn init(vm: &mut Vm) {
    let define_fn = |name: &str, nfn, vm: &mut Vm| {
        let hash = object::intern(name.to_owned(), vm);
        vm.funcs.insert(hash, Value::nfn(hash, nfn));
    };

    // Content
    define_fn(
        "buildrowsetfromstring",
        content::build_rowset_from_string,
        vm,
    );

    // Data Extension
    define_fn(
        "dataextensionrowcount",
        data_extension::data_extension_row_count,
        vm,
    );
    define_fn("deletedata", data_extension::delete_data, vm);
    define_fn("deletede", data_extension::delete_de, vm);
    define_fn("field", data_extension::field, vm);
    define_fn("insertdata", data_extension::insert_data, vm);
    define_fn("insertde", data_extension::insert_de, vm);
    define_fn("lookup", data_extension::lookup, vm);
    define_fn("lookuporderedrows", data_extension::lookup_ordered_rows, vm);
    define_fn(
        "lookuporderedrowscs",
        data_extension::lookup_ordered_rows_cs,
        vm,
    );
    define_fn("lookuprows", data_extension::lookup_rows, vm);
    define_fn("lookuprowscs", data_extension::lookup_rows_cs, vm);
    define_fn("row", data_extension::row, vm);
    define_fn("rowcount", data_extension::row_count, vm);
    define_fn("updatedata", data_extension::update_data, vm);
    define_fn("updatede", data_extension::update_de, vm);
    define_fn("upsertdata", data_extension::upsert_data, vm);
    define_fn("upsertde", data_extension::upsert_de, vm);

    // Date & Time (.NET)
    define_fn("dateadd", datetime::date_add, vm);
    define_fn("datediff", datetime::date_diff, vm);
    define_fn("dateparse", datetime::date_parse, vm);
    define_fn("datepart", datetime::date_part, vm);
    define_fn("formatdate", datetime::format_date, vm);
    define_fn(
        "localdatetosystemdate",
        datetime::local_date_to_system_date,
        vm,
    );
    define_fn("now", datetime::now, vm);
    define_fn(
        "systemdatetolocaldate",
        datetime::system_date_to_local_date,
        vm,
    );

    // Encoding
    define_fn("base64decode", encoding::base64_decode, vm);
    define_fn("base64encode", encoding::base64_encode, vm);
    define_fn("guid", encoding::guid, vm);

    // Encryption
    define_fn("md5", encryption::md5, vm);
    define_fn("sha1", encryption::sha1, vm);
    define_fn("sha256", encryption::sha256, vm);
    define_fn("sha512", encryption::sha512, vm);

    // Math
    define_fn("add", math::add, vm);
    define_fn("divide", math::divide, vm);
    define_fn("formatcurrency", math::format_currency, vm); // .NET
    define_fn("formatnumber", math::format_number, vm); // .NET
    define_fn("mod", math::mod_, vm);
    define_fn("multiply", math::multiply, vm);
    define_fn("random", math::random, vm);
    define_fn("subtract", math::subtract, vm);

    // String
    define_fn("char", string::char_, vm);
    define_fn("concat", string::concat, vm);
    define_fn("format", string::format, vm); // .NET
    define_fn("indexof", string::index_of, vm);
    define_fn("length", string::length, vm);
    define_fn("lowercase", string::lowercase, vm);
    define_fn("propercase", string::proper_case, vm); // .NET
    define_fn("regexmatch", string::regex_match, vm); // .NET
    define_fn("replace", string::replace, vm);
    define_fn("replacelist", string::replace_list, vm);
    define_fn("stringtodate", string::string_to_date, vm); // .NET
    define_fn("stringtohex", string::string_to_hex, vm);
    define_fn("substring", string::substring, vm);
    define_fn("trim", string::trim, vm);
    define_fn("uppercase", string::uppercase, vm);

    // Utilities
    define_fn("empty", utilities::empty, vm);
    define_fn("iif", utilities::iif, vm);
    define_fn("isemailaddress", utilities::is_email_address, vm);
    define_fn("isnull", utilities::is_null, vm);
    // `output` and `outputline` are handled at compile time
    define_fn("v", utilities::v, vm);
}

//

pub fn check_arity(arity: u8, arg_count: u8) -> Result<(), Value> {
    if arg_count != arity {
        return Err(Value::error(format!("Unexpected number of arguments passed to function — got {arg_count}, but expected {arity}.")));
    }

    Ok(())
}

pub fn check_arity_min(arity_min: u8, arg_count: u8) -> Result<(), Value> {
    if arg_count < arity_min {
        return Err(Value::error(format!("Unexpected number of arguments passed to function — got {arg_count}, but expected at least {arity_min}.")));
    }

    Ok(())
}

pub fn check_arity_max(arity_max: u8, arg_count: u8) -> Result<(), Value> {
    if arg_count > arity_max {
        return Err(Value::error(format!("Unexpected number of arguments passed to function — got {arg_count}, but expected {arity_max} at most.")));
    }

    Ok(())
}

pub fn check_arity_range(arity_min: u8, arity_max: u8, arg_count: u8) -> Result<(), Value> {
    if arg_count < arity_min || arg_count > arity_max {
        return Err(Value::error(format!("Unexpected number of arguments passed to function — got {arg_count}, but expected at least {arity_min}, {arity_max} at most.")));
    }

    Ok(())
}

pub enum ArgType {
    Number,
    Boolean,
    String,
    Row,
    Rowset,
    Value,
}

pub fn check_arg_types(arg_types: &[ArgType], arg_start: usize, vm: &Vm) -> Result<(), Value> {
    let stack = &vm.stack;
    let mut offset = arg_start;

    for arg_type in arg_types {
        let val = &stack[offset];
        let result = match arg_type {
            ArgType::Number => ("number", val.is_num_arg(vm).0),
            ArgType::Boolean => ("boolean", val.is_bool_arg(vm).0),
            ArgType::String => ("string", val.is_string()),
            ArgType::Row => ("row", val.is_row()),
            ArgType::Rowset => ("rowset", val.is_rowset()),
            ArgType::Value => ("value", true),
        };
        if !result.1 {
            return Err(Value::error(format!(
                "Unexpected type passed to function — expected a {}.",
                result.0
            )));
        }
        offset += 1
    }

    Ok(())
}
