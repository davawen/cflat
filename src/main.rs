use ast::parser;
use chumsky::{Parser, prelude::Input};
use lexer::lexer;
use error::show_errs;

mod lexer;
mod ast;
mod ir;
mod error;

fn main() {
    let input = r#"
struct Vec2 {
    i32 x;
    i32 y;
}

void print(msg: u8[] str) {}

void main() {
    i32 i = 0;
    loop {
        if i >= 100 { break }

        if i % 15 == 0 { print(msg = "fizzbuzz") }
        else if i % 3 == 0 { print(msg = "fizz") }
        else if i % 5 == 0 { print(msg = "buzz") }
        else { print(msg = "num") }
    }
}
    "#;

    println!("lexing");
    let (lexed, errs) = lexer().parse(input).into_output_errors();
    show_errs(input, "stdin", errs);

    let Some(lexed) = lexed else { return };

    println!("parsing");
    let (parsed, errs) = parser().parse(Input::spanned(&lexed, (input.len()..input.len()).into())).into_output_errors();
    show_errs(input, "stdin", errs);

    let Some(parsed) = parsed else { return };
    for parsed in &parsed {
        println!("{parsed}");
    }

    let p = ir::Program::lower(&parsed);
    let mut p = match p {
        Ok(p) => p,
        Err(e) => {
            show_errs(input, "stdin", vec![e]);
            return;
        }
    };
    let type_errs = p.typecheck();
    if !type_errs.is_empty() {
        show_errs(input, "stdin", type_errs);
    }
    println!("{p}");
}
