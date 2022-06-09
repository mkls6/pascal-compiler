use crate::error::CompilerError;
use crate::lexer::Lexer;
use crate::syntax::*;
use crate::token::Token;
use std::iter::Peekable;

pub struct Parser {
    lexer: Peekable<Lexer>,
    current_token: Option<Result<Token, CompilerError>>,
    pub(crate) errors: Vec<CompilerError>,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        let mut parser = Self {
            lexer: lexer.peekable(),
            current_token: None,
            errors: Vec::new(),
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
                Token::Identifier(s) => Ok(Factor::Identifier(Identifier { name: s.clone() })),
                Token::LBrace => {
                    Ok(Factor::Expression(Box::new(self.parse_expr()?)))
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
        // TODO: this check probably should not be here
        if let Some(Ok(Token::RBrace)) = self.current_token {
            return Ok(None);
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
        // TODO: this check probably should not be here
        if let Some(Ok(Token::RBrace)) = self.current_token {
            return Ok(None);
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
                Token::DivOp | Token::MulOp | Token::ModOp | Token::Semicolon | Token::EndKeyword => Ok(None),
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

    fn parse_statement(&mut self) -> Result<Statement, CompilerError> {
        match &self.current_token {
            Some(Ok(Token::Identifier(_))) => {
                Ok(Statement::Simple(self.parse_assignment()?))
            }
            _ => Err(CompilerError::syntax("Illegal statement".into(), 0, 0))
        }
    }

    fn parse_compound(&mut self) -> Result<Compound, CompilerError> {
        if let Some(Ok(Token::BeginKeyword)) = self.current_token {
            // Consume
            self.next_token();
        };

        let mut statements = Vec::new();

        loop {
            if let Some(Ok(Token::EndKeyword)) = self.current_token {
                self.next_token();
                break;
            } else {
                let statement = self.parse_statement();
                match statement {
                    Ok(st) => statements.push(st),
                    Err(e) => {
                        self.errors.push(e);

                        loop {
                            match self.current_token {
                                // TODO: check for proper starters
                                Some(Ok(Token::Identifier(_))) => break,
                                _ => self.next_token()
                            }
                        }
                    }
                }
            };
        }

        Ok(Compound {
            statements
        })
    }

    fn parse_expr(&mut self) -> Result<Expression, CompilerError> {
        let mut inside_braces = false;

        if let &Some(Ok(Token::LBrace)) = &self.current_token {
            inside_braces = true;
            self.next_token();
        };

        let expr = Expression {
            term: self.parse_term()?,
            sub_expr: self.parse_sub_expr()?,
        };

        if inside_braces {
            match self.current_token.clone() {
                Some(Ok(Token::RBrace)) => {
                    // Do not consume RBrace => it is consumed inside parse_factor
                    Ok(expr)
                }
                Some(Ok(t)) => Err(CompilerError::syntax(format!("Expected closing brace, got {}", t), 0, 0)),
                _ => Err(CompilerError::syntax("Unexpected EOF (expected '}}'".into(), 0, 0))
            }
        } else {
            Ok(expr)
        }
    }

    fn parse_identifier(&mut self) -> Result<Identifier, CompilerError> {
        match &self.current_token {
            Some(Ok(Token::Identifier(s))) => {
                let id = Identifier { name: s.clone() };
                self.next_token();
                Ok(id)
            }
            Some(Ok(t)) => Err(CompilerError::syntax(format!("Expected identifier, found {}", t), 0, 0)),
            Some(Err(e)) => Err(e.clone()),
            None => Err(CompilerError::syntax("Expected identifier, found EOF".into(), 0, 0))
        }
    }

    fn parse_assignment(&mut self) -> Result<VarAssignment, CompilerError> {
        let id = self.parse_identifier()?;

        match &self.current_token {
            Some(Ok(Token::AssignOp)) => {
                self.next_token();

                let assignment = VarAssignment {
                    name: id,
                    value: self.parse_expr()?,
                };

                if let Some(Ok(Token::Semicolon)) = self.current_token {
                    // Consume
                    self.next_token();
                }

                Ok(assignment)
            }
            Some(Ok(t)) => Err(CompilerError::syntax(format!("Expected :=, found {}", t), 0, 0)),
            Some(Err(e)) => Err(e.clone()),
            _ => Err(CompilerError::syntax("Expected :=, found EOF".into(), 0, 0))
        }
    }

    pub fn parse(&mut self) -> Result<Compound, CompilerError> {
        self.parse_compound()
        // self.parse_assignment()
        // self.parse_expr()
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
            Some(Ok(Token::Identifier(s))) => {
                Err(CompilerError::syntax(format!("Expected ; but found {}", s), 0, 0))
            }
            Some(Ok(Token::PlusOp)) |
            Some(Ok(Token::MinusOp)) |
            Some(Ok(Token::Semicolon)) |
            Some(Ok(Token::EndKeyword)) |
            None => Ok(None),
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
