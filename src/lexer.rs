use crate::error::LexicalError;
use crate::io::CharReader;
use crate::token::Token;
use std::iter::Iterator;

#[allow(dead_code)]
pub struct Lexer<'a> {
    chars: CharReader<'a>,
    current_token: String,
}

#[allow(dead_code)]
impl<'a> Lexer<'a> {
    pub fn new(chars: CharReader<'a>) -> Self {
        let current_token = String::new();

        Self {
            chars,
            current_token,
        }
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

    fn number(&mut self) -> Result<Token, LexicalError> {
        let mut num = String::new();

        loop {
            match self.chars.by_ref().current_char() {
                Some(ch) if ch.is_digit(10) => num.push(ch),
                _ => break,
            }

            self.chars.by_ref().next();
        }

        let parsed = num.parse::<i32>();

        match parsed {
            Ok(i) => Ok(Token::INTEGER(i)),
            _ => Err(LexicalError {
                description: String::from("Failed to parse int"),
            }),
        }
    }

    fn operator(&mut self) -> Result<Token, LexicalError> {
        let op = if self.chars.current_char().is_none() {
            Ok(Token::EOF)
        } else {
            match self.chars.current_char().unwrap() {
                '+' => Ok(Token::PLUS),
                '-' => Ok(Token::MINUS),
                _ => Err(LexicalError {
                    description: String::from("Invalid operator"),
                }),
            }
        };

        self.chars.by_ref().next();
        op
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token, LexicalError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_ws();

        let token = match self.chars.by_ref().current_char() {
            Some(ch) => match ch {
                '0'..='9' => self.number(),
                '+' | '-' => self.operator(),
                _ => {
                    self.chars.by_ref().next();

                    Err(LexicalError {
                        description: String::from(format!(
                            "Unsupported character '{}' in input stream",
                            ch
                        )),
                    })
                }
            },
            None => Ok(Token::EOF),
        };

        match token {
            Ok(Token::EOF) => None,
            _ => Some(token),
        }
    }
}
