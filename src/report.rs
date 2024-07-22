use ariadne::{ColorGenerator, Fmt, Label, Report, ReportKind};

use crate::syntax::Span;

pub fn undeclared_variable(name: &str, span: &Span) -> Report<'static> {
    let mut c = ColorGenerator::new();
    let name_color = c.next();
    let name = name.fg(name_color);

    Report::build(ReportKind::Error, (), span.start)
        .with_label(
            Label::new(span.into_range())
                .with_message(format!("Variable `{}` was not declared", name,))
                .with_color(name_color),
        )
        .with_help(format!("Use `let {name} = …;`"))
        .finish()
}

pub fn parse_err(err: chumsky::error::Rich<char>) -> Report<'static> {
    let mut c = ColorGenerator::new();
    let span = err.span();

    Report::build(ReportKind::Error, (), span.start)
        .with_label(
            Label::new(span.into_range())
                .with_message(format!("{}", err.reason()))
                .with_color(c.next()),
        )
        .finish()
}
