#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Import,
    From,
    Arrow,         // ->
    Assign,        // =
    FnStart,       // __fn
    BlockEnd,      // __
    Identifier(String),
    StringLiteral(String),
    NumberLiteral(f64),
    Keyword(String),   // if, then, otherwise, run, ret, etc.
    Operator(String),  // plus, minus, equal, not equal
    LParen, RParen,
    Colon, Comma,
    Lt, Gt,          // for <types>
    EOF,
}

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self { input: input.chars().collect(), pos: 0 }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).cloned()
    }

    fn advance(&mut self) -> Option<char> {
        if self.pos < self.input.len() {
            let c = self.input[self.pos];
            self.pos += 1;
            Some(c)
        } else {
            None
        }
    }

    fn consume_while<F>(&mut self, cond: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut result = String::new();
        while let Some(c) = self.peek() {
            if cond(c) {
                result.push(c);
                self.advance();
            } else {
                break;
            }
        }
        result
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = vec![];

        while let Some(c) = self.peek() {
            match c {
                ' ' | '\t' | '\n' | '\r' => { self.advance(); },
                '(' => { tokens.push(Token::LParen); self.advance(); },
                ')' => { tokens.push(Token::RParen); self.advance(); },
                ',' => { tokens.push(Token::Comma); self.advance(); },
                ':' => { tokens.push(Token::Colon); self.advance(); },
                '<' => { tokens.push(Token::Lt); self.advance(); },
                '>' => { tokens.push(Token::Gt); self.advance(); },
                '"' => {
                    self.advance(); // skip quote
                    let s = self.consume_while(|ch| ch != '"');
                    self.advance(); // skip ending quote
                    tokens.push(Token::StringLiteral(s));
                }
                '0'..='9' => {
                    let num = self.consume_while(|ch| ch.is_ascii_digit() || ch == '.');
                    tokens.push(Token::NumberLiteral(num.parse().unwrap()));
                }
                '-' => {
                    self.advance();
                    if self.peek() == Some('>') {
                        self.advance();
                        tokens.push(Token::Arrow);
                    }
                }
                '=' => { tokens.push(Token::Assign); self.advance(); },
                '_' => {
                    let word = self.consume_while(|ch| ch.is_alphanumeric() || ch == '_');
                    if word == "__fn" {
                        tokens.push(Token::FnStart);
                    } else if word == "__" {
                        tokens.push(Token::BlockEnd);
                    } else {
                        tokens.push(Token::Identifier(word));
                    }
                }
                'a'..='z' | 'A'..='Z' => {
                    let word = self.consume_while(|ch| ch.is_alphanumeric() || ch == '_');
                    match word.as_str() {
                        "import" => tokens.push(Token::Import),
                        "from" => tokens.push(Token::From),
                        "if" | "then" | "otherwise" | "run" | "ret" => tokens.push(Token::Keyword(word)),
                        "plus" | "minus" | "equal" | "not" => tokens.push(Token::Operator(word)),
                        _ => tokens.push(Token::Identifier(word)),
                    }
                }
                _ => { self.advance(); }
            }
        }

        tokens.push(Token::EOF);
        tokens
    }
}