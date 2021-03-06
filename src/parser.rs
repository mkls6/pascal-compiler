use crate::analyzer::Analyzer;
use crate::error::CompilerError;
use crate::lexer::Lexer;
use crate::scope::Usage;
use crate::syntax::*;
use crate::token::{Token, TokenType};
use std::iter::Peekable;

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
                    let usage = self.analyzer.find_identifier(&id)?;
                    match usage {
                        Usage::Variable(_) | Usage::Constant(_) => {
                            Ok(Factor::Identifier(Identifier { id: token.clone() }))
                        }
                        _ => Err(CompilerError::semantic(
                            "Identifier is not a variable".into(),
                            token.pos,
                        )),
                    }
                }
                Token {
                    token: TokenType::LBrace,
                    ..
                } => Ok(Factor::Expression(Box::new(self.parse_simple_expr()?))),
                tok => Err(CompilerError::syntax(
                    format!("Expected literal or identifier, found {:?}", tok),
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
        let factor = Box::new(self.parse_factor()?);
        let sub_term = self.parse_sub_term()?;

        let factor_type = self.analyzer.get_factor_type(&factor)?;
        let fact_type_str = match factor_type {
            Usage::Variable(s) | Usage::Constant(s) => s,
            _ => todo!(),
        };
        let sub_term_type;
        if sub_term.is_some() {
            sub_term_type = self
                .analyzer
                .get_sub_term_type(&sub_term.as_ref().unwrap())?;
        } else {
            sub_term_type = String::new();
        }

        let term_type =
            self.analyzer
                .merge_types(&fact_type_str, &sub_term_type, self.current_pos, false)?;

        let term = Term {
            factor,
            sub_term,
            term_type,
        };

        Ok(term)
    }

    fn parse_sub_term(&mut self) -> Result<Option<Box<SubTerm>>, CompilerError> {
        // TODO: this check probably should not be here
        match &self.current_token {
            Some(Ok(Token {
                token: TokenType::RBrace,
                ..
            })) => Ok(None),
            Some(Ok(tok)) if tok.is_rel_op() => Ok(None),
            _ => {
                match &self.current_token {
                    Some(Ok(t)) if t.is_mul_op() => {
                        let op = self.parse_multiplicative_op()?;
                        let factor = Box::new(self.parse_factor()?);
                        let sub_term_res = self.parse_sub_term()?;
                        let sub_term = sub_term_res;

                        let factor_type = self.analyzer.get_factor_type(&factor)?;
                        let fact_type_str = match factor_type {
                            Usage::Variable(s) | Usage::Constant(s) => s,
                            _ => todo!(),
                        };
                        let sub_term_type;
                        if sub_term.is_some() {
                            sub_term_type = self
                                .analyzer
                                .get_sub_term_type(sub_term.as_ref().unwrap())?;
                        } else {
                            sub_term_type = String::new();
                        }

                        let res = self.analyzer.merge_types(
                            &fact_type_str,
                            &sub_term_type,
                            self.current_pos,
                            false,
                        )?;
                        // let sub_term_type = self.analyzer.get_subterm_type(&sub_term)?;

                        Ok(Some(Box::new(SubTerm {
                            op,
                            factor,
                            sub_term,
                            sub_term_type: res,
                        })))
                    }
                    Some(Ok(t)) if t.is_add_op() || t.is_expression_end() || t.is_rel_op() => {
                        Ok(None)
                    }
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
        }
    }

    fn parse_comma(&mut self) -> Result<(), CompilerError> {
        let tok = self.current_token.take();
        self.next_token();

        match tok {
            Some(Ok(t)) => match t {
                Token {
                    token: TokenType::Comma,
                    ..
                } => Ok(()),
                _ => Err(CompilerError::syntax(
                    format!("Expected ',', found {:?}", t),
                    t.pos,
                )),
            },
            Some(Err(e)) => Err(e),
            _ => Err(CompilerError::syntax(
                "Unexpected EOF".into(),
                self.current_pos,
            )),
        }
    }

    fn parse_type_declaration(&mut self) -> Result<Vec<TypeDeclaration>, CompilerError> {
        // id {,id} : type_id
        let mut types = Vec::new();

        loop {
            match self.current_token.take() {
                Some(Ok(token)) => match token {
                    Token {
                        token: TokenType::Identifier(_),
                        ..
                    } => {
                        types.push(Ok(token));
                        self.next_token();

                        match &self.current_token {
                            Some(Ok(Token {
                                token: TokenType::Colon,
                                ..
                            })) => break,
                            _ => self.parse_comma()?,
                        }
                    }
                    _ => {
                        types.push(Err(CompilerError::syntax(
                            format!("Expected identifier, found {:?}", token),
                            token.pos,
                        )));
                    }
                },
                Some(Err(e)) => types.push(Err(e)),
                None => types.push(Err(CompilerError::syntax(
                    "Unexpected EOF".into(),
                    self.current_pos,
                ))),
            }
        }

        let parent_type = match &self.current_token {
            Some(Ok(Token {
                token: TokenType::Colon,
                ..
            })) => {
                self.next_token();
                match self.current_token.take() {
                    Some(Ok(token)) => {
                        let type_id = Identifier { id: token.clone() };
                        self.next_token();
                        self.parse_semicolon()?;
                        let usage = self.analyzer.find_identifier(&type_id)?;

                        match usage {
                            Usage::Type(_) => Ok(type_id),
                            _ => Err(CompilerError::semantic(
                                "Identifier is not a type".into(),
                                token.pos,
                            )),
                        }
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

        match parent_type {
            Ok(type_id) => {
                let mut declarations = Vec::new();

                for id in types {
                    if let Ok(token) = id {
                        declarations.push(TypeDeclaration {
                            id: Identifier { id: token },
                            parent: type_id.clone(),
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

    fn parse_type_section(&mut self) -> Result<TypeSection, CompilerError> {
        // [type
        //      <type_declaration>
        //      {<type_declaration>}
        let mut declarations = Vec::new();
        match &self.current_token {
            Some(Ok(Token {
                token: TokenType::TypeKeyword,
                ..
            })) => {
                self.next_token();

                loop {
                    match &self.current_token {
                        Some(Ok(Token {
                            token: TokenType::BeginKeyword,
                            ..
                        }))
                        | Some(Ok(Token {
                            token: TokenType::VarKeyword,
                            ..
                        })) => break,
                        _ => {
                            let decl = self.parse_type_declaration();
                            match decl {
                                Ok(v) => {
                                    for i in v {
                                        let check_res = self.analyzer.check_type_declaration(i);
                                        match check_res {
                                            Ok(decl) => declarations.push(decl),
                                            Err(e) => self.errors.push(e),
                                        }
                                    }
                                }
                                Err(e) => {
                                    self.errors.push(e);
                                    self.skip_until_starters();
                                }
                            }
                        }
                    }
                }

                Ok(TypeSection {
                    types: declarations,
                })
            }
            Some(Ok(t)) => Err(CompilerError::syntax(
                format!("Expected TYPE, found {:?}", t),
                t.pos,
            )),
            Some(Err(e)) => Err(e.clone()),
            _ => Err(CompilerError::syntax(
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
                                token: TokenType::Colon,
                                ..
                            })) => break,
                            _ => self.parse_comma()?,
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
                        let type_id = Identifier { id: token.clone() };
                        self.next_token();
                        self.parse_semicolon()?;
                        let usage = self.analyzer.find_identifier(&type_id)?;
                        match usage {
                            Usage::Type(_) => Ok(type_id),
                            _ => Err(CompilerError::semantic(
                                "Identifier is not a type".into(),
                                token.pos,
                            )),
                        }
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
                            let check_res = self.analyzer.check_var_declaration(i);
                            match check_res {
                                Ok(decl) => declarations.push(decl),
                                Err(e) => self.errors.push(e),
                            }
                        }
                    }
                    Err(e) => {
                        self.errors.push(e);
                        self.skip_until_starters();
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
                } else if t.is_rel_op() {
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

                let type_section = match self.current_token {
                    Some(Ok(Token {
                        token: TokenType::TypeKeyword,
                        ..
                    })) => Some(Box::new(self.parse_type_section()?)),
                    _ => None,
                };
                let var_section = match self.current_token {
                    Some(Ok(Token {
                        token: TokenType::VarKeyword,
                        ..
                    })) => Some(Box::new(self.parse_var_section()?)),
                    _ => None,
                };
                let compound = Box::new(self.parse_compound()?);

                self.parse_period()?;
                self.analyzer.leave_scope();

                Ok(Program {
                    identifier: id,
                    var_section,
                    type_section,
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
                Token {
                    token: TokenType::OrOp,
                    ..
                } => {
                    self.next_token();
                    Ok(AdditiveOp::Or)
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
            Some(Ok(t)) if t.is_expression_end() || t.is_rel_op() => Ok(None),
            Some(Ok(t)) if t.is_add_op() => {
                let op = Box::new(self.parse_additive_op()?);
                let term = Box::new(self.parse_term()?);
                let sub_expr_res = self.parse_sub_expr()?;
                let sub_expr = sub_expr_res.map(Box::new);

                let term_type = &term.term_type;
                let sub_expr_type;

                if sub_expr.is_some() {
                    sub_expr_type = sub_expr.as_ref().unwrap().sub_expr_type.clone();
                } else {
                    sub_expr_type = String::new();
                }

                let sub_expr_type = self.analyzer.merge_types(
                    term_type,
                    &sub_expr_type,
                    self.current_pos,
                    false,
                )?;

                Ok(Some(SubExpression {
                    op,
                    term,
                    sub_expr,
                    sub_expr_type,
                }))
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
            Some(Ok(Token {
                token: TokenType::IfKeyword,
                ..
            })) => Ok(Statement::Cond(self.parse_conditional()?)),
            Some(Ok(Token {
                token: TokenType::WhileKeyword,
                ..
            })) => Ok(Statement::While(self.parse_while_loop()?)),
            _ => Err(CompilerError::syntax(
                "Illegal statement".into(),
                self.current_pos,
            )),
        }
    }

    fn parse_expr(&mut self) -> Result<Expression, CompilerError> {
        // Expr ::= <Simple Expr> | <Simple Expr> <Rel Op> <Simple Expr>
        let first = Box::new(self.parse_simple_expr()?);

        match &self.current_token {
            Some(Ok(token)) if token.is_rel_op() => {
                let op = Box::new(self.parse_relational_op()?);
                let second = Box::new(self.parse_simple_expr()?);
                self.analyzer.merge_types(
                    &first.expr_type,
                    &second.expr_type,
                    self.current_pos,
                    true,
                )?;

                Ok(Expression::Relational(Box::new(RelationalExpression {
                    first,
                    op,
                    second,
                })))
            }
            _ => Ok(Expression::Simple(first)),
        }
    }

    fn parse_if(&mut self) -> Result<(), CompilerError> {
        let tok = self.current_token.take();
        self.next_token();

        match tok {
            Some(Ok(Token {
                token: TokenType::IfKeyword,
                ..
            })) => Ok(()),
            Some(Ok(t)) => Err(CompilerError::syntax(
                format!("Expected 'if', found {:?}", t),
                t.pos,
            )),
            _ => Err(CompilerError::syntax(
                "Unexpected EOF".into(),
                self.current_pos,
            )),
        }
    }

    fn parse_then(&mut self) -> Result<(), CompilerError> {
        let tok = self.current_token.take();
        self.next_token();

        match tok {
            Some(Ok(Token {
                token: TokenType::ThenKeyword,
                ..
            })) => Ok(()),
            Some(Ok(t)) => Err(CompilerError::syntax(
                format!("Expected 'Then', found {:?}", t),
                t.pos,
            )),
            Some(Err(e)) => Err(e),
            _ => Err(CompilerError::syntax(
                "Unexpected EOF".into(),
                self.current_pos,
            )),
        }
    }

    fn parse_else(&mut self) -> Result<Option<()>, CompilerError> {
        match &self.current_token {
            Some(Ok(Token {
                token: TokenType::ElseKeyword,
                ..
            })) => {
                self.next_token();
                Ok(Some(()))
            }
            Some(Err(e)) => Err(e.clone()),
            _ => Ok(None),
        }
    }

    fn parse_conditional(&mut self) -> Result<IfStatement, CompilerError> {
        self.parse_if()?;
        let condition = Box::new(self.parse_expr()?);
        self.analyzer
            .check_expr(&condition, &String::from("boolean"), self.current_pos)?;
        self.parse_then()?;
        let statement = self.parse_statement()?;
        let else_ = self.parse_else()?;

        let else_statement = if else_.is_some() {
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };

        Ok(IfStatement {
            condition,
            statement: Box::new(statement),
            else_statement,
        })
    }

    fn parse_while(&mut self) -> Result<(), CompilerError> {
        let tok = self.current_token.take();
        self.next_token();

        match tok {
            Some(Ok(Token {
                token: TokenType::WhileKeyword,
                ..
            })) => Ok(()),
            Some(Ok(t)) => Err(CompilerError::syntax(
                format!("Expected 'While', found {:?}", t),
                t.pos,
            )),
            Some(Err(e)) => Err(e),
            _ => Err(CompilerError::syntax(
                "Unexpected EOF".into(),
                self.current_pos,
            )),
        }
    }

    fn parse_do(&mut self) -> Result<(), CompilerError> {
        let tok = self.current_token.take();
        self.next_token();

        match tok {
            Some(Ok(Token {
                token: TokenType::DoKeyword,
                ..
            })) => Ok(()),
            Some(Ok(t)) => Err(CompilerError::syntax(
                format!("Expected 'Do', found {:?}", t),
                t.pos,
            )),
            Some(Err(e)) => Err(e),
            _ => Err(CompilerError::syntax(
                "Unexpected EOF".into(),
                self.current_pos,
            )),
        }
    }

    fn parse_while_loop(&mut self) -> Result<WhileLoop, CompilerError> {
        self.parse_while()?;
        let expr = self.parse_expr()?;
        self.analyzer
            .check_expr(&expr, &String::from("boolean"), self.current_pos)?;
        self.parse_do()?;
        let statement = self.parse_statement()?;

        Ok(WhileLoop {
            condition: Box::new(expr),
            statement: Box::new(statement),
        })
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
                        token: TokenType::EndKeyword,
                        ..
                    }
                    | Token {
                        token: TokenType::BeginKeyword,
                        ..
                    }
                    | Token {
                        token: TokenType::IfKeyword,
                        ..
                    }
                    | Token {
                        token: TokenType::WhileKeyword,
                        ..
                    } => {
                        return;
                    }
                    Token {
                        token: TokenType::Identifier(_),
                        ..
                    } => match self.lexer.peek() {
                        Some(Ok(Token {
                            token: TokenType::AssignOp,
                            ..
                        })) => return,
                        _ => {
                            self.next_token();
                            continue;
                        }
                    },
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

    fn parse_relational_op(&mut self) -> Result<RelationalOp, CompilerError> {
        let op = match &self.current_token {
            Some(Ok(Token {
                token: TokenType::BiggerEq,
                ..
            })) => {
                self.next_token();
                Ok(RelationalOp::BiggerEq)
            }
            Some(Ok(Token {
                token: TokenType::Bigger,
                ..
            })) => {
                self.next_token();
                Ok(RelationalOp::Bigger)
            }
            Some(Ok(Token {
                token: TokenType::LessEq,
                ..
            })) => {
                self.next_token();
                Ok(RelationalOp::LessEq)
            }
            Some(Ok(Token {
                token: TokenType::Less,
                ..
            })) => {
                self.next_token();
                Ok(RelationalOp::Less)
            }
            Some(Ok(Token {
                token: TokenType::Eq,
                ..
            })) => {
                self.next_token();
                Ok(RelationalOp::Eq)
            }
            Some(Ok(Token {
                token: TokenType::UnEq,
                ..
            })) => {
                self.next_token();
                Ok(RelationalOp::UnEq)
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

    fn parse_simple_expr(&mut self) -> Result<SimpleExpression, CompilerError> {
        let mut inside_braces = false;

        if let &Some(Ok(Token {
            token: TokenType::LBrace,
            ..
        })) = &self.current_token
        {
            inside_braces = true;
            self.next_token();
        };

        let term = Box::new(self.parse_term()?);
        let sub_expr = self.parse_sub_expr()?;
        let sub_expr_type;

        if sub_expr.is_some() {
            sub_expr_type = sub_expr.as_ref().unwrap().sub_expr_type.clone();
        } else {
            sub_expr_type = String::new();
        }

        let pos = self.current_pos;

        let expr_type = self
            .analyzer
            .merge_types(&term.term_type, &sub_expr_type, pos, false)?;

        let expr = SimpleExpression {
            term,
            sub_expr,
            expr_type,
        };

        let expr = if inside_braces {
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
        };

        expr
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
                    name: Box::new(id),
                    value: Box::new(self.parse_expr()?),
                };

                self.parse_semicolon()?;
                Ok(self.analyzer.check_assignment(assignment)?)
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
                token: TokenType::AndOp,
                ..
            })) => {
                self.next_token();
                Ok(MultiplicativeOp::And)
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
