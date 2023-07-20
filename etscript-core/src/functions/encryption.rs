use crate::functions::{
    self,
    encoding::{self, CharEncoding},
    ArgType,
};
use crate::value::Value;
use crate::vm::Vm;

use md5::Md5;
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha512};

/// Warning: The MD5 algorithm should be considered cryptographically broken and
/// unsuitable for further use. It is included for compatibility only.
/// See: https://www.kb.cert.org/vuls/id/836068
pub fn md5(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    hash(Hasher::Md5, arg_start, arg_count, vm)
}

/// Warning: The SHA-1 algorithm should be considered cryptographically broken and
/// unsuitable for further use. It is included for compatibility only.
/// See: https://sha-mbles.github.io/
pub fn sha1(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    hash(Hasher::Sha1, arg_start, arg_count, vm)
}

pub fn sha256(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    hash(Hasher::Sha256, arg_start, arg_count, vm)
}

pub fn sha512(arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
    hash(Hasher::Sha512, arg_start, arg_count, vm)
}

//

enum Hasher {
    Md5,
    Sha1,
    Sha256,
    Sha512,
}

fn hash(hasher: Hasher, arg_start: usize, arg_count: u8, vm: &mut Vm) -> Result<Value, Value> {
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

    let hash = match hasher {
        Hasher::Md5 => {
            let mut h = Md5::new();
            h.update(bytes);
            format!("{:x}", h.finalize())
        }
        Hasher::Sha1 => {
            let mut h = Sha1::new();
            h.update(bytes);
            format!("{:x}", h.finalize())
        }
        Hasher::Sha256 => {
            let mut h = Sha256::new();
            h.update(bytes);
            format!("{:x}", h.finalize())
        }
        Hasher::Sha512 => {
            let mut h = Sha512::new();
            h.update(bytes);
            format!("{:x}", h.finalize())
        }
    };

    Ok(Value::string(hash, vm))
}
