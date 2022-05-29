use crate::error::CompilerError;
use crate::io::CharReader;
use crate::token::Token;
use std::iter::Iterator;

pub struct Lexer {
    chars: CharReader,
}

impl Lexer {
    pub fn new(chars: CharReader) -> Self {
        Self { chars }
    }

    fn skip_ws(&mut self) {
        loop {
            let cur = self.chars.current_char();

            if cur.is_none() {
                break;
            }

            match cur.unwrap() {
                c if c.is_whitespace() => {
                    self.chars.by_ref().next();
                    continue;
                }
                _ => {
                    break;
                }
            }
        }
    }

    fn number(&mut self) -> Result<Token, CompilerError> {
        let mut num = String::new();
        let mut is_real = false;

        loop {
            match self.chars.by_ref().current_char() {
                Some(ch) if ch.is_digit(10) => num.push(ch),
                Some(ch) if ch == '.' => {
                    num.push(ch);
                    is_real = true;
                }
                Some(ch) if ch.is_whitespace() => break,
                Some(ch) if ch.is_alphanumeric() => {
                    // Consume everything until whitespace or EOF
                    num.push(ch);

                    while let Some(ch) = self.chars.next() {
                        if ch.is_whitespace() {
                            break;
                        } else {
                            num.push(ch);
                        }

                        break;
                    }
                }
                _ => break,
            }

            self.chars.by_ref().next();
        }

        if is_real {
            let parsed = num.parse::<f32>();

            match parsed {
                Ok(f) => Ok(Token::Real(f)),
                _ => {
                    let pos = self.chars.position();
                    Err(CompilerError::lexical(
                        String::from(format!("Invalid real literal {}", num)),
                        pos.0,
                        pos.1,
                    ))
                }
            }
        } else {
            let parsed = num.parse::<i32>();

            match parsed {
                Ok(i) => Ok(Token::Integer(i)),
                _ => {
                    let pos = self.chars.position();

                    Err(CompilerError::lexical(
                        String::from(format!("Invalid int literal {}", num)),
                        pos.0,
                        pos.1,
                    ))
                }
            }
        }
    }

    fn maybe_keyword(&mut self) -> Result<Token, CompilerError> {
        if self.chars.by_ref().current_char().is_none() {
            Ok(Token::EOF)
        } else {
            let mut s = String::new();
            s.push(self.chars.by_ref().current_char().unwrap());

            loop {
                match self.chars.next() {
                    Some(ch) if ch.is_alphanumeric() => s.push(ch),
                    _ => break,
                }
            }

            match s.to_lowercase().as_str() {
                "div" => Ok(Token::DivOp),
                "mod" => Ok(Token::ModOp),
                "program" => Ok(Token::ProgramKeyword),
                "begin" => Ok(Token::BeginKeyword),
                "end" => Ok(Token::EndKeyword),
                "integer" => Ok(Token::IntegerKeyword),
                "real" => Ok(Token::RealKeyword),
                "var" => Ok(Token::VarKeyword),
                _ => Ok(Token::Identifier(s)),
            }
        }
    }

    fn operator(&mut self) -> Result<Token, CompilerError> {
        let op = if self.chars.current_char().is_none() {
            Ok(Token::EOF)
        } else {
            match self.chars.current_char().unwrap() {
                '+' => Ok(Token::PlusOp),
                '-' => Ok(Token::MinusOp),
                '*' => Ok(Token::MulOp),
                ':' => match self.chars.by_ref().peek() {
                    Some(ch) if ch == &'=' => {
                        self.chars.by_ref().next();
                        Ok(Token::AssignOp)
                    }
                    _ => Ok(Token::Colon),
                },
                _ => {
                    let pos = self.chars.position();

                    Err(CompilerError::lexical(
                        String::from("Invalid operator"),
                        pos.0,
                        pos.1,
                    ))
                }
            }
        };

        self.chars.by_ref().next();
        op
    }

    fn symbol(&mut self) -> Result<Token, CompilerError> {
        let sym = if self.chars.current_char().is_none() {
            Ok(Token::EOF)
        } else {
            match self.chars.current_char().unwrap() {
                ';' => Ok(Token::Semicolon),
                '.' => Ok(Token::Period),
                '(' => Ok(Token::LBrace),
                ')' => Ok(Token::RBrace),
                '\'' => {
                    // Read chars until string literal is closed
                    let literal: String = self
                        .chars
                        .by_ref()
                        .take_while(|x: &char| x != &'\'')
                        .collect();

                    match self.chars.current_char() {
                        Some('\'') => Ok(Token::StringLiteral(literal)),
                        _ => {
                            let pos = self.chars.position();

                            Err(CompilerError::lexical(
                                String::from("Invalid string literal"),
                                pos.0,
                                pos.1,
                            ))
                        }
                    }
                }
                _ => {
                    let pos = self.chars.position();

                    Err(CompilerError::lexical(
                        String::from(format!(
                            "Unsupported symbol {}",
                            self.chars.current_char().unwrap()
                        )),
                        pos.0,
                        pos.1,
                    ))
                }
            }
        };

        self.chars.next();
        sym
    }
}

impl Iterator for Lexer {
    type Item = Result<Token, CompilerError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_ws();

        let token = match self.chars.by_ref().current_char() {
            Some(ch) => match ch {
                '0'..='9' => self.number(),
                '+' | '-' | '*' | ':' => self.operator(),
                _ if ch.is_alphanumeric() => self.maybe_keyword(),
                _ => self.symbol(),
            },
            None => Ok(Token::EOF),
        };

        match token {
            Ok(Token::EOF) => None,
            _ => Some(token),
        }
    }
}
