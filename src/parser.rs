use crate::error::CompilerError;
use crate::lexer::Lexer;
use crate::syntax::*;
use crate::token::{Token, TokenType};
use std::iter::Peekable;

pub struct Parser {
    lexer: Peekable<Lexer>,
    current_token: Option<Result<Token, CompilerError>>,
    current_pos: (usize, usize),
    pub(crate) errors: Vec<CompilerError>,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        let mut parser = Self {
            lexer: lexer.peekable(),
            current_token: None,
            errors: Vec::new(),
            current_pos: (0, 0),
        };

        parser.next_token();
        parser
    }

    fn next_token(&mut self) {
        let res = self.lexer.next();
        self.current_token = res;

        if let Some(Ok(t)) = &self.current_token {
            self.current_pos = t.pos
        }
    }

    fn parse_factor(&mut self) -> Result<Factor, CompilerError> {
        let factor = match &self.current_token {
            Some(Ok(token)) => match token {
                Token {
                    token: TokenType::Integer(_),
                    ..
                } => Ok(Factor::Integer(token.clone())),
                Token {
                    token: TokenType::Real(_),
                    ..
                } => Ok(Factor::Real(token.clone())),
                Token {
                    token: TokenType::Identifier(_),
                    ..
                } => Ok(Factor::Identifier(Identifier { id: token.clone() })),
                Token {
                    token: TokenType::LBrace,
                    ..
                } => Ok(Factor::Expression(Box::new(self.parse_expr()?))),
                tok => Err(CompilerError::syntax(
                    format!("Expected int or real literal, found {:#?}", tok),
                    tok.pos.0,
                    tok.pos.1,
                )),
            },
            Some(Err(e)) => Err((*e).clone()),
            _ => Err(CompilerError::syntax(
                format!("Expected int or real, found EOF"),
                self.current_pos.0,
                self.current_pos.1,
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
        if let Some(Ok(Token {
            token: TokenType::RBrace,
            ..
        })) = self.current_token
        {
            return Ok(None);
        };

        let op_res = self.parse_multiplicative_op()?;
        match op_res {
            Some(v) => {
                let factor = self.parse_factor()?;
                let sub_term_res = self.parse_sub_term()?;
                let sub_term = sub_term_res.map(Box::new);

                Ok(Some(SubTerm {
                    op: v,
                    factor,
                    sub_term,
                }))
            }
            None => Ok(None),
        }
    }

    fn parse_var_declaration(&mut self) -> Result<Vec<VarDeclaration>, CompilerError> {
        // id {,id} : type_id
        let tok = self.current_token.clone();

        let mut identifiers = Vec::new();
        loop {
            match tok {
                Some(Ok(ref token)) => {
                    match token {
                        Token {
                            token: TokenType::Identifier(_),
                            ..
                        } => {
                            identifiers.push(Ok(token.clone()));
                            self.next_token();

                            match &self.current_token {
                                Some(Ok(Token {
                                            token: TokenType::Comma,
                                            ..
                                        })) => self.next_token(),
                                _ => break,
                            }
                        }
                        _ => {
                            identifiers.push(Err(CompilerError::syntax(
                                format!("Expected identifier, found {:#?}", token),
                                token.pos.0,
                                token.pos.1,
                            )));
                        }
                    }
                }
                Some(Err(ref e)) => identifiers.push(Err(e.clone())),
                None => identifiers.push(Err(CompilerError::syntax(
                    "Unexpected EOF".into(),
                    self.current_pos.0,
                    self.current_pos.1,
                ))),
            }
        }

        let var_type = match &self.current_token {
            Some(Ok(Token {
                token: TokenType::Colon,
                ..
            })) => {
                self.next_token();
                match &self.current_token {
                    Some(Ok(token)) => {
                        let pos = token.pos;
                        let type_id = Ok(Identifier { id: token.clone() });
                        self.next_token();

                        match &self.current_token {
                            Some(Ok(Token {
                                token: TokenType::Semicolon,
                                ..
                            })) => {
                                self.next_token();
                                type_id
                            }
                            Some(Ok(t)) => Err(CompilerError::syntax(
                                "';' expected".into(),
                                t.pos.0,
                                t.pos.1,
                            )),
                            Some(Err(e)) => Err(e.clone()),
                            _ => Err(CompilerError::syntax("Unexpected EOF".into(), pos.0, pos.1)),
                        }
                    }
                    _ => Err(CompilerError::syntax(
                        "Expected identifier".into(),
                        self.current_pos.0,
                        self.current_pos.1,
                    )),
                }
            }
            _ => Err(CompilerError::syntax(
                "Expected ','".into(),
                self.current_pos.0,
                self.current_pos.1,
            )),
        };

        match var_type {
            Ok(type_id) => {
                let mut declarations = Vec::new();

                for id in identifiers {
                    if let Ok(token) = id {
                        declarations.push(VarDeclaration {
                            id: Identifier { id: token },
                            type_name: type_id.clone(),
                        });
                    } else {
                        if let Err(e) = id {
                            self.errors.push(e);
                        }
                    }
                }

                Ok(declarations)
            }
            Err(e) => Err(e),
        }
    }

    fn parse_var_section(&mut self) -> Result<VarSection, CompilerError> {
        // [var
        //      <var_declaration>
        //      {<var_declaration>}]
        let mut declarations = Vec::new();
        let check_section = match &self.current_token {
            Some(Ok(Token {
                token: TokenType::VarKeyword,
                ..
            })) => {
                self.next_token();
                Ok(())
            }
            Some(Ok(t)) => Err(CompilerError::syntax(
                format!("Expected VAR, found {:#?}", t),
                t.pos.0,
                t.pos.1,
            )),
            _ => Err(CompilerError::syntax(
                "Expected VAR, found EOF".into(),
                self.current_pos.0,
                self.current_pos.1,
            )),
        };

        match check_section {
            Ok(_) => loop {
                let decl = self.parse_var_declaration();
                match decl {
                    Ok(v) => {
                        for i in v {
                            declarations.push(i);
                        }
                    }
                    Err(e) => {
                        self.errors.push(e);

                        loop {
                            if let Some(Ok(Token {
                                token: TokenType::Identifier(_),
                                ..
                            })) = self.current_token
                            {
                                break;
                            };
                            self.next_token();
                        }
                    }
                }

                match self.current_token {
                    Some(Ok(Token {
                        token: TokenType::BeginKeyword,
                        pos,
                    })) => {
                        if declarations.len() == 0 {
                            self.errors.push(CompilerError::syntax(
                                "Expected identifier, found BEGIN".into(),
                                pos.0,
                                pos.1,
                            ));
                        };
                        return Ok(VarSection { declarations });
                    }
                    _ => continue,
                }
            },
            Err(e) => Err(e),
        }
    }

    fn parse_identifier(&mut self) -> Result<Identifier, CompilerError> {
        match &self.current_token.clone() {
            Some(Ok(token)) => {
                if let Token {
                    token: TokenType::Identifier(_),
                    ..
                } = token
                {
                    self.next_token();
                    Ok(Identifier { id: token.clone() })
                } else {
                    self.next_token();
                    Err(CompilerError::syntax(
                        format!("Expected identifier, found {:#?}", token),
                        token.pos.0,
                        token.pos.1,
                    ))
                }
            }
            Some(Err(e)) => Err(e.clone()),
            _ => Err(CompilerError::syntax(
                "Unexpected EOF".into(),
                self.current_pos.0,
                self.current_pos.1,
            )),
        }
    }

    // ';' and other such tokens aren't actually present inside
    // result tree, so we simple check if it's present and
    // return empty Ok [Ok(()) where () is a unit type]
    fn parse_semicolon(&mut self) -> Result<(), CompilerError> {
        // Move token here
        let target_token = self.current_token.clone();
        self.next_token();

        match target_token {
            Some(Ok(t)) => {
                if let Token {
                    token: TokenType::Semicolon,
                    ..
                } = t
                {
                    Ok(())
                } else {
                    Err(CompilerError::syntax(
                        format!("Expected ';', found {:#?}", t),
                        t.pos.0,
                        t.pos.1,
                    ))
                }
            }
            Some(Err(e)) => Err(e),
            _ => Err(CompilerError::syntax(
                "Unexpected EOF".into(),
                self.current_pos.0,
                self.current_pos.1,
            )),
        }
    }

    fn parse_period(&mut self) -> Result<(), CompilerError> {
        let tok = self.current_token.clone();
        self.next_token();

        match tok {
            Some(Ok(t)) => {
                match t {
                    Token { token: TokenType::Period, ..} => Ok(()),
                    _ => Err(CompilerError::syntax(format!("Expected '.', found {:#?}", t), t.pos.0, t.pos.1))
                }
            }
            Some(Err(e)) => Err(e),
            None => Err(CompilerError::syntax("Unexpected EOF".into(), self.current_pos.0, self.current_pos.1))
        }
    }

    fn parse_program(&mut self) -> Result<Program, CompilerError> {
        // program <identifier>;
        // <vars>
        // <types>
        // <procedures>
        // <compound>
        // end.
        match &self.current_token {
            Some(Ok(Token {
                token: TokenType::ProgramKeyword,
                ..
            })) => {
                self.next_token();
                let id = self.parse_identifier()?;
                // Semicolon check
                self.parse_semicolon()?;
                let var_section = Some(self.parse_var_section()?);
                let compound = self.parse_compound()?;

                self.parse_period()?;

                Ok(Program {
                    identifier: id,
                    var_section,
                    compound,
                })
            }
            Some(Ok(t)) => Err(CompilerError::syntax(
                format!("Expected 'PROGRAM', found {:#?}", t),
                t.pos.0,
                t.pos.1,
            )),
            Some(Err(e)) => Err(e.clone()),
            _ => Err(CompilerError::syntax(
                "Unexpected EOF".into(),
                self.current_pos.0,
                self.current_pos.1,
            )),
        }
    }

    fn parse_additive_op(&mut self) -> Result<Option<AdditiveOp>, CompilerError> {
        // TODO: this check probably should not be here
        if let Some(Ok(Token {
            token: TokenType::RBrace,
            ..
        })) = self.current_token
        {
            return Ok(None);
        };

        let op = match &self.current_token {
            Some(Ok(t)) => match t {
                Token {
                    token: TokenType::PlusOp,
                    ..
                } => {
                    self.next_token();
                    Ok(Some(AdditiveOp::Plus))
                }
                Token {
                    token: TokenType::MinusOp,
                    ..
                } => {
                    self.next_token();
                    Ok(Some(AdditiveOp::Minus))
                }
                // TODO: this should be checked using lookahead parsing
                Token {
                    token: TokenType::DivOp,
                    ..
                }
                | Token {
                    token: TokenType::MulOp,
                    ..
                }
                | Token {
                    token: TokenType::ModOp,
                    ..
                }
                | Token {
                    token: TokenType::Semicolon,
                    ..
                }
                | Token {
                    token: TokenType::EndKeyword,
                    ..
                } => Ok(None),
                tok => Err(CompilerError::syntax(
                    format!("Expected operator, found {:#?}", tok),
                    tok.pos.0,
                    tok.pos.1,
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
            Some(Ok(Token {
                token: TokenType::Identifier(_),
                ..
            })) => Ok(Statement::Simple(self.parse_assignment()?)),
            _ => Err(CompilerError::syntax(
                "Illegal statement".into(),
                self.current_pos.0,
                self.current_pos.1,
            )),
        }
    }

    fn parse_compound(&mut self) -> Result<Compound, CompilerError> {
        if let Some(Ok(Token {
            token: TokenType::BeginKeyword,
            ..
        })) = self.current_token
        {
            // Consume
            self.next_token();
        };

        let mut statements = Vec::new();

        loop {
            if let Some(Ok(Token {
                token: TokenType::EndKeyword,
                ..
            })) = self.current_token
            {
                self.next_token();
                break;
            } else {
                let statement = self.parse_statement();
                match statement {
                    Ok(st) => statements.push(st),
                    Err(e) => {
                        self.errors.push(e);

                        loop {
                            match self.current_token.clone() {
                                Some(Ok(token)) => {
                                    match token {
                                        Token { token: TokenType::Identifier(_), ..} |
                                        Token { token: TokenType::EndKeyword, .. } => {
                                            break
                                        }
                                        _ => self.next_token()
                                    }
                                }
                                Some(Err(e)) => {
                                    self.errors.push(e);
                                    self.next_token();
                                },
                                None => break
                            }
                        }
                    }
                }
            };
        }

        Ok(Compound { statements })
    }

    fn parse_expr(&mut self) -> Result<Expression, CompilerError> {
        let mut inside_braces = false;

        if let &Some(Ok(Token {
            token: TokenType::LBrace,
            ..
        })) = &self.current_token
        {
            inside_braces = true;
            self.next_token();
        };

        let expr = Expression {
            term: self.parse_term()?,
            sub_expr: self.parse_sub_expr()?,
        };

        if inside_braces {
            match self.current_token.clone() {
                Some(Ok(Token {
                    token: TokenType::RBrace,
                    ..
                })) => {
                    // Do not consume RBrace => it is consumed inside parse_factor
                    Ok(expr)
                }
                Some(Ok(t)) => Err(CompilerError::syntax(
                    format!("Expected closing brace, got {:#?}", t),
                    t.pos.0,
                    t.pos.1,
                )),
                _ => Err(CompilerError::syntax(
                    "Unexpected EOF (expected '}}'".into(),
                    self.current_pos.0,
                    self.current_pos.1,
                )),
            }
        } else {
            Ok(expr)
        }
    }
    fn parse_assignment(&mut self) -> Result<VarAssignment, CompilerError> {
        let id = self.parse_identifier()?;

        match &self.current_token {
            Some(Ok(Token {
                token: TokenType::AssignOp,
                ..
            })) => {
                self.next_token();

                let assignment = VarAssignment {
                    name: id,
                    value: self.parse_expr()?,
                };

                if let Some(Ok(Token {
                    token: TokenType::Semicolon,
                    ..
                })) = self.current_token
                {
                    // Consume
                    self.next_token();
                }

                Ok(assignment)
            }
            Some(Ok(t)) => Err(CompilerError::syntax(
                format!("Expected :=, found {:#?}", t),
                t.pos.0,
                t.pos.1,
            )),
            Some(Err(e)) => Err(e.clone()),
            _ => Err(CompilerError::syntax(
                "Expected :=, found EOF".into(),
                self.current_pos.0,
                self.current_pos.1,
            )),
        }
    }

    pub fn parse(&mut self) -> Result<Program, CompilerError> {
        // let var_section = self.parse_var_section()?;
        // let compound = self.parse_compound()?;

        // Ok((var_section, compound))
        self.parse_program()
    }

    fn parse_multiplicative_op(&mut self) -> Result<Option<MultiplicativeOp>, CompilerError> {
        let op = match &self.current_token {
            Some(Ok(Token {
                token: TokenType::MulOp,
                ..
            })) => {
                self.next_token();
                Ok(Some(MultiplicativeOp::Mul))
            }
            Some(Ok(Token {
                token: TokenType::DivOp,
                ..
            })) => {
                self.next_token();
                Ok(Some(MultiplicativeOp::Div))
            }
            Some(Ok(Token {
                token: TokenType::ModOp,
                ..
            })) => {
                self.next_token();
                Ok(Some(MultiplicativeOp::Mod))
            }
            Some(Ok(Token {
                token: TokenType::PlusOp,
                ..
            }))
            | Some(Ok(Token {
                token: TokenType::MinusOp,
                ..
            }))
            | Some(Ok(Token {
                token: TokenType::Semicolon,
                ..
            }))
            | Some(Ok(Token {
                token: TokenType::EndKeyword,
                ..
            }))
            | None => Ok(None),
            Some(Ok(Token {
                token: TokenType::Identifier(s),
                pos,
            })) => Err(CompilerError::syntax(
                format!("Expected ; but found identifier {:#?}", s),
                pos.0,
                pos.1,
            )),
            Some(Ok(t)) => Err(CompilerError::syntax(
                format!("Expected *, div or mod, found {:#?}", t),
                t.pos.0,
                t.pos.1,
            )),
            Some(Err(e)) => Err((*e).clone()),
        };

        op
    }
}
