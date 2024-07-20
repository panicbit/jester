#![allow(clippy::result_large_err)]

mod expr;
mod parser;

use std::borrow::Cow;
use std::{env, fmt, fs};

use ariadne::{ColorGenerator, Fmt, Label, Report, ReportKind, Source};
use chumsky::Parser;
use parser::parser;

use crate::expr::{Expr, Span};

fn main() {
    let path = env::args().nth(1).unwrap();
    let input = fs::read_to_string(path).unwrap();

    let expr = match parser().parse(&input).into_result() {
        Ok(expr) => expr,
        Err(errs) => {
            for err in errs {
                eprintln!("Error: {err:?}");
            }
            return;
        }
    };

    let js = match Trans::new(Span::new(0, input.len())).trans(&expr) {
        Ok(js) => js,
        Err(report) => {
            report.print(Source::from(&input)).unwrap();
            return;
        }
    };

    println!("{js}");
    // println!("{:#?}", js);
}

#[derive(Debug)]
struct Variable {
    jester_name: String,
    js_name: String,
}

#[derive(Debug)]
struct Scope {
    variables: Vec<Variable>,
    span: Span,
}

impl Scope {
    fn new(span: Span) -> Self {
        Self {
            variables: Vec::new(),
            span,
        }
    }

    fn has_js_variable(&self, js_name: &str) -> bool {
        for variable in &self.variables {
            if variable.js_name == js_name {
                return true;
            }
        }

        false
    }

    fn declare_variable<'a>(&mut self, jester_name: &'a str) -> Cow<'a, str> {
        if !self.has_js_variable(jester_name) {
            self.declare_variable_unchecked(jester_name, jester_name);

            return Cow::Borrowed(jester_name);
        }

        let n = 2;

        loop {
            let js_name = format!("{jester_name}__{n}");

            if !self.has_js_variable(&js_name) {
                self.declare_variable_unchecked(jester_name, &js_name);

                return Cow::Owned(js_name);
            }
        }
    }

    fn declare_variable_unchecked(
        &mut self,
        jester_name: impl Into<String>,
        js_name: impl Into<String>,
    ) {
        self.variables.push(Variable {
            jester_name: jester_name.into(),
            js_name: js_name.into(),
        });
    }
}

struct Trans {
    scopes: Vec<Scope>,
}

impl Trans {
    fn new(span: Span) -> Self {
        Self {
            scopes: vec![Scope::new(span)],
        }
    }

    fn last_scope_mut(&mut self) -> &mut Scope {
        self.scopes
            .last_mut()
            .expect("BUG: expected at least one scope")
    }

    fn declare_variable<'a>(&mut self, jester_name: &'a str) -> Cow<'a, str> {
        self.last_scope_mut().declare_variable(jester_name)
    }

    fn resolve_variable(&mut self, jester_name: &str) -> Option<&str> {
        self.scopes
            .iter()
            .rev()
            .flat_map(|scope| scope.variables.iter().rev())
            .find(|variable| variable.jester_name == jester_name)
            .map(|variable| variable.js_name.as_str())
    }

    fn trans<'a>(&mut self, expr: &'a Expr) -> Result<JSExpr<'a>, Report<'static>> {
        Ok(match expr {
            Expr::Int(n) => JSExpr::Number((*n).into()),
            Expr::Parens(expr) => JSExpr::Parens(self.trans(expr)?.boxed()),
            Expr::Var(name, span) => {
                let name = self
                    .resolve_variable(name)
                    .ok_or_else(|| report_undeclared_variable(name, span))?;

                JSExpr::Var(Cow::Owned(name.into()))
            }
            Expr::Neg(expr) => JSExpr::Neg(self.trans(expr)?.boxed()),
            Expr::Add(lhs, rhs) => JSExpr::Add(self.trans(lhs)?.boxed(), self.trans(rhs)?.boxed()),
            Expr::Sub(lhs, rhs) => JSExpr::Sub(self.trans(lhs)?.boxed(), self.trans(rhs)?.boxed()),
            Expr::Mul(lhs, rhs) => JSExpr::Mul(self.trans(lhs)?.boxed(), self.trans(rhs)?.boxed()),
            Expr::Div(lhs, rhs) => JSExpr::Div(self.trans(lhs)?.boxed(), self.trans(rhs)?.boxed()),
            Expr::Call(_, _) => todo!(),
            Expr::Let { name, rhs, then } => {
                JSExpr::Let {
                    // order is significant
                    rhs: self.trans(rhs)?.boxed(),
                    name: self.declare_variable(name),
                    then: self.trans(then)?.boxed(),
                }
            }
            Expr::Fn {
                name,
                args,
                body,
                then,
            } => todo!(),
        })
    }
}

fn report_undeclared_variable(name: &str, span: &Span) -> Report<'static> {
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

#[derive(Debug)]
enum JSExpr<'a> {
    Number(f64),
    Let {
        name: Cow<'a, str>,
        rhs: Box<JSExpr<'a>>,
        then: Box<JSExpr<'a>>,
    },
    Parens(Box<JSExpr<'a>>),
    Var(Cow<'a, str>),
    Neg(Box<JSExpr<'a>>),
    Mul(Box<JSExpr<'a>>, Box<JSExpr<'a>>),
    Div(Box<JSExpr<'a>>, Box<JSExpr<'a>>),
    Add(Box<JSExpr<'a>>, Box<JSExpr<'a>>),
    Sub(Box<JSExpr<'a>>, Box<JSExpr<'a>>),
}

impl JSExpr<'_> {
    fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}

impl fmt::Display for JSExpr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let expr = DisplayJSExpr {
            indent: 0,
            expr: self,
        };

        write!(f, "{expr}")
    }
}

struct DisplayJSExpr<'a> {
    indent: usize,
    expr: &'a JSExpr<'a>,
}

impl<'a> DisplayJSExpr<'a> {
    fn write_indent(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for _ in 0..self.indent {
            write!(f, "    ")?;
        }

        Ok(())
    }

    fn with(&self, expr: &'a JSExpr<'a>) -> Self {
        Self {
            indent: self.indent,
            expr,
        }
    }
}

impl<'a> fmt::Display for DisplayJSExpr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write_indent(f)?;

        match &self.expr {
            JSExpr::Number(n) => write!(f, "{n}"),
            JSExpr::Let { name, rhs, then } => {
                writeln!(f, "let {name} = {rhs};")?;
                write!(f, "{then}")
            }
            JSExpr::Parens(expr) => write!(f, "({expr})"),
            JSExpr::Var(name) => write!(f, "{name}"),
            JSExpr::Neg(expr) => match **expr {
                JSExpr::Neg(_) => write!(f, "-({expr})"),
                _ => write!(f, "-{expr}"),
            },
            JSExpr::Mul(lhs, rhs) => write!(f, "{lhs} * {rhs}"),
            JSExpr::Div(lhs, rhs) => write!(f, "{lhs} / {rhs}"),
            JSExpr::Add(lhs, rhs) => write!(f, "{lhs} + {rhs}"),
            JSExpr::Sub(lhs, rhs) => write!(f, "{lhs} - {rhs}"),
        }
    }
}
