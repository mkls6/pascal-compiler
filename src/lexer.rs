use crate::error::LexicalError;
use crate::io::CharReader;
use crate::token::Token;
use std::iter::Iterator;

pub struct Lexer<'a> {
    chars: CharReader<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(chars: CharReader<'a>) -> Self {
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

    fn maybe_keyword(&mut self) -> Result<Token, LexicalError> {
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
                "div" => Ok(Token::DIV),
                "mod" => Ok(Token::MOD),
                "program" => Ok(Token::PROGRAM),
                "begin" => Ok(Token::BEGIN),
                "end" => Ok(Token::END),
                _ => Ok(Token::IDENTIFIER(s)),
            }
        }
    }

    fn operator(&mut self) -> Result<Token, LexicalError> {
        let op = if self.chars.current_char().is_none() {
            Ok(Token::EOF)
        } else {
            match self.chars.current_char().unwrap() {
                '+' => Ok(Token::PLUS),
                '-' => Ok(Token::MINUS),
                '*' => Ok(Token::MUL),
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
                '+' | '-' | '*' => self.operator(),
                _ if ch.is_alphanumeric() => self.maybe_keyword(),
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
