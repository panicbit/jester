#![allow(clippy::result_large_err)]

mod expr;
mod js;
mod parser;

use std::borrow::Cow;
use std::{env, fmt, fs};

use ariadne::{ColorGenerator, Fmt, Label, Report, ReportKind, Source};
use chumsky::Parser;
use parser::parser;

use crate::expr::{Expr, Span};
use crate::js::DisplayJS;
use crate::parser::{Block, Let, Stmt};

fn main() {
    let path = env::args().nth(1).unwrap();
    let input = fs::read_to_string(path).unwrap();
    let source = Source::from(&input);

    let expr = match parser().parse(&input).into_result() {
        Ok(expr) => expr,
        Err(errs) => {
            for err in errs {
                let report = report_parse_err(err);
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

    println!("{}", DisplayJS::new(&js));
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

    fn trans_block<'a>(&mut self, block: &'a Block) -> Result<js::Block<'a>, Report<'static>> {
        let stmts = block
            .stmts
            .iter()
            .map(|stmt| self.trans_stmt(stmt))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(js::Block { stmts })
    }

    fn trans_stmt<'a>(&mut self, stmt: &'a Stmt) -> Result<js::Stmt<'a>, Report<'static>> {
        Ok(match stmt {
            Stmt::Let(Let {
                name: _,
                ty: _,
                rhs,
            }) if rhs.contains_block() => {
                todo!("blocks in let not supported right now")
            }
            Stmt::Let(Let { name, ty, rhs }) => {
                let name = self.declare_variable(name);

                js::Stmt::Let(js::Let {
                    name: name.into(),
                    rhs: self.trans_expr(rhs)?.boxed(),
                })
            }
            Stmt::Expr(expr) => js::Stmt::Expr(self.trans_expr(expr)?),
        })
    }

    fn trans_expr<'a>(&mut self, expr: &'a Expr) -> Result<js::Expr<'a>, Report<'static>> {
        Ok(match expr {
            Expr::Int(n) => js::Expr::Number((*n).into()),
            Expr::Parens(expr) => js::Expr::Parens(self.trans_expr(expr)?.boxed()),
            Expr::Var {
                name,
                name_span: span,
            } => {
                let name = self
                    .resolve_variable(name)
                    .ok_or_else(|| report_undeclared_variable(name, span))?;

                js::Expr::Var(Cow::Owned(name.into()))
            }
            Expr::Neg(expr) => js::Expr::Neg(self.trans_expr(expr)?.boxed()),
            Expr::Add(lhs, rhs) => {
                js::Expr::Add(self.trans_expr(lhs)?.boxed(), self.trans_expr(rhs)?.boxed())
            }
            Expr::Sub(lhs, rhs) => {
                js::Expr::Sub(self.trans_expr(lhs)?.boxed(), self.trans_expr(rhs)?.boxed())
            }
            Expr::Mul(lhs, rhs) => {
                js::Expr::Mul(self.trans_expr(lhs)?.boxed(), self.trans_expr(rhs)?.boxed())
            }
            Expr::Div(lhs, rhs) => {
                js::Expr::Div(self.trans_expr(lhs)?.boxed(), self.trans_expr(rhs)?.boxed())
            }
            Expr::Call(_, _) => todo!(),
            Expr::Fn {
                name,
                args,
                body,
                then,
            } => todo!(),
            Expr::Block(block) => todo!("block!"),
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

fn report_parse_err(err: chumsky::error::Rich<char>) -> Report<'static> {
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
