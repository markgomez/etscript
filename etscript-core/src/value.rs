use crate::object::{NativeFn, NativeFnObj, Obj, ObjType, RowObj, RowsetObj, StringObj};
use crate::vm::Vm;

use std::collections::HashMap;
use std::ffi::CString;

#[derive(Clone)]
pub enum ValueType {
    Null(u8),
    Number(f64),
    Boolean(bool),
    Obj(Obj),
}

#[derive(Clone)]
pub struct Value {
    pub type_: ValueType,
}

#[macro_export]
macro_rules! truthy_pattern {
    () => {
        "1" | "true" | "t" | "yes" | "y"
    };
}

#[macro_export]
macro_rules! falsey_pattern {
    () => {
        "0" | "false" | "f" | "no" | "n"
    };
}

impl Value {
    pub fn null() -> Self {
        Self {
            type_: ValueType::Null(0u8),
        }
    }

    pub fn num(num: f64) -> Self {
        Self {
            type_: ValueType::Number(num),
        }
    }

    pub fn boolean(boolean: bool) -> Self {
        Self {
            type_: ValueType::Boolean(boolean),
        }
    }

    pub fn nfn(hash: u64, nfn: NativeFn) -> Self {
        Self::from(Obj {
            type_: ObjType::NativeFn(NativeFnObj::new(hash, nfn)),
        })
    }

    pub fn string(string: String, vm: &mut Vm) -> Self {
        Self::from(Obj {
            type_: ObjType::String(StringObj::new(string, vm)),
        })
    }

    pub fn datetime(data: (String, i64), vm: &mut Vm) -> Self {
        let mut string_obj = StringObj::new(data.0, vm);

        string_obj.data = data.1;
        string_obj.is_datetime = true;

        Self::from(Obj {
            type_: ObjType::String(string_obj),
        })
    }

    pub fn row(map: HashMap<u64, Self>) -> Self {
        Self::from(Obj {
            type_: ObjType::Row(RowObj::new(map)),
        })
    }

    pub fn rowset(vec: Vec<Self>) -> Self {
        Self::from(Obj {
            type_: ObjType::Rowset(RowsetObj::new(vec)),
        })
    }

    pub fn error(string: String) -> Self {
        Self::from(Obj {
            type_: ObjType::Error(string),
        })
    }

    //

    pub fn is_null(&self) -> bool {
        if let ValueType::Null(_) = self.type_ {
            return true;
        }
        false
    }

    pub fn is_num(&self) -> bool {
        if let ValueType::Number(_) = self.type_ {
            return true;
        }
        false
    }

    pub fn is_num_arg(&self, vm: &Vm) -> (bool, Option<String>) {
        if let ValueType::Number(num) = self.type_ {
            return (true, Some(format!("{num}")));
        }
        if let ValueType::Obj(obj) = &self.type_ {
            if obj.is_string() {
                let string = self.to_string(vm);
                let Ok(_) = string.parse::<f64>() else {
                    return (false, None)
                };
                return (true, Some(string));
            }
        }
        (false, None)
    }

    pub fn is_bool(&self) -> bool {
        if let ValueType::Boolean(_) = self.type_ {
            return true;
        }
        false
    }

    pub fn is_bool_arg(&self, vm: &Vm) -> (bool, Option<String>) {
        if self.is_bool() {
            return (true, None);
        }
        if let ValueType::Number(num) = self.type_ {
            if num == 1f64 || num == 0f64 {
                return (true, Some(format!("{num}")));
            }
        }
        if let ValueType::Obj(obj) = &self.type_ {
            if obj.is_string() {
                let string = self.to_lower(vm);
                match string.as_str() {
                    truthy_pattern!() | falsey_pattern!() => return (true, Some(string)),
                    _ => (),
                }
            }
        }
        (false, None)
    }

    pub fn is_obj(&self) -> bool {
        if let ValueType::Obj(_) = self.type_ {
            return true;
        }
        false
    }

    pub fn is_nfn(&self) -> bool {
        if let ValueType::Obj(obj) = &self.type_ {
            return obj.is_nfn();
        }
        false
    }

    pub fn is_string(&self) -> bool {
        if let ValueType::Obj(obj) = &self.type_ {
            return obj.is_string();
        }
        false
    }

    pub fn is_datetime(&self) -> bool {
        if let ValueType::Obj(obj) = &self.type_ {
            if obj.is_string() {
                let string_obj: StringObj = obj.clone().into();
                return string_obj.is_datetime;
            }
            return false;
        }
        false
    }

    pub fn is_row(&self) -> bool {
        if let ValueType::Obj(obj) = &self.type_ {
            return obj.is_row();
        }
        false
    }

    pub fn is_rowset(&self) -> bool {
        if let ValueType::Obj(obj) = &self.type_ {
            return obj.is_rowset();
        }
        false
    }

    pub fn is_error(&self) -> bool {
        if let ValueType::Obj(obj) = &self.type_ {
            return obj.is_error();
        }
        false
    }

    //

    pub fn to_num(&self, vm: &Vm) -> f64 {
        if let ValueType::Number(num) = self.type_ {
            return num;
        }
        if let ValueType::Obj(obj) = &self.type_ {
            if obj.is_string() {
                if let Ok(num) = self.to_string(vm).parse::<f64>() {
                    return num;
                };
            }
        }
        panic!("Incorrect value was used. Expected `f64` or parsable string.");
    }

    pub fn to_bool(&self, vm: &Vm) -> bool {
        if let ValueType::Boolean(boolean) = self.type_ {
            return boolean;
        }
        if let ValueType::Number(num) = self.type_ {
            match num {
                n if n == 1f64 => return true,
                n if n == 0f64 => return false,
                _ => panic!("Incorrect number value was used. Expected `1f64` or `0f64`."),
            }
        }
        match self.to_lower(vm).as_str() {
            truthy_pattern!() => true,
            falsey_pattern!() => false,
            _ => panic!(
                r"Incorrect string value was used. Expected truthy (`1`, `true`, `t`, `yes`, `y`)
or falsey (`0`, `false`, `f`, `no`, `n`)."
            ),
        }
    }

    pub fn to_string(&self, vm: &Vm) -> String {
        let obj: StringObj = self.clone().into();
        let strings = &vm.strings.borrow();

        obj.get(strings).to_owned()
    }

    pub fn to_string_hash(&self, vm: &Vm) -> (String, u64) {
        let obj: StringObj = self.clone().into();
        let strings = &vm.strings.borrow();

        (obj.get(strings).to_owned(), obj.hash)
    }

    pub fn to_lower(&self, vm: &Vm) -> String {
        let obj: StringObj = self.clone().into();
        let strings = &vm.strings.borrow();

        obj.get(strings).to_lowercase()
    }

    pub fn to_ascii_lower(&self, vm: &Vm) -> String {
        let obj: StringObj = self.clone().into();
        let strings = &vm.strings.borrow();

        obj.get(strings).to_ascii_lowercase()
    }

    pub fn to_upper(&self, vm: &Vm) -> String {
        let obj: StringObj = self.clone().into();
        let strings = &vm.strings.borrow();

        obj.get(strings).to_uppercase()
    }

    pub fn to_ascii_upper(&self, vm: &Vm) -> String {
        let obj: StringObj = self.clone().into();
        let strings = &vm.strings.borrow();

        obj.get(strings).to_ascii_uppercase()
    }

    pub fn to_c_string(&self, vm: &Vm) -> Result<CString, Value> {
        let obj: StringObj = self.clone().into();
        let strings = &vm.strings.borrow();
        let string = obj.get(strings).as_bytes();

        let Ok(c_string) = CString::new(string) else {
            return Err(Value::error(r"Interior nul byte (`\0`) was found.".to_owned()));
        };

        Ok(c_string)
    }

    pub fn to_row(&self) -> RowObj {
        let row: RowObj = self.clone().into();

        row
    }

    pub fn to_rowset(&self) -> RowsetObj {
        let rowset: RowsetObj = self.clone().into();

        rowset
    }

    //

    pub fn is_truthy(&self, vm: &Vm) -> bool {
        if let ValueType::Number(num) = self.type_ {
            return matches!(num, n if n == 1f64);
        }
        if let ValueType::Boolean(boolean) = self.type_ {
            return boolean;
        }
        if let ValueType::Obj(obj) = &self.type_ {
            match obj.type_ {
                ObjType::String(string_obj) => {
                    let string = string_obj.get(&vm.strings.borrow()).to_lowercase();
                    matches!(string.as_str(), truthy_pattern!())
                }
                _ => false,
            }
        } else {
            false
        }
    }

    pub fn are_vals_eq(a: &Self, b: &Self, strings: &HashMap<u64, String>) -> bool {
        match &a.type_ {
            ValueType::Null(_) => b.is_null(),
            ValueType::Number(a) => match &b.type_ {
                ValueType::Null(_) => false,
                ValueType::Number(num) => *a == *num,
                ValueType::Boolean(boolean) => match *a {
                    n if n == 1f64 => *boolean,
                    _ => !(*boolean),
                },
                ValueType::Obj(obj) => match obj.type_ {
                    ObjType::String(string_obj) => {
                        let string = string_obj.get(strings);
                        if let Ok(num) = string.parse::<f64>() {
                            *a == num
                        } else {
                            false
                        }
                    }
                    _ => false,
                },
            },
            ValueType::Boolean(a) => match &b.type_ {
                ValueType::Null(_) => false,
                ValueType::Number(num) => match *num {
                    n if n == 1f64 => *a,
                    _ => !(*a),
                },
                ValueType::Boolean(boolean) => *a == *boolean,
                ValueType::Obj(obj) => match obj.type_ {
                    ObjType::String(string_obj) => {
                        let string = string_obj.get(strings).to_lowercase();
                        match string.as_str() {
                            truthy_pattern!() => *a,
                            _ => !(*a),
                        }
                    }
                    _ => false,
                },
            },
            ValueType::Obj(a) => match a.type_ {
                ObjType::String(obj_a) => match &b.type_ {
                    ValueType::Null(_) => false,
                    ValueType::Number(num_b) => {
                        let string = obj_a.get(strings);
                        if let Ok(num_a) = string.parse::<f64>() {
                            num_a == *num_b
                        } else {
                            false
                        }
                    }
                    ValueType::Boolean(boolean) => {
                        let string = obj_a.get(strings).to_lowercase();
                        match string.as_str() {
                            truthy_pattern!() => *boolean,
                            _ => !(*boolean),
                        }
                    }
                    ValueType::Obj(b) => match b.type_ {
                        ObjType::String(obj_b) => {
                            if obj_a.is_datetime && obj_b.is_datetime {
                                obj_a.data == obj_b.data
                            } else {
                                obj_a.get(strings) == obj_b.get(strings)
                            }
                        }
                        _ => false,
                    },
                },
                _ => false,
            },
        }
    }

    pub fn print(&self, strings: &HashMap<u64, String>) {
        match &self.type_ {
            ValueType::Null(_) => {
                print!("<null>");
            }
            ValueType::Number(num) => {
                print!("{num}");
            }
            ValueType::Boolean(boolean) => {
                print!("{boolean}");
            }
            ValueType::Obj(obj) => match &obj.type_ {
                ObjType::NativeFn(_) => {
                    print!("<fn>");
                }
                ObjType::String(string_obj) => {
                    print!("{}", string_obj.get(strings));
                }
                ObjType::Row(_) => {
                    print!("<row>");
                }
                ObjType::Rowset(_) => {
                    print!("<rowset>");
                }
                ObjType::Error(string) => {
                    print!("Error: {string}");
                }
            },
        }
    }
}

//

impl From<f64> for Value {
    fn from(num: f64) -> Self {
        Self {
            type_: ValueType::Number(num),
        }
    }
}

impl From<Value> for f64 {
    fn from(val: Value) -> Self {
        let ValueType::Number(num) = val.type_ else {
            panic!("Incorrect variant of `ValueType` was used. Expected `ValueType::Number`.");
        };
        num
    }
}

impl From<bool> for Value {
    fn from(boolean: bool) -> Self {
        Self {
            type_: ValueType::Boolean(boolean),
        }
    }
}

impl From<Value> for bool {
    fn from(val: Value) -> Self {
        let ValueType::Boolean(boolean) = val.type_ else {
            panic!("Incorrect variant of `ValueType` was used. Expected `ValueType::Boolean`.");
        };
        boolean
    }
}

impl From<Obj> for Value {
    fn from(obj: Obj) -> Self {
        Self {
            type_: ValueType::Obj(obj),
        }
    }
}

impl From<Value> for Obj {
    fn from(val: Value) -> Self {
        let ValueType::Obj(obj) = val.type_ else {
            panic!("Incorrect variant of `ValueType` was used. Expected `ValueType::Obj`.");
        };
        obj
    }
}

impl From<Value> for NativeFnObj {
    fn from(val: Value) -> Self {
        let ValueType::Obj(obj) = val.type_ else {
            panic!("Incorrect variant of `ValueType` was used. Expected `ValueType::Obj`.");
        };
        obj.into()
    }
}

impl From<Value> for StringObj {
    fn from(val: Value) -> Self {
        let ValueType::Obj(obj) = val.type_ else {
            panic!("Incorrect variant of `ValueType` was used. Expected `ValueType::Obj`.");
        };
        obj.into()
    }
}

impl From<Value> for RowObj {
    fn from(val: Value) -> Self {
        let ValueType::Obj(obj) = val.type_ else {
            panic!("Incorrect variant of `ValueType` was used. Expected `ValueType::Obj`.");
        };
        obj.into()
    }
}

impl From<Value> for RowsetObj {
    fn from(val: Value) -> Self {
        let ValueType::Obj(obj) = val.type_ else {
            panic!("Incorrect variant of `ValueType` was used. Expected `ValueType::Obj`.");
        };
        obj.into()
    }
}
