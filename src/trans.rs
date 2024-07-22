use std::borrow::Cow;

use ariadne::Report;

use crate::syntax::{Block, Expr, Let, Span, Stmt};
use crate::{js, report};

pub struct Trans {
    scopes: Vec<Scope>,
}

impl Trans {
    pub fn new(span: Span) -> Self {
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

    pub fn trans_block<'a>(&mut self, block: &'a Block) -> Result<js::Block<'a>, Report<'static>> {
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
            Stmt::Let(Let { name, ty: _, rhs }) => {
                let name = self.declare_variable(name);

                js::Stmt::Let(js::Let {
                    name,
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
                    .ok_or_else(|| report::undeclared_variable(name, span))?;

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

#[derive(Debug)]
struct Variable {
    jester_name: String,
    js_name: String,
}
