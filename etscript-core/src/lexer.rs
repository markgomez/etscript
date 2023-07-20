use std::{
    cmp::Ordering,
    iter::{Enumerate, Peekable},
    slice::Iter,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenType {
    // delimiters
    AttrDelim,       // `%%`
    FnDelimOpen,     // `%%=`
    FnDelimClose,    // `=%%`
    BlockDelimOpen,  // `%%[`
    BlockDelimClose, // `]%%`

    LeftParen,
    RightParen,
    Comma,
    Minus,

    // constants
    Number,
    String,
    Null,

    // boolean
    True,
    False,

    // logical
    Not,
    Or,
    And,

    // comparison
    EqualEqual,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,

    // identifiers
    Var,
    Set,
    Equal,
    Identifier,
    FnIdentifier,

    // control flow
    If,
    Then,
    ElseIf,
    Else,
    EndIf,

    For,
    To,
    DownTo,
    Do,
    Next,

    Output,
    OutputLine,
    Pass,
    Error,
    Eof,
}

#[derive(Clone, Copy)]
pub struct Token {
    pub type_: TokenType,
    pub offset: usize,
    pub length: usize,
    pub line_num: u16,
}

impl Token {
    pub fn new(type_: TokenType) -> Self {
        Self {
            type_,
            offset: 0,
            length: 0,
            line_num: 1,
        }
    }
}

impl Default for Token {
    fn default() -> Self {
        Self::new(TokenType::Null)
    }
}

#[derive(PartialEq, Eq)]
pub enum Mode {
    Pass,
    Attr,
    Fn,
    Block,
}

pub struct Lexer<'a> {
    pub mode: Mode,
    pub source: &'static str,
    src_iter: Peekable<Enumerate<Iter<'a, u8>>>,
    starting_offset: usize,
    current_offset: usize,
    line_num: u16,
    pub err_fmt_string: String,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'static str) -> Self {
        Self {
            mode: Mode::Pass,
            source,
            src_iter: source.as_bytes().iter().enumerate().peekable(),
            starting_offset: 0,
            current_offset: 0,
            line_num: 1,
            err_fmt_string: String::new(),
        }
    }

    pub fn init(&mut self) {
        self.mode = Mode::Pass;
        self.src_iter = self.source.as_bytes().iter().enumerate().peekable();
        self.starting_offset = 0;
        self.current_offset = 0;
        self.line_num = 1;
        self.err_fmt_string.clear();
    }

    //

    fn is_alpha(&self, c: char) -> bool {
        c.is_ascii_alphabetic() || c == '_'
    }

    fn is_digit(&self, c: char) -> bool {
        c.is_ascii_digit()
    }

    fn is_attr(&self, c: char) -> bool {
        self.is_alpha(c) || self.is_digit(c) || c == ' ' || c == '-'
    }

    #[allow(clippy::wrong_self_convention)]
    fn is_next(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        let Some(value) = self.src_iter.next_if(|&item| item == (self.current_offset, &(expected as u8))) else {
            return false;
        };
        self.current_offset = value.0 + 1;

        true
    }

    #[allow(clippy::wrong_self_convention)]
    fn is_at_end(&mut self) -> bool {
        self.src_iter.peek().is_none()
    }

    //

    fn advance(&mut self) -> Option<char> {
        let Some(value) = self.src_iter.next() else {
            return None;
        };
        self.current_offset = value.0 + 1;

        Some(*value.1 as char)
    }

    fn advance_by(&mut self, n: usize) -> Option<char> {
        let mut char_ = None;
        let mut count = 0;

        while count < n {
            let Some(value) = self.src_iter.next() else {
                return None;
            };
            self.current_offset = value.0 + 1;
            char_ = Some(*value.1 as char);
            count += 1;
        }

        char_
    }

    fn peek(&mut self) -> Option<char> {
        let Some(value) = self.src_iter.peek() else {
            return None;
        };

        Some(*value.1 as char)
    }

    fn peek_plus(&mut self, n: usize) -> Option<char> {
        if n == 0 {
            return self.peek();
        }

        let outer_bound = self.source.as_bytes().len() - self.current_offset;
        if self.is_at_end() || n >= outer_bound {
            return None;
        }

        Some(self.source.as_bytes()[self.current_offset + n] as char)
    }

    fn char_at(&self, offset: usize) -> Option<char> {
        let outer_bound = self.source.as_bytes().len() - self.current_offset;
        if offset >= outer_bound {
            return None;
        }

        Some(self.source.as_bytes()[self.starting_offset + offset] as char)
    }

    //

    fn create_token(&self, type_: TokenType) -> Token {
        let mut token = Token::new(type_);

        token.offset = self.starting_offset;
        token.length = self.current_offset - self.starting_offset;
        token.line_num = self.line_num;

        token
    }

    fn create_err_token(&mut self, fmt_str: &str) -> Token {
        self.err_fmt_string = fmt_str.to_owned();

        self.create_token(TokenType::Error)
    }

    fn match_keyword(
        &self,
        offset: usize,
        length: usize,
        remainder: &str,
        type_: TokenType,
    ) -> TokenType {
        if self.current_offset - self.starting_offset != offset + length {
            return TokenType::Identifier;
        }

        let lexeme = &mut self.source.as_bytes()
            [self.starting_offset + offset..self.current_offset]
            .to_ascii_lowercase();

        if remainder.as_bytes().iter().cmp(lexeme.iter()) == Ordering::Equal {
            return type_;
        }

        TokenType::Identifier
    }

    fn ident_or_keyword(&self) -> TokenType {
        let Some(first_char) = self.char_at(0) else {
            return TokenType::Identifier;
        };
        match first_char {
            'A' | 'a' => self.match_keyword(1, 2, "nd", TokenType::And),
            'D' | 'd' => match self.current_offset - self.starting_offset > 1 {
                true => {
                    let Some(next_char) = self.char_at(1) else {
                        return TokenType::Identifier;
                    };
                    match next_char {
                        'O' | 'o' => {
                            if self.match_keyword(0, 2, "do", TokenType::Do) == TokenType::Do {
                                return TokenType::Do;
                            }
                            self.match_keyword(2, 4, "wnto", TokenType::DownTo)
                        }
                        _ => TokenType::Identifier,
                    }
                }
                _ => TokenType::Identifier,
            },
            'E' | 'e' => match self.current_offset - self.starting_offset > 1 {
                true => {
                    let Some(next_char) = self.char_at(1) else {
                        return TokenType::Identifier;
                    };
                    match next_char {
                        'L' | 'l' => {
                            if self.match_keyword(2, 2, "se", TokenType::Else) == TokenType::Else {
                                return TokenType::Else;
                            }
                            self.match_keyword(2, 4, "seif", TokenType::ElseIf)
                        }
                        'N' | 'n' => self.match_keyword(2, 3, "dif", TokenType::EndIf),
                        _ => TokenType::Identifier,
                    }
                }
                _ => TokenType::Identifier,
            },
            'F' | 'f' => match self.current_offset - self.starting_offset > 1 {
                true => {
                    let Some(next_char) = self.char_at(1) else {
                        return TokenType::Identifier;
                    };
                    match next_char {
                        'A' | 'a' => self.match_keyword(2, 3, "lse", TokenType::False),
                        'O' | 'o' => self.match_keyword(2, 1, "r", TokenType::For),
                        _ => TokenType::Identifier,
                    }
                }
                _ => TokenType::Identifier,
            },
            'I' | 'i' => self.match_keyword(1, 1, "f", TokenType::If),
            'N' | 'n' => match self.current_offset - self.starting_offset > 1 {
                true => {
                    let Some(next_char) = self.char_at(1) else {
                        return TokenType::Identifier;
                    };
                    match next_char {
                        'E' | 'e' => self.match_keyword(2, 2, "xt", TokenType::Next),
                        'O' | 'o' => self.match_keyword(2, 1, "t", TokenType::Not),
                        'U' | 'u' => self.match_keyword(2, 2, "ll", TokenType::Null),
                        _ => TokenType::Identifier,
                    }
                }
                _ => TokenType::Identifier,
            },
            'O' | 'o' => self.match_keyword(1, 1, "r", TokenType::Or),
            'S' | 's' => self.match_keyword(1, 2, "et", TokenType::Set),
            'T' | 't' => match self.current_offset - self.starting_offset > 1 {
                true => {
                    let Some(next_char) = self.char_at(1) else {
                        return TokenType::Identifier;
                    };
                    match next_char {
                        'H' | 'h' => self.match_keyword(2, 2, "en", TokenType::Then),
                        'O' | 'o' => self.match_keyword(0, 2, "to", TokenType::To),
                        'R' | 'r' => self.match_keyword(2, 2, "ue", TokenType::True),
                        _ => TokenType::Identifier,
                    }
                }
                _ => TokenType::Identifier,
            },
            'V' | 'v' => self.match_keyword(1, 2, "ar", TokenType::Var),
            _ => TokenType::Identifier,
        }
    }

    fn create_callable_token(&mut self, type_: TokenType) -> Option<Token> {
        let mut token = None;
        let mut count = 0;

        while let Some(next_char) = self.peek_plus(count) {
            match next_char {
                c if c.is_ascii_whitespace() => count += 1,
                '(' => {
                    token = Some(self.create_token(type_));
                    break;
                }
                _ => break,
            }
        }

        token
    }

    fn create_ident_token(&mut self) -> Token {
        if let Some(first_char) = self.char_at(0) {
            if first_char == '%' || first_char == '[' {
                let mut name_len = 0;

                while let Some(next_char) = self.peek() {
                    if next_char != '%' && next_char != ']' && !self.is_at_end() {
                        if self.is_attr(next_char) {
                            name_len += 1;
                        } else {
                            return self.create_err_token(" — unexpected character: `{}`.");
                        }
                        if first_char == '%' {
                            self.advance();
                        }
                        self.advance();
                    } else {
                        break;
                    }
                }
                if name_len < 1 {
                    return self.create_err_token(" — attribute name cannot be empty.");
                }
                if self.is_at_end() {
                    if first_char == '%' {
                        return self.create_err_token(" — missing `%%` terminator.");
                    }
                    return self.create_err_token(" — missing `]` terminator.");
                }
                if first_char == '%' {
                    self.advance();
                }
                self.advance();

                let mut token = Token::new(TokenType::Identifier);

                if first_char == '%' {
                    token.offset = self.starting_offset + 2;
                } else {
                    token.offset = self.starting_offset + 1;
                }
                token.length = name_len;
                token.line_num = self.line_num;

                return token;
            }

            while let Some(next_char) = self.peek() {
                if self.is_alpha(next_char) || self.is_digit(next_char) {
                    self.advance();
                } else {
                    break;
                }
            }

            if first_char == '@' {
                if self.current_offset - self.starting_offset == 1 {
                    return self.create_err_token(
                        " — variable names must include at least one other letter, number, or underscore.",
                    );
                }
                return self.create_token(TokenType::Identifier);
            }

            let lexeme = &self.source.as_bytes()[self.starting_offset..self.current_offset];
            let Ok(str) = std::str::from_utf8(lexeme) else {
                return self.create_err_token(" — identifier is not a valid UTF-8 string.");
            };
            if "output".eq_ignore_ascii_case(str) {
                if let Some(token) = self.create_callable_token(TokenType::Output) {
                    return token;
                }
            }
            if "outputline".eq_ignore_ascii_case(str) {
                if let Some(token) = self.create_callable_token(TokenType::OutputLine) {
                    return token;
                }
            }
            if let Some(token) = self.create_callable_token(TokenType::FnIdentifier) {
                return token;
            }
        }

        self.create_token(self.ident_or_keyword())
    }

    fn create_number_token(&mut self) -> Token {
        while let Some(next_char) = self.peek() {
            if self.is_digit(next_char) {
                self.advance();
            } else {
                break;
            }
        }

        if self.peek() == Some('.') {
            if let Some(char_after_next) = self.peek_plus(1) {
                if self.is_digit(char_after_next) {
                    self.advance();

                    while let Some(next_char) = self.peek() {
                        if self.is_digit(next_char) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
            }
        }

        self.create_token(TokenType::Number)
    }

    fn create_string_token(&mut self, quote_mark: char) -> Token {
        let mut is_closed = false;

        while !self.is_at_end() {
            let Some(next_char) = self.peek() else {
              break;
            };
            if next_char == '\n' {
                self.line_num += 1;
            }
            if next_char == quote_mark && self.peek_plus(1) == Some(quote_mark) {
                self.advance_by(2); // skip over escape sequence (`""` or `''`)
            } else if next_char == quote_mark {
                is_closed = true;
                self.advance();
                break;
            } else {
                self.advance();
            }
        }
        if !is_closed {
            return self.create_err_token(" — unterminated string.");
        }

        self.create_token(TokenType::String)
    }

    //

    fn skip_whitespace(&mut self) -> Option<Token> {
        loop {
            let Some(next_char) = self.peek() else {
                return None;
            };
            match next_char {
                c if c.is_ascii_whitespace() => {
                    if c == '\n' {
                        self.line_num += 1;
                    }
                    self.advance();
                }
                '/' => {
                    let Some(next_char) = self.peek_plus(1) else {
                        return None;
                    };
                    match next_char {
                        '/' => {
                            while self.peek() != Some('\n') && !self.is_at_end() {
                                self.advance();
                            }
                        }
                        '*' => {
                            let mut is_closed = false;
                            let mut length = -2;

                            while !self.is_at_end() {
                                if self.peek() == Some('\n') {
                                    self.line_num += 1;
                                }
                                if self.peek() == Some('*') && self.peek_plus(1) == Some('/') {
                                    is_closed = true;
                                    self.advance_by(2);
                                    break;
                                }
                                self.advance();
                                length += 1;
                            }
                            if !is_closed || length < 0 {
                                return Some(self.create_err_token(" — unterminated comment."));
                            }
                        }
                        _ => break,
                    }
                }
                _ => break,
            }
        }

        None
    }

    //

    pub fn scan(&mut self) -> Token {
        if self.mode == Mode::Fn || self.mode == Mode::Block {
            if let Some(err_token) = self.skip_whitespace() {
                return err_token; // unterminated comment
            }
        }

        self.starting_offset = self.current_offset;

        if self.is_at_end() {
            return self.create_token(TokenType::Eof);
        }

        // intercept delimiters
        if self.peek() == Some('%') {
            match self.mode {
                Mode::Attr => {
                    if self.peek_plus(1) == Some('%') {
                        self.advance_by(2);
                        return self.create_token(TokenType::AttrDelim);
                    }
                }
                Mode::Fn => {
                    if self.peek_plus(1) == Some('%') && self.peek_plus(2) == Some('=') {
                        self.advance_by(3);
                        return self.create_token(TokenType::FnDelimOpen);
                    }
                }
                Mode::Block => {
                    if self.peek_plus(1) == Some('%') && self.peek_plus(2) == Some('[') {
                        self.advance_by(3);
                        return self.create_token(TokenType::BlockDelimOpen);
                    }
                }
                _ => (), // `Mode::Pass` needs to `advance()` until the next delimiter is
                         // found (or EOF is reached). Confining that task here will cause
                         // the lexer to be "stuck" if the next character is not `%`.
                         // The succeeding `match` statement will handle this instead.
            }
        }

        match self.mode {
            Mode::Pass => {
                while !self.is_at_end() {
                    if self.peek() == Some('%') && self.peek_plus(1) == Some('%') {
                        match self.peek_plus(2) {
                            Some('=') => {
                                self.mode = Mode::Fn;
                            }
                            Some('[') => {
                                self.mode = Mode::Block;
                            }
                            _ => {
                                self.mode = Mode::Attr;
                            }
                        }
                        return self.create_token(TokenType::Pass);
                    }
                    self.advance();
                    if self.peek() == Some('\n') {
                        self.line_num += 1;
                    }
                }
                self.create_token(TokenType::Pass)
            }
            _ => {
                let Some(char_) = self.advance() else {
                    return self.create_token(TokenType::Eof);
                };
                if self.is_alpha(char_) {
                    return self.create_ident_token();
                }

                if self.is_digit(char_) {
                    return self.create_number_token();
                }

                let err_token = self.create_err_token(" — unexpected character: `{}`.");

                match char_ {
                    '[' => self.create_ident_token(),
                    ']' => {
                        if self.peek() == Some('%') && self.peek_plus(1) == Some('%') {
                            self.advance_by(2);
                            self.mode = Mode::Pass;
                            self.create_token(TokenType::BlockDelimClose)
                        } else {
                            err_token
                        }
                    }
                    '@' => self.create_ident_token(),
                    '(' => self.create_token(TokenType::LeftParen),
                    ')' => self.create_token(TokenType::RightParen),
                    ',' => self.create_token(TokenType::Comma),
                    '-' => self.create_token(TokenType::Minus),
                    '=' => {
                        if self.peek() == Some('%') && self.peek_plus(1) == Some('%') {
                            self.advance_by(2);
                            self.mode = Mode::Pass;
                            self.create_token(TokenType::FnDelimClose)
                        } else {
                            match self.is_next('=') {
                                true => self.create_token(TokenType::EqualEqual),
                                _ => self.create_token(TokenType::Equal),
                            }
                        }
                    }
                    '!' => match self.is_next('=') {
                        true => self.create_token(TokenType::NotEqual),
                        _ => err_token,
                    },
                    '<' => match self.is_next('=') {
                        true => self.create_token(TokenType::LessEqual),
                        _ => self.create_token(TokenType::Less),
                    },
                    '>' => match self.is_next('=') {
                        true => self.create_token(TokenType::GreaterEqual),
                        _ => self.create_token(TokenType::Greater),
                    },
                    '"' | '\'' => self.create_string_token(char_),
                    _ => err_token,
                }
            }
        }
    }
}
