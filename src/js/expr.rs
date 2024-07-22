use std::borrow::Cow;
use std::fmt;

use crate::js;

#[derive(Debug)]
pub enum Expr<'a> {
    Number(f64),
    Parens(Box<Expr<'a>>),
    Var(Cow<'a, str>),
    Neg(Box<Expr<'a>>),
    Mul(Box<Expr<'a>>, Box<Expr<'a>>),
    Div(Box<Expr<'a>>, Box<Expr<'a>>),
    Add(Box<Expr<'a>>, Box<Expr<'a>>),
    Sub(Box<Expr<'a>>, Box<Expr<'a>>),
}

impl Expr<'_> {
    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}

impl fmt::Display for Expr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        js::Display::new(self).fmt(f)
    }
}
