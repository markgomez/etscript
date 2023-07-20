use crate::value::Value;

pub enum Opcode {
    Constant,
    ConstantShort,
    DefineGlobal,
    DefineGlobalShort,
    GetGlobal,
    GetGlobalShort,
    SetGlobal,
    SetGlobalShort,
    NativeFn,
    NativeFnShort,
    Call,

    GetLocal,
    SetLocal,

    Jump,
    JumpIfFalse,
    Loop,
    Pass,

    Add,
    Negate,
    Null,
    LineFeed,

    True,
    False,
    Not,

    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,

    Pop,
    Write,
    Return,
}

pub struct Bytecode {
    bytes: Vec<u8>,
    constants: Vec<Value>,
    line_nums: Vec<u16>,
}

impl Default for Bytecode {
    fn default() -> Self {
        Self::new()
    }
}

impl Bytecode {
    pub fn new() -> Self {
        Self {
            bytes: Vec::new(),
            constants: Vec::new(),
            line_nums: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.bytes.clear();
        self.bytes.shrink_to_fit();
        self.constants.clear();
        self.constants.shrink_to_fit();
        self.line_nums.clear();
        self.line_nums.shrink_to_fit();
    }

    //

    pub fn push_byte(&mut self, byte: u8, line_num: u16) {
        self.bytes.push(byte);
        self.line_nums.push(line_num);
    }

    pub fn assign(&mut self, offset: usize, byte: u8) {
        self.bytes[offset] = byte;
    }

    pub fn byte_at(&self, offset: usize) -> u8 {
        self.bytes[offset]
    }

    pub fn byte_count(&self) -> usize {
        self.bytes.len()
    }

    //

    pub fn push_const(&mut self, value: Value) -> usize {
        self.constants.push(value);

        self.constants.len() - 1
    }

    pub fn const_at(&self, offset: usize) -> &Value {
        &self.constants[offset]
    }

    pub fn const_count(&self) -> usize {
        self.constants.len()
    }

    //

    pub fn line_num_at(&self, offset: usize) -> u16 {
        self.line_nums[offset]
    }
}
