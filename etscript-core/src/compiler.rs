use crate::bytecode::Opcode;
use crate::debug;
use crate::lexer::{Lexer, Mode, Token, TokenType};
use crate::value::Value;
use crate::vm::{Status, Vm};

use std::collections::HashMap;
use unicode_segmentation::UnicodeSegmentation;

struct Local {
    name: Token,
    depth: isize,
}

#[derive(Clone, Copy)]
enum Precedence {
    None,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < <= > >=
    Unary,      // - not
    Call,       // ()
}

impl From<u8> for Precedence {
    fn from(level: u8) -> Self {
        match level {
            1 => Precedence::Assignment,
            2 => Precedence::Or,
            3 => Precedence::And,
            4 => Precedence::Equality,
            5 => Precedence::Comparison,
            6 => Precedence::Unary,
            7 => Precedence::Call,
            _ => Precedence::None,
        }
    }
}

type ParseFn = fn(&mut Compiler, bool);

struct ParseRule {
    prefix_fn: Option<ParseFn>,
    infix_fn: Option<ParseFn>,
    prec: Precedence,
}

impl ParseRule {
    fn new(prefix_fn: Option<ParseFn>, infix_fn: Option<ParseFn>, prec: Precedence) -> Self {
        Self {
            prefix_fn,
            infix_fn,
            prec,
        }
    }
}

impl Default for ParseRule {
    fn default() -> Self {
        Self::new(None, None, Precedence::None)
    }
}

struct Parser<'a> {
    lexer: Lexer<'a>,
    rules: HashMap<u8, ParseRule>,
    previous_token: Token,
    current_token: Token,
    had_error: bool,
    in_panic_mode: bool,
}

impl<'a> Parser<'a> {
    #[rustfmt::skip]
    fn init_rules(r: &mut HashMap<u8, ParseRule>) {
r.insert(TokenType::AttrDelim as u8,       ParseRule::default());
r.insert(TokenType::FnDelimOpen as u8,     ParseRule::default());
r.insert(TokenType::FnDelimClose as u8,    ParseRule::default());
r.insert(TokenType::BlockDelimOpen as u8,  ParseRule::default());
r.insert(TokenType::BlockDelimClose as u8, ParseRule::default());

r.insert(TokenType::LeftParen as u8,       ParseRule::new(Some(|c,b| Compiler::grouping(c,b)),  Some(|c,b| Compiler::call(c,b)),   Precedence::Call));
r.insert(TokenType::RightParen as u8,      ParseRule::default());
r.insert(TokenType::Comma as u8,           ParseRule::default());
r.insert(TokenType::Minus as u8,           ParseRule::new(Some(|c,b| Compiler::unary(c,b)),     None,                              Precedence::None));

r.insert(TokenType::Number as u8,          ParseRule::new(Some(|c,b| Compiler::number(c,b)),    None,                              Precedence::None));
r.insert(TokenType::String as u8,          ParseRule::new(Some(|c,b| Compiler::string(c,b)),    None,                              Precedence::None));
r.insert(TokenType::Null as u8,            ParseRule::new(Some(|c,b| Compiler::literal(c,b)),   None,                              Precedence::None));

r.insert(TokenType::True as u8,            ParseRule::new(Some(|c,b| Compiler::literal(c,b)),   None,                              Precedence::None));
r.insert(TokenType::False as u8,           ParseRule::new(Some(|c,b| Compiler::literal(c,b)),   None,                              Precedence::None));

r.insert(TokenType::Not as u8,             ParseRule::new(Some(|c,b| Compiler::unary(c,b)),     None,                              Precedence::None));
r.insert(TokenType::Or as u8,              ParseRule::new(None,                                 Some(|c,b| Compiler::or(c,b)),     Precedence::Or));
r.insert(TokenType::And as u8,             ParseRule::new(None,                                 Some(|c,b| Compiler::and(c,b)),    Precedence::And));

r.insert(TokenType::EqualEqual as u8,      ParseRule::new(None,                                 Some(|c,b| Compiler::binary(c,b)), Precedence::Equality));
r.insert(TokenType::NotEqual as u8,        ParseRule::new(None,                                 Some(|c,b| Compiler::binary(c,b)), Precedence::Equality));
r.insert(TokenType::Less as u8,            ParseRule::new(None,                                 Some(|c,b| Compiler::binary(c,b)), Precedence::Comparison));
r.insert(TokenType::LessEqual as u8,       ParseRule::new(None,                                 Some(|c,b| Compiler::binary(c,b)), Precedence::Comparison));
r.insert(TokenType::Greater as u8,         ParseRule::new(None,                                 Some(|c,b| Compiler::binary(c,b)), Precedence::Comparison));
r.insert(TokenType::GreaterEqual as u8,    ParseRule::new(None,                                 Some(|c,b| Compiler::binary(c,b)), Precedence::Comparison));

r.insert(TokenType::Var as u8,             ParseRule::default());
r.insert(TokenType::Set as u8,             ParseRule::default());
r.insert(TokenType::Equal as u8,           ParseRule::default());
r.insert(TokenType::Identifier as u8,      ParseRule::new(Some(|c,b| Compiler::variable(c,b)),  None,                              Precedence::None));
r.insert(TokenType::FnIdentifier as u8,    ParseRule::new(Some(|c,b| Compiler::native_fn(c,b)), None,                              Precedence::None));

r.insert(TokenType::If as u8,              ParseRule::default());
r.insert(TokenType::Then as u8,            ParseRule::default());
r.insert(TokenType::ElseIf as u8,          ParseRule::default());
r.insert(TokenType::Else as u8,            ParseRule::default());
r.insert(TokenType::EndIf as u8,           ParseRule::default());

r.insert(TokenType::For as u8,             ParseRule::default());
r.insert(TokenType::To as u8,              ParseRule::default());
r.insert(TokenType::DownTo as u8,          ParseRule::default());
r.insert(TokenType::Do as u8,              ParseRule::default());
r.insert(TokenType::Next as u8,            ParseRule::default());

r.insert(TokenType::Output as u8,          ParseRule::default());
r.insert(TokenType::OutputLine as u8,      ParseRule::default());
r.insert(TokenType::Pass as u8,            ParseRule::new(Some(|c,b| Compiler::pass(c,b)),      None,                              Precedence::None));
r.insert(TokenType::Error as u8,           ParseRule::default());
r.insert(TokenType::Eof as u8,             ParseRule::default());
    }
}

pub struct Compiler<'a> {
    vm: &'a mut Vm,
    locals: Vec<Local>,
    scope_depth: isize,
    parser: Parser<'a>,
}

impl<'a> Compiler<'a> {
    const CONSTANTS_MAX: usize = u16::MAX as usize + 1;
    const BYTE_JUMP_MAX: u16 = u16::MAX;
    const ARG_COUNT_MAX: u8 = u8::MAX;

    pub fn new(vm: &'a mut Vm, source: &'static str) -> Self {
        Self {
            vm,
            locals: Vec::with_capacity(Self::CONSTANTS_MAX),
            scope_depth: 0,
            parser: Parser {
                lexer: Lexer::new(source),
                rules: HashMap::new(),
                previous_token: Token::default(),
                current_token: Token::default(),
                had_error: false,
                in_panic_mode: false,
            },
        }
    }

    fn init_locals(&mut self) {
        self.locals.clear();
        self.scope_depth = 0;
    }

    fn init_parser(&mut self) {
        self.parser.lexer.init();
        if self.parser.rules.is_empty() {
            Parser::init_rules(&mut self.parser.rules);
        }
        self.parser.previous_token = Token::default();
        self.parser.current_token = Token::default();
        self.parser.had_error = false;
        self.parser.in_panic_mode = false;
    }

    fn init(&mut self) {
        self.init_locals();
        self.init_parser();
    }

    fn end_compiler(&mut self) {
        self.emit_byte(Opcode::Return as u8);

        if (!self.parser.had_error)
            && cfg!(debug_assertions)
            && option_env!("PRINT_BYTECODE").is_some()
        {
            debug::disassemble_bytecode(&self.vm.bc, "Instruction Set", &self.vm.strings.borrow());
        }
    }

    //

    fn str_from_src(&self, start: usize, length: usize) -> &'static str {
        &self.parser.lexer.source[start..start + length]
    }

    fn str_from_token(&self, token: Token) -> &'static str {
        self.str_from_src(token.offset, token.length)
    }

    fn error_at(&mut self, token: Token, err_msg: &str) {
        if self.parser.in_panic_mode {
            return; // skip cascading errors until `synchronize()` is called
        }
        self.parser.in_panic_mode = true;

        *self.vm.result.borrow_mut() += &format!("[line {}] Error", token.line_num);

        match token.type_ {
            TokenType::Error => {
                *self.vm.result.borrow_mut() +=
                    &self.parser.lexer.err_fmt_string.replace("{}", err_msg);
            }
            TokenType::Eof => {
                *self.vm.result.borrow_mut() += " at end.";
            }
            _ => {
                *self.vm.result.borrow_mut() += &format!(" at `{}`.", self.str_from_token(token));
            }
        }

        if token.type_ != TokenType::Error {
            *self.vm.result.borrow_mut() += &format!(" {err_msg}");
        }
        *self.vm.result.borrow_mut() += "\n";

        self.parser.had_error = true;
    }

    fn error(&mut self, err_msg: &str) {
        self.error_at(self.parser.previous_token, err_msg);
    }

    fn error_at_current(&mut self, err_msg: &str) {
        self.error_at(self.parser.current_token, err_msg);
    }

    //

    fn advance(&mut self) {
        self.parser.previous_token = self.parser.current_token;

        loop {
            self.parser.current_token = self.parser.lexer.scan();

            // parse only valid tokens
            if self.parser.current_token.type_ != TokenType::Error {
                break;
            }

            let lexeme = self.str_from_token(self.parser.current_token);
            let err_msg = match lexeme {
                "\t" => "<tab>",
                "\r" => "<carriage return>",
                "\n" => "<line feed>",
                _ => lexeme,
            };
            self.error_at_current(err_msg);
        }
    }

    fn consume(&mut self, type_: TokenType, err_msg: &str) {
        if self.parser.current_token.type_ == type_ {
            self.advance();
            return;
        }
        self.error_at_current(err_msg);
    }

    //

    fn is_token_type(&self, type_: TokenType) -> bool {
        self.parser.current_token.type_ == type_
    }

    #[allow(clippy::wrong_self_convention)]
    fn is_at_token(&mut self, type_: TokenType) -> bool {
        if !self.is_token_type(type_) {
            return false;
        }
        self.advance();

        true
    }

    fn are_idents_eq(&self, a: Token, b: Token) -> bool {
        if a.length != b.length {
            return false;
        }

        self.str_from_token(a) == self.str_from_token(b)
    }

    fn is_const_short(&self, offset: usize) -> bool {
        offset > u8::MAX as usize
    }

    //

    fn emit_byte(&mut self, byte: u8) {
        self.vm
            .bc
            .push_byte(byte, self.parser.previous_token.line_num);
    }

    fn emit_bytes(&mut self, byte: u8, offset: usize) {
        self.emit_byte(byte);
        if self.is_const_short(offset) {
            self.emit_byte(((offset >> 8) & 0xff) as u8);
            self.emit_byte((offset & 0xff) as u8);
        } else {
            self.emit_byte(offset as u8);
        }
    }

    // todo: handle scopes when attribute support is added

    fn _begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn _end_scope(&mut self) {
        self.scope_depth -= 1;
        while !self.locals.is_empty() {
            match self.locals.last() {
                Some(local) => {
                    if local.depth > self.scope_depth {
                        self.emit_byte(Opcode::Pop as u8);
                        self.locals.pop();
                    } else {
                        break;
                    }
                }
                None => break,
            }
        }
    }

    //

    fn emit_jump(&mut self, byte: u8) -> usize {
        self.emit_byte(byte);
        self.emit_byte(0xff);
        self.emit_byte(0xff);

        self.vm.bc.byte_count() - 2
    }

    fn patch_jump(&mut self, offset: usize) {
        let jump = self.vm.bc.byte_count() - offset - 2;
        if jump > Self::BYTE_JUMP_MAX as usize {
            self.error("Jump size limit exceeded.");
        }

        self.vm.bc.assign(offset, ((jump >> 8) & 0xff) as u8);
        self.vm.bc.assign(offset + 1, (jump & 0xff) as u8);
    }

    //

    fn push_const(&mut self, val: Value) -> usize {
        if self.vm.bc.const_count() >= Self::CONSTANTS_MAX {
            self.error("Constant pool size limit reached.");
            return 0;
        }

        self.vm.bc.push_const(val)
    }

    fn emit_const(&mut self, val: Value) {
        let offset = self.push_const(val);
        let opcode = if self.is_const_short(offset) {
            Opcode::ConstantShort
        } else {
            Opcode::Constant
        };

        self.emit_bytes(opcode as u8, offset);
    }

    //

    fn push_ident_const(&mut self, name: Token) -> usize {
        let ident = self.str_from_token(name).to_ascii_lowercase();

        let val = Value::string(ident, self.vm);

        self.push_const(val)
    }

    fn resolve_local(&mut self, name: &Token) -> isize {
        let mut local_offset = -1;
        let mut had_error = false;

        for local in self.locals[..self.locals.len()].iter().enumerate().rev() {
            if self.are_idents_eq(*name, local.1.name) {
                if local.1.depth == -1 {
                    had_error = true;
                    local_offset = local.0 as isize;
                    break;
                }
                return local.0 as isize;
            }
        }
        if had_error {
            self.error("Can't read local variable in its own initializer.")
        }

        local_offset
    }

    fn declare_var(&mut self) {
        if self.scope_depth == 0 {
            return; // global scope
        }
        let name = self.parser.previous_token;
        let mut had_error = false;

        for local in self.locals[..self.locals.len()].iter().rev() {
            if local.depth != -1 && local.depth < self.scope_depth {
                break;
            }
            if self.are_idents_eq(name, local.name) {
                had_error = true;
            }
        }
        if had_error {
            self.error("A local variable with this name already exists in this scope.");
        }

        if self.locals.len() >= Self::CONSTANTS_MAX {
            self.error("Local variable limit reached.");
            return;
        }
        self.locals.push(Local { name, depth: -1 }); // uninitialized until `define_var()`
    }

    fn parse_var(&mut self, err_msg: &str) -> usize {
        if !self
            .str_from_token(self.parser.current_token)
            .starts_with('@')
        {
            self.error("Variable names must begin with `@`.");
        }
        self.consume(TokenType::Identifier, err_msg);

        self.declare_var();

        if self.scope_depth > 0 {
            return 0; // local scope; exit before globals are parsed
        }

        self.push_ident_const(self.parser.previous_token)
    }

    fn define_var(&mut self, ident_const: usize) {
        if self.scope_depth > 0 {
            if let Some(local) = self.locals.last_mut() {
                local.depth = self.scope_depth; // local is now initialized
            }
            return; // local scope; exit before globals are parsed
        }

        let opcode = if self.is_const_short(ident_const) {
            Opcode::DefineGlobalShort
        } else {
            Opcode::DefineGlobal
        };

        self.emit_bytes(opcode as u8, ident_const);
    }

    fn emit_var(&mut self, name: Token, can_assign: bool) {
        let is_var = self.str_from_token(name).starts_with('@');
        let get_op;
        let set_op;

        let mut ident = self.resolve_local(&name);
        if ident != -1 {
            get_op = Opcode::GetLocal;
            set_op = Opcode::SetLocal;
        } else {
            ident = self.push_ident_const(name) as isize;
            if self.is_const_short(ident as usize) {
                get_op = Opcode::GetGlobalShort;
                set_op = Opcode::SetGlobalShort;
            } else {
                get_op = Opcode::GetGlobal;
                set_op = Opcode::SetGlobal;
            }
        }

        if can_assign && self.is_at_token(TokenType::Equal) {
            if !is_var {
                self.error("Variable names must begin with `@`.");
            }
            self.expr();
            self.emit_bytes(set_op as u8, ident as usize);
        } else {
            self.emit_bytes(get_op as u8, ident as usize);
        }
    }

    //

    fn grouping(&mut self, _can_assign: bool) {
        self.expr();
        self.consume(TokenType::RightParen, "Expected `)` after expression.");
    }

    fn unary(&mut self, _can_assign: bool) {
        let unary_op = self.parser.previous_token.type_;

        self.parse_prec(Precedence::Unary);

        match unary_op {
            TokenType::Minus => self.emit_byte(Opcode::Negate as u8),
            TokenType::Not => self.emit_byte(Opcode::Not as u8),
            _ => (),
        }
    }

    fn number(&mut self, _can_assign: bool) {
        let Ok(num) = self
            .str_from_token(self.parser.previous_token)
            .parse::<f64>()
        else {
            self.error("Unable to parse number.");
            return;
        };

        self.emit_const(Value::num(num));
    }

    fn string(&mut self, _can_assign: bool) {
        let quote_mark =
            self.parser.lexer.source.as_bytes()[self.parser.previous_token.offset] as char;
        let lexeme = self.str_from_src(
            self.parser.previous_token.offset + 1,
            self.parser.previous_token.length - 2,
        );
        let do_escape = match quote_mark {
            '"' => lexeme.contains("\"\""),
            '\'' => lexeme.contains("''"),
            _ => false,
        };

        let string = if do_escape {
            let mut char_buff = [0u8; 4];
            let qm = quote_mark.encode_utf8(&mut char_buff).to_owned();
            let mut prev_clstr = qm.as_str();
            let mut is_escape = false;
            let mut new_string = String::new();
            let graphemes = lexeme.graphemes(true);

            for clstr in graphemes {
                if clstr == qm.as_str() && prev_clstr == qm.as_str() && !is_escape {
                    // preceding quote mark (the escape) already pushed; skip this one
                    is_escape = true;
                } else {
                    is_escape = false;
                    new_string.push_str(clstr);
                }
                prev_clstr = clstr;
            }

            new_string
        } else {
            lexeme.to_owned()
        };

        let val = Value::string(string, self.vm);

        self.emit_const(val);
    }

    fn literal(&mut self, _can_assign: bool) {
        match self.parser.previous_token.type_ {
            TokenType::False => self.emit_byte(Opcode::False as u8),
            TokenType::Null => self.emit_byte(Opcode::Null as u8),
            TokenType::True => self.emit_byte(Opcode::True as u8),
            _ => (),
        }
    }

    fn variable(&mut self, can_assign: bool) {
        self.emit_var(self.parser.previous_token, can_assign);
    }

    fn native_fn(&mut self, _can_assign: bool) {
        let ident = self.push_ident_const(self.parser.previous_token);
        let opcode = if self.is_const_short(ident) {
            Opcode::NativeFnShort
        } else {
            Opcode::NativeFn
        };

        self.emit_bytes(opcode as u8, ident);
    }

    fn pass(&mut self, _can_assign: bool) {
        let start = self.parser.previous_token.offset as f64;
        let end = (self.parser.previous_token.offset + self.parser.previous_token.length) as f64;

        self.emit_const(Value::num(start));
        self.emit_const(Value::num(end));
        self.emit_byte(Opcode::Pass as u8);
    }

    fn call(&mut self, _can_assign: bool) {
        let mut arg_count = 0;

        if !self.is_token_type(TokenType::RightParen) {
            loop {
                self.expr();
                if arg_count == Self::ARG_COUNT_MAX {
                    let arg_count_max = Self::ARG_COUNT_MAX;
                    self.error(&format!(
                        "Function argument limit of {arg_count_max} reached."
                    ));
                } else {
                    arg_count += 1;
                }
                if !self.is_at_token(TokenType::Comma) {
                    break;
                }
            }
        }

        self.consume(TokenType::RightParen, "Expected `)` after arguments.");
        self.emit_bytes(Opcode::Call as u8, arg_count as usize);
    }

    fn or(&mut self, _can_assign: bool) {
        let else_jump = self.emit_jump(Opcode::JumpIfFalse as u8);
        let end_jump = self.emit_jump(Opcode::Jump as u8);

        self.patch_jump(else_jump);
        self.emit_byte(Opcode::Pop as u8);
        self.parse_prec(Precedence::Or);
        self.patch_jump(end_jump);
    }

    fn and(&mut self, _can_assign: bool) {
        let end_jump = self.emit_jump(Opcode::JumpIfFalse as u8);

        self.emit_byte(Opcode::Pop as u8);
        self.parse_prec(Precedence::And);
        self.patch_jump(end_jump);
    }

    fn binary(&mut self, _can_assign: bool) {
        let binary_op = self.parser.previous_token.type_;
        let rule = self.get_rule(binary_op);

        // binary operators are left-associative, so a right-hand operand's precedence
        // will be one level higher than its own
        let prec = (rule.prec as u8 + 1).into();

        self.parse_prec(prec);

        match binary_op {
            TokenType::EqualEqual => self.emit_byte(Opcode::Equal as u8),
            TokenType::NotEqual => self.emit_byte(Opcode::NotEqual as u8),
            TokenType::Less => self.emit_byte(Opcode::Less as u8),
            TokenType::LessEqual => self.emit_byte(Opcode::LessEqual as u8),
            TokenType::Greater => self.emit_byte(Opcode::Greater as u8),
            TokenType::GreaterEqual => self.emit_byte(Opcode::GreaterEqual as u8),
            _ => (),
        }
    }

    //

    fn get_rule(&self, type_: TokenType) -> &ParseRule {
        let Some(rule) = self.parser.rules.get(&(type_ as u8)) else {
            panic!("Missing `ParseRule` for token type: {type_:?}.");
        };
        rule
    }

    fn parse_prec(&mut self, prec: Precedence) {
        self.advance();

        let Some(prefix_fn) = self.get_rule(self.parser.previous_token.type_).prefix_fn else {
            self.error("Expected expression.");
            return;
        };
        let can_assign = prec as u8 <= Precedence::Assignment as u8;

        prefix_fn(self, can_assign);

        while prec as u8 <= self.get_rule(self.parser.current_token.type_).prec as u8 {
            self.advance();
            if let Some(infix_fn) = self.get_rule(self.parser.previous_token.type_).infix_fn {
                infix_fn(self, can_assign);
            }
        }
        if can_assign && self.is_at_token(TokenType::Equal) {
            self.error("Invalid assignment target.");
        }
    }

    //

    fn expr(&mut self) {
        self.parse_prec(Precedence::Assignment); // start at lowest level
    }

    fn inline_expr(&mut self) {
        if self.is_at_token(TokenType::AttrDelim) {
            if !self.is_token_type(TokenType::Identifier) {
                self.error_at_current("Expected attribute.");
            }
            if self.is_token_type(TokenType::Identifier)
                && self
                    .str_from_token(self.parser.current_token)
                    .starts_with('@')
            {
                self.error_at_current("Expected attribute.");
            }

            self.expr();
            self.emit_byte(Opcode::Write as u8);

            if !self.is_token_type(TokenType::AttrDelim) {
                self.error_at_current("Expected `%%` delimiter.");
            } else {
                self.parser.lexer.mode = Mode::Pass;
                self.advance();
            }
        } else if self.is_at_token(TokenType::FnDelimOpen) {
            if !self.is_token_type(TokenType::FnIdentifier) {
                self.error_at_current("Unknown function after `%%=` delimiter.");
            }

            self.expr();
            self.emit_byte(Opcode::Write as u8);

            self.consume(
                TokenType::FnDelimClose,
                "Expected `=%%` delimiter after function.",
            );
        }
    }

    //

    fn expr_stmt(&mut self) {
        let do_pass = self.is_token_type(TokenType::Pass);

        self.expr();
        if !do_pass {
            self.emit_byte(Opcode::Pop as u8);
        }
    }

    fn var_decl_stmt(&mut self) {
        let ident = self.parse_var("Expected variable name.");

        self.emit_byte(Opcode::Null as u8);
        self.define_var(ident);
        if self.is_at_token(TokenType::Equal) {
            self.error("A `var` declaration is not for assignment. Use `set` instead.");
        }
        if self.is_at_token(TokenType::Comma) {
            self.var_decl_stmt();
        }
    }

    fn set_decl_stmt(&mut self) {
        let ident = self.parse_var("Expected variable name.");

        if self.is_at_token(TokenType::Equal) {
            self.expr();
        } else {
            self.error("Expected variable value.");
        }

        self.define_var(ident);
    }

    fn parse_if_body(&mut self) {
        while !self.is_token_type(TokenType::EndIf) && !self.is_token_type(TokenType::Eof) {
            if self.is_token_type(TokenType::ElseIf) || self.is_token_type(TokenType::Else) {
                break;
            }
            self.decl_stmt();
        }
    }

    fn if_stmt(&mut self) {
        self.expr(); // `Boolean` value pushed onto stack
        self.consume(TokenType::Then, "Expected `Then` after condition.");

        // determine `VM.ip` offset adjustment, i.e., how many bytes to jump over to next branch
        let then_jump = self.emit_jump(Opcode::JumpIfFalse as u8); // opcode + placeholder operand
        self.emit_byte(Opcode::Pop as u8);

        self.parse_if_body();

        let mut else_jump = 0;
        let mut elseif_jumps = Vec::new();

        if self.is_token_type(TokenType::ElseIf) {
            else_jump = self.emit_jump(Opcode::Jump as u8); // jump-off point (to end) if `true`

            self.patch_jump(then_jump); // landing spot if `false`
            self.emit_byte(Opcode::Pop as u8);

            while self.is_at_token(TokenType::ElseIf) {
                self.expr();
                self.consume(TokenType::Then, "Expected `Then` after condition.");

                let then_jump = self.emit_jump(Opcode::JumpIfFalse as u8);

                self.emit_byte(Opcode::Pop as u8);

                self.parse_if_body();

                elseif_jumps.push(self.emit_jump(Opcode::Jump as u8));
                self.patch_jump(then_jump);
                self.emit_byte(Opcode::Pop as u8);
            }
        }

        if self.is_at_token(TokenType::Else) {
            if else_jump == 0 {
                else_jump = self.emit_jump(Opcode::Jump as u8);
                self.patch_jump(then_jump);
                self.emit_byte(Opcode::Pop as u8);
            }

            while !self.is_token_type(TokenType::EndIf) && !self.is_token_type(TokenType::Eof) {
                self.decl_stmt();
            }
        }

        if else_jump == 0 {
            else_jump = self.emit_jump(Opcode::Jump as u8);
            self.patch_jump(then_jump);
            self.emit_byte(Opcode::Pop as u8);
        }

        self.consume(TokenType::EndIf, "Expected `EndIf` after branch.");
        if !elseif_jumps.is_empty() {
            for i in &elseif_jumps {
                self.patch_jump(*i);
            }
        }
        self.patch_jump(else_jump);
    }

    fn for_stmt(&mut self) {
        let ident = self.parse_var("Expected starting index variable name.");
        let init_var = self.parser.previous_token;
        if self.is_at_token(TokenType::Equal) {
            self.expr();
            self.define_var(ident);
            self.emit_var(init_var, true);
        } else {
            self.error("Expected starting index assignment.");
        }

        let increment;
        let comp_op;
        if self.is_token_type(TokenType::DownTo) {
            increment = -1f64;
            comp_op = Opcode::GreaterEqual as u8;
            self.consume(
                TokenType::DownTo,
                "Expected `DownTo` after starting index assignment.",
            );
        } else {
            increment = 1f64;
            comp_op = Opcode::LessEqual as u8;
            self.consume(
                TokenType::To,
                "Expected either `DownTo` or `To` after starting index assignment.",
            );
        }

        // ending index expression
        let loop_ = self.vm.bc.byte_count();
        self.expr();
        self.emit_byte(comp_op);

        self.consume(
            TokenType::Do,
            "Expected `Do` after ending index expression.",
        );

        let end_jump = self.emit_jump(Opcode::JumpIfFalse as u8);
        self.emit_byte(Opcode::Pop as u8);

        while !self.is_token_type(TokenType::Next) && !self.is_token_type(TokenType::Eof) {
            self.decl_stmt();
        }

        let mut index = ident;
        let mut get_op = Opcode::GetGlobal;
        let mut set_op = Opcode::SetGlobal;
        if self.is_const_short(ident) {
            get_op = Opcode::GetGlobalShort;
            set_op = Opcode::SetGlobalShort;
        }

        let local = self.resolve_local(&init_var);
        if local != -1 {
            index = local as usize;
            get_op = Opcode::GetLocal;
            set_op = Opcode::SetLocal;
        }

        // increment
        self.emit_bytes(get_op as u8, index);
        self.emit_const(Value::num(increment));
        self.emit_byte(Opcode::Add as u8);
        self.emit_bytes(set_op as u8, index);

        // emit loop
        self.emit_byte(Opcode::Loop as u8);

        let offset = self.vm.bc.byte_count() - loop_ + 2;
        if offset > Self::BYTE_JUMP_MAX as usize {
            self.error("Loop size limit exceeded.");
        }
        self.emit_byte(((offset >> 8) & 0xff) as u8);
        self.emit_byte((offset & 0xff) as u8);

        self.patch_jump(end_jump);
        self.emit_byte(Opcode::Pop as u8);
        self.consume(TokenType::Next, "Expected `Next` after block.");

        if self.parser.previous_token.type_ == TokenType::Next
            && self.parser.current_token.type_ == TokenType::Identifier
        {
            self.consume(
                TokenType::Identifier,
                "Error at optional variable trailing `Next`",
            );
        }
    }

    fn output(&mut self) {
        let with_lf = self.parser.previous_token.type_ == TokenType::OutputLine;

        self.consume(TokenType::LeftParen, "Expected `(`.");
        let do_write = self.parser.current_token.type_ == TokenType::FnIdentifier;

        self.expr();
        if do_write {
            self.emit_byte(Opcode::Write as u8);
        } else {
            self.emit_byte(Opcode::Pop as u8);
        }

        self.consume(TokenType::RightParen, "Expected `)` after arguments.");
        if with_lf {
            self.emit_byte(Opcode::LineFeed as u8);
        }
    }

    //

    fn synchronize(&mut self) {
        self.parser.in_panic_mode = false;

        while self.parser.current_token.type_ != TokenType::Eof {
            if self.parser.previous_token.type_ == TokenType::RightParen {
                return;
            }
            match self.parser.current_token.type_ {
                TokenType::Var => return,
                TokenType::Set => return,
                TokenType::If => return,
                TokenType::For => return,
                TokenType::Output => return,
                TokenType::OutputLine => return,
                _ => (),
            }
            self.advance();
        }
    }

    //

    fn block_ingress(&mut self) {
        while !self.is_token_type(TokenType::BlockDelimClose) && !self.is_token_type(TokenType::Eof)
        {
            self.decl_stmt();
        }
        self.consume(TokenType::BlockDelimClose, "Expected `]%%` delimiter.");
    }

    fn block_egress(&mut self) {
        while !self.is_token_type(TokenType::BlockDelimOpen) && !self.is_token_type(TokenType::Eof)
        {
            self.expr_stmt();
            self.inline_expr();
        }

        // Mode::PassThru

        if !self.is_token_type(TokenType::Eof) {
            self.consume(TokenType::BlockDelimOpen, "Expected `%%[` delimiter.");
        }
    }

    fn stmt(&mut self) {
        if self.is_at_token(TokenType::Output) || self.is_at_token(TokenType::OutputLine) {
            self.output();
        } else if self.is_at_token(TokenType::If) {
            self.if_stmt();
        } else if self.is_at_token(TokenType::For) {
            self.for_stmt();
        } else if self.is_at_token(TokenType::BlockDelimOpen) {
            self.block_ingress();
        } else if self.is_at_token(TokenType::BlockDelimClose) {
            self.block_egress();
        } else {
            self.expr_stmt();
        }
    }

    fn decl_stmt(&mut self) {
        if self.is_token_type(TokenType::AttrDelim) || self.is_token_type(TokenType::FnDelimOpen) {
            self.inline_expr();
        } else if self.is_at_token(TokenType::Var) {
            self.var_decl_stmt();
        } else if self.is_at_token(TokenType::Set) {
            self.set_decl_stmt();
        } else {
            self.stmt();
        }

        if self.parser.in_panic_mode {
            self.synchronize();
        }
    }

    //

    pub fn compile(&mut self) -> Result<(), Status> {
        self.init();

        //self.begin_scope();
        self.advance();
        while !self.is_at_token(TokenType::Eof) {
            self.decl_stmt();
        }
        //self.end_scope();
        self.end_compiler();

        if self.parser.had_error {
            return Err(Status::CompileError);
        }

        Ok(())
    }
}
