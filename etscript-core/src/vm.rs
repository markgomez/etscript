use crate::bytecode::{Bytecode, Opcode};
use crate::compiler::Compiler;
use crate::debug::{self, STYLE_DIM, STYLE_RESET, STYLE_YELLOW};
use crate::functions;
use crate::object::{NativeFnObj, ObjType, StringObj};
use crate::value::{Value, ValueType};

use std::{
    collections::HashMap,
    {cell::RefCell, rc::Rc},
};

pub enum Status {
    Ok,
    InputError,
    DatabaseError,
    CompileError,
    RuntimeError,
}

pub struct Vm {
    pub stack: Vec<Value>,
    current_offset: usize,
    pub bc: Bytecode,
    pub strings: Rc<RefCell<HashMap<u64, String>>>,
    globals: HashMap<u64, Value>,
    pub funcs: HashMap<u64, Value>,
    pub result: Rc<RefCell<String>>,
}

impl Vm {
    const STACK_MAX: usize = (u8::MAX as usize + 1) * 64;

    pub fn new(strings: Rc<RefCell<HashMap<u64, String>>>, result: Rc<RefCell<String>>) -> Self {
        Self {
            stack: Vec::with_capacity(Self::STACK_MAX),
            current_offset: 0,
            bc: Bytecode::default(),
            strings,
            globals: HashMap::new(),
            funcs: HashMap::new(),
            result,
        }
    }

    pub fn reset(&mut self) {
        self.stack.clear();
        self.current_offset = 0;
        self.bc.clear();

        self.strings.borrow_mut().clear();
        self.strings.borrow_mut().shrink_to_fit();
        self.globals.clear();
        self.globals.shrink_to_fit();
        self.funcs.clear();
        self.funcs.shrink_to_fit();
    }

    //

    fn push(&mut self, val: Value) {
        self.stack.push(val);
    }

    fn pop(&mut self) -> Value {
        if let Some(val) = self.stack.pop() {
            return val;
        }

        Value::null()
    }

    //

    fn peek(&self, offset: usize) -> &Value {
        &self.stack[self.stack.len() - 1 - offset]
    }

    fn read_byte(&mut self) -> u8 {
        self.current_offset += 1;

        self.bc.byte_at(self.current_offset - 1)
    }

    fn read_bytes(&mut self) -> u16 {
        self.current_offset += 2;

        ((self.bc.byte_at(self.current_offset - 2) as u16) << 8)
            | self.bc.byte_at(self.current_offset - 1) as u16
    }

    fn read_const(&self, offset: usize) -> Value {
        self.bc.const_at(offset).clone()
    }

    fn write(&mut self, val: &Value) {
        match &val.type_ {
            ValueType::Null(_) => {
                *self.result.borrow_mut() += "";
            }
            ValueType::Number(num) => {
                *self.result.borrow_mut() += &format!("{num}");
            }
            ValueType::Boolean(boolean) => {
                *self.result.borrow_mut() += &format!("{boolean}");
            }
            ValueType::Obj(obj) => match &obj.type_ {
                ObjType::NativeFn(_) => {
                    *self.result.borrow_mut() += "";
                }
                ObjType::String(string_obj) => {
                    *self.result.borrow_mut() += string_obj.get(&self.strings.borrow());
                }
                ObjType::Row(_) => {
                    *self.result.borrow_mut() += "";
                }
                ObjType::Rowset(_) => {
                    *self.result.borrow_mut() += "";
                }
                ObjType::Error(string) => {
                    self.result.borrow_mut().clear();
                    *self.result.borrow_mut() += "Error: ";
                    *self.result.borrow_mut() += string;
                }
            },
        }
    }

    //

    pub fn run(&mut self, source: &'static str) -> Result<(), Status> {
        macro_rules! binary_op {
            ($op:tt) => {
                let mut do_compare = true;
                let mut b = 0f64;
                let mut a = 0f64;

                if self.peek(0).is_num() {
                    b = (*self).pop().into();
                } else if self.peek(0).is_datetime() {
                    let obj_b: StringObj = (*self).pop().into();
                    b = obj_b.data as f64;
                } else {
                    do_compare = false;
                }

                if do_compare {
                    if self.peek(0).is_num() {
                        a = (*self).pop().into();
                    } else if self.peek(0).is_datetime() {
                        let obj_a: StringObj = (*self).pop().into();
                        a = obj_a.data as f64;
                    } else {
                        do_compare = false;
                    }
                }

                if !do_compare {
                    *self.result.borrow_mut() = "Operands must be numbers.".to_owned();
                    return Err(Status::RuntimeError);
                }
                self.push(Value::from(a $op b));
            }
        }

        if self.funcs.is_empty() {
            functions::init(self);
        }

        Compiler::new(&mut *self, source).compile()?;

        let trace_exec = cfg!(debug_assertions) && option_env!("TRACE_EXEC").is_some();

        loop {
            if trace_exec {
                if self.current_offset > 0 {
                    if self.stack.is_empty() {
                        print!("{STYLE_DIM}{: >12}{STYLE_RESET}", "[]");
                    } else {
                        print!("{: >10}", "");
                    }
                    for val in &self.stack {
                        print!("[ {STYLE_YELLOW}");
                        val.print(&self.strings.borrow());
                        print!("{STYLE_RESET} ]");
                    }
                    println!();
                }
                debug::disassemble_instruction(
                    &self.bc,
                    self.current_offset,
                    &self.strings.borrow(),
                );
            }

            let opcode = self.read_byte();

            match opcode {
                b if b == Opcode::Constant as u8 || b == Opcode::ConstantShort as u8 => {
                    let offset = if b == Opcode::ConstantShort as u8 {
                        self.read_bytes() as usize
                    } else {
                        self.read_byte() as usize
                    };
                    let val = self.read_const(offset);

                    self.push(val);
                }

                b if b == Opcode::DefineGlobal as u8 || b == Opcode::DefineGlobalShort as u8 => {
                    let offset = if b == Opcode::DefineGlobalShort as u8 {
                        self.read_bytes() as usize
                    } else {
                        self.read_byte() as usize
                    };
                    let ident = self.read_const(offset);
                    let string_obj: StringObj = ident.into();
                    let hash = string_obj.hash;

                    self.globals.insert(hash, self.peek(0).clone());
                    self.pop();
                }

                b if b == Opcode::GetGlobal as u8 || b == Opcode::GetGlobalShort as u8 => {
                    let offset = if b == Opcode::GetGlobalShort as u8 {
                        self.read_bytes() as usize
                    } else {
                        self.read_byte() as usize
                    };
                    let ident = self.read_const(offset);
                    let string_obj: StringObj = ident.into();
                    let name = string_obj.get(&self.strings.borrow()).to_owned();
                    let hash = string_obj.hash;

                    let Some(val) = self.globals.get(&hash) else {
                        *self.result.borrow_mut() = format!("Undefined variable: `{name}`.");
                        return Err(Status::RuntimeError);
                    };

                    self.push(val.clone());
                }

                b if b == Opcode::SetGlobal as u8 || b == Opcode::SetGlobalShort as u8 => {
                    let offset = if b == Opcode::SetGlobalShort as u8 {
                        self.read_bytes() as usize
                    } else {
                        self.read_byte() as usize
                    };
                    let ident = self.read_const(offset);
                    let string_obj: StringObj = ident.into();
                    let name = string_obj.get(&self.strings.borrow()).to_owned();
                    let hash = string_obj.hash;
                    if !self.globals.contains_key(&hash) {
                        *self.result.borrow_mut() = format!("Undefined variable: `{name}`.");

                        return Err(Status::RuntimeError);
                    }
                    self.globals.insert(hash, self.peek(0).clone());
                }

                b if b == Opcode::NativeFn as u8 || b == Opcode::NativeFnShort as u8 => {
                    let offset = if b == Opcode::NativeFnShort as u8 {
                        self.read_bytes() as usize
                    } else {
                        self.read_byte() as usize
                    };
                    let ident = self.read_const(offset);
                    let string_obj: StringObj = ident.into();
                    let hash = string_obj.hash;

                    let Some(val) = self.funcs.get(&hash) else {
                        *self.result.borrow_mut() = format!(
                            "Undefined function: `{}`.",
                            string_obj.get(&self.strings.borrow())
                        );
                        return Err(Status::RuntimeError);
                    };

                    self.push(val.clone());
                }

                b if b == Opcode::Call as u8 => {
                    let arg_count = self.read_byte();
                    let callee = self.peek(arg_count as usize);

                    if callee.is_nfn() {
                        let obj: NativeFnObj = (*callee).clone().into();
                        let nfn = obj.nfn;
                        let arg_start = self.stack.len() - arg_count as usize;
                        let result = nfn(arg_start, arg_count, self);

                        // only returned `Value` should remain
                        for _ in 0..=arg_count {
                            self.pop();
                        }

                        match result {
                            Ok(val) => self.push(val),
                            Err(val) => {
                                self.write(&val);
                                return Err(Status::RuntimeError);
                            }
                        }
                    } else {
                        *self.result.borrow_mut() = "Callee is not callable.".to_owned();

                        return Err(Status::RuntimeError);
                    }
                }

                //
                b if b == Opcode::GetLocal as u8 => {
                    let offset = self.read_byte() as usize;

                    self.push(self.stack[offset].clone());
                }

                b if b == Opcode::SetLocal as u8 => {
                    let offset = self.read_byte() as usize;

                    self.stack[offset] = self.peek(0).clone();
                }

                //
                b if b == Opcode::Jump as u8 => {
                    let offset = self.read_bytes() as usize;

                    self.current_offset += offset;
                }

                b if b == Opcode::JumpIfFalse as u8 => {
                    let offset = self.read_bytes() as usize;
                    if !self.peek(0).is_truthy(self) {
                        self.current_offset += offset;
                    }
                }

                b if b == Opcode::Loop as u8 => {
                    let offset = self.read_bytes() as usize;

                    self.current_offset -= offset;
                }

                b if b == Opcode::Pass as u8 => {
                    let end: f64 = (*self).pop().into();
                    let start: f64 = (*self).pop().into();

                    *self.result.borrow_mut() += &source[start as usize..end as usize];
                }

                //
                b if b == Opcode::Add as u8 => {
                    binary_op!(+);
                }

                b if b == Opcode::Negate as u8 => {
                    if !self.peek(0).is_num() {
                        *self.result.borrow_mut() = "Operand must be a number.".to_owned();

                        return Err(Status::RuntimeError);
                    }
                    let num: f64 = (*self).pop().into();

                    self.push(Value::num(-num));
                }

                b if b == Opcode::Null as u8 => {
                    self.push(Value::null());
                }

                b if b == Opcode::LineFeed as u8 => {
                    *self.result.borrow_mut() += "\n";
                }

                //
                b if b == Opcode::True as u8 => {
                    self.push(Value::boolean(true));
                }

                b if b == Opcode::False as u8 => {
                    self.push(Value::boolean(false));
                }

                b if b == Opcode::Not as u8 => {
                    let val = self.pop();
                    self.push(Value::boolean(!val.is_truthy(self)));
                }

                //
                b if b == Opcode::Equal as u8 => {
                    let b = self.pop();
                    let a = self.pop();
                    let result = Value::are_vals_eq(&a, &b, &self.strings.borrow());

                    self.push(Value::boolean(result));
                }

                b if b == Opcode::NotEqual as u8 => {
                    let b = self.pop();
                    let a = self.pop();
                    let result = !Value::are_vals_eq(&a, &b, &self.strings.borrow());

                    self.push(Value::boolean(result));
                }

                b if b == Opcode::Less as u8 => {
                    binary_op!(<);
                }

                b if b == Opcode::LessEqual as u8 => {
                    binary_op!(<=);
                }

                b if b == Opcode::Greater as u8 => {
                    binary_op!(>);
                }

                b if b == Opcode::GreaterEqual as u8 => {
                    binary_op!(>=);
                }

                //
                b if b == Opcode::Pop as u8 => {
                    self.pop();
                }

                b if b == Opcode::Write as u8 => {
                    let val = self.pop();
                    self.write(&val);
                }

                b if b == Opcode::Return as u8 => {
                    return Ok(());
                }

                _ => {
                    *self.result.borrow_mut() = format!("Undefined instruction: {opcode}.");

                    return Err(Status::RuntimeError);
                }
            }
        }
    }
}
