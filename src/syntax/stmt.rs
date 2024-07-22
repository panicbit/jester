use crate::syntax::{Expr, Let};

#[derive(Debug)]
pub enum Stmt<'a> {
    Let(Let<'a>),
    Expr(Expr<'a>),
}
