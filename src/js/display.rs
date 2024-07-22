use std::fmt;

use crate::js::Expr;

pub struct Display<'a, T: 'a> {
    pub(crate) indent: usize,
    pub(crate) value: &'a T,
}

impl<'a, T: 'a> Display<'a, T> {
    pub fn new(value: &'a T) -> Self {
        Self { indent: 0, value }
    }

    pub fn write_indent(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for _ in 0..self.indent {
            write!(f, "    ")?;
        }

        Ok(())
    }

    pub fn with<'b, U: 'b>(&self, value: &'b U) -> Display<'b, U> {
        Display {
            indent: self.indent,
            value,
        }
    }

    pub fn with_indented<'b, U: 'b>(&self, value: &'b U) -> Display<'b, U> {
        Display {
            indent: self.indent + 1,
            value,
        }
    }
}

impl<'a> fmt::Display for Display<'a, Expr<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write_indent(f)?;

        match &self.value {
            Expr::Number(n) => write!(f, "{n}"),
            Expr::Parens(expr) => write!(f, "({expr})"),
            Expr::Var(name) => write!(f, "{name}"),
            Expr::Neg(expr) => match **expr {
                Expr::Neg(_) => write!(f, "-({expr})"),
                _ => write!(f, "-{expr}"),
            },
            Expr::Mul(lhs, rhs) => write!(f, "{lhs} * {rhs}"),
            Expr::Div(lhs, rhs) => write!(f, "{lhs} / {rhs}"),
            Expr::Add(lhs, rhs) => write!(f, "{lhs} + {rhs}"),
            Expr::Sub(lhs, rhs) => write!(f, "{lhs} - {rhs}"),
        }
    }
}
