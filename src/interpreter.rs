use crate::ast::{Stmt, Expr, Literal};
use crate::environment::{Environment, Value, Function};
use std::rc::Rc;

pub fn interpret(program: Vec<Stmt>, env: &Environment) {
    for s in program {
        match s {
            Stmt::ImportStmt(e) => {
                if let Expr::Import { name, alias, module } = e {
                    // For prototype: simulate import by registering a dummy function or builtin
                    // e.g., "http_request" provides "request" which returns string "ok"
                    if module == "http_request" {
                        env.set(&alias.unwrap_or(name.clone()), Value::Func(Function {
                            params: vec!["url".into()],
                            body: vec![],
                            types: vec![],
                        }));
                    } else {
                        // unknown module: store Null
                        env.set(&alias.unwrap_or(name.clone()), Value::Null);
                    }
                }
            }
            Stmt::FunctionDef(e) => {
                if let Expr::Function { params, types, body } = e {
                    // store anonymous function under special name? For now require user to assign externally.
                    // For prototype let's register with a generated name or allow retrieving by index.
                    // Simpler: store function under name "__anonN" not ideal but enough for demonstration.
                    let fn_name = format!("__fn_{}", rand::random::<u32>());
                    env.set(&fn_name, Value::Func(Function { params, body, types }));
                    // print registration
                    println!("Registered function {}", fn_name);
                }
            }
            Stmt::Expr(expr) => {
                eval_expr(expr, env);
            }
            _ => {}
        }
    }
}

fn eval_expr(expr: Expr, env: &Environment) -> Value {
    match expr {
        Expr::Lit(l) => Value::from(l),
        Expr::Var(name) => {
            env.get(&name).unwrap_or(Value::Null)
        }
        Expr::Binary { left, op, right } => {
            let l = eval_expr(*left, env);
            let r = eval_expr(*right, env);
            match op.as_str() {
                "plus" => {
                    match (l, r) {
                        (Value::Num(a), Value::Num(b)) => Value::Num(a + b),
                        (Value::Str(a), Value::Num(b)) => {
                            Value::Str(format!("{}{}", a, b))
                        }
                        (Value::Str(a), Value::Str(b)) => Value::Str(format!("{}{}", a, b)),
                        _ => Value::Null,
                    }
                }
                "and" => {
                    match (l, r) {
                        (Value::Bool(a), Value::Bool(b)) => Value::Bool(a && b),
                        _ => Value::Null,
                    }
                }
                "same" => {
                    // check string equality or number equality
                    match (l, r) {
                        (Value::Str(a), Value::Str(b)) => Value::Bool(a == b),
                        (Value::Num(a), Value::Num(b)) => Value::Bool((a - b).abs() < 1e-9),
                        _ => Value::Bool(false),
                    }
                }
                "not_equal" => {
                    match (l, r) {
                        (Value::Num(a), Value::Num(b)) => Value::Bool((a - b).abs() > 1e-9),
                        (Value::Str(a), Value::Str(b)) => Value::Bool(a != b),
                        _ => Value::Bool(true),
                    }
                }
                _ => Value::Null,
            }
        }
        Expr::If { cond, then_body, else_body } => {
            let c = eval_expr(*cond, env);
            let take_then = match c {
                Value::Bool(b) => b,
                _ => false,
            };
            if take_then {
                for st in then_body {
                    if let Stmt::Expr(e) = st {
                        let val = eval_expr(e, env);
                        // return on Return
                        if let Value::Null = val { } // ignore
                    }
                }
            } else if let Some(else_block) = else_body {
                for st in else_block {
                    if let Stmt::Expr(e) = st {
                        let val = eval_expr(e, env);
                    }
                }
            }
            Value::Null
        }
        Expr::Run(boxed) => {
            // handle as a function call where callee is Var(name) or Call
            match *boxed {
                Expr::Call{ callee, args } => {
                    // not implemented heavy: return Null
                    Value::Null
                }
                Expr::Var(name) => {
                    // call builtin 'add' or imported functions: for prototype, if name == "add" and args absent, return dummy
                    if name == "add" {
                        Value::Num(42.0)
                    } else {
                        env.get(&name).unwrap_or(Value::Null)
                    }
                }
                _ => Value::Null,
            }
        }
        Expr::Return(boxed) => {
            // return expression value: here we just evaluate and print result
            let v = eval_expr(*boxed, env);
            println!("Return => {:?}", v);
            v
        }
        Expr::Call { callee, args } => {
            // support simple form: callee is Var(name)
            if let Expr::Var(name) = *callee {
                // find in env
                if let Some(Value::Func(f)) = env.get(&name) {
                    // for prototype: do not execute body, just return Null or dummy
                    return Value::Null;
                } else {
                    return Value::Null;
                }
            }
            Value::Null
        }
        _ => Value::Null,
    }
}