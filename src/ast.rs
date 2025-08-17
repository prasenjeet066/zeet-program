#[derive(Debug, Clone)]
pub enum Literal {
    Str(String),
    Num(f64),
    Bool(bool),
    Array(Vec<Literal>),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Import { name: String, alias: Option<String>, module: String },
    Function { params: Vec<String>, types: Vec<(String,String)>, body: Vec<Stmt> },
    Call { callee: Box<Expr>, args: Vec<Expr> },
    If { cond: Box<Expr>, then_body: Vec<Stmt>, else_body: Option<Vec<Stmt>> },
    Run(Box<Expr>),
    Return(Box<Expr>),
    Binary { left: Box<Expr>, op: String, right: Box<Expr> },
    Var(String),
    Lit(Literal),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(Expr),
    FunctionDef(Expr),
    ImportStmt(Expr),
    Empty,
}