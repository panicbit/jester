#![allow(clippy::result_large_err)]

mod js;
mod parser;
mod report;
mod syntax;
mod trans;

use std::{env, fs};

use ariadne::Source;
use chumsky::Parser;
use parser::parser;

use crate::js::Display;
use crate::syntax::Span;
use crate::trans::Trans;

fn main() {
    let path = env::args().nth(1).unwrap();
    let input = fs::read_to_string(path).unwrap();
    let source = Source::from(&input);

    let expr = match parser().parse(&input).into_result() {
        Ok(expr) => expr,
        Err(errs) => {
            for err in errs {
                let report = report::parse_err(err);
                report.print(source.clone());
            }
            return;
        }
    };

    println!("{:#?}", expr);

    let js = match Trans::new(Span::new(0, input.len())).trans_block(&expr) {
        Ok(js) => js,
        Err(report) => {
            report.print(source).unwrap();
            return;
        }
    };

    println!("{}", Display::new(&js));
    // println!("{:#?}", js);
}
