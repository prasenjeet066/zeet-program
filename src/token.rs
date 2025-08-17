#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords / Symbols
    Import,
    From,
    Arrow,       // ->
    FnKw,        // __fn
    Equals,      // =
    Colon,       // :
    LAngle,      // <
    RAngle,      // >
    LParen,
    RParen,
    Comma,
    If,
    Then,        // recognized via "- then," but we normalize
    Otherwise,
    Run,
    Ret,
    Underscore,  // __ (end of function marker)
    Eof,

    // Operators/words (we keep as identifiers mostly)
    Identifier(String),

    // Literals
    StringLit(String),
    NumberLit(f64),
    BoolLit(bool),
}