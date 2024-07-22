use std::{env, fs};

use ariadne::Source;
use chumsky::Parser;

use jester_script::js::Display;
use jester_script::syntax::Span;
use jester_script::trans::Trans;
use jester_script::{parser, report};

fn main() {
    let path = env::args().nth(1).unwrap();
    let input = fs::read_to_string(path).unwrap();
    let source = Source::from(&input);

    let file = match parser::file().parse(&input).into_result() {
        Ok(file) => file,
        Err(errs) => {
            for err in errs {
                let report = report::parse_err(err);
                report.print(source.clone());
            }
            return;
        }
    };

    println!("{:#?}", file);

    let js = match Trans::new(Span::new(0, input.len())).trans_file(&file) {
        Ok(js) => js,
        Err(report) => {
            report.print(source).unwrap();
            return;
        }
    };

    println!("{}", Display::new(&js));
    // println!("{:#?}", js);
}
