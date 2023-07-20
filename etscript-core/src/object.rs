use crate::value::Value;
use crate::vm::Vm;

use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
};

pub type NativeFn = fn(usize, u8, &mut Vm) -> Result<Value, Value>;

#[derive(Clone, Copy)]
pub struct NativeFnObj {
    pub hash: u64,
    pub nfn: NativeFn,
}

impl NativeFnObj {
    pub fn new(hash: u64, nfn: NativeFn) -> Self {
        Self { hash, nfn }
    }
}

#[derive(Clone, Copy)]
pub struct StringObj {
    pub hash: u64,
    pub data: i64,
    pub is_datetime: bool,
}

impl StringObj {
    pub fn new(string: String, vm: &mut Vm) -> Self {
        let hash = intern(string, vm);
        let data = i64::MIN;
        let is_datetime = false;

        Self {
            hash,
            data,
            is_datetime,
        }
    }

    pub fn get<'a>(&self, strings: &'a HashMap<u64, String>) -> &'a String {
        &strings[&self.hash]
    }
}

#[derive(Clone)]
pub struct RowObj {
    pub map: HashMap<u64, Value>,
}

impl RowObj {
    pub fn new(map: HashMap<u64, Value>) -> Self {
        Self { map }
    }
}

#[derive(Clone)]
pub struct RowsetObj {
    pub vec: Vec<Value>,
}

impl RowsetObj {
    pub fn new(vec: Vec<Value>) -> Self {
        Self { vec }
    }
}

#[derive(Clone)]
pub enum ObjType {
    NativeFn(NativeFnObj),
    String(StringObj),
    Row(RowObj),
    Rowset(RowsetObj),
    Error(String),
}

#[derive(Clone)]
pub struct Obj {
    pub type_: ObjType,
}

impl Obj {
    pub fn is_nfn(&self) -> bool {
        if let ObjType::NativeFn(_) = self.type_ {
            return true;
        }
        false
    }

    pub fn is_string(&self) -> bool {
        if let ObjType::String(_) = self.type_ {
            return true;
        }
        false
    }

    pub fn is_row(&self) -> bool {
        if let ObjType::Row(_) = self.type_ {
            return true;
        }
        false
    }

    pub fn is_rowset(&self) -> bool {
        if let ObjType::Rowset(_) = self.type_ {
            return true;
        }
        false
    }

    pub fn is_error(&self) -> bool {
        if let ObjType::Error(_) = self.type_ {
            return true;
        }
        false
    }
}

//

impl From<NativeFnObj> for Obj {
    fn from(nfn_obj: NativeFnObj) -> Self {
        Self {
            type_: ObjType::NativeFn(nfn_obj),
        }
    }
}

impl From<Obj> for NativeFnObj {
    fn from(obj: Obj) -> Self {
        let ObjType::NativeFn(nfn_obj) = obj.type_ else {
            panic!("Incorrect variant of `ObjType` was used. Expected `ObjType::NativeFn`.");
        };
        nfn_obj
    }
}

impl From<StringObj> for Obj {
    fn from(string_obj: StringObj) -> Self {
        Self {
            type_: ObjType::String(string_obj),
        }
    }
}

impl From<Obj> for StringObj {
    fn from(obj: Obj) -> Self {
        let ObjType::String(string_obj) = obj.type_ else {
            panic!("Incorrect variant of `ObjType` was used. Expected `ObjType::String`.");
        };
        string_obj
    }
}

impl From<RowObj> for Obj {
    fn from(row_obj: RowObj) -> Self {
        Self {
            type_: ObjType::Row(row_obj),
        }
    }
}

impl From<Obj> for RowObj {
    fn from(obj: Obj) -> Self {
        let ObjType::Row(row_obj) = obj.type_ else {
            panic!("Incorrect variant of `ObjType` was used. Expected `ObjType::Row`.");
        };
        row_obj
    }
}

impl From<RowsetObj> for Obj {
    fn from(rs_obj: RowsetObj) -> Self {
        Self {
            type_: ObjType::Rowset(rs_obj),
        }
    }
}

impl From<Obj> for RowsetObj {
    fn from(obj: Obj) -> Self {
        let ObjType::Rowset(rs_obj) = obj.type_ else {
            panic!("Incorrect variant of `ObjType` was used. Expected `ObjType::Rowset`.");
        };
        rs_obj
    }
}

//

pub fn hash_of<T: Hash>(t: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    t.hash(&mut hasher);
    hasher.finish()
}

pub fn intern(string: String, vm: &mut Vm) -> u64 {
    let hash = hash_of(&string);

    vm.strings.borrow_mut().entry(hash).or_insert(string);

    hash
}
