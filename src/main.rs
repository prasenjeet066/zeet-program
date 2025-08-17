mod token;
mod lexer;
mod ast;
mod parser;
mod environment;
mod interpreter;

use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::environment::Environment;
use crate::interpreter::interpret;
use std::fs;

fn main() {
    // Example: read file from first arg or run built-in example
    let args: Vec<String> = std::env::args().collect();
    let code = if args.len() > 1 {
        fs::read_to_string(&args[1]).expect("cannot read file")
    } else {
        // sample program (from your example)
        r#"
import request from http_request
import request -> req from http_request

__fn = (a,b):<a is string, b is string Array>
       if(a and b same) - then,
         run add(a plus 3)
       otherwise - ret false
__
        
__fn = (a):<a is number>
       if ( a is realNumber and a not equal 0 ) - then,
          ret a
        __
__
"#
        .to_string()
    };

    let mut lx = Lexer::new(&code);
    let toks = lx.tokenize();
    // println!("TOKENS: {:?}", toks);

    let mut p = Parser::new(toks);
    let prog = p.parse_program();
    // println!("AST: {:#?}", prog);

    let env = Environment::new();
    // add a builtin function 'add' to demonstrate run
    env.set("add", Value::Func(crate::environment::Function {
        params: vec!["x".into()],
        body: vec![],
        types: vec![],
    }));

    interpret(prog, &env);
}