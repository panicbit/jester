use std::fmt;

use crate::js::{self, Block, Expr, Fn, Let};

#[derive(Debug)]
pub enum Stmt<'a> {
    Fn(Fn<'a>),
    Let(Let<'a>),
    // TODO: add `Option<Semi>` or similar
    Expr(Expr<'a>),
    Block(Block<'a>),
}

impl fmt::Display for Stmt<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        js::Display::new(self).fmt(f)
    }
}

impl<'a> fmt::Display for js::Display<'a, Stmt<'a>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.value {
            Stmt::Fn(r#fn) => self.with(r#fn).fmt(f),
            Stmt::Let(let_stmt) => self.with(let_stmt).fmt(f),
            Stmt::Expr(expr) => writeln!(f, "{}", self.with(expr)),
            Stmt::Block(block) => self.with(block).fmt(f),
        }
    }
}
