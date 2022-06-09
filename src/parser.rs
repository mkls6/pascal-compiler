use crate::error::CompilerError;
use crate::lexer::Lexer;
use crate::syntax::*;
use crate::token::Token;
use std::iter::Peekable;

pub struct Parser {
    lexer: Peekable<Lexer>,
    current_token: Option<Result<Token, CompilerError>>,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        let mut parser = Self {
            lexer: lexer.peekable(),
            current_token: None,
        };

        parser.next_token();
        parser
    }

    fn next_token(&mut self) {
        let res = self.lexer.next();
        self.current_token = res;
    }

    fn parse_factor(&mut self) -> Result<Factor, CompilerError> {
        let factor = match &self.current_token {
            Some(Ok(t)) => match t {
                Token::Integer(i) => Ok(Factor::Integer(*i)),
                Token::Real(f) => Ok(Factor::Real(*f)),
                Token::LBrace => {
                    // Consume left brace
                    self.next_token();
                    let expr = Ok(Factor::Expression(Box::new(self.parse_expr()?)));

                    match self.current_token {
                        Some(Ok(Token::RBrace)) => {
                            self.next_token();
                            expr
                        }
                        _ => Err(CompilerError::syntax("Expected closing brace".into(), 0, 0))
                    }
                }
                tok => Err(CompilerError::syntax(
                    String::from(format!("Expected int or real literal, found {}", tok)),
                    0,
                    0,
                )),
            },
            Some(Err(e)) => Err((*e).clone()),
            _ => Err(CompilerError::syntax(
                String::from(format!("Expected int or real, found EOF")),
                0,
                0,
            )),
        };
        self.next_token();
        factor
    }

    fn parse_term(&mut self) -> Result<Term, CompilerError> {
        let term = Term {
            factor: self.parse_factor()?,
            sub_term: self.parse_sub_term()?,
        };

        Ok(term)
    }

    fn parse_sub_term(&mut self) -> Result<Option<SubTerm>, CompilerError> {
        if let Some(Ok(Token::RBrace)) = self.current_token {
            return Ok(None)
        };

        let op_res = self.parse_multiplicative_op()?;
        match op_res {
            Some(v) => {
                let factor = self.parse_factor()?;
                let sub_term_res = self.parse_sub_term()?;
                let sub_term = match sub_term_res {
                    Some(t) => Some(Box::new(t)),
                    None => None,
                };

                Ok(Some(SubTerm {
                    op: v,
                    factor,
                    sub_term,
                }))
            }
            None => Ok(None),
        }
    }

    fn parse_additive_op(&mut self) -> Result<Option<AdditiveOp>, CompilerError> {
        if let Some(Ok(Token::RBrace)) = self.current_token {
            return Ok(None)
        };

        let op = match &self.current_token {
            Some(Ok(t)) => match t {
                Token::PlusOp => {
                    self.next_token();
                    Ok(Some(AdditiveOp::Plus))
                }
                Token::MinusOp => {
                    self.next_token();
                    Ok(Some(AdditiveOp::Minus))
                }
                Token::DivOp | Token::MulOp | Token::ModOp => Ok(None),
                tok => Err(CompilerError::syntax(
                    String::from(format!("Expected operator, found {}", tok)),
                    0,
                    0,
                )),
            },
            Some(Err(e)) => Err((*e).clone()),
            None => Ok(None),
        };

        op
    }

    fn parse_sub_expr(&mut self) -> Result<Option<SubExpression>, CompilerError> {
        let op_res = self.parse_additive_op()?;
        match op_res {
            Some(o) => {
                let term = self.parse_term()?;
                let sub_expr_res = self.parse_sub_expr()?;
                let sub_expr;

                if sub_expr_res.is_none() {
                    sub_expr = None;
                } else {
                    sub_expr = Some(Box::new(sub_expr_res.unwrap()));
                };

                Ok(Some(SubExpression {
                    op: o,
                    term,
                    sub_expr,
                }))
            }
            None => Ok(None),
        }
    }

    fn parse_expr(&mut self) -> Result<Expression, CompilerError> {
        let expr = Expression {
            term: self.parse_term()?,
            sub_expr: self.parse_sub_expr()?,
        };

        Ok(expr)
    }

    pub fn parse(&mut self) -> Result<Expression, CompilerError> {
        match &mut self.current_token {
            Some(Err(e)) => Err((*e).clone()),
            Some(Ok(t)) => match (*t).clone() {
                Token::Integer(_) | Token::Real(_) => self.parse_expr(),
                tok => Err(CompilerError::syntax(
                    String::from(format!("Expected int or real literal, found {}", tok)),
                    0,
                    0,
                )),
            },
            None => Err(CompilerError::syntax(
                String::from("Expected program start, found EOF (empty program)"),
                0,
                0,
            )),
        }
    }

    fn parse_multiplicative_op(&mut self) -> Result<Option<MultiplicativeOp>, CompilerError> {
        let op = match &self.current_token {
            Some(Ok(Token::MulOp)) => {
                self.next_token();
                Ok(Some(MultiplicativeOp::Mul))
            }
            Some(Ok(Token::DivOp)) => {
                self.next_token();
                Ok(Some(MultiplicativeOp::Div))
            }
            Some(Ok(Token::ModOp)) => {
                self.next_token();
                Ok(Some(MultiplicativeOp::Mod))
            }
            Some(Ok(Token::PlusOp)) | Some(Ok(Token::MinusOp)) | None => Ok(None),
            Some(Ok(t)) => Err(CompilerError::syntax(
                String::from(format!("Expected *, div or mod, found {}", t)),
                0,
                0,
            )),
            Some(Err(e)) => Err((*e).clone()),
        };

        op
    }
}
