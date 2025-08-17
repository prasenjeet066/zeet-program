use crate::token::Token;
use crate::ast::{Expr, Literal, Stmt};
use std::iter::Peekable;
use std::vec::IntoIter;

pub struct Parser {
    tokens: Peekable<IntoIter<Token>>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens: tokens.into_iter().peekable() }
    }

    fn peek(&mut self) -> Token {
        self.tokens.peek().cloned().unwrap_or(Token::Eof)
    }

    fn next(&mut self) -> Token {
        self.tokens.next().unwrap_or(Token::Eof)
    }

    pub fn parse_program(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        loop {
            match self.peek() {
                Token::Eof => break,
                Token::Import => {
                    let imp = self.parse_import();
                    stmts.push(Stmt::ImportStmt(imp));
                }
                Token::FnKw => {
                    let f = self.parse_function();
                    stmts.push(Stmt::FunctionDef(f));
                }
                _ => {
                    // skip unknown top-level tokens
                    self.next();
                }
            }
        }
        stmts
    }

    fn parse_import(&mut self) -> Expr {
        // import name [-> alias] from module
        self.next(); // consume import
        let name = if let Token::Identifier(s) = self.next() { s } else { "".into() };
        let mut alias = None;
        if let Token::Arrow = self.peek() {
            self.next(); // ->
            if let Token::Identifier(a) = self.next() { alias = Some(a); }
        }
        // expect from
        if let Token::From = self.next() {} // consume
        let module = if let Token::Identifier(m) = self.next() {
            m
        } else if let Token::StringLit(s) = self.next() {
            s
        } else { String::new() };
        Expr::Import { name, alias, module }
    }

    fn parse_function(&mut self) -> Expr {
        self.next(); // consume __fn
        // optional '='
        if let Token::Equals = self.peek() {
            self.next();
        }
        // expect '(' params ')'
        let mut params = Vec::new();
        if let Token::LParen = self.next() {
            loop {
                match self.peek() {
                    Token::Identifier(s) => {
                        if let Token::Identifier(name) = self.next() {
                            params.push(name);
                        }
                    }
                    Token::Comma => { self.next(); }
                    Token::RParen => { self.next(); break; }
                    _ => { self.next(); }
                }
            }
        }
        // optional type annotations like : < ... >
        let mut types = Vec::new();
        if let Token::Colon = self.peek() {
            self.next(); // :
            if let Token::LAngle = self.next() {
                // parse simple "a is string, b is string Array" until '>'
                loop {
                    match self.peek() {
                        Token::Identifier(name) => {
                            if let Token::Identifier(param_name) = self.next() {
                                // expect 'is'
                                if let Token::Identifier(is_kw) = self.peek() {
                                    if is_kw.to_lowercase() == "is" {
                                        self.next(); // consume 'is'
                                        if let Token::Identifier(typ) = self.next() {
                                            types.push((param_name, typ));
                                        } else {
                                            // skip
                                            self.next();
                                        }
                                    }
                                }
                            }
                        }
                        Token::Comma => { self.next(); }
                        Token::RAngle => { self.next(); break; }
                        Token::Identifier(_) => { self.next(); }
                        _ => { self.next(); }
                    }
                }
            }
        }

        // parse body lines until Underscore token '__' (function end)
        let mut body: Vec<Stmt> = Vec::new();
        loop {
            match self.peek() {
                Token::Underscore => { self.next(); break; }
                Token::If => {
                    let ifstmt = self.parse_if();
                    body.push(Stmt::Expr(ifstmt));
                }
                Token::Run => {
                    self.next();
                    let expr = self.parse_simple_expr();
                    body.push(Stmt::Expr(Expr::Run(Box::new(expr))));
                }
                Token::Ret => {
                    self.next();
                    let expr = self.parse_simple_expr();
                    body.push(Stmt::Expr(Expr::Return(Box::new(expr))));
                }
                Token::Eof => break,
                _ => { self.next(); } // skip unknown
            }
        }

        Expr::Function { params, types, body }
    }

    fn parse_if(&mut self) -> Expr {
        self.next(); // consume if
        // parse condition inside parentheses or simple tokens until Then
        let cond = self.parse_simple_expr();
        // consume optional Then or "- then," which lexer normalizes to Then token
        if let Token::Then = self.peek() { self.next(); }
        // parse then-body until Otherwise or Underscore or EOF
        let mut then_body = Vec::new();
        loop {
            match self.peek() {
                Token::Otherwise => {
                    self.next(); // consume otherwise
                    break;
                }
                Token::Ret => {
                    self.next();
                    let expr = self.parse_simple_expr();
                    then_body.push(Stmt::Expr(Expr::Return(Box::new(expr))));
                }
                Token::Run => {
                    self.next();
                    let expr = self.parse_simple_expr();
                    then_body.push(Stmt::Expr(Expr::Run(Box::new(expr))));
                }
                Token::Underscore | Token::Eof => break,
                _ => { self.next(); } // skip
            }
        }

        // optional else part after 'otherwise'
        let mut else_body = None;
        if let Token::Ret = self.peek() {
            self.next();
            let expr = self.parse_simple_expr();
            else_body = Some(vec![Stmt::Expr(Expr::Return(Box::new(expr)))]);
        } else if let Token::Identifier(_) = self.peek() {
            // some syntaxes might put 'ret' after 'otherwise -', handled above
        }

        Expr::If { cond: Box::new(cond), then_body, else_body }
    }

    // extremely simple expression parser that recognizes:
    // identifiers, literals, and binary ops with keyword operators (plus, and, same, not equal)
    fn parse_simple_expr(&mut self) -> Expr {
        // get first operand
        let left = match self.next() {
            Token::Identifier(s) => {
                Expr::Var(s)
            }
            Token::StringLit(s) => Expr::Lit(Literal::Str(s)),
            Token::NumberLit(n) => Expr::Lit(Literal::Num(n)),
            Token::BoolLit(b) => Expr::Lit(Literal::Bool(b)),
            Token::LParen => {
                // not heavy parsing: read until RParen as a single identifier or literal
                if let Token::Identifier(s) = self.next() {
                    let _ = self.next(); // consume ')'
                    Expr::Var(s)
                } else { Expr::Lit(Literal::Bool(false)) }
            }
            other => {
                // unknown; return false literal
                Expr::Lit(Literal::Bool(false))
            }
        };

        // lookahead for operator
        match self.peek() {
            Token::Identifier(op) => {
                // operators like plus, and, same, not...
                let op_word = if let Token::Identifier(op2) = self.next() { op2 } else { String::new() };
                // handle 'not equal' sequence
                if op_word == "not" {
                    if let Token::Identifier(next_word) = self.peek() {
                        if next_word == "equal" {
                            let _ = self.next(); // consume 'equal'
                            // parse right operand
                            let right = self.parse_simple_expr();
                            return Expr::Binary { left: Box::new(left), op: "not_equal".into(), right: Box::new(right) };
                        }
                    }
                }
                // else normal binary operator
                let right = self.parse_simple_expr();
                Expr::Binary { left: Box::new(left), op: op_word, right: Box::new(right) }
            }
            _ => left,
        }
    }
}