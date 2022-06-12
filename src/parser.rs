use crate::analyzer::Analyzer;
use crate::error::CompilerError;
use crate::lexer::Lexer;
use crate::syntax::*;
use crate::token::{Token, TokenType};
use std::iter::Peekable;
use crate::scope::Usage;

pub struct Parser {
    lexer: Peekable<Lexer>,
    current_token: Option<Result<Token, CompilerError>>,
    current_pos: (usize, usize),
    pub(crate) errors: Vec<CompilerError>,
    analyzer: Analyzer,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        let mut parser = Self {
            lexer: lexer.peekable(),
            current_token: None,
            errors: Vec::new(),
            current_pos: (0, 0),
            analyzer: Analyzer::new(),
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
                } => {
                    let id = Identifier { id: token.clone() };
                    self.analyzer.find_identifier(&id, &Usage::Variable)?;
                    Ok(Factor::Identifier(Identifier { id: token.clone() }))
                },
                Token {
                    token: TokenType::LBrace,
                    ..
                } => Ok(Factor::Expression(Box::new(self.parse_expr()?))),
                tok => Err(CompilerError::syntax(
                    format!("Expected int or real literal, found {:?}", tok),
                    tok.pos,
                )),
            },
            Some(Err(e)) => Err((*e).clone()),
            _ => Err(CompilerError::syntax(
                "Expected int or real, found EOF".into(),
                self.current_pos,
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

        match &self.current_token {
            Some(Ok(t)) if t.is_mul_op() => {
                let op = self.parse_multiplicative_op()?;
                let factor = self.parse_factor()?;
                let sub_term_res = self.parse_sub_term()?;
                let sub_term = sub_term_res.map(Box::new);

                Ok(Some(SubTerm {
                    op,
                    factor,
                    sub_term,
                }))
            }
            Some(Ok(t)) if t.is_add_op() || t.is_expression_end() => Ok(None),
            Some(Ok(t)) => Err(CompilerError::syntax(
                format!("Expected *, div or mod, found {:?}", t),
                t.pos,
            )),
            Some(Err(e)) => Err(e.clone()),
            None => Err(CompilerError::syntax(
                "Unexpected EOF".into(),
                self.current_pos,
            )),
        }
    }

    fn parse_var_declaration(&mut self) -> Result<Vec<VarDeclaration>, CompilerError> {
        // id {,id} : type_id
        let mut identifiers = Vec::new();

        loop {
            match self.current_token.take() {
                Some(Ok(token)) => match token {
                    Token {
                        token: TokenType::Identifier(_),
                        ..
                    } => {
                        identifiers.push(Ok(token));
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
                            format!("Expected identifier, found {:?}", token),
                            token.pos,
                        )));
                    }
                },
                Some(Err(e)) => identifiers.push(Err(e)),
                None => identifiers.push(Err(CompilerError::syntax(
                    "Unexpected EOF".into(),
                    self.current_pos,
                ))),
            }
        }

        let var_type = match &self.current_token {
            Some(Ok(Token {
                token: TokenType::Colon,
                ..
            })) => {
                self.next_token();
                match self.current_token.take() {
                    Some(Ok(token)) => {
                        let type_id = Ok(Identifier { id: token });
                        self.next_token();
                        self.parse_semicolon()?;
                        type_id
                    }
                    _ => Err(CompilerError::syntax(
                        "Expected identifier".into(),
                        self.current_pos,
                    )),
                }
            }
            _ => Err(CompilerError::syntax(
                "Expected ','".into(),
                self.current_pos,
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
                    } else if let Err(e) = id {
                        self.errors.push(e);
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
                format!("Expected VAR, found {:?}", t),
                t.pos,
            )),
            _ => Err(CompilerError::syntax(
                "Expected VAR, found EOF".into(),
                self.current_pos,
            )),
        };

        match check_section {
            Ok(_) => loop {
                let decl = self.parse_var_declaration();
                match decl {
                    Ok(v) => {
                        for i in v {
                            let check_res = self.analyzer.check_declaration(i);
                            match check_res {
                                Ok(decl) => declarations.push(decl),
                                Err(e) => self.errors.push(e)
                            }
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
                        if declarations.is_empty() {
                            self.errors.push(CompilerError::syntax(
                                "Expected identifier, found BEGIN".into(),
                                pos,
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
                        format!("Expected identifier, found {:?}", token),
                        token.pos,
                    ))
                }
            }
            Some(Err(e)) => Err(e.clone()),
            _ => Err(CompilerError::syntax(
                "Unexpected EOF".into(),
                self.current_pos,
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
                        format!("Expected ';', found {:?}", t),
                        t.pos,
                    ))
                }
            }
            Some(Err(e)) => Err(e),
            _ => Err(CompilerError::syntax(
                "Unexpected EOF".into(),
                self.current_pos,
            )),
        }
    }

    fn parse_period(&mut self) -> Result<(), CompilerError> {
        let tok = self.current_token.clone();
        self.next_token();

        match tok {
            Some(Ok(t)) => match t {
                Token {
                    token: TokenType::Period,
                    ..
                } => Ok(()),
                _ => Err(CompilerError::syntax(
                    format!("Expected '.', found {:?}", t),
                    t.pos,
                )),
            },
            Some(Err(e)) => Err(e),
            None => Err(CompilerError::syntax(
                "Unexpected EOF".into(),
                self.current_pos,
            )),
        }
    }

    fn parse_program(&mut self) -> Result<Program, CompilerError> {
        // program <identifier>;
        // <vars>
        // <types>
        // <procedures>
        // <compound>
        // end.
        self.analyzer.enter_scope();
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
                self.analyzer.leave_scope();

                Ok(Program {
                    identifier: id,
                    var_section,
                    compound,
                })
            }
            Some(Ok(t)) => Err(CompilerError::syntax(
                format!("Expected 'PROGRAM', found {:?}", t),
                t.pos,
            )),
            Some(Err(e)) => Err(e.clone()),
            _ => Err(CompilerError::syntax(
                "Unexpected EOF".into(),
                self.current_pos,
            )),
        }
    }

    fn parse_additive_op(&mut self) -> Result<AdditiveOp, CompilerError> {
        let op = match self.current_token.take() {
            Some(Ok(t)) => match t {
                Token {
                    token: TokenType::PlusOp,
                    ..
                } => {
                    self.next_token();
                    Ok(AdditiveOp::Plus)
                }
                Token {
                    token: TokenType::MinusOp,
                    ..
                } => {
                    self.next_token();
                    Ok(AdditiveOp::Minus)
                }
                tok => Err(CompilerError::syntax(
                    format!("Expected additive operator, found {:?}", tok),
                    tok.pos,
                )),
            },
            Some(Err(e)) => Err(e),
            None => Err(CompilerError::syntax(
                "Unexpected EOF".into(),
                self.current_pos,
            )),
        };

        op
    }

    fn parse_sub_expr(&mut self) -> Result<Option<SubExpression>, CompilerError> {
        match &self.current_token {
            Some(Ok(t)) if t.is_expression_end() => Ok(None),
            Some(Ok(t)) if t.is_add_op() => {
                let op = self.parse_additive_op()?;
                let term = self.parse_term()?;
                let sub_expr_res = self.parse_sub_expr()?;
                let sub_expr = sub_expr_res.map(Box::new);

                Ok(Some(SubExpression { op, term, sub_expr }))
            }
            Some(Ok(t)) => Err(CompilerError::syntax(
                format!("Expected +, - or statement end, found {:?}", t),
                t.pos,
            )),
            Some(Err(e)) => Err(e.clone()),
            None => Err(CompilerError::syntax(
                "Unexpected EOF".into(),
                self.current_pos,
            )),
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
                self.current_pos,
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
                        self.skip_until_starters();
                    }
                }
            };
        }

        Ok(Compound { statements })
    }

    fn skip_until_starters(&mut self) {
        loop {
            match &self.current_token {
                Some(Ok(token)) => match token {
                    Token {
                        token: TokenType::Identifier(_),
                        ..
                    }
                    | Token {
                        token: TokenType::EndKeyword,
                        ..
                    } => {
                        return;
                    }
                    _ => self.next_token(),
                },
                Some(Err(e)) => {
                    self.errors.push(e.clone());
                    self.next_token();
                }
                None => return,
            }
        }
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
                    format!("Expected closing brace, got {:?}", t),
                    t.pos,
                )),
                _ => Err(CompilerError::syntax(
                    "Unexpected EOF (expected '}}'".into(),
                    self.current_pos,
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

                self.parse_semicolon()?;

                Ok(assignment)
            }
            Some(Ok(t)) => Err(CompilerError::syntax(
                format!("Expected :=, found {:?}", t),
                t.pos,
            )),
            Some(Err(e)) => Err(e.clone()),
            _ => Err(CompilerError::syntax(
                "Expected :=, found EOF".into(),
                self.current_pos,
            )),
        }
    }

    pub fn parse(&mut self) -> Result<Program, CompilerError> {
        // let var_section = self.parse_var_section()?;
        // let compound = self.parse_compound()?;

        // Ok((var_section, compound))
        self.parse_program()
    }

    fn parse_multiplicative_op(&mut self) -> Result<MultiplicativeOp, CompilerError> {
        let op = match &self.current_token {
            Some(Ok(Token {
                token: TokenType::MulOp,
                ..
            })) => {
                self.next_token();
                Ok(MultiplicativeOp::Mul)
            }
            Some(Ok(Token {
                token: TokenType::DivOp,
                ..
            })) => {
                self.next_token();
                Ok(MultiplicativeOp::Div)
            }
            Some(Ok(Token {
                token: TokenType::ModOp,
                ..
            })) => {
                self.next_token();
                Ok(MultiplicativeOp::Mod)
            }
            Some(Ok(Token {
                token: TokenType::Identifier(s),
                pos,
            })) => Err(CompilerError::syntax(
                format!("Expected ; but found identifier {:?}", s),
                *pos,
            )),
            Some(Ok(t)) => Err(CompilerError::syntax(
                format!("Expected *, div or mod, found {:?}", t),
                t.pos,
            )),
            Some(Err(e)) => Err((*e).clone()),
            None => Err(CompilerError::syntax(
                "Unexpected EOF".into(),
                self.current_pos,
            )),
        };

        op
    }
}
