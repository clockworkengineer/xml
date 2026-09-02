use crate::error::{Result, XmlError};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Slash,
    DoubleSlash,
    At,
    Dot,
    DoubleDot,
    Star,
    ColonColon,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    Comma,
    Pipe,
    Plus,
    Minus,
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    Name(String),
    LiteralString(String),
    LiteralNumber(f64),
    Eof,
}

#[derive(Debug, Clone)]
pub struct XPathLexer<'a> {
    chars: Vec<char>,
    pos: usize,
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> XPathLexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.chars.get(self.pos).copied();
        if ch.is_some() {
            self.pos += 1;
        }
        ch
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_ascii_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    pub fn next_token(&mut self) -> Result<Token> {
        self.skip_whitespace();
        let ch = match self.peek() {
            Some(c) => c,
            None => return Ok(Token::Eof),
        };

        match ch {
            '/' => {
                self.advance();
                if self.peek() == Some('/') {
                    self.advance();
                    Ok(Token::DoubleSlash)
                } else {
                    Ok(Token::Slash)
                }
            }
            '.' => {
                self.advance();
                if self.peek() == Some('.') {
                    self.advance();
                    Ok(Token::DoubleDot)
                } else if self.peek().map_or(false, |c| c.is_ascii_digit()) {
                    self.pos -= 1;
                    self.read_number()
                } else {
                    Ok(Token::Dot)
                }
            }
            '@' => {
                self.advance();
                Ok(Token::At)
            }
            '*' => {
                self.advance();
                Ok(Token::Star)
            }
            '(' => {
                self.advance();
                Ok(Token::LeftParen)
            }
            ')' => {
                self.advance();
                Ok(Token::RightParen)
            }
            '[' => {
                self.advance();
                Ok(Token::LeftBracket)
            }
            ']' => {
                self.advance();
                Ok(Token::RightBracket)
            }
            ',' => {
                self.advance();
                Ok(Token::Comma)
            }
            '|' => {
                self.advance();
                Ok(Token::Pipe)
            }
            '+' => {
                self.advance();
                Ok(Token::Plus)
            }
            '-' => {
                self.advance();
                Ok(Token::Minus)
            }
            '=' => {
                self.advance();
                Ok(Token::Eq)
            }
            '!' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Ok(Token::NotEq)
                } else {
                    Err(XmlError::XPathError("Expected '=' after '!'".into()))
                }
            }
            '<' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Ok(Token::LtEq)
                } else {
                    Ok(Token::Lt)
                }
            }
            '>' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Ok(Token::GtEq)
                } else {
                    Ok(Token::Gt)
                }
            }
            ':' => {
                self.advance();
                if self.peek() == Some(':') {
                    self.advance();
                    Ok(Token::ColonColon)
                } else {
                    Ok(Token::Name(":".into()))
                }
            }
            '"' | '\'' => self.read_string(ch),
            _ if ch.is_ascii_digit() => self.read_number(),
            _ if ch.is_alphabetic() || ch == '_' => self.read_name(),
            _ => Err(XmlError::XPathError(format!(
                "Unexpected character in XPath expression: '{ch}'"
            ))),
        }
    }

    fn read_string(&mut self, quote: char) -> Result<Token> {
        self.advance(); // consume quote
        let mut s = String::new();
        while let Some(ch) = self.peek() {
            if ch == quote {
                self.advance();
                return Ok(Token::LiteralString(s));
            }
            s.push(self.advance().unwrap());
        }
        Err(XmlError::XPathError("Unterminated string literal in XPath".into()))
    }

    fn read_number(&mut self) -> Result<Token> {
        let mut num_str = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() || ch == '.' {
                num_str.push(self.advance().unwrap());
            } else {
                break;
            }
        }
        num_str
            .parse::<f64>()
            .map(Token::LiteralNumber)
            .map_err(|_| XmlError::XPathError(format!("Invalid number literal '{num_str}'")))
    }

    fn read_name(&mut self) -> Result<Token> {
        let mut name = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' || ch == '-' || ch == ':' || ch == '.' {
                name.push(self.advance().unwrap());
            } else {
                break;
            }
        }
        Ok(Token::Name(name))
    }
}
