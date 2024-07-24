use std::fs;

use ariadne::Source;
use chumsky::error::Rich;
use chumsky::primitive::end;
use chumsky::{ParseResult, Parser as _};
use eyre::{eyre, Context, Result};
use glob::glob;
use jester_script::parser::Parser;

fn parse<'a, T: 'a>(input: &'a str, parser: impl Parser<'a, T>) -> ParseResult<T, Rich<char>> {
    parser.then_ignore(end()).parse(input)
}

fn parse_file<T>(path: &str, parser: impl for<'a> Parser<'a, T>) -> Result<()> {
    let input =
        fs::read_to_string(path).with_context(|| format!("failed to read test file: {path}"))?;
    let source = Source::from(&input);

    let value = match parse(&input, parser).into_result() {
        Ok(value) => value,
        Err(errs) => {
            let mut error_report = Err(eyre!("Failed to parse `{path:?}`"));

            for err in errs {
                let report = jester_script::report::parse_err(err);
                let mut output = Vec::new();
                report
                    .write_for_stdout(source.clone(), &mut output)
                    .context("failed to write ariadne report")?;

                let output =
                    String::from_utf8(output).context("Ariadne report contains invalid UTF-8")?;

                error_report = error_report.context(output);
            }

            return error_report;
        }
    };

    Ok(())
}

#[test]
fn end_to_end() {
    glob("tests/parsing/*.jst").unwrap();
}
